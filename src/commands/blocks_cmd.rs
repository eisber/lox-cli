use anyhow::{Result, bail};
use std::collections::HashMap;

use crate::BlocksCmd;
use crate::commands::RunContext;

// ── Keyword alias table ──────────────────────────────────────────────────────
// Maps natural-language phrases → block xml_types for intent-based search.

const ALIASES: &[(&str, &[&str])] = &[
    // Timer patterns
    ("timed light", &["StairwayLS"]),
    ("light timer", &["StairwayLS"]),
    ("stairway", &["StairwayLS"]),
    ("delayed pulse", &["OnPulseDelay"]),
    ("delay before", &["OnPulseDelay"]),
    ("off delay", &["OffDelay"]),
    ("stay on after", &["OffDelay"]),
    ("monoflop", &["Monoflop"]),
    ("pulse", &["Monoflop", "PulseGen", "PulseAt"]),
    ("schedule", &["DayTimer", "AlarmClock", "PulseAt"]),
    // Logic
    ("and", &["And"]),
    ("or", &["Or"]),
    ("not", &["Not"]),
    ("xor", &["Xor"]),
    ("flip flop", &["FlipFlop", "RSFlipFlop", "SRFlipFlop"]),
    ("memory", &["AMemory"]),
    ("latch", &["AMemory", "FlipFlop"]),
    ("counter", &["Counter", "UpDownCounter"]),
    // Comparison
    (
        "threshold",
        &["GreaterEqual", "Less", "AnalogThresholdTrigger"],
    ),
    ("greater", &["GreaterEqual", "Greater"]),
    ("less", &["Less", "LessEqual"]),
    ("compare", &["GreaterEqual", "Less", "Equal", "NotEqual"]),
    ("equal", &["Equal", "NotEqual"]),
    // Math
    ("multiply", &["Mult"]),
    ("add", &["Add"]),
    ("subtract", &["Sub"]),
    ("divide", &["Div"]),
    ("average", &["Average", "Avg"]),
    ("formula", &["Formula"]),
    ("scale", &["AnalogScaler", "Mult"]),
    // Controls
    ("blind", &["JalousieUpDown2", "AutoJalousie"]),
    ("jalousie", &["JalousieUpDown2"]),
    (
        "shading",
        &["JalousieUpDown2", "AutoJalousie", "DaylightController"],
    ),
    ("light", &["LightController2", "StairwayLS"]),
    ("dimmer", &["LightController2", "Mult"]),
    ("heating", &["HeatIRoomController2", "Heatmixer2"]),
    ("cooling", &["HeatIRoomController2", "AcControl"]),
    ("ventilation", &["Ventilation", "ToiletFan"]),
    ("fan", &["Ventilation", "ToiletFan", "Fan"]),
    ("irrigation", &["Irrigation", "DayTimer"]),
    ("alarm", &["Alarm", "AlarmClock"]),
    ("presence", &["Presence", "PresenceController"]),
    ("motion", &["PresenceDetector", "StairwayLS"]),
    ("door", &["Door", "Doorcontroller"]),
    ("energy", &["EnergyManager2", "SpotOpt"]),
    ("meter", &["MeterAbsUni", "MeterDig"]),
    ("wallbox", &["Wallbox"]),
    ("music", &["MusicPlayer"]),
    ("button", &["PushButton", "PushButton2"]),
    ("switch", &["PushButton", "State"]),
    ("state", &["State", "StateV"]),
    ("status", &["StatusMonitor"]),
    // Intent phrases
    ("turn on for", &["StairwayLS"]),
    ("turn on when", &["And", "GreaterEqual"]),
    ("close when", &["GreaterEqual", "Less"]),
    ("open when", &["Less", "GreaterEqual"]),
    ("protect", &["GreaterEqual", "Less", "And"]),
    ("frost", &["Less"]),
    ("wind", &["GreaterEqual"]),
    ("rain", &["GreaterEqual", "Not"]),
    (
        "temperature",
        &["GreaterEqual", "Less", "HeatIRoomController2"],
    ),
    ("humidity", &["GreaterEqual", "Less", "DewPoint"]),
    ("co2", &["GreaterEqual", "StatusMonitor"]),
    ("night", &["DayTimer", "Less"]),
    ("morning", &["DayTimer", "AlarmClock", "PulseAt"]),
    // German aliases
    ("treppenlicht", &["StairwayLS"]),
    ("zeitschalter", &["StairwayLS", "Monoflop"]),
    ("licht timer", &["StairwayLS"]),
    ("einschalten für", &["StairwayLS"]),
    ("verzögerung", &["OnPulseDelay", "OffDelay", "OnDelay"]),
    ("ausschaltverzögerung", &["OffDelay"]),
    ("einschaltverzögerung", &["OnDelay"]),
    ("impuls", &["Monoflop", "PulseGen", "PulseAt"]),
    ("schaltuhr", &["DayTimer"]),
    ("zeitplan", &["DayTimer", "AlarmClock"]),
    ("wecker", &["AlarmClock"]),
    ("und", &["And"]),
    ("oder", &["Or"]),
    ("nicht", &["Not"]),
    ("speicher", &["AMemory"]),
    ("merker", &["AMemory", "FlipFlop"]),
    ("zähler", &["Counter", "UpDownCounter"]),
    ("schwellwert", &["GreaterEqual", "Less", "AnalogThresholdTrigger"]),
    ("größer", &["GreaterEqual", "Greater"]),
    ("kleiner", &["Less", "LessEqual"]),
    ("vergleich", &["GreaterEqual", "Less", "Equal"]),
    ("multiplikation", &["Mult"]),
    ("addition", &["Add"]),
    ("subtraktion", &["Sub"]),
    ("division", &["Div"]),
    ("mittelwert", &["Average", "Avg"]),
    ("formel", &["Formula"]),
    ("beschattung", &["JalousieUpDown2", "AutoJalousie"]),
    ("rolladen", &["JalousieUpDown2"]),
    ("rollo", &["JalousieUpDown2"]),
    ("markise", &["JalousieUpDown2", "AutoJalousie"]),
    ("beleuchtung", &["LightController2", "StairwayLS"]),
    ("dimmen", &["LightController2", "Mult"]),
    ("heizung", &["HeatIRoomController2", "Heatmixer2"]),
    ("kühlung", &["HeatIRoomController2", "AcControl"]),
    ("klimaanlage", &["AcControl", "HVACController"]),
    ("lüftung", &["Ventilation", "ToiletFan"]),
    ("bewässerung", &["Irrigation", "DayTimer"]),
    ("bewegung", &["PresenceDetector", "StairwayLS"]),
    ("präsenz", &["Presence", "PresenceController"]),
    ("tür", &["Door", "Doorcontroller"]),
    ("türklingel", &["PushButton", "Monoflop"]),
    ("energie", &["EnergyManager2", "SpotOpt"]),
    ("taster", &["PushButton", "PushButton2"]),
    ("schalter", &["PushButton", "State"]),
    ("sonnenschutz", &["GreaterEqual", "AutoJalousie"]),
    ("frostschutz", &["Less"]),
    ("windschutz", &["GreaterEqual"]),
    ("regenschutz", &["GreaterEqual", "Not"]),
    ("temperatur", &["GreaterEqual", "Less", "HeatIRoomController2"]),
    ("feuchtigkeit", &["GreaterEqual", "Less", "DewPoint"]),
    ("taupunkt", &["DewPoint"]),
    ("nacht", &["DayTimer", "Less"]),
    ("morgen", &["DayTimer", "AlarmClock", "PulseAt"]),
    ("garagentor", &["StairwayLS", "Monoflop"]),
    ("poolpumpe", &["StairwayLS", "DayTimer"]),
];

