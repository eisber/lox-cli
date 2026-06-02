use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::Subcommand;
use lox_sim::{engine::SimEngine, graph::SimGraph, parser};
use serde::Deserialize;
use serde_json::json;

#[derive(Subcommand)]
pub enum SimCmd {
    /// Parse config and report block/connector counts
    Check {
        /// Path to .Loxone config file
        file: String,
    },
    /// Execute simulation specs and check expected outputs
    Run {
        /// Path to .Loxone config file
        file: String,
        /// Sim spec as inline JSON (single object or array)
        #[arg(long)]
        sim: Option<String>,
        /// Path to a JSON file containing sim spec(s)
        #[arg(long)]
        sim_file: Option<String>,
    },
    /// Step-by-step simulation showing signal changes per tick
    Step {
        /// Path to .Loxone config file
        file: String,
        /// Sim spec as inline JSON
        #[arg(long)]
        sim: Option<String>,
        /// Path to a JSON file containing sim spec
        #[arg(long)]
        sim_file: Option<String>,
    },
    /// Dump full graph structure (blocks, connectors, wires)
    Dump {
        /// Path to .Loxone config file
        file: String,
        /// Optional sim spec to apply before dumping
        #[arg(long)]
        sim: Option<String>,
    },
}

#[derive(Deserialize)]
struct SimSpec {
    #[serde(default)]
    name: String,
    #[serde(default)]
    inputs: HashMap<String, f64>,
    #[serde(default = "default_ticks")]
    ticks: usize,
    #[serde(default = "default_dt")]
    dt: f64,
    #[serde(default)]
    expected_outputs: HashMap<String, HashMap<String, f64>>,
    /// Optional time injection (minutes since midnight, 0-1440).
    /// Automatically injected into ALL DayTimer blocks.
    #[serde(default)]
    time: Option<f64>,
    /// Optional multi-step sequence. Each step sets new inputs, ticks forward,
    /// then checks outputs. Enables temporal flow testing (e.g. heat rises,
    /// then cools, then re-triggers).
    #[serde(default)]
    steps: Vec<SimStep>,
    /// When true, include all non-zero output signals in the result JSON.
    /// Enables auto-discovery of what the agent circuit actually produces.
    #[serde(default)]
    trace: bool,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct SimStep {
    #[serde(default)]
    inputs: HashMap<String, f64>,
    #[serde(default = "default_ticks")]
    ticks: usize,
    #[serde(default)]
    dt: f64,
    #[serde(default)]
    time: Option<f64>,
    #[serde(default)]
    expected_outputs: HashMap<String, HashMap<String, f64>>,
}

fn default_ticks() -> usize {
    10
}
fn default_dt() -> f64 {
    0.1
}

struct ScenarioResult {
    name: String,
    pass: bool,
    checks: Vec<serde_json::Value>,
    trace: Vec<serde_json::Value>,
}

/// Collect all non-zero output signals from the engine as structured JSON.
fn collect_trace(engine: &SimEngine, graph: &SimGraph) -> Vec<serde_json::Value> {
    let mut signals = Vec::new();
    for bid in 0..graph.block_count() {
        let info = graph.block_info(bid);
        for &cid in &info.outputs {
            let val = engine.signal(cid);
            if val.abs() > 1e-9 {
                let key = &graph.connector(cid).key;
                let name = if let Some(ref room) = info.room {
                    format!("{} [{}].{}", info.name, room, key)
                } else {
                    format!("{}.{}", info.name, key)
                };
                signals.push(serde_json::json!({
                    "output": name,
                    "value": val,
                    "block": info.name,
                    "room": info.room,
                    "connector": key,
                }));
            }
        }
    }
    signals
}

fn load_sim_json(sim: Option<&str>, sim_file: Option<&str>) -> Result<String> {
    if let Some(json_str) = sim {
        Ok(json_str.to_string())
    } else if let Some(path) = sim_file {
        std::fs::read_to_string(path).with_context(|| format!("failed to read sim-file: {path}"))
    } else {
        let mut buf = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf)
            .context("failed to read sim spec from stdin")?;
        Ok(buf)
    }
}

