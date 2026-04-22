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