// ── Category table ───────────────────────────────────────────────────────────

const CATEGORIES: &[(&str, &[&str])] = &[
    (
        "logic",
        &[
            "And",
            "Or",
            "Not",
            "Xor",
            "FlipFlop",
            "RSFlipFlop",
            "SRFlipFlop",
        ],
    ),
    (
        "math",
        &[
            "Add", "Add4", "Sub", "Mult", "Div", "Mod", "Formula", "Average", "Avg", "Power",
        ],
    ),
    (
        "compare",
        &[
            "GreaterEqual",
            "Greater",
            "Less",
            "LessEqual",
            "Equal",
            "NotEqual",
            "AnalogThresholdTrigger",
            "AnalogComparator",
        ],
    ),
    (
        "timer",
        &[
            "StairwayLS",
            "Monoflop",
            "OnPulseDelay",
            "OffDelay",
            "OnOffDelay",
            "OnDelay",
            "PulseGen",
            "EdgeDetection",
        ],
    ),
    (
        "schedule",
        &["DayTimer", "AlarmClock", "PulseAt", "PulseBy", "Calendar"],
    ),
    (
        "state",
        &[
            "AMemory",
            "State",
            "StateV",
            "Counter",
            "UpDownCounter",
            "AnalogMultiplexer",
            "AnalogMultiplexer2",
        ],
    ),
    (
        "lighting",
        &[
            "LightController2",
            "LightControllerH",
            "BrightnessControl",
            "CentralLight",
        ],
    ),
    (
        "shading",
        &[
            "JalousieUpDown2",
            "AutoJalousie",
            "DaylightController",
            "CentralShade",
        ],
    ),
    (
        "hvac",
        &[
            "HeatIRoomController2",
            "Heatmixer2",
            "HVACController",
            "Ventilation",
            "Fan",
            "Fancoil",
            "DewPoint",
            "AcControl",
            "ToiletFan",
        ],
    ),
    (
        "security",
        &[
            "Alarm",
            "AlarmChain",
            "CentralAlarm",
            "SmokeAlarm",
            "Door",
            "Doorcontroller",
            "NfcCodeTouch",
        ],
    ),
    (
        "energy",
        &[
            "EnergyManager2",
            "SpotOpt",
            "LoadShed",
            "Wallbox",
            "MeterAbsUni",
            "MeterDig",
            "EFM",
            "Fronius",
        ],
    ),
    (
        "io",
        &[
            "InputRef",
            "OutputRef",
            "EIBsensor",
            "EIBactor",
            "VirtualIn",
        ],
    ),
    (
        "button",
        &["PushButton", "PushButton2", "LongClick", "MultiClick"],
    ),
    (
        "misc",
        &[
            "StatusMonitor",
            "Presence",
            "PresenceController",
            "Irrigation",
            "PoolController",
            "MusicPlayer",
        ],
    ),
];

