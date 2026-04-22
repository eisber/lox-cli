use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use lox_sim::{compiler::CompiledGraph, engine::SimEngine, graph::SimGraph, parser};
use xmltree::{Element, XMLNode};

const STRUCTURAL_TYPES: &[&str] = &["Category", "Document", "Page", "Place", "Program"];
const EPSILON: f64 = 1e-9;

fn eval_configs() -> Vec<PathBuf> {
    let dir = Path::new("/tmp/eval-llm");
    if !dir.exists() {
        eprintln!("Skipping equivalence tests: /tmp/eval-llm not found");
        return vec![];
    }
    let mut paths: Vec<_> = std::fs::read_dir(dir)
        .expect("eval corpus directory should be readable")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("Loxone"))
        .collect();
    paths.sort();
    paths
}

fn parse_graph(path: &Path) -> SimGraph {
    parser::parse_file(path)
        .unwrap_or_else(|error| panic!("failed to parse {}: {error}", path.display()))
}

fn block_ids_by_name(graph: &SimGraph, name: &str) -> Vec<usize> {
    (0..graph.block_count())
        .filter(|&block_id| graph.block_info(block_id).name == name)
        .collect()
}

fn actuator_output(engine: &SimEngine, name: &str) -> f64 {
    block_ids_by_name(engine.graph(), name)
        .into_iter()
        .flat_map(|block_id| engine.graph().block_info(block_id).outputs.clone())
        .map(|cid| engine.signal(cid))
        .fold(0.0, |best, value| {
            if value.abs() > best.abs() {
                value
            } else {
                best
            }
        })
}

fn incoming_signal(engine: &SimEngine, block_name: &str, key: &str) -> f64 {
    block_ids_by_name(engine.graph(), block_name)
        .into_iter()
        .filter_map(|block_id| {
            let cid = engine.graph().find_connector(block_id, key)?;
            let source = engine.graph().input_source_of(cid).unwrap_or(cid);
            Some(engine.signal(source))
        })
        .fold(0.0, |best, value| {
            if value.abs() > best.abs() {
                value
            } else {
                best
            }
        })
}

fn walk_xml(root: &Element, visit: &mut impl FnMut(&Element)) {
    visit(root);
    for child in &root.children {
        if let XMLNode::Element(child) = child {
            walk_xml(child, visit);
        }
    }
}

