use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

use lox_sim::blocks::{create_block, Block};
use lox_sim::engine::SimEngine;
use lox_sim::graph::SimGraph;

const EPSILON: f64 = 1e-6;

#[derive(Debug, Deserialize)]
struct TestVectorFile {
    vectors: Vec<TestVector>,
}

#[derive(Debug, Deserialize)]
struct TestVector {
    id: String,
    description: String,
    #[serde(default)]
    dt: Option<f64>,
    blocks: Vec<BlockSpec>,
    #[serde(default)]
    wiring: Vec<WireSpec>,
    steps: Vec<StepSpec>,
}

#[derive(Debug, Deserialize)]
struct BlockSpec {
    #[serde(rename = "type")]
    block_type: String,
    title: String,
    #[serde(default)]
    connectors: Vec<String>,
    #[serde(default)]
    params: HashMap<String, f64>,
}

#[derive(Debug, Deserialize)]
struct WireSpec {
    from: String,
    to: String,
}

#[derive(Debug, Deserialize)]
struct StepSpec {
    inputs: Vec<f64>,
    ticks: usize,
    #[serde(default)]
    dt: Option<f64>,
    expected: HashMap<String, f64>,
}

#[derive(Clone, Copy)]
struct Schema {
    inputs: &'static [&'static str],
    outputs: &'static [&'static str],
    params: &'static [&'static str],
}

fn manifest_path(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn load_vectors() -> TestVectorFile {
    let path = manifest_path("tests/test_vectors.json");
    let data = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    serde_json::from_str(&data)
        .unwrap_or_else(|err| panic!("failed to parse {}: {err}", path.display()))
}

fn block_schema(block_type: &str) -> Option<Schema> {
    Some(match block_type {
        "And" => Schema {
            inputs: &["I1", "I2"],
            outputs: &["Q"],
            params: &[],
        },
        "Or" => Schema {
            inputs: &["I1", "I2"],
            outputs: &["Q"],
            params: &[],
        },
        "Not" => Schema {
            inputs: &["I"],
            outputs: &["Q"],
            params: &[],
        },
        "GreaterEqual" => Schema {
            inputs: &["Input1", "Input2"],
            outputs: &["Q"],
            params: &[],
        },
        "Less" => Schema {
            inputs: &["Input1", "Input2"],
            outputs: &["Q"],
            params: &[],
        },
        "AnalogThresholdTrigger" => Schema {
            inputs: &["Input"],
            outputs: &["Q", "RisingEdge", "FallingEdge"],
            params: &["On", "Off"],
        },
        "Add" => Schema {
            inputs: &["Input1", "Input2"],
            outputs: &["AQ"],
            params: &[],
        },
        "Sub" => Schema {
            inputs: &["Input1", "Input2"],
            outputs: &["AQ"],
            params: &[],
        },
        "Mult" => Schema {
            inputs: &["Input1", "Input2"],
            outputs: &["AQ"],
            params: &[],
        },
        "Div" => Schema {
            inputs: &["Input1", "Input2"],
            outputs: &["AQ"],
            params: &[],
        },
        "Monoflop" => Schema {
            inputs: &["InputTrigger"],
            outputs: &["Q"],
            params: &["Time"],
        },
        "OffDelay" => Schema {
            inputs: &["InputTrigger"],
            outputs: &["Q"],
            params: &["Time"],
        },
        "StairwayLS" => Schema {
            inputs: &["InputTrigger", "On"],
            outputs: &["Q"],
            params: &["TimeHigh", "TimeWarn", "WarnTime"],
        },
        "OnPulseDelay" => Schema {
            inputs: &["InputTrigger"],
            outputs: &["Q"],
            params: &["Delay", "Time"],
        },
        "EdgeDetection" => Schema {
            inputs: &["Input"],
            outputs: &["Edge", "RisingEdge", "FallingEdge"],
            params: &["PulseTime"],
        },
        "JalousieUpDown2" => Schema {
            inputs: &[
                "InputTriggerUp",
                "InputTriggerDown",
                "InputPos",
                "InputDisable",
            ],
            outputs: &["Pos", "Dir", "Moving"],
            params: &["TimeEnd"],
        },
        "HeatIRoomController2" => Schema {
            inputs: &["Temp", "Setpoint", "Reset", "InputDisable"],
            outputs: &["AQh"],
            params: &["Kp", "Ki"],
        },
        "Heatcurve" => Schema {
            inputs: &["OutdoorTemp", "Setpoint"],
            outputs: &["SupplyTemp"],
            params: &["BaseTemp", "Slope"],
        },
        "Ventilation2" => Schema {
            inputs: &["Speed"],
            outputs: &["Level", "SpeedNorm"],
            params: &["MaxLevels"],
        },
        "PassThrough" => Schema {
            inputs: &["Input"],
            outputs: &["Q"],
            params: &[],
        },
        _ => return None,
    })
}

fn validate_connectors(spec: &BlockSpec, schema: Schema) -> Result<(), String> {
    if spec.connectors.is_empty() {
        return Ok(());
    }

    let known: std::collections::HashSet<&str> = schema
        .inputs
        .iter()
        .chain(schema.outputs.iter())
        .chain(schema.params.iter())
        .copied()
        .collect();

    for connector in &spec.connectors {
        if !known.contains(connector.as_str()) {
            return Err(format!(
                "block '{}' ({}) declares unknown connector '{}'",
                spec.title, spec.block_type, connector
            ));
        }
    }
    Ok(())
}

fn create_block_impl(spec: &BlockSpec) -> Box<dyn Block> {
    create_block(&spec.block_type)
}

fn parse_input_index(reference: &str) -> Option<usize> {
    reference.strip_prefix("input:")?.parse().ok()
}

fn referenced_input_count(vector: &TestVector) -> usize {
    let mut max_index = 0usize;
    let mut saw_input = false;

    for wire in &vector.wiring {
        for endpoint in [&wire.from, &wire.to] {
            let head = endpoint.split('.').next().unwrap_or(endpoint);
            if let Some(index) = parse_input_index(head) {
                max_index = max_index.max(index);
                saw_input = true;
            }
        }
    }

    for step in &vector.steps {
        if !step.inputs.is_empty() {
            max_index = max_index.max(step.inputs.len().saturating_sub(1));
            saw_input = true;
        }
    }

    if saw_input {
        max_index + 1
    } else {
        0
    }
}

fn split_endpoint<'a>(raw: &'a str, default_key: &'static str) -> (&'a str, &'a str) {
    if let Some((left, right)) = raw.rsplit_once('.') {
        (left, right)
    } else {
        (raw, default_key)
    }
}