// ── Confusion pairs ──────────────────────────────────────────────────────────

const CONFUSIONS: &[(&str, &str, &str)] = &[
    (
        "StairwayLS",
        "OnPulseDelay",
        "StairwayLS turns on IMMEDIATELY for a duration. OnPulseDelay WAITS first, then pulses briefly.",
    ),
    (
        "AMemory",
        "FlipFlop",
        "AMemory stores analog values on trigger. FlipFlop toggles digital 0/1.",
    ),
    (
        "GreaterEqual",
        "AnalogThresholdTrigger",
        "Both compare, but AnalogThresholdTrigger has hysteresis.",
    ),
    (
        "OffDelay",
        "OnPulseDelay",
        "OffDelay keeps output ON after input drops. OnPulseDelay fires a delayed pulse.",
    ),
];

// ── Block index loaded from embedded JSON ────────────────────────────────────

struct BlockEntry {
    xml_type: String,
    name: String,
    connectors: Vec<ConnectorEntry>,
}

struct ConnectorEntry {
    conn_type: String, // "Input", "Output", "Parameter"
    name: String,
    short: String,
    default: Option<String>,
    min: Option<String>,
    max: Option<String>,
    unit: Option<String>,
}

fn load_block_index() -> Vec<BlockEntry> {
    let json_str = include_str!("../../docs/schemas/loxone-block-types-full.json");
    let raw: HashMap<String, serde_json::Value> =
        serde_json::from_str(json_str).unwrap_or_default();

    let mut blocks = Vec::with_capacity(raw.len());
    for (xml_type, val) in &raw {
        let name = val
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(xml_type)
            .to_string();

        let connectors = val
            .get("connectors")
            .and_then(|c| c.as_array())
            .map(|arr| {
                arr.iter()
                    .map(|c| ConnectorEntry {
                        conn_type: c
                            .get("type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        name: c
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        short: c
                            .get("short")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        default: c.get("default").and_then(|v| v.as_str()).map(String::from),
                        min: c.get("min").and_then(|v| v.as_str()).map(String::from),
                        max: c.get("max").and_then(|v| v.as_str()).map(String::from),
                        unit: c.get("unit").and_then(|v| v.as_str()).map(String::from),
                    })
                    .collect()
            })
            .unwrap_or_default();

        blocks.push(BlockEntry {
            xml_type: xml_type.clone(),
            name,
            connectors,
        });
    }
    blocks.sort_by(|a, b| a.xml_type.cmp(&b.xml_type));
    blocks
}

// ── Search scoring ───────────────────────────────────────────────────────────

fn search_blocks(query: &str, blocks: &[BlockEntry]) -> Vec<(String, f64, String)> {
    let query_lower = query.to_lowercase();
    let tokens: Vec<&str> = query_lower.split_whitespace().collect();
    if tokens.is_empty() {
        return Vec::new();
    }

    let mut scores: HashMap<String, f64> = HashMap::new();
    // Track names for output
    let name_map: HashMap<&str, &str> = blocks
        .iter()
        .map(|b| (b.xml_type.as_str(), b.name.as_str()))
        .collect();

    // 1. Keyword alias boost (0.4)
    for (alias, types) in ALIASES {
        if query_lower.contains(alias) {
            for t in *types {
                *scores.entry(t.to_string()).or_default() += 0.4;
            }
        }
    }

    // 2. Title / type name match (0.3)
    for block in blocks {
        let name_lower = block.name.to_lowercase();
        let type_lower = block.xml_type.to_lowercase();
        let token_hits = tokens
            .iter()
            .filter(|t| name_lower.contains(*t) || type_lower.contains(*t))
            .count();
        if token_hits > 0 {
            let frac = token_hits as f64 / tokens.len().max(1) as f64;
            *scores.entry(block.xml_type.clone()).or_default() += 0.3 * frac;
        }
    }

    // 3. Connector name match (0.2)
    for block in blocks {
        let conn_text: String = block
            .connectors
            .iter()
            .map(|c| format!("{} {}", c.name, c.short))
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
        let token_hits = tokens.iter().filter(|t| conn_text.contains(*t)).count();
        if token_hits > 0 {
            let frac = token_hits as f64 / tokens.len().max(1) as f64;
            *scores.entry(block.xml_type.clone()).or_default() += 0.2 * frac;
        }
    }

    // 4. Category name match (0.1)
    for (cat_name, types) in CATEGORIES {
        if tokens.iter().any(|t| cat_name.contains(t)) {
            for t in *types {
                *scores.entry(t.to_string()).or_default() += 0.1;
            }
        }
    }

    let mut results: Vec<(String, f64, String)> = scores
        .into_iter()
        .filter(|(_, score)| *score > 0.0)
        .map(|(xml_type, score)| {
            let name = name_map
                .get(xml_type.as_str())
                .unwrap_or(&xml_type.as_str())
                .to_string();
            (xml_type, score, name)
        })
        .collect();

    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(10);
    results
}

// ── Category lookup ──────────────────────────────────────────────────────────

fn category_for(xml_type: &str) -> Option<&'static str> {
    for (cat, types) in CATEGORIES {
        if types.contains(&xml_type) {
            return Some(cat);
        }
    }
    None
}

