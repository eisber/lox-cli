use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::Subcommand;
use lox_sim::{engine::SimEngine, graph::SimGraph, parser};
use serde::Deserialize;

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

    for (key, value) in &spec.inputs {
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
        // Fallback: inject into output connector (for non-source blocks)
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

    for _ in 0..spec.ticks {
        engine.tick(spec.dt);
    }

    let mut checks = Vec::new();
    let mut all_pass = true;

    for (output_key, comparators) in &spec.expected_outputs {
        let actual = resolve_output(&engine, graph, output_key);

        for (op, expected) in comparators {
            let pass = check_comparator(actual, op, *expected);

            if !pass {
                all_pass = false;
            }

            checks.push(serde_json::json!({
                "output": output_key,
                "actual": actual,
                "comparator": op,
                "expected": expected,
                "pass": pass,
            }));
        }
    }

    ScenarioResult {
        name: spec.name.clone(),
        pass: all_pass,
        checks,
    }
}

pub fn cmd_sim(action: SimCmd) -> Result<()> {
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
        SimCmd::Dump { file, sim } => cmd_dump(&file, sim.as_deref()),
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

    let output = serde_json::json!({
        "pass": all_pass,
        "total": results.len(),
        "passed": results.iter().filter(|r| r.pass).count(),
        "scenarios": results.iter().map(|r| {
            serde_json::json!({
                "name": r.name,
                "pass": r.pass,
                "checks": r.checks,
            })
        }).collect::<Vec<_>>(),
    });
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

fn cmd_dump(file: &str, sim: Option<&str>) -> Result<()> {
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
