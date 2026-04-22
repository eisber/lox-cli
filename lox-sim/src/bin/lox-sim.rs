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
        "dump" => {
            // Dump graph structure: blocks, connectors, wires
            let mut engine = SimEngine::new(graph.clone());

            // Apply inputs from --sim if provided
            if let Some(pos) = args.iter().position(|a| a == "--sim") {
                if let Some(sim_json) = args.get(pos + 1) {
                    if let Ok(spec) = serde_json::from_str::<SimSpec>(sim_json) {
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
                }
            }

            println!("=== BLOCKS ({}) ===", graph.block_count());
            for bid in 0..graph.block_count() {
                let info = graph.block_info(bid);
                let room = info.room.as_deref().unwrap_or("-");
                println!("  [{bid:3}] {name:30} ({room:15}) ins={ni} outs={no}",
                    name=info.name, ni=info.inputs.len(), no=info.outputs.len());
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
                    println!("    IN  {key:20} cid={cid:3} val={val:8.2}  {wire_info}", key=c.key);
                }
                for &cid in &info.outputs {
                    let c = graph.connector(cid);
                    let val = engine.signal(cid);
                    println!("    OUT {key:20} cid={cid:3} val={val:8.2}", key=c.key);
                }
            }

            println!("\n=== WIRES ===");
            for cid in 0..graph.connector_count() {
                if let Some(src) = graph.input_source_of(cid) {
                    let dc = graph.connector(cid);
                    let db = graph.block_info(dc.block_id);
                    let sc = graph.connector(src);
                    let sb = graph.block_info(sc.block_id);
                    println!("  {sn}.{sk} → {dn}.{dk}  (val={sv:.2} → {dv:.2})",
                        sn=sb.name, sk=sc.key, dn=db.name, dk=dc.key,
                        sv=engine.signal(src), dv=engine.signal(cid));
                }
            }
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
        let mut actual = engine.get_output(output_key);

        // If 0, try reading the input connector's wire source (engine reads
        // from source directly during eval, doesn't copy to input cid)
        if actual == 0.0 && output_key.contains('.') {
            let parts: Vec<&str> = output_key.splitn(2, '.').collect();
            let block_name = parts[0];
            let conn_name = parts.get(1).unwrap_or(&"");
            for bid in 0..graph.block_count() {
                let info = graph.block_info(bid);
                let name_matches = info.name == block_name
                    || info.room.as_ref().map_or(false, |r| {
                        format!("{} [{}]", info.name, r) == block_name
                    });
                if !name_matches {
                    continue;
                }
                if let Some(cid) = graph.find_connector(bid, conn_name) {
                    // Read from wire source if wired
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