fn category_names() -> Vec<&'static str> {
    CATEGORIES.iter().map(|(name, _)| *name).collect()
}

// ── Command handlers ─────────────────────────────────────────────────────────

pub fn cmd_blocks(ctx: &RunContext, action: BlocksCmd) -> Result<()> {
    match action {
        BlocksCmd::Search { query } => blocks_search(ctx, &query),
        BlocksCmd::Info { block_type } => blocks_info(ctx, &block_type),
        BlocksCmd::List { category } => blocks_list(ctx, category.as_deref()),
    }
}

fn blocks_search(ctx: &RunContext, query: &str) -> Result<()> {
    let blocks = load_block_index();
    let results = search_blocks(query, &blocks);

    if results.is_empty() {
        if ctx.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "ok": true,
                    "query": query,
                    "results": [],
                }))?
            );
        } else {
            eprintln!("No blocks matched '{}'.", query);
            eprintln!(
                "Try: lox blocks list --category <{}>",
                category_names().join("|")
            );
        }
        return Ok(());
    }

    if ctx.json {
        let items: Vec<serde_json::Value> = results
            .iter()
            .map(|(xml_type, score, name)| {
                serde_json::json!({
                    "type": xml_type,
                    "name": name,
                    "score": (*score * 100.0).round() / 100.0,
                    "category": category_for(xml_type),
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "ok": true,
                "query": query,
                "results": items,
            }))?
        );
    } else {
        println!("Search: \"{}\"\n", query);
        println!(
            "{:<24} {:<30} {:>6}  CATEGORY",
            "TYPE", "NAME", "SCORE"
        );
        println!("{}", "─".repeat(76));
        for (xml_type, score, name) in &results {
            let cat = category_for(xml_type).unwrap_or("-");
            println!("{:<24} {:<30} {:>5.2}  {}", xml_type, name, score, cat);
        }
        println!("\nUse: lox blocks info <TYPE> for full details");
    }
    Ok(())
}