fn parse_graph(file: &str) -> Result<SimGraph> {
    let path = PathBuf::from(file);
    parser::parse_file(&path).map_err(|e| anyhow::anyhow!("parse_failed: {e}"))
}

fn resolve_output(engine: &SimEngine, graph: &SimGraph, output_key: &str) -> f64 {
    let mut actual = engine.get_output(output_key);
    if actual == 0.0 && output_key.contains('.') {
        let parts: Vec<&str> = output_key.splitn(2, '.').collect();
        let block_name = parts[0];
        let conn_name = parts.get(1).unwrap_or(&"");
        for bid in 0..graph.block_count() {
            let info = graph.block_info(bid);
            let name_matches = info.name == block_name
                || info
                    .room
                    .as_ref()
                    .is_some_and(|r| format!("{} [{}]", info.name, r) == block_name);
            if !name_matches {
                continue;
            }
            if let Some(cid) = graph.find_connector(bid, conn_name) {
                if let Some(src) = graph.input_source_of(cid) {
                    let sig = engine.signal(src);
                    if sig.abs() > actual.abs() {
                        actual = sig;
                    }
                } else {
                    let sig = engine.signal(cid);
                    if sig.abs() > actual.abs() {
                        actual = sig;
                    }
                }
            }
        }
    }
    actual
}

fn check_comparator(actual: f64, op: &str, expected: f64) -> bool {
    match op {
        ">" => actual > expected,
        ">=" => actual >= expected,
        "<" => actual < expected,
        "<=" => actual <= expected,
        "==" => (actual - expected).abs() < 1e-9,
        "~=" => (actual - expected).abs() < expected.abs() * 0.05 + 1e-9,
        _ => false,
    }
}

fn run_one(graph: &SimGraph, spec: &SimSpec) -> ScenarioResult {
    let mut engine = SimEngine::new(graph.clone());
    let mut checks = Vec::new();
    let mut all_pass = true;

    // Inject time into all DayTimer blocks if specified
    if let Some(minutes) = spec.time {
        engine.set_time(minutes);
    }

    apply_inputs(&mut engine, &spec.inputs);

    for _ in 0..spec.ticks {
        engine.tick(spec.dt);
    }

    // Check initial expected_outputs
    check_outputs(
        &engine,
        graph,
        &spec.expected_outputs,
        &mut checks,
        &mut all_pass,
    );

    // Multi-step: run each step, check outputs after each one
    for step in &spec.steps {
        if let Some(minutes) = step.time {
            engine.set_time(minutes);
        }
        apply_inputs(&mut engine, &step.inputs);
        let step_dt = if step.dt > 0.0 { step.dt } else { spec.dt };
        for _ in 0..step.ticks {
            engine.tick(step_dt);
        }
        // Check this step's outputs immediately
        check_outputs(
            &engine,
            graph,
            &step.expected_outputs,
            &mut checks,
            &mut all_pass,
        );
    }

    ScenarioResult {
        name: spec.name.clone(),
        pass: all_pass,
        checks,
        trace: if spec.trace {
            collect_trace(&engine, graph)
        } else {
            Vec::new()
        },
    }
}

fn apply_inputs(engine: &mut SimEngine, inputs: &HashMap<String, f64>) {
    for (key, value) in inputs {
        if engine.set_input(key, *value) {
            continue;
        }
        let block_name = key.split('.').next().unwrap_or(key);
        if engine.set_input(block_name, *value) {
            continue;
        }
        let candidates = [
            format!("{block_name}.AQ"),
            format!("{block_name}.Q"),
            format!("{block_name}.InputTrigger"),
            format!("{block_name}.I1"),
            format!("{block_name}.Input"),
            format!("{block_name}.OutputPresence"),
        ];
        let mut found = false;
        for c in &candidates {
            if engine.set_input(c, *value) {
                found = true;
                break;
            }
        }
        if found {
            continue;
        }
        if engine.inject_output(key, *value) {
            continue;
        }
        if engine.inject_output(block_name, *value) {
            continue;
        }
        for c in &candidates {
            if engine.inject_output(c, *value) {
                found = true;
                break;
            }
        }
        if !found {
            eprintln!("warning: could not set input '{key}'");
        }
    }
}

