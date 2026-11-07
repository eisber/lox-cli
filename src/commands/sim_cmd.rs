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
        /// Exit non-zero if any block is unimplemented (unreliable) or a wire
        /// could not be honored.
        #[arg(long)]
        strict: bool,
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

// Unknown keys are rejected: a typo such as `expect` for `expected_outputs`
// would otherwise drop the checks and the scenario would pass.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
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
    /// Optional simulated wall-clock. Drives time/astro blocks (Time, Hour,
    /// Sunrise, NightTime, …) and DayTimer/AlarmClock time inputs, advancing
    /// by `dt` each tick.
    #[serde(default)]
    clock: Option<ClockSpec>,
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
#[serde(deny_unknown_fields)]
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
    clock: Option<ClockSpec>,
    #[serde(default)]
    expected_outputs: HashMap<String, HashMap<String, f64>>,
}

/// Simulated wall-clock specification (see [`SimSpec::clock`]).
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ClockSpec {
    /// Wall-clock time, `"HH:MM"` or `"HH:MM:SS"`.
    #[serde(default)]
    time: Option<String>,
    /// Calendar date, `"YYYY-MM-DD"`.
    #[serde(default)]
    date: Option<String>,
    /// Minutes since midnight (alternative to `time`).
    #[serde(default)]
    minutes_since_midnight: Option<f64>,
    /// Latitude for sunrise/sunset (defaults to Vienna).
    #[serde(default)]
    latitude: Option<f64>,
    /// Longitude for sunrise/sunset (defaults to Vienna).
    #[serde(default)]
    longitude: Option<f64>,
}

impl ClockSpec {
    fn build(&self) -> Result<lox_sim::clock::SimClock> {
        use lox_sim::clock::SimClock;
        let mut clock = if let Some(m) = self.minutes_since_midnight {
            let mut c = SimClock::parse(None, self.date.as_deref())
                .map_err(|e| anyhow::anyhow!("invalid clock: {e}"))?;
            c.advance(m * 60.0);
            c
        } else {
            SimClock::parse(self.time.as_deref(), self.date.as_deref())
                .map_err(|e| anyhow::anyhow!("invalid clock: {e}"))?
        };
        if let (Some(lat), Some(lon)) = (self.latitude, self.longitude) {
            clock.set_location(lat, lon);
        }
        Ok(clock)
    }
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
    // Suppress the raw, un-ranked per-type "Unknown block type" spam; we
    // present a categorized summary instead (see `sim_diagnostics`).
    lox_sim::blocks::set_block_warnings(false);
    parser::parse_file(&path).map_err(|e| anyhow::anyhow!("parse_failed: {e}"))
}

/// Categorized simulation-fidelity summary for a parsed graph.
struct SimDiagnostics {
    simulated: usize,
    structural: usize,
    /// Unimplemented block type -> count of instances.
    unimplemented: std::collections::BTreeMap<String, usize>,
    /// Non-fatal wire diagnostics from the parser (e.g. skipped wires).
    wire_warnings: Vec<String>,
}

impl SimDiagnostics {
    fn collect(graph: &SimGraph) -> Self {
        use lox_sim::blocks::{BlockSupport, block_support};
        let mut simulated = 0;
        let mut structural = 0;
        let mut unimplemented: std::collections::BTreeMap<String, usize> = Default::default();
        for bid in 0..graph.block_count() {
            let bt = &graph.block_info(bid).block_type;
            match block_support(bt) {
                BlockSupport::Simulated => simulated += 1,
                BlockSupport::Structural => structural += 1,
                BlockSupport::Unimplemented => {
                    *unimplemented.entry(bt.clone()).or_default() += 1;
                }
            }
        }
        SimDiagnostics {
            simulated,
            structural,
            unimplemented,
            wire_warnings: graph.warnings().to_vec(),
        }
    }

    fn unreliable_count(&self) -> usize {
        self.unimplemented.values().sum()
    }