fn blocks_info(ctx: &RunContext, block_type: &str) -> Result<()> {
    let blocks = load_block_index();

    // Find by exact xml_type (case-insensitive)
    let block = blocks
        .iter()
        .find(|b| b.xml_type.eq_ignore_ascii_case(block_type));

    // Fallback: fuzzy substring match on name or xml_type
    let block = match block {
        Some(b) => b,
        None => {
            let lower = block_type.to_lowercase();
            let candidates: Vec<&BlockEntry> = blocks
                .iter()
                .filter(|b| {
                    b.xml_type.to_lowercase().contains(&lower)
                        || b.name.to_lowercase().contains(&lower)
                })
                .collect();
            if candidates.len() == 1 {
                candidates[0]
            } else if candidates.is_empty() {
                // Try search as fallback
                let results = search_blocks(block_type, &blocks);
                if !results.is_empty() {
                    let suggestions: Vec<String> =
                        results.iter().take(5).map(|(t, _, _)| t.clone()).collect();
                    bail!(
                        "Unknown block type '{}'. Did you mean: {}?\n\nTry: lox blocks search \"{}\"",
                        block_type,
                        suggestions.join(", "),
                        block_type
                    );
                }
                bail!("Unknown block type '{}'. Run: lox blocks list", block_type);
            } else {
                let names: Vec<String> = candidates
                    .iter()
                    .take(8)
                    .map(|b| format!("{} ({})", b.xml_type, b.name))
                    .collect();
                bail!(
                    "Ambiguous block type '{}'. Matches:\n  {}\n\nSpecify the exact type.",
                    block_type,
                    names.join("\n  ")
                );
            }
        }
    };

    let inputs: Vec<&ConnectorEntry> = block
        .connectors
        .iter()
        .filter(|c| c.conn_type == "Input")
        .collect();
    let outputs: Vec<&ConnectorEntry> = block
        .connectors
        .iter()
        .filter(|c| c.conn_type == "Output")
        .collect();
    let params: Vec<&ConnectorEntry> = block
        .connectors
        .iter()
        .filter(|c| c.conn_type == "Parameter")
        .collect();

    if ctx.json {
        let conn_to_json = |c: &ConnectorEntry| -> serde_json::Value {
            let mut obj = serde_json::json!({
                "name": c.name,
                "short": c.short,
            });
            if let Some(d) = &c.default {
                obj["default"] = serde_json::Value::String(d.clone());
            }
            if let Some(m) = &c.min {
                obj["min"] = serde_json::Value::String(m.clone());
            }
            if let Some(m) = &c.max {
                obj["max"] = serde_json::Value::String(m.clone());
            }
            if let Some(u) = &c.unit {
                obj["unit"] = serde_json::Value::String(u.clone());
            }
            obj
        };

        let mut result = serde_json::json!({
            "ok": true,
            "type": block.xml_type,
            "name": block.name,
            "category": category_for(&block.xml_type),
            "inputs": inputs.iter().map(|c| conn_to_json(c)).collect::<Vec<_>>(),
            "outputs": outputs.iter().map(|c| conn_to_json(c)).collect::<Vec<_>>(),
            "parameters": params.iter().map(|c| conn_to_json(c)).collect::<Vec<_>>(),
        });

        // Add confusion pairs
        let confusions: Vec<serde_json::Value> = CONFUSIONS
            .iter()
            .filter(|(a, b, _)| *a == block.xml_type || *b == block.xml_type)
            .map(|(a, b, note)| {
                let other = if *a == block.xml_type { *b } else { *a };
                serde_json::json!({ "other": other, "note": note })
            })
            .collect();
        if !confusions.is_empty() {
            result["confusions"] = serde_json::Value::Array(confusions);
        }

        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("{} ({})", block.xml_type, block.name);
        if let Some(cat) = category_for(&block.xml_type) {
            println!("Category: {}", cat);
        }
        println!();

        if !inputs.is_empty() {
            println!("  Inputs:");
            for c in &inputs {
                let detail = format_connector_detail(c);
                if detail.is_empty() {
                    println!("    {:<20} {}", c.name, c.short);
                } else {
                    println!("    {:<20} {} {}", c.name, c.short, detail);
                }
            }
            println!();
        }

        if !outputs.is_empty() {
            println!("  Outputs:");
            for c in &outputs {
                let detail = format_connector_detail(c);
                if detail.is_empty() {
                    println!("    {:<20} {}", c.name, c.short);
                } else {
                    println!("    {:<20} {} {}", c.name, c.short, detail);
                }
            }
            println!();
        }

        if !params.is_empty() {
            println!("  Parameters:");
            for c in &params {
                let detail = format_connector_detail(c);
                if detail.is_empty() {
                    println!("    {:<20} {}", c.name, c.short);
                } else {
                    println!("    {:<20} {} {}", c.name, c.short, detail);
                }
            }
            println!();
        }

        // Confusion pairs
        let confusions: Vec<&(&str, &str, &str)> = CONFUSIONS
            .iter()
            .filter(|(a, b, _)| *a == block.xml_type || *b == block.xml_type)
            .collect();
        if !confusions.is_empty() {
            println!("  Don't confuse with:");
            for (a, b, note) in confusions {
                let other = if *a == block.xml_type { b } else { a };
                println!("    {} — {}", other, note);
            }
            println!();
        }

        // CLI example
        println!("  Use with:");
        println!(
            "    lox config add <file> --type {} --title \"My {}\"",
            block.xml_type, block.name
        );

        // Related scenarios from eval cases
        let scenarios = load_scenarios_for_type(&block.xml_type);
        if !scenarios.is_empty() {
            println!();
            println!("  Scenarios:");
            for (i, (difficulty, utterance)) in scenarios.iter().take(5).enumerate() {
                println!("    {}. [{}] {}", i + 1, difficulty, utterance);
            }
            if scenarios.len() > 5 {
                println!("    ... and {} more", scenarios.len() - 5);
            }
        }
    }

    Ok(())
}