fn check_outputs(
    engine: &SimEngine,
    graph: &SimGraph,
    expected: &HashMap<String, HashMap<String, f64>>,
    checks: &mut Vec<serde_json::Value>,
    all_pass: &mut bool,
) {
    for (output_key, comparators) in expected {
        let actual = resolve_output(engine, graph, output_key);

        for (op, exp) in comparators {
            let pass = check_comparator(actual, op, *exp);

            if !pass {
                *all_pass = false;
            }

            checks.push(serde_json::json!({
                "output": output_key,
                "actual": actual,
                "comparator": op,
                "expected": exp,
                "pass": pass,
            }));
        }
    }
}

pub fn cmd_sim(ctx: &super::RunContext, action: SimCmd) -> Result<()> {
    match action {
        SimCmd::Check { file } => cmd_check(&file),
        SimCmd::Run {
            file,
            sim,
            sim_file,
        } => cmd_run(&file, sim.as_deref(), sim_file.as_deref()),
        SimCmd::Step {
            file,
            sim,
            sim_file,
        } => cmd_step(&file, sim.as_deref(), sim_file.as_deref()),
        SimCmd::Dump { file, sim } => cmd_dump(&file, sim.as_deref(), ctx.json),
    }
}

fn cmd_check(file: &str) -> Result<()> {
    let graph = parse_graph(file)?;
    let output = serde_json::json!({
        "status": "ok",
        "blocks": graph.block_count(),
        "connectors": graph.connector_count(),
    });
    println!("{output}");
    Ok(())
}

fn cmd_run(file: &str, sim: Option<&str>, sim_file: Option<&str>) -> Result<()> {
    let graph = parse_graph(file)?;
    let sim_json = load_sim_json(sim, sim_file)?;

    let specs: Vec<SimSpec> = match serde_json::from_str::<SimSpec>(&sim_json) {
        Ok(single) => vec![single],
        Err(_) => serde_json::from_str(&sim_json).context("invalid sim JSON")?,
    };

    let mut results = Vec::new();
    let mut all_pass = true;

    for spec in &specs {
        let result = run_one(&graph, spec);
        if !result.pass {
            all_pass = false;
        }
        results.push(result);
    }

    // Build top-level trace dict from last scenario (for convenience)
    let trace_dict: serde_json::Map<String, serde_json::Value> = results
        .last()
        .map(|r| {
            r.trace
                .iter()
                .filter_map(|t| {
                    let obj = t.as_object()?;
                    let key = obj.get("output")?.as_str()?;
                    let val = obj.get("value")?.as_f64()?;
                    Some((key.to_string(), serde_json::json!(val)))
                })
                .collect()
        })
        .unwrap_or_default();

    let mut output = serde_json::json!({
        "pass": all_pass,
        "total": results.len(),
        "passed": results.iter().filter(|r| r.pass).count(),
        "scenarios": results.iter().map(|r| {
            let mut s = serde_json::json!({
                "name": r.name,
                "pass": r.pass,
                "checks": r.checks,
            });
            if !r.trace.is_empty() {
                s["trace"] = serde_json::json!(r.trace);
            }
            s
        }).collect::<Vec<_>>(),
    });
    if !trace_dict.is_empty() {
        output["trace"] = serde_json::Value::Object(trace_dict);
    }
    println!("{}", serde_json::to_string(&output).unwrap());

    if !all_pass {
        bail!(
            "simulation failed: {}/{} scenarios passed",
            results.iter().filter(|r| r.pass).count(),
            results.len()
        );
    }
    Ok(())
}