    /// Print the human-readable summary to stderr. Returns `true` if the config
    /// is fully reliable (no unimplemented blocks and no skipped wires).
    fn report(&self, graph: &SimGraph) -> bool {
        for w in &self.wire_warnings {
            eprintln!("warning: {w}");
        }
        if !self.unimplemented.is_empty() {
            // Rank unimplemented types by impact (instance count, desc).
            let mut ranked: Vec<(&String, &usize)> = self.unimplemented.iter().collect();
            ranked.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
            eprintln!(
                "warning: {} block(s) across {} type(s) are not implemented \
                 (results unreliable; PicoC outputs are not evaluated, other unknown types use PassThrough):",
                self.unreliable_count(),
                self.unimplemented.len()
            );
            for (bt, count) in ranked.iter().take(15) {
                eprintln!("    {bt} ×{count}");
            }
            if ranked.len() > 15 {
                eprintln!("    … and {} more type(s)", ranked.len() - 15);
            }
        }
        eprintln!(
            "summary: {} blocks, {} simulated, {} unsupported (unreliable), {} structural",
            graph.block_count(),
            self.simulated,
            self.unreliable_count(),
            self.structural,
        );
        self.unimplemented.is_empty() && self.wire_warnings.is_empty()
    }
}

fn resolve_output(engine: &SimEngine, graph: &SimGraph, output_key: &str) -> f64 {
    let mut actual = engine.get_output(output_key);
    if actual == 0.0 && output_key.contains('.') {
        let parts: Vec<&str> = output_key.splitn(2, '.').collect();
        let block_name = parts[0];
        let conn_name = parts.get(1).unwrap_or(&"");

        // Extract room qualifier from "Name [Room]" syntax
        let (bare_name, expected_room) = if block_name.contains(" [") && block_name.ends_with(']') {
            let idx = block_name.rfind(" [").unwrap();
            (
                &block_name[..idx],
                Some(&block_name[idx + 2..block_name.len() - 1]),
            )
        } else {
            (block_name, None)
        };

        let mut name_matched = false;
        for bid in 0..graph.block_count() {
            let info = graph.block_info(bid);
            let name_matches = info.name == block_name
                || info.name == bare_name
                    && info
                        .room
                        .as_ref()
                        .is_some_and(|r| Some(r.as_str()) == expected_room)
                || info
                    .room
                    .as_ref()
                    .is_some_and(|r| format!("{} [{}]", info.name, r) == block_name);
            if !name_matches {
                continue;
            }
            name_matched = true;
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

        // Type+room fallback: if no block matched by name but we have a room,
        // search for ANY block in that room with a matching connector.
        // This handles cases where the agent names blocks differently from specs.
        if actual == 0.0
            && !name_matched
            && let Some(room) = expected_room
        {
            for bid in 0..graph.block_count() {
                let info = graph.block_info(bid);
                let in_room = info.room.as_ref().is_some_and(|r| r == room);
                if !in_room {
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

    // Install a simulated wall-clock if specified (drives time/astro blocks).
    if let Some(clock_spec) = &spec.clock {
        match clock_spec.build() {
            Ok(clock) => engine.set_clock(clock),
            Err(e) => eprintln!("warning: {e}"),
        }
    }
    // Inject time into all DayTimer blocks if specified
    if let Some(minutes) = spec.time {
        engine.set_time(minutes);
    }

    let unresolved = apply_inputs(&mut engine, &spec.inputs);
    fail_unresolved_inputs(&unresolved, &mut checks, &mut all_pass);

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
        if let Some(clock_spec) = &step.clock {
            match clock_spec.build() {
                Ok(clock) => engine.set_clock(clock),
                Err(e) => eprintln!("warning: {e}"),
            }
        }
        if let Some(minutes) = step.time {
            engine.set_time(minutes);
        }
        let unresolved = apply_inputs(&mut engine, &step.inputs);
        fail_unresolved_inputs(&unresolved, &mut checks, &mut all_pass);
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

    // A simulation with no output assertions has not verified any behavior.
    // This also catches trace-only scenarios and steps without expectations.
    if !checks.iter().any(|check| check.get("output").is_some()) {
        all_pass = false;
        checks.push(serde_json::json!({
            "pass": false,
            "error": "scenario has no output assertions",
        }));
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

/// An input that matches nothing leaves the circuit at its defaults, so the
/// checks would test something else than intended: fail the scenario.
fn fail_unresolved_inputs(
    keys: &[String],
    checks: &mut Vec<serde_json::Value>,
    all_pass: &mut bool,
) {
    for key in keys {
        *all_pass = false;
        checks.push(serde_json::json!({
            "input": key,
            "pass": false,
            "error": "could not set input",
        }));
    }
}

/// Inject `inputs`; returns the keys that matched no connector.
fn apply_inputs(engine: &mut SimEngine, inputs: &HashMap<String, f64>) -> Vec<String> {
    let mut unresolved = Vec::new();
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
            unresolved.push(key.clone());
        }
    }
    unresolved
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
        SimCmd::Check { file, strict } => cmd_check(&file, strict),
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

fn cmd_check(file: &str, strict: bool) -> Result<()> {
    let graph = parse_graph(file)?;
    let diag = SimDiagnostics::collect(&graph);
    let output = serde_json::json!({
        "status": "ok",
        "blocks": graph.block_count(),
        "connectors": graph.connector_count(),
        "simulated": diag.simulated,
        "unsupported": diag.unreliable_count(),
        "passthrough_unreliable": diag.unreliable_count(),
        "structural": diag.structural,
        "skipped_wires": diag.wire_warnings.len(),
    });
    println!("{output}");
    let reliable = diag.report(&graph);
    if strict && !reliable {
        bail!(
            "strict: config is not fully simulatable ({} unreliable block(s), {} skipped wire(s))",
            diag.unreliable_count(),
            diag.wire_warnings.len()
        );
    }
    Ok(())
}

fn parse_sim_specs(json: &str) -> Result<Vec<SimSpec>> {
    // Choose the JSON shape first: serde permits an empty sequence when
    // deserializing a struct with defaults, so "[]" could become one spec.
    let value: serde_json::Value = serde_json::from_str(json).context("invalid sim JSON")?;
    let specs: Vec<SimSpec> = if value.is_array() {
        serde_json::from_value(value).context("invalid sim JSON")?
    } else {
        vec![serde_json::from_value(value).context("invalid sim JSON")?]
    };
    if specs.is_empty() {
        bail!("simulation requires at least one scenario");
    }
    Ok(specs)
}

fn cmd_run(file: &str, sim: Option<&str>, sim_file: Option<&str>) -> Result<()> {
    let graph = parse_graph(file)?;
    SimDiagnostics::collect(&graph).report(&graph);
    let sim_json = load_sim_json(sim, sim_file)?;
    let specs = parse_sim_specs(&sim_json)?;

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
    SimDiagnostics::collect(&graph).report(&graph);
    let sim_json = load_sim_json(sim, sim_file)?;

    let spec: SimSpec = serde_json::from_str(&sim_json).context("invalid sim JSON")?;

    let mut engine = SimEngine::new(graph.clone());

    // Install a simulated wall-clock if specified.
    if let Some(clock_spec) = &spec.clock {
        match clock_spec.build() {
            Ok(clock) => engine.set_clock(clock),
            Err(e) => eprintln!("warning: {e}"),
        }
    }
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
    SimDiagnostics::collect(&graph).report(&graph);
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

#[cfg(test)]
mod false_pass_tests {
    use super::*;

    #[test]
    fn empty_scenario_list_is_rejected() {
        let error = parse_sim_specs("[]").err().expect("empty list must fail");
        assert!(error.to_string().contains("at least one scenario"));
    }

    #[test]
    fn scenario_with_no_output_assertions_fails() {
        let spec = parse_sim_specs("{}").expect("valid JSON").remove(0);
        let result = run_one(&SimGraph::new(), &spec);
        assert!(!result.pass);
        assert!(
            result
                .checks
                .iter()
                .any(|check| { check["error"] == "scenario has no output assertions" })
        );
    }

    #[test]
    fn trace_only_steps_without_assertions_still_fail() {
        let json = r#"{"trace":true,"steps":[{"inputs":{},"ticks":1}]}"#;
        let spec = parse_sim_specs(json).expect("valid JSON").remove(0);
        let result = run_one(&SimGraph::new(), &spec);
        assert!(!result.pass);
        assert!(
            result
                .checks
                .iter()
                .any(|check| { check["error"] == "scenario has no output assertions" })
        );
    }

    #[test]
    fn empty_comparators_are_not_output_assertions() {
        let spec = parse_sim_specs(r#"{"expected_outputs":{"Source.Q":{}}}"#)
            .unwrap()
            .remove(0);
        assert!(!run_one(&SimGraph::new(), &spec).pass);
    }

    #[test]
    fn assertions_in_steps_can_pass_without_initial_assertions() {
        let graph = lox_sim::parser::parse_bytes(
            br#"<ControlList><C Type="VirtualIn" U="s" Title="Source"><Co K="Q" U="q"/></C></ControlList>"#,
        )
        .unwrap();
        let specs = parse_sim_specs(
            r#"[{"trace":true,"steps":[{"inputs":{"Source":1},"expected_outputs":{"Source.Q":{"==":1}}}]}]"#,
        )
        .unwrap();
        assert_eq!(specs.len(), 1);
        let result = run_one(&graph, &specs[0]);
        assert!(result.pass);
        assert_eq!(result.checks.len(), 1);
        assert!(!result.trace.is_empty());
    }
}

#[cfg(test)]
mod strict_spec_tests {
    use super::*;

    #[test]
    fn picoc_is_unreliable_and_rejected_by_strict_check() {
        use std::io::Write;

        for block_type in ["Code1", "Code4", "Code8", "Code16"] {
            let xml = format!(
                r#"<ControlList><C Type="{block_type}" U="code-1" Title="Synthetic" Code="int x = 1;"/></ControlList>"#
            );
            let graph = parser::parse_bytes(xml.as_bytes()).unwrap();
            let diag = SimDiagnostics::collect(&graph);
            assert_eq!(diag.simulated, 0);
            assert_eq!(diag.unreliable_count(), 1);
            assert_eq!(diag.unimplemented.get(block_type), Some(&1));
            let mut file = tempfile::NamedTempFile::new_in(".").unwrap();
            file.write_all(xml.as_bytes()).unwrap();
            let path = file.path().to_str().unwrap();
            assert!(cmd_check(path, false).is_ok());
            let error = cmd_check(path, true).unwrap_err();
            assert!(error.to_string().contains("1 unreliable block(s)"));
        }
    }

    #[test]
    fn unknown_keys_are_rejected_at_every_spec_level() {
        for json in [
            r#"{"expect":{}}"#,
            r#"{"steps":[{"expect":{}}]}"#,
            r#"{"clock":{"timezone":"UTC"}}"#,
            r#"{"steps":[{"clock":{"timezone":"UTC"}}]}"#,
        ] {
            let error = serde_json::from_str::<SimSpec>(json).err().unwrap();
            assert!(error.to_string().contains("unknown field"), "{error}");
        }
    }

    #[test]
    fn unresolved_inputs_fail_initial_and_step_checks() {
        let graph = lox_sim::parser::parse_bytes(
            br#"<ControlList><C Type="VirtualIn" U="s" Title="Source"><Co K="Q" U="q"/></C></ControlList>"#,
        )
        .unwrap();
        for json in [
            r#"{"inputs":{"Missing":1},"expected_outputs":{"Source.Q":{"==":0}}}"#,
            r#"{"steps":[{"inputs":{"Missing":1},"expected_outputs":{"Source.Q":{"==":0}}}]}"#,
        ] {
            let spec: SimSpec = serde_json::from_str(json).unwrap();
            let result = run_one(&graph, &spec);
            assert!(!result.pass);
            assert!(result.checks.iter().any(|check| {
                check["input"] == "Missing"
                    && check["pass"] == false
                    && check["error"] == "could not set input"
            }));
            assert!(
                result
                    .checks
                    .iter()
                    .any(|check| check["output"] == "Source.Q" && check["pass"] == true)
            );
        }
    }
}