/// Load eval case scenarios that use a given block type.
fn load_scenarios_for_type(xml_type: &str) -> Vec<(String, String)> {
    let eval_dir = std::path::Path::new("tests/eval/cases");
    if !eval_dir.exists() {
        return Vec::new();
    }
    let mut scenarios = Vec::new();
    let Ok(entries) = std::fs::read_dir(eval_dir) else {
        return Vec::new();
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Ok(data) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(cases) = serde_json::from_str::<Vec<serde_json::Value>>(&data) else {
            continue;
        };
        for case in &cases {
            let blocks = case
                .get("expected")
                .and_then(|e| e.get("new_blocks"))
                .and_then(|b| b.as_array());
            if let Some(blocks) = blocks {
                let uses_type = blocks.iter().any(|b| {
                    b.get("type").and_then(|t| t.as_str()) == Some(xml_type)
                });
                if uses_type {
                    let difficulty = case
                        .get("difficulty")
                        .and_then(|d| d.as_str())
                        .unwrap_or("?")
                        .to_string();
                    let utterance = case
                        .get("utterance")
                        .and_then(|u| u.as_str())
                        .unwrap_or("")
                        .chars()
                        .take(90)
                        .collect::<String>();
                    if !utterance.is_empty() {
                        scenarios.push((difficulty, utterance));
                    }
                }
            }
        }
    }
    scenarios
}