fn cmd_step(file: &str, sim: Option<&str>, sim_file: Option<&str>) -> Result<()> {
    let graph = parse_graph(file)?;
    let sim_json = load_sim_json(sim, sim_file)?;

    let spec: SimSpec = serde_json::from_str(&sim_json).context("invalid sim JSON")?;

    let mut engine = SimEngine::new(graph.clone());

    // Inject time into all DayTimer blocks if specified
    if let Some(minutes) = spec.time {
        engine.set_time(minutes);
    }

    for (key, value) in &spec.inputs {
        let block_name = key.split('.').next().unwrap_or(key);
        if !engine.set_input(key, *value) {
            let _ = engine.set_input(block_name, *value);
        }
    }

    println!("t=0.000  inputs: {:?}", spec.inputs);

    // Build watch list: blocks with wired inputs + expected outputs
    let mut watch: Vec<(String, usize, String)> = Vec::new();
    for bid in 0..graph.block_count() {
        let info = graph.block_info(bid);
        for &cid in &info.outputs {
            if graph.input_source_of(cid).is_some()
                || !info
                    .inputs
                    .iter()
                    .all(|&ic| graph.input_source_of(ic).is_none())
            {
                let key = graph.connector(cid).key.clone();
                let room = info.room.as_deref().unwrap_or("");
                let display = if room.is_empty() {
                    format!("{}.{}", info.name, key)
                } else {
                    format!("{} [{}].{}", info.name, room, key)
                };
                watch.push((display, bid, key));
            }
        }
    }
    for output_key in spec.expected_outputs.keys() {
        if !watch.iter().any(|(d, _, _)| d == output_key) {
            watch.push((output_key.clone(), usize::MAX, String::new()));
        }
    }

    let mut prev_values: Vec<f64> = watch
        .iter()
        .map(|(_, bid, key)| {
            if *bid < graph.block_count() {
                graph
                    .find_connector(*bid, key)
                    .map(|cid| engine.signal(cid))
                    .unwrap_or(0.0)
            } else {
                0.0
            }
        })
        .collect();

    for tick in 0..spec.ticks {
        engine.tick(spec.dt);
        let t = (tick + 1) as f64 * spec.dt;

        let mut changes = Vec::new();
        for (i, (display, bid, key)) in watch.iter().enumerate() {
            let val = if *bid < graph.block_count() {
                graph
                    .find_connector(*bid, key)
                    .map(|cid| {
                        graph
                            .input_source_of(cid)
                            .map(|src| engine.signal(src))
                            .unwrap_or(engine.signal(cid))
                    })
                    .unwrap_or(0.0)
            } else {
                0.0
            };

            if (val - prev_values[i]).abs() > 1e-9 {
                changes.push(format!("{display}={val:.2}"));
                prev_values[i] = val;
            }
        }

        if !changes.is_empty() {
            println!("t={t:.3}  {}", changes.join("  "));
        }
    }

    println!("\n--- final ---");
    for (output_key, comparators) in &spec.expected_outputs {
        let actual = resolve_output(&engine, &graph, output_key);
        for (op, expected) in comparators {
            let pass = check_comparator(actual, op, *expected);
            let icon = if pass { "✅" } else { "❌" };
            println!("{icon} {output_key} = {actual:.4}  {op} {expected}");
        }
    }
    Ok(())
}

