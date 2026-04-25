use std::fs;
use std::path::{Path, PathBuf};

use lox_sim::engine::SimEngine;
use lox_sim::graph::SimGraph;
use lox_sim::parser::parse_file;
use lox_sim::trace::{reachable_from, trace_signal};
use xmltree::{Element, XMLNode};

fn fixture_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("lox-sim has repo parent")
        .join("tests/eval/fixture.Loxone")
}

fn eval_case(name: &str) -> PathBuf {
    Path::new("/tmp/eval-llm").join(name)
}

/// Return true if the eval corpus directory exists (skips in CI)
fn eval_corpus_available() -> bool {
    Path::new("/tmp/eval-llm").exists()
}

macro_rules! skip_if_no_corpus {
    () => {
        if !eval_corpus_available() {
            eprintln!("skipping: /tmp/eval-llm not available");
            return;
        }
    };
}

fn parse_graph(path: &Path) -> SimGraph {
    parse_file(path).unwrap_or_else(|error| panic!("failed to parse {}: {error}", path.display()))
}

fn parse_engine(path: &Path) -> SimEngine {
    SimEngine::new(parse_graph(path))
}

fn count_places(path: &Path) -> usize {
    fn walk(elem: &Element, count: &mut usize) {
        if elem.name == "C" && elem.attributes.get("Type").map(String::as_str) == Some("Place") {
            *count += 1;
        }
        for child in &elem.children {
            if let XMLNode::Element(child) = child {
                walk(child, count);
            }
        }
    }

    let data = fs::read(path).expect("fixture readable");
    let root = Element::parse(data.as_slice()).expect("fixture XML parses");
    let mut count = 0;
    walk(&root, &mut count);
    count
}

fn block_ids_by_name(graph: &SimGraph, name: &str) -> Vec<usize> {
    (0..graph.block_count())
        .filter(|&block_id| graph.block_info(block_id).name == name)
        .collect()
}

fn incoming_signal(engine: &SimEngine, block_id: usize, key: &str) -> f64 {
    let cid = engine
        .graph()
        .find_connector(block_id, key)
        .unwrap_or_else(|| {
            panic!(
                "missing connector {key} on block {}",
                engine.graph().block_info(block_id).name
            )
        });
    let source = engine.graph().input_source_of(cid).unwrap_or(cid);
    engine.signal(source)
}

fn has_wired_input(graph: &SimGraph, block_id: usize, key: &str) -> bool {
    let cid = graph
        .find_connector(block_id, key)
        .unwrap_or_else(|| panic!("missing connector {key}"));
    graph.input_source_of(cid).is_some()
}

#[test]
fn test_parse_fixture() {
    let path = fixture_path();
    let graph = parse_graph(&path);

    assert_eq!(count_places(&path), 7);
    assert_eq!(graph.block_count(), 73);
    assert!(graph.connector_count() >= 119);
}

#[test]
fn test_piano_protection() {
    skip_if_no_corpus!();
    let path = eval_case("s01-piano-protection.Loxone");
    let mut engine = parse_engine(&path);

    assert!(engine.set_input("Außentemperatur", 25.0));
    assert!(engine.set_input("Sonnenschein", 1.0));
    for _ in 0..3 {
        engine.tick(0.1);
    }

    let graph = engine.graph();
    let jalousie = block_ids_by_name(graph, "Jalousie 1")
        .into_iter()
        .find(|&block_id| has_wired_input(graph, block_id, "InputTriggerDown"))
        .expect("wired Jalousie 1");

    assert!(incoming_signal(&engine, jalousie, "InputTriggerDown") > 0.5);
}

#[test]
fn test_presence_hallway() {
    skip_if_no_corpus!();
    let path = eval_case("s05-presence-hallway.Loxone");
    let mut engine = parse_engine(&path);

    assert!(engine.set_input("Bewegungsmelder.InputTrigger", 1.0));
    engine.tick(0.1);

    let graph = engine.graph();
    let hallway_light = block_ids_by_name(graph, "Lichtsteuerung")
        .into_iter()
        .find(|&block_id| has_wired_input(graph, block_id, "Presence"))
        .expect("hallway light controller");

    assert!(incoming_signal(&engine, hallway_light, "Presence") > 0.5);
}

#[test]
fn test_frost_protection() {
    skip_if_no_corpus!();
    let path = eval_case("s06-frost-blinds-up.Loxone");
    let mut engine = parse_engine(&path);

    assert!(engine.set_input("Außentemperatur", -2.0));
    for _ in 0..2 {
        engine.tick(0.1);
    }

    let graph = engine.graph();
    let jalousie = block_ids_by_name(graph, "Jalousie 1")
        .into_iter()
        .find(|&block_id| has_wired_input(graph, block_id, "InputTriggerUp"))
        .expect("frost-protected Jalousie 1");

    assert!(incoming_signal(&engine, jalousie, "InputTriggerUp") > 0.5);
}

#[test]
fn test_trace_temp_to_blind() {
    skip_if_no_corpus!();
    let path = eval_case("s01-piano-protection.Loxone");
    let graph = parse_graph(&path);
    let reachable = reachable_from(&graph, "Außentemperatur");
    let jalousie = block_ids_by_name(&graph, "Jalousie 1")
        .into_iter()
        .find(|&block_id| has_wired_input(&graph, block_id, "InputTriggerDown"))
        .expect("wired Jalousie 1");

    assert!(trace_signal(
        &graph,
        "Außentemperatur",
        "Temp über 20",
        "Input1"
    ));
    assert!(reachable.contains(&jalousie));
}