fn resolve_connector(
    graph: &SimGraph,
    block_ids: &HashMap<String, usize>,
    raw: &str,
    default_key: &'static str,
) -> Result<usize, String> {
    let (block_name, key) = split_endpoint(raw, default_key);
    let block_id = block_ids
        .get(block_name)
        .copied()
        .ok_or_else(|| format!("unknown block '{block_name}' in reference '{raw}'"))?;
    graph.find_connector(block_id, key).ok_or_else(|| {
        format!(
            "unknown connector '{}.{}' for block type {}",
            block_name,
            key,
            graph.block_info(block_id).name
        )
    })
}

fn build_graph(vector: &TestVector) -> Result<(SimGraph, usize), String> {
    let mut graph = SimGraph::new();
    let mut block_ids = HashMap::new();
    let input_count = referenced_input_count(vector);

    for index in 0..input_count {
        let title = format!("input:{index}");
        let bid = graph.add_block(
            title.clone(),
            create_block("VirtualIn"),
            &["Input"],
            &["Q"],
            &[],
        );
        block_ids.insert(title, bid);
    }

    for spec in &vector.blocks {
        let schema = block_schema(&spec.block_type)
            .ok_or_else(|| format!("unsupported test block type '{}'", spec.block_type))?;
        validate_connectors(spec, schema)?;
        let bid = graph.add_block(
            spec.title.clone(),
            create_block_impl(spec),
            schema.inputs,
            schema.outputs,
            schema.params,
        );
        for (param, value) in &spec.params {
            let cid = graph.find_connector(bid, param).ok_or_else(|| {
                format!(
                    "block '{}' has no parameter connector '{}'",
                    spec.title, param
                )
            })?;
            graph.set_connector_default(cid, *value).map_err(|err| {
                format!(
                    "failed to set default for '{}.{}': {err}",
                    spec.title, param
                )
            })?;
        }
        block_ids.insert(spec.title.clone(), bid);
    }

    for wire in &vector.wiring {
        let from = resolve_connector(&graph, &block_ids, &wire.from, "Q")?;
        let to = resolve_connector(&graph, &block_ids, &wire.to, "Input")?;
        graph
            .add_wire(from, to)
            .map_err(|err| format!("failed to wire '{}' -> '{}': {err}", wire.from, wire.to))?;
    }

    Ok((graph, input_count))
}

fn nearly_equal(left: f64, right: f64) -> bool {
    (left - right).abs() <= EPSILON
}

fn run_vector(vector: &TestVector) -> Result<(), String> {
    let (graph, input_count) = build_graph(vector)?;
    let mut engine = SimEngine::new(graph);
    let default_dt = vector.dt.unwrap_or(1.0);

    for (step_index, step) in vector.steps.iter().enumerate() {
        if step.inputs.len() != input_count {
            return Err(format!(
                "step {} expected {} inputs but got {}",
                step_index + 1,
                input_count,
                step.inputs.len()
            ));
        }

        let dt = step.dt.unwrap_or(default_dt);
        for _ in 0..step.ticks {
            for (input_index, value) in step.inputs.iter().enumerate() {
                let name = format!("input:{input_index}");
                if !engine.set_input(&name, *value) {
                    return Err(format!("failed to set {name} on step {}", step_index + 1));
                }
            }
            engine.mark_all_dirty();
            engine.tick(dt);
        }

        for (output, expected) in &step.expected {
            let actual = engine.get_output(output);
            if !nearly_equal(actual, *expected) {
                return Err(format!(
                    "step {} output '{}' expected {} but got {}",
                    step_index + 1,
                    output,
                    expected,
                    actual
                ));
            }
        }
    }

    Ok(())
}

#[test]
fn test_vector_runner() {
    let file = load_vectors();
    let mut failures = Vec::new();

    for vector in &file.vectors {
        match run_vector(vector) {
            Ok(()) => println!("PASS {} - {}", vector.id, vector.description),
            Err(err) => failures.push(format!(
                "FAIL {} - {}: {err}",
                vector.id, vector.description
            )),
        }
    }

    if !failures.is_empty() {
        panic!("{}", failures.join("\n"));
    }
}