fn cmd_dump(file: &str, sim: Option<&str>, json_output: bool) -> Result<()> {
    let graph = parse_graph(file)?;
    let mut engine = SimEngine::new(graph.clone());

    if let Some(sim_json) = sim
        && let Ok(spec) = serde_json::from_str::<SimSpec>(sim_json)
    {
        for (key, value) in &spec.inputs {
            let block_name = key.split('.').next().unwrap_or(key);
            if !engine.set_input(key, *value) {
                let _ = engine.set_input(block_name, *value);
            }
        }
        for _ in 0..spec.ticks {
            engine.tick(spec.dt);
        }
    }

    if json_output {
        return cmd_dump_json(&graph, &engine);
    }

    println!("=== BLOCKS ({}) ===", graph.block_count());
    for bid in 0..graph.block_count() {
        let info = graph.block_info(bid);
        let room = info.room.as_deref().unwrap_or("-");
        println!(
            "  [{bid:3}] {name:30} ({room:15}) ins={ni} outs={no}",
            name = info.name,
            ni = info.inputs.len(),
            no = info.outputs.len()
        );
        for &cid in &info.inputs {
            let c = graph.connector(cid);
            let src = graph.input_source_of(cid);
            let val = engine.signal(cid);
            let wire_info = if let Some(src_cid) = src {
                let sc = graph.connector(src_cid);
                let sb = graph.block_info(sc.block_id);
                format!("← {}.{} (={:.2})", sb.name, sc.key, engine.signal(src_cid))
            } else {
                String::from("(unwired)")
            };
            println!(
                "    IN  {key:20} cid={cid:3} val={val:8.2}  {wire_info}",
                key = c.key
            );
        }
        for &cid in &info.outputs {
            let c = graph.connector(cid);
            let val = engine.signal(cid);
            println!("    OUT {key:20} cid={cid:3} val={val:8.2}", key = c.key);
        }
    }

    println!("\n=== WIRES ===");
    for cid in 0..graph.connector_count() {
        if let Some(src) = graph.input_source_of(cid) {
            let dc = graph.connector(cid);
            let db = graph.block_info(dc.block_id);
            let sc = graph.connector(src);
            let sb = graph.block_info(sc.block_id);
            println!(
                "  {sn}.{sk} → {dn}.{dk}  (val={sv:.2} → {dv:.2})",
                sn = sb.name,
                sk = sc.key,
                dn = db.name,
                dk = dc.key,
                sv = engine.signal(src),
                dv = engine.signal(cid)
            );
        }
    }
    Ok(())
}

fn cmd_dump_json(graph: &SimGraph, _engine: &SimEngine) -> Result<()> {
    let mut blocks_json = Vec::new();
    let mut wires_json = Vec::new();
    let mut block_types: HashMap<String, Vec<usize>> = HashMap::new();

    for bid in 0..graph.block_count() {
        let info = graph.block_info(bid);
        block_types
            .entry(info.block_type.clone())
            .or_default()
            .push(bid);

        let inputs: Vec<_> = info
            .inputs
            .iter()
            .map(|&cid| {
                let c = graph.connector(cid);
                let wired_from = graph.input_source_of(cid);
                json!({
                    "cid": cid,
                    "key": c.key,
                    "wired_from": wired_from,
                })
            })
            .collect();

        // Build output wired_to by scanning all wires
        let outputs: Vec<_> = info
            .outputs
            .iter()
            .map(|&cid| {
                let c = graph.connector(cid);
                let wired_to: Vec<usize> = graph
                    .wires()
                    .iter()
                    .filter(|(from, _)| *from == cid)
                    .map(|(_, to)| *to)
                    .collect();
                json!({
                    "cid": cid,
                    "key": c.key,
                    "wired_to": wired_to,
                })
            })
            .collect();

        blocks_json.push(json!({
            "id": bid,
            "name": info.name,
            "type": info.block_type,
            "room": info.room,
            "inputs": inputs,
            "outputs": outputs,
        }));
    }

    for &(from_cid, to_cid) in graph.wires() {
        let sc = graph.connector(from_cid);
        let sb = graph.block_info(sc.block_id);
        let dc = graph.connector(to_cid);
        let db = graph.block_info(dc.block_id);
        wires_json.push(json!({
            "from_cid": from_cid,
            "to_cid": to_cid,
            "from_block": sb.name,
            "to_block": db.name,
        }));
    }

    let output = json!({
        "blocks": blocks_json,
        "wires": wires_json,
        "block_types": block_types,
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
