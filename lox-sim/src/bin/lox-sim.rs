//! lox-sim CLI — run simulation specs against Loxone config files.
//!
//! Usage:
//!   lox-sim run config.Loxone --sim '{"inputs":{"Temp.AQ":25},"ticks":10,"dt":0.1,"expected_outputs":{"Blind.Down":{">":0.5}}}'
//!   lox-sim run config.Loxone --sim-file sim-spec.json
//!   lox-sim check config.Loxone   # just parse and report block count

use std::path::PathBuf;

use lox_sim::{engine::SimEngine, parser};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: lox-sim <run|check> <config.Loxone> [--sim JSON | --sim-file FILE]");
        std::process::exit(2);
    }

    let command = &args[1];
    let config_path = PathBuf::from(&args[2]);

    let graph = match parser::parse_file(&config_path) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("{{\"error\":\"parse_failed\",\"detail\":\"{e}\"}}");
            std::process::exit(1);
        }
    };

    match command.as_str() {
        "check" => {
            let output = serde_json::json!({
                "status": "ok",
                "blocks": graph.block_count(),
                "connectors": graph.connector_count(),
            });
            println!("{output}");
        }
        "run" => {
            let sim_json = if let Some(pos) = args.iter().position(|a| a == "--sim") {
                args.get(pos + 1)
                    .expect("--sim requires a JSON argument")
                    .clone()
            } else if let Some(pos) = args.iter().position(|a| a == "--sim-file") {
                let path = args
                    .get(pos + 1)
                    .expect("--sim-file requires a path argument");
                std::fs::read_to_string(path).expect("failed to read sim-file")
            } else {
                // Read from stdin
                let mut buf = String::new();
                std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf)
                    .expect("failed to read stdin");
                buf
            };

            let specs: Vec<SimSpec> = match serde_json::from_str::<SimSpec>(&sim_json) {
                Ok(single) => vec![single],
                Err(_) => serde_json::from_str(&sim_json).unwrap_or_else(|e| {
                    eprintln!("{{\"error\":\"invalid_sim_json\",\"detail\":\"{e}\"}}");
                    std::process::exit(1);
                }),
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
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("Unknown command: {command}. Use 'run' or 'check'.");
            std::process::exit(2);
        }
    }
}

#[derive(serde::Deserialize)]
struct SimSpec {
    #[serde(default)]
    name: String,
    #[serde(default)]
    inputs: std::collections::HashMap<String, f64>,
    #[serde(default = "default_ticks")]
    ticks: usize,
    #[serde(default = "default_dt")]
    dt: f64,
    #[serde(default)]
    expected_outputs: std::collections::HashMap<String, std::collections::HashMap<String, f64>>,
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

fn run_one(graph: &lox_sim::graph::SimGraph, spec: &SimSpec) -> ScenarioResult {
    let mut engine = SimEngine::new(graph.clone());

    // Set inputs — try key as-is, then strip connector suffix, then add suffixes
    for (key, value) in &spec.inputs {
        if engine.set_input(key, *value) {
            continue;
        }
        // Strip ".AQ", ".Q", etc. and try just the block name
        let block_name = key.split('.').next().unwrap_or(key);
        if engine.set_input(block_name, *value) {
            continue;
        }
        // Try common suffixes on the block name
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
        if !found {
            eprintln!("warning: could not set input '{key}'");
        }
    }

    // Tick
    for _ in 0..spec.ticks {
        engine.tick(spec.dt);
    }

    // Check outputs
    let mut checks = Vec::new();
    let mut all_pass = true;

    for (output_key, comparators) in &spec.expected_outputs {
        // Try as output first, then as input (for checking signals arriving at actuators)
        let mut actual = engine.get_output(output_key);
        if actual == 0.0 {
            // Try as named input (actuator inputs like "Jalousie 1.InputTriggerDown")
            if let Some(cids) = engine.named_input_cids(output_key) {
                actual = cids.iter().map(|&cid| engine.signal(cid)).fold(0.0_f64, |a, b| if b.abs() > a.abs() { b } else { a });
            }
        }
        // Try stripping to block.connector and resolving manually  
        if actual == 0.0 && output_key.contains('.') {
            let parts: Vec<&str> = output_key.splitn(2, '.').collect();
            let block_name = parts[0];
            let conn_name = parts.get(1).unwrap_or(&"");
            for bid in 0..graph.block_count() {
                let info = graph.block_info(bid);
                if info.name == block_name {
                    if let Some(cid) = graph.find_connector(bid, conn_name) {
                        let sig = engine.signal(cid);
                        if sig.abs() > actual.abs() {
                            actual = sig;
                        }
                    }
                }
            }
        }

        for (op, expected) in comparators {
            let pass = match op.as_str() {
                ">" => actual > *expected,
                ">=" => actual >= *expected,
                "<" => actual < *expected,
                "<=" => actual <= *expected,
                "==" => (actual - expected).abs() < 1e-9,
                "~=" => (actual - expected).abs() < expected.abs() * 0.05 + 1e-9,
                _ => false,
            };

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
