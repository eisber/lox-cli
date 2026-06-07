//! Regression tests against a sanitized multi-room Gen-2 export fixture.
//!
//! Covers the gaps reported in REQUIREMENTS-sim-config-gaps.md:
//!   * P0-1 — parsing must not hard-fail on an input used as a wire source.
//!   * P0-2 — a simulated clock drives time/astro blocks so schedules are testable.
//!   * P0-3 — Time -> GreaterEqual/Less -> And -> Mult night-gating works.

use std::path::{Path, PathBuf};

use lox_sim::clock::SimClock;
use lox_sim::engine::SimEngine;
use lox_sim::parser::parse_file;

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/gen2_nightlight.Loxone")
}

#[test]
fn parses_without_hard_fail_and_warns_on_bad_wire() {
    let graph = parse_file(fixture()).expect("real Gen-2 export must parse");
    assert!(graph.block_count() > 5);
    assert!(
        graph.warnings().iter().any(|w| w.contains("skipped wire")),
        "expected a graceful skipped-wire warning, got {:?}",
        graph.warnings()
    );
}

fn brightness_at(time: &str) -> f64 {
    let graph = parse_file(fixture()).expect("parse");
    let mut engine = SimEngine::new(graph);
    engine.set_clock(SimClock::parse(Some(time), Some("2026-06-07")).expect("clock"));
    for _ in 0..5 {
        engine.tick(0.1);
    }
    engine.get_output("Brightness")
}

#[test]
fn night_gating_schedule_is_simulatable() {
    // Daytime (12:00): after 05:00 AND before 22:00 -> 1 * 30 = 30.
    assert!(
        (brightness_at("12:00") - 30.0).abs() < 1e-6,
        "expected 30 at noon"
    );
    // Night (23:00): not before 22:00 -> 0 * 30 = 0.
    assert!(brightness_at("23:00").abs() < 1e-6, "expected 0 at 23:00");
    // Early (04:00): not after 05:00 -> 0.
    assert!(brightness_at("04:00").abs() < 1e-6, "expected 0 at 04:00");
}

#[test]
fn time_block_outputs_minutes_since_midnight() {
    let graph = parse_file(fixture()).expect("parse");
    let mut engine = SimEngine::new(graph);
    engine.set_clock(SimClock::parse(Some("10:30"), Some("2026-06-07")).expect("clock"));
    engine.tick(0.1);
    let aq = engine.get_output("Minuten seit Mitternacht");
    assert!((aq - 630.0).abs() < 1.0, "expected ~630 min, got {aq}");
}