fn format_connector_detail(c: &ConnectorEntry) -> String {
    let mut parts = Vec::new();
    if let Some(d) = &c.default {
        parts.push(format!("default={}", d));
    }
    if let Some(min) = &c.min {
        if let Some(max) = &c.max {
            parts.push(format!("range={}..{}", min, max));
        } else {
            parts.push(format!("min={}", min));
        }
    } else if let Some(max) = &c.max {
        parts.push(format!("max={}", max));
    }
    if let Some(u) = &c.unit {
        parts.push(u.clone());
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!("({})", parts.join(", "))
    }
}

fn blocks_list(ctx: &RunContext, category: Option<&str>) -> Result<()> {
    let blocks = load_block_index();

    // Build type → name lookup
    let name_map: HashMap<&str, &str> = blocks
        .iter()
        .map(|b| (b.xml_type.as_str(), b.name.as_str()))
        .collect();

    if let Some(cat) = category {
        let cat_lower = cat.to_lowercase();
        let matched = CATEGORIES.iter().find(|(name, _)| *name == cat_lower);

        let (cat_name, types) = match matched {
            Some(c) => c,
            None => {
                let available: Vec<&str> = category_names();
                bail!(
                    "Unknown category '{}'. Available: {}",
                    cat,
                    available.join(", ")
                );
            }
        };

        if ctx.json {
            let items: Vec<serde_json::Value> = types
                .iter()
                .map(|t| {
                    let name = name_map.get(t).copied().unwrap_or(*t);
                    let io = blocks.iter().find(|b| b.xml_type == *t);
                    let (ni, no) = io
                        .map(|b| {
                            let i = b
                                .connectors
                                .iter()
                                .filter(|c| c.conn_type == "Input")
                                .count();
                            let o = b
                                .connectors
                                .iter()
                                .filter(|c| c.conn_type == "Output")
                                .count();
                            (i, o)
                        })
                        .unwrap_or((0, 0));
                    serde_json::json!({
                        "type": t,
                        "name": name,
                        "inputs": ni,
                        "outputs": no,
                    })
                })
                .collect();
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "ok": true,
                    "category": cat_name,
                    "blocks": items,
                }))?
            );
        } else {
            println!("Category: {}\n", cat_name);
            println!("{:<24} {:<30} {:>3} {:>3}", "TYPE", "NAME", "IN", "OUT");
            println!("{}", "─".repeat(64));
            for t in *types {
                let name = name_map.get(t).copied().unwrap_or(t);
                let io = blocks.iter().find(|b| b.xml_type == *t);
                let (ni, no) = io
                    .map(|b| {
                        let i = b
                            .connectors
                            .iter()
                            .filter(|c| c.conn_type == "Input")
                            .count();
                        let o = b
                            .connectors
                            .iter()
                            .filter(|c| c.conn_type == "Output")
                            .count();
                        (i, o)
                    })
                    .unwrap_or((0, 0));
                println!("{:<24} {:<30} {:>3} {:>3}", t, name, ni, no);
            }
        }
    } else {
        // List all categories
        if ctx.json {
            let cats: Vec<serde_json::Value> = CATEGORIES
                .iter()
                .map(|(name, types)| {
                    serde_json::json!({
                        "category": name,
                        "count": types.len(),
                        "types": types,
                    })
                })
                .collect();
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "ok": true,
                    "categories": cats,
                    "total_types": blocks.len(),
                }))?
            );
        } else {
            println!("{:<12} {:>5}  TYPES", "CATEGORY", "COUNT");
            println!("{}", "─".repeat(76));
            for (name, types) in CATEGORIES {
                let preview: Vec<&str> = types.iter().take(4).copied().collect();
                let suffix = if types.len() > 4 { " …" } else { "" };
                println!(
                    "{:<12} {:>5}  {}{}",
                    name,
                    types.len(),
                    preview.join(", "),
                    suffix
                );
            }
            println!(
                "\n{} block types total. Use: lox blocks list --category <name>",
                blocks.len()
            );
        }
    }
    Ok(())
}