#[test]
fn test_nighttime_dimming() {
    skip_if_no_corpus!();
    let path = eval_case("s02-night-hallway-dim.Loxone");
    let mut engine = parse_engine(&path);

    // 2:00 AM falls within the DayTimer "Nachtdimmung" active range (0–360 min, value 30)
    assert!(engine.set_input("Nachtdimmung.minutes_since_midnight", 120.0));
    assert!(engine.set_input("Nachtdimmung.day_of_week", 0.0));
    engine.tick(0.1);

    let graph = engine.graph();
    let hallway_light = block_ids_by_name(graph, "Lichtsteuerung")
        .into_iter()
        .find(|&block_id| has_wired_input(graph, block_id, "Brightness"))
        .expect("hallway light brightness input");

    assert!((incoming_signal(&engine, hallway_light, "Brightness") - 30.0).abs() < 1e-6);
}

/// Test simulator trace against multiple eval configs
#[test]
fn test_trace_eval_configs() {
    skip_if_no_corpus!();
    use std::path::Path;

    // Cases with clear source → destination expectations
    let trace_cases = vec![
        (
            "k043-direct",
            "Außentemperatur",
            "Raumregler",
            Some("TempO"),
        ),
        (
            "k071-override",
            "Schalter 1",
            "Jalousie 1",
            Some("InputDisable"),
        ),
        (
            "d01-flur-colloquial-motion-light",
            "Bewegungsmelder",
            "Lichtsteuerung",
            Some("Presence"),
        ),
        (
            "k042-direct",
            "Bewegungsmelder Garten",
            "Gartenbeleuchtung",
            Some("Presence"),
        ),
    ];

    let eval_dir = Path::new("/tmp/eval-llm");
    let fixture_path = Path::new("../tests/eval/fixture.Loxone");
    let fixture_size = std::fs::metadata(fixture_path)
        .map(|m| m.len())
        .unwrap_or(0);

    let mut tested = 0;
    let mut passed = 0;

    for (case_id, src, dst, conn) in &trace_cases {
        let config_path = eval_dir.join(format!("{}.Loxone", case_id));
        if !config_path.exists() {
            continue;
        }

        // Skip unmodified
        let size = std::fs::metadata(&config_path)
            .map(|m| m.len())
            .unwrap_or(0);
        if size == fixture_size {
            continue;
        }

        tested += 1;

        // Parse and trace
        match lox_sim::parser::parse_file(config_path.to_str().unwrap()) {
            Ok(graph) => {
                let engine = lox_sim::engine::SimEngine::new(graph);
                let result = engine.trace(src, dst);
                if result.found {
                    passed += 1;
                } else {
                    eprintln!("TRACE FAIL: {} — {} → {}.{:?}", case_id, src, dst, conn);
                }
            }
            Err(e) => {
                eprintln!("PARSE FAIL: {} — {}", case_id, e);
            }
        }
    }

    eprintln!("Trace results: {}/{} passed", passed, tested);
    // At least some should pass
    assert!(passed > 0, "No trace tests passed");
    assert_eq!(
        passed, tested,
        "Not all trace tests passed ({passed}/{tested})"
    );
}

/// Test actual simulation — set inputs, tick, check outputs
#[test]
fn test_simulate_eval_configs() {
    skip_if_no_corpus!();
    let eval_dir = Path::new("/tmp/eval-llm");
    let fixture_path = Path::new("../tests/eval/fixture.Loxone");
    let fixture_size = std::fs::metadata(fixture_path)
        .map(|m| m.len())
        .unwrap_or(0);

    // Simulate configs that we can verify
    let sim_cases: Vec<(&str, Vec<(&str, f64)>, &str, f64, &str)> = vec![
        // (case_id, inputs, output_block, expected_threshold, description)
        // Temp below setpoint (21°C) → heating valve opens (Kp·error > 0)
        (
            "s16-temp-to-thermostat",
            vec![("Außentemperatur", 15.0), ("Raumregler.Setpoint", 21.0)],
            "Raumregler",
            20.0,
            "temp feeds thermostat",
        ),
        (
            "s05-presence-hallway",
            vec![("Bewegungsmelder.InputTrigger", 1.0)],
            "Lichtsteuerung.PresenceActive",
            0.5,
            "motion activates presence",
        ),
    ];

    let mut tested = 0;
    let mut passed = 0;

    for (case_id, inputs, output_block, threshold, desc) in &sim_cases {
        let config_path = eval_dir.join(format!("{}.Loxone", case_id));
        if !config_path.exists() {
            continue;
        }
        let size = std::fs::metadata(&config_path)
            .map(|m| m.len())
            .unwrap_or(0);
        if size == fixture_size {
            continue;
        }

        tested += 1;

        match lox_sim::parser::parse_file(config_path.to_str().unwrap()) {
            Ok(graph) => {
                let mut engine = SimEngine::new(graph);

                // Set inputs
                for (name, value) in inputs {
                    engine.set_input(name, *value);
                }

                // Tick multiple times to propagate
                for _ in 0..10 {
                    engine.tick(0.1);
                }

                // Check if any output on the target block is above threshold
                let output = engine.get_output(output_block);
                eprintln!(
                    "  SIM {}: {} = {} (threshold {}): {}",
                    case_id, output_block, output, threshold, desc
                );

                if output >= *threshold {
                    passed += 1;
                }
            }
            Err(e) => eprintln!("  PARSE FAIL: {} — {}", case_id, e),
        }
    }

    eprintln!("Simulation results: {}/{} passed", passed, tested);
    assert!(tested > 0, "No simulation tests ran");
    assert_eq!(
        passed, tested,
        "Not all simulation tests passed ({passed}/{tested})"
    );
}
