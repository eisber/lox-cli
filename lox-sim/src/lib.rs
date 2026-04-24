//! # lox-sim — Loxone Miniserver SPS Simulator
//!
//! A differentiable, high-performance simulator for Loxone automation configs.
//!
//! ## Architecture
//!
//! ```text
//! .Loxone XML → Parser → SimGraph → Evaluator → Outputs
//!                           ↓
//!                     Block Library (195 types)
//!                           ↓
//!                     State Management
//!                           ↓
//!                     Autodiff Engine (optional)
//! ```
//!
//! ## Modules
//!
//! - `types` — Core signal types (f64 values, connector IDs)
//! - `graph` — Simulation graph: blocks, connectors, wiring, topological sort
//! - `blocks` — Block implementations (logic, math, timers, controllers)
//! - `engine` — Tick loop, evaluation, dirty propagation
//! - `clock` — Simulated time, sunrise/sunset, calendar
//! - `state` — Snapshot, restore, remanence
//! - `io` — Input injection, output reading, CSV time-series
//! - `autodiff` — Dual numbers, Wengert tape, smooth relaxations
//! - `trace` — Signal flow tracer (BFS reachability)
//! - `parser` — .Loxone XML → SimGraph loader

pub mod autodiff;
pub mod batch;
pub mod blocks;
pub mod clock;
pub mod compiler;
pub mod engine;
pub mod graph;
pub mod io;
pub mod parser;
pub mod profiler;
pub mod state;
pub mod trace;
pub mod types;

#[cfg(test)]
mod debug_smoke {
    use crate::engine::SimEngine;
    use crate::parser::parse_file;

    #[test]
    #[ignore] // requires external fixture file
    fn debug_smoke_alarm() {
        let path = "/tmp/lox-eval-full/uc-20592-acoustic-fire-and-water-alarm.Loxone";
        let graph = parse_file(path).unwrap();

        for bid in 0..graph.block_count() {
            let info = graph.block_info(bid);
            if info.name.contains("Brand")
                || info.name.contains("Rauch")
                || info.name.contains("Alarm")
            {
                eprintln!("Block {}: {}", bid, info.name);
                eprintln!("  Inputs ({}):", info.inputs.len());
                for &cid in &info.inputs {
                    let conn = graph.connector(cid);
                    let src = graph.input_source_of(cid);
                    eprintln!(
                        "    cid={}: key={}, dir={:?}, src={:?}",
                        cid, conn.key, conn.dir, src
                    );
                }
                eprintln!("  Outputs ({}):", info.outputs.len());
                for &cid in &info.outputs {
                    let conn = graph.connector(cid);
                    eprintln!("    cid={}: key={}, dir={:?}", cid, conn.key, conn.dir);
                }
            }
        }

        let mut engine = SimEngine::new(graph.clone());
        engine.set_input("Rauchmelder.AQ", 1.0);
        engine.tick(0.1);

        let out1 = engine.get_output("Brand- und Wasseralarm.OutAlarm1");
        let out_num = engine.get_output("Brand- und Wasseralarm.OutNumAlarms");
        eprintln!("OutAlarm1 = {}, OutNumAlarms = {}", out1, out_num);
        assert!(out1 > 0.5, "OutAlarm1 should be high, got {}", out1);
    }
}