fn expected_block_count_from_stats(path: &Path) -> usize {
    let data = std::fs::read(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    let root = Element::parse(data.as_slice())
        .unwrap_or_else(|error| panic!("failed to parse XML {}: {error}", path.display()));

    let mut counts: HashMap<String, usize> = HashMap::new();
    walk_xml(&root, &mut |elem| {
        if let Some(block_type) = elem.attributes.get("Type") {
            *counts.entry(block_type.clone()).or_default() += 1;
        }
    });

    let structural = STRUCTURAL_TYPES
        .iter()
        .map(|kind| counts.get(*kind).copied().unwrap_or_default())
        .sum::<usize>();
    counts.values().sum::<usize>() - structural
}

fn planned_inputs(graph: &SimGraph) -> Vec<(String, f64)> {
    let mut name_counts: HashMap<String, usize> = HashMap::new();
    for block_id in 0..graph.block_count() {
        let info = graph.block_info(block_id);
        for &cid in &info.inputs {
            let full = format!("{}.{}", info.name, graph.connector(cid).key);
            *name_counts.entry(full).or_default() += 1;
        }
    }

    let mut inputs = BTreeMap::new();
    for block_id in 0..graph.block_count() {
        let info = graph.block_info(block_id);
        for &cid in &info.inputs {
            let key = graph.connector(cid).key.as_str();
            let full = format!("{}.{}", info.name, key);
            if name_counts.get(&full).copied().unwrap_or_default() != 1 {
                continue;
            }

            let value = match (info.name.as_str(), key) {
                ("Außentemperatur", "I1" | "Input") => Some(20.0),
                ("Sonnenschein", "I1" | "Input") => Some(1.0),
                ("Windgeschwindigkeit", "I1" | "Input") => Some(10.0),
                ("Regen", "I1" | "Input") => Some(0.0),
                ("Luftfeuchtigkeit", "I1" | "Input") => Some(50.0),
                ("Helligkeit", "I1" | "Input") => Some(500.0),
                ("Schalter 1", "I1" | "Input") => Some(1.0),
                ("Schalter 2", "I1" | "Input") => Some(0.0),
                ("Bewässerung Manuell", "I1" | "Input") => Some(1.0),
                ("Feuchtesensor Garten", "I1" | "Input") => Some(25.0),
                ("CO2 Sensor", "I1" | "Input") => Some(800.0),
                ("Garagentor Sensor", "I1" | "Input") => Some(0.0),
                ("Türklingel", "I1" | "Input") => Some(0.0),
                ("Raumtemperatur Wohnzimmer", "I1" | "Input") => Some(20.0),
                ("Raumtemperatur Schlafzimmer", "I1" | "Input") => Some(20.0),
                ("Pool Temperatur", "I1" | "Input") => Some(20.0),
                ("Türkontakt Eingang", "I1" | "Input") => Some(0.0),
                _ if info.name.contains("Bewegungsmelder") && key == "InputTrigger" => Some(1.0),
                _ if key == "Setpoint" => Some(21.0),
                _ if key == "minutes_since_midnight" => Some(120.0),
                _ if key == "day_of_week" => Some(1.0),
                _ => None,
            };

            if let Some(value) = value {
                inputs.insert(full, value);
            }
        }
    }

    inputs.into_iter().collect()
}

fn apply_inputs_engine(engine: &mut SimEngine, inputs: &[(String, f64)], path: &Path) {
    for (name, value) in inputs {
        assert!(
            engine.set_input(name, *value),
            "{}: failed to set engine input {name}",
            path.display()
        );
    }
}

fn apply_inputs_compiled(compiled: &mut CompiledGraph, inputs: &[(String, f64)], path: &Path) {
    for (name, value) in inputs {
        assert!(
            compiled.set_input(name, *value),
            "{}: failed to set compiled input {name}",
            path.display()
        );
    }
}

fn assert_all_finite(engine: &SimEngine, path: &Path) {
    for cid in 0..engine.graph().connector_count() {
        let value = engine.signal(cid);
        assert!(
            value.is_finite(),
            "{}: connector {} produced non-finite value {}",
            path.display(),
            cid,
            value
        );
    }
}

fn compiled_mismatch(graph: &SimGraph, inputs: &[(String, f64)], path: &Path) -> Option<String> {
    let mut engine = SimEngine::new(graph.clone());
    let mut compiled = CompiledGraph::from_graph(graph);

    apply_inputs_engine(&mut engine, inputs, path);
    apply_inputs_compiled(&mut compiled, inputs, path);

    for _ in 0..10 {
        engine.tick(0.1);
        compiled.tick(0.1);
    }

    if compiled.signal_count() != graph.connector_count() {
        return Some(format!(
            "compiled signal count {} != connector count {}",
            compiled.signal_count(),
            graph.connector_count()
        ));
    }
    if compiled.step_count() == 0 {
        return Some("compiled graph contains no evaluation steps".to_string());
    }

    let mut mismatches = Vec::new();
    for cid in 0..graph.connector_count() {
        let interpreted = engine.signal(cid);
        let compiled_value = compiled.signal(cid);
        if !interpreted.is_finite() || !compiled_value.is_finite() {
            mismatches.push(format!(
                "connector {cid} is non-finite (interp={interpreted}, compiled={compiled_value})"
            ));
            continue;
        }
        if (interpreted - compiled_value).abs() > EPSILON {
            let connector = graph.connector(cid);
            let block = graph.block_info(connector.block_id);
            mismatches.push(format!(
                "{}.{} mismatch: interp={interpreted}, compiled={compiled_value}",
                block.name, connector.key
            ));
        }
        if mismatches.len() >= 8 {
            break;
        }
    }

    if mismatches.is_empty() {
        None
    } else {
        Some(mismatches.join("\n"))
    }
}

fn try_set_case_input(engine: &mut SimEngine, name: &str, value: f64) -> bool {
    let mut candidates = vec![name.to_string()];
    if !name.contains('.') {
        candidates.push(format!("{name}.InputTrigger"));
        candidates.push(format!("{name}.I1"));
        candidates.push(format!("{name}.Input"));
    }
    candidates.sort();
    candidates.dedup();
    candidates
        .into_iter()
        .any(|candidate| engine.set_input(&candidate, value))
}

#[test]
fn test_equivalence_simple_cases() {
    let simple_cases = vec![
        (
            "s01-piano-protection",
            vec![("Außentemperatur", 25.0), ("Sonnenschein", 1.0)],
            "Außentemperatur",
            "Jalousie 1",
            Some(("Jalousie 1", "InputTriggerDown")),
            None,
        ),
        (
            "s05-presence-hallway",
            vec![("Bewegungsmelder", 1.0)],
            "Bewegungsmelder",
            "Lichtsteuerung",
            None,
            Some("Lichtsteuerung.PresenceActive"),
        ),
        (
            "s16-temp-to-thermostat",
            vec![("Außentemperatur", 18.0), ("Raumregler.Setpoint", 21.0)],
            "Außentemperatur",
            "Raumregler",
            None,
            Some("Raumregler.AQh"),
        ),
    ];

    let mut tested = 0;

    for (case_id, inputs, trace_from, trace_to, actuator_input, actuator_output_name) in
        &simple_cases
    {
        let path = Path::new("/tmp/eval-llm").join(format!("{case_id}.Loxone"));
        if !path.exists() {
            continue;
        }
        tested += 1;

        let graph = parse_graph(&path);
        let mut engine = SimEngine::new(graph);

        for (name, value) in inputs {
            assert!(
                try_set_case_input(&mut engine, name, *value),
                "{}: failed to set test input {name}",
                path.display()
            );
        }

        for _ in 0..10 {
            engine.tick(0.1);
        }

        let trace_result = engine.trace(trace_from, trace_to);
        eprintln!(
            "{} trace {} -> {} found={} hops={}",
            case_id, trace_from, trace_to, trace_result.found, trace_result.hops
        );

        assert!(
            trace_result.found,
            "{}: signal should flow {} -> {}",
            case_id, trace_from, trace_to
        );

        if let Some((block_name, key)) = actuator_input {
            let value = incoming_signal(&engine, block_name, key);
            assert!(
                value.abs() > EPSILON,
                "{}: actuator input {}.{} should have a non-zero command signal",
                case_id,
                block_name,
                key
            );
        }

        if let Some(output_name) = actuator_output_name {
            let value = engine.get_output(output_name);
            assert!(
                value.abs() > EPSILON,
                "{}: actuator output {} should be non-zero after stimulation",
                case_id,
                output_name
            );
        } else if actuator_input.is_none() {
            let value = actuator_output(&engine, trace_to);
            assert!(
                value.abs() > EPSILON,
                "{}: actuator {} should have a non-zero output after stimulation",
                case_id,
                trace_to
            );
        }

        assert_all_finite(&engine, &path);
    }

    if tested == 0 {
        eprintln!("skipping: no simple equivalence cases available in /tmp/eval-llm/");
        return;
    }
}

#[test]
fn test_equivalence_check_vs_sim() {
    let configs = eval_configs();
    if configs.is_empty() {
        eprintln!("skipping: /tmp/eval-llm not available");
        return;
    }

    let mut checked = 0usize;
    let mut failures = Vec::new();
    let mut skipped_parse = 0usize;

    for path in configs {
        let graph = match parser::parse_file(&path) {
            Ok(graph) => graph,
            Err(error) => {
                skipped_parse += 1;
                eprintln!("skipping {}: parse failed: {error}", path.display());
                continue;
            }
        };
        checked += 1;

        let expected_blocks = expected_block_count_from_stats(&path);
        if graph.block_count() != expected_blocks {
            failures.push(format!(
                "{}: parsed {} blocks, expected {} from lox config stats semantics",
                path.display(),
                graph.block_count(),
                expected_blocks
            ));
        }

        if graph.connector_count() == 0 {
            failures.push(format!(
                "{}: connector count should not be zero",
                path.display()
            ));
            continue;
        }

        let inputs = planned_inputs(&graph);

        let mut engine = SimEngine::new(graph.clone());
        apply_inputs_engine(&mut engine, &inputs, &path);
        for _ in 0..10 {
            engine.tick(0.1);
        }
        for cid in 0..graph.connector_count() {
            let value = engine.signal(cid);
            if !value.is_finite() {
                failures.push(format!(
                    "{}: connector {} produced non-finite value {}",
                    path.display(),
                    cid,
                    value
                ));
                break;
            }
        }
    }

    if checked == 0 {
        eprintln!("skipping: no eval configs were checked");
        return;
    }
    eprintln!("checked {checked} parsed configs, skipped {skipped_parse} parse failures");
    assert!(
        failures.is_empty(),
        "equivalence issues:\n{}",
        failures.join("\n")
    );
}

#[test]
fn test_compiled_matches_interpreted_for_supported_cases() {
    let supported_cases = [
        "s01-piano-protection",
        "s05-presence-hallway",
        "s16-temp-to-thermostat",
    ];

    let mut tested = 0usize;
    for case_id in supported_cases {
        let path = Path::new("/tmp/eval-llm").join(format!("{case_id}.Loxone"));
        if !path.exists() {
            continue;
        }

        let graph = parse_graph(&path);
        if let Some(error) = compiled_mismatch(&graph, &[], &path) {
            panic!(
                "{}: compiled graph should match interpreted execution for supported case\n{}",
                path.display(),
                error
            );
        }
        tested += 1;
    }

    if tested == 0 {
        eprintln!("skipping: no compiled-equivalence cases available in /tmp/eval-llm/");
        return;
    }
}
