//! JIT vs interpreter equivalence test for ALL block types.
//!
//! Verifies that `SimEngine` (interpreter) and `CompiledGraph` (JIT) produce
//! identical outputs for the same inputs across all 212+ registered block types.

use lox_sim::blocks::create_block;
use lox_sim::compiler::CompiledGraph;
use lox_sim::engine::SimEngine;
use lox_sim::graph::SimGraph;

const EPSILON: f64 = 1e-9;

/// All block type names from the `create_block` registry.
fn all_block_types() -> Vec<&'static str> {
    vec![
        "AMemory",
        "AMinmax",
        "AalEmergency",
        "AalSmartAlarm",
        "AcControl",
        "Access",
        "Add",
        "Add4",
        "Alarm",
        "AlarmChain",
        "AlarmClock",
        "AnalogComparator",
        "AnalogDiffTrigger",
        "AnalogMultiplexer",
        "AnalogMultiplexer2",
        "AnalogScaler",
        "AnalogStepper",
        "AnalogThresholdTrigger",
        "AnalogWatchdog",
        "And",
        "AutoJalousie",
        "AutomaticScene",
        "AutopilotRule",
        "Average",
        "Avg",
        "BinDecoder",
        "BinEncoder",
        "BrightnessControl",
        "Calendar",
        "CallGen",
        "CarCharger",
        "CentralAlarm",
        "CentralFancoil",
        "CentralGate",
        "CentralLight",
        "CentralMusic",
        "CentralPresence",
        "CentralRoofwindow",
        "CentralShade",
        "ClimateControllerUS",
        "CmdRecognition",
        "Code1",
        "Code16",
        "Code4",
        "Code8",
        "Constant",
        "Counter",
        "DayTimer",
        "DaylightController",
        "DbConE",
        "DbConS",
        "Device Tablet",
        "DewPoint",
        "Div",
        "Door",
        "Doorcontroller",
        "EFM",
        "EIBJalousie",
        "EIBPush",
        "EIBactor",
        "EIBextactor",
        "EIBsensor",
        "EdgeDetection",
        "EdgeWipingRelay",
        "EibDimmer",
        "Energy",
        "EnergyManager",
        "EnergyManager2",
        "Equal",
        "Fan",
        "Fancoil",
        "FancoilFreshAir",
        "FlipFlop",
        "Formula",
        "Fronius",
        "Gain",
        "Greater",
        "GreaterEqual",
        "HVACController",
        "HeatCentral",
        "HeatIRoomController2",
        "Heatcurve",
        "Heatmixer",
        "Heatmixer2",
        "HourCounter",
        "HvacAC",
        "IRcontroller",
        "IRoomcontrol",
        "InputRef",
        "Int",
        "Intercom",
        "Irrigation",
        "JalousieUpDown2",
        "Jalousiemotor",
        "JoinWindowSensor",
        "Leaf",
        "Less",
        "LessEqual",
        "LightController",
        "LightController2",
        "LightControllerH",
        "Lightscene",
        "LightsceneLearn",
        "LightsceneRGB",
        "LoadShed",
        "LongClick",
        "MPGroup",
        "MailBox",
        "MailGen",
        "Media",
        "MediaClient",
        "Memory",
        "MeterAbsBi",
        "MeterAbsSt",
        "MeterAbsUni",
        "MeterDig",
        "MeterPBi",
        "MeterPSt",
        "MeterPUni",
        "Minmax",
        "Mod",
        "Monoflop",
        "Mult",
        "MultiClick",
        "MultiFuncSW",
        "MusicPlayer",
        "Nand",
        "Nevo",
        "NfcCodeTouch",
        "Nor",
        "Not",
        "NotEqual",
        "OffDelay",
        "OnDelay",
        "OnOffDelay",
        "OnPulseDelay",
        "Or",
        "OutputRef",
        "OutputRefLM",
        "OvertempShutdown",
        "PButtonT",
        "PI",
        "PID",
        "PVProductionForecast",
        "PWM",
        "PassThrough",
        "Pergola",
        "Ping",
        "PoolController",
        "Power",
        "PowerUnit",
        "Presence",
        "PresenceController",
        "PresenceDetector",
        "PulseAt",
        "PulseBy",
        "PulseGen",
        "PushButton",
        "PushButton2",
        "PushButton2Sel",
        "PushButtonSel",
        "PushDimmer",
        "RSFlipFlop",
        "Radio",
        "Radio2",
        "Ramp",
        "Rand",
        "RandomGen",
        "RetOnDelay",
        "RisingEdge",
        "RoofWindow",
        "Roomcontrol",
        "SRFlipFlop",
        "SampleHold",
        "Sauna",
        "SaunaVapor",
        "Sequencer",
        "SequenceController",
        "ShadeRoof",
        "Shift",
        "Skylight",
        "SmokeAlarm",
        "Solarpumpcontrol",
        "SpotOpt",
        "StairwayLS",
        "State",
        "StateV",
        "StatusMonitor",
        "SteakThermo",
        "StepSel",
        "Sub",
        "SwitchingTimer",
        "SysVar",
        "Tablet",
        "TextGenerator",
        "TimeMinmax",
        "ToiletFan",
        "TpfController",
        "TrackingPassThrough",
        "UpDownCounter",
        "Validator",
        "VentInternorm",
        "Ventilation",
        "Ventilation2",
        "VirtualIn",
        "VirtualOut",
        "VirtualState",
        "WBEM",
        "Wallbox",
        "Weed",
        "Wind",
        "WindowsMonitor",
        "Xor",
        "2Point",
        "3Point",
    ]
}

/// Connector schema for a block type: (inputs, outputs, params).
///
/// Mirrors `block_signature` from parser.rs — extended with schemas for all
/// block types that have specific connector layouts.
fn block_connectors(block_type: &str) -> (Vec<&'static str>, Vec<&'static str>, Vec<&'static str>) {
    match block_type {
        "AMemory" => (vec!["Input", "Trigger", "Reset"], vec!["AQ"], vec![]),
        "Add" | "Add4" => (
            vec!["Input1", "Input2", "Input3", "Input4"],
            vec!["AQ", "Q"],
            vec![],
        ),
        "AnalogThresholdTrigger" => (
            vec!["Input"],
            vec!["Q", "RisingEdge", "FallingEdge"],
            vec!["On", "Off", "PulseTime"],
        ),
        "AnalogComparator" => (vec!["Input1", "Input2"], vec!["Q"], vec!["Hysteresis"]),
        "AnalogDiffTrigger" => (
            vec!["Input1", "Input2"],
            vec!["Q"],
            vec!["OnDiff", "OffDiff"],
        ),
        "AnalogMultiplexer" => (vec!["I1", "I2", "I3", "I4", "Select"], vec!["AQ"], vec![]),
        "AnalogMultiplexer2" => (vec!["I1", "I2", "I3", "I4", "Select"], vec!["AQ"], vec![]),
        "AnalogScaler" => (
            vec!["Input"],
            vec!["AQ"],
            vec!["InMin", "InMax", "OutMin", "OutMax"],
        ),
        "AnalogStepper" => (
            vec!["Up", "Down", "Reset"],
            vec!["AQ"],
            vec!["Step", "Min", "Max"],
        ),
        "AnalogWatchdog" => (vec!["Input"], vec!["Q"], vec!["Min", "Max", "Timeout"]),
        "And" => (vec!["I1", "I2"], vec!["Q"], vec![]),
        "Constant" => (vec![], vec!["Q"], vec!["Value"]),
        "Counter" => (
            vec!["Trigger", "I1"],
            vec!["Q", "AQ"],
            vec!["EndValue", "Mode"],
        ),
        "DayTimer" => (
            vec!["minutes_since_midnight", "day_of_week", "InputTrigger"],
            vec!["AQ", "AQm", "Qon", "Qoff", "AQmt"],
            vec!["Manual", "Mode", "PulseTime"],
        ),
        "Div" => (vec!["Input1", "Input2"], vec!["AQ", "Q"], vec![]),
        "EdgeDetection" => (
            vec!["Input", "I1"],
            vec!["Edge", "RisingEdge", "FallingEdge"],
            vec!["PulseTime"],
        ),
        "FlipFlop" | "RSFlipFlop" | "SRFlipFlop" => {
            (vec!["InputS", "InputR", "InputTrigger"], vec!["Q"], vec![])
        }
        "Formula" => (vec!["I1", "I2", "I3", "I4"], vec!["Q"], vec![]),
        "Gain" => (vec!["I1"], vec!["Q"], vec!["Factor"]),
        "Greater" | "GreaterEqual" | "Less" | "LessEqual" | "Equal" | "NotEqual" => {
            (vec!["Input1", "Input2"], vec!["Q"], vec![])
        }
        "Monoflop" => (vec!["InputTrigger"], vec!["Q"], vec!["Time"]),
        "Mult" => (vec!["Input1", "Input2"], vec!["AQ", "Q"], vec![]),
        "Not" => (vec!["I", "I1"], vec!["Q"], vec![]),
        "OffDelay" => (vec!["InputTrigger"], vec!["Q"], vec!["Time"]),
        "OnDelay" => (vec!["InputTrigger"], vec!["Q"], vec!["Time"]),
        "OnOffDelay" => (vec!["InputTrigger"], vec!["Q"], vec!["OnDelay", "OffDelay"]),
        "OnPulseDelay" => (vec!["InputTrigger"], vec!["Q"], vec!["Delay", "Time"]),
        "Or" => (vec!["I1", "I2"], vec!["Q"], vec![]),
        "PassThrough" | "TrackingPassThrough" => (vec!["I1", "Input"], vec!["Q", "AQ"], vec![]),
        "PulseGen" => (
            vec!["InputEnable", "InputInvert"],
            vec!["Q"],
            vec!["TimeHigh", "TimeLow"],
        ),
        "PushButton" | "PushButton2" | "PushButtonSel" | "PushButton2Sel" => (
            vec!["InputTrigger", "On"],
            vec!["Q", "Qoff", "Qon", "AQ"],
            vec!["Min", "Max"],
        ),
        "RisingEdge" => (vec!["I1", "Input"], vec!["Q", "RisingEdge"], vec![]),
        "LightController2" => (
            vec![
                "I1",
                "Presence",
                "Brightness",
                "Move",
                "Sel1",
                "Sel2",
                "Sel3",
                "Sel4",
                "Sel5",
                "Sel6",
                "Sel7",
                "Sel8",
                "Reset",
                "InputDisable",
            ],
            vec!["AQ1", "AQ2", "Scene", "PresenceActive"],
            vec!["FadingTime", "SceneMixTime"],
        ),
        "LightController" | "LightControllerH" => (
            vec!["I1", "Presence", "Brightness", "Reset", "InputDisable"],
            vec!["AQ1", "Scene", "PresenceActive"],
            vec!["FadingTime"],
        ),
        "JalousieUpDown2" | "Jalousiemotor" | "EIBJalousie" | "Pergola" | "RoofWindow"
        | "ShadeRoof" | "Skylight" => (
            vec![
                "InputTriggerUp",
                "InputTriggerDown",
                "InputPos",
                "InputDisable",
            ],
            vec!["Pos", "Dir", "Moving"],
            vec!["TimeEnd"],
        ),
        "HeatIRoomController2" | "ClimateControllerUS" | "HVACController" => (
            vec!["Temp", "Setpoint", "Reset", "InputDisable"],
            vec!["AQh"],
            vec!["Kp", "Ki"],
        ),
        "StairwayLS" => (
            vec!["InputTrigger", "On"],
            vec!["Q"],
            vec!["TimeHigh", "TimeWarn", "WarnTime"],
        ),
        "Sub" => (vec!["Input1", "Input2"], vec!["AQ", "Q"], vec![]),
        "SysVar" => (vec!["I1"], vec!["AQ"], vec![]),
        "VirtualIn" | "VirtualOut" | "VirtualState" => {
            (vec!["I1", "Input"], vec!["Q", "AQ"], vec![])
        }
        "Xor" | "Nand" | "Nor" => (vec!["I1", "I2"], vec!["Q"], vec![]),
        "PresenceDetector" => (vec!["InputTrigger"], vec!["OutputPresence"], vec![]),
        "Calendar" => (
            vec!["minutes_since_midnight", "day_of_week"],
            vec!["Q"],
            vec![],
        ),
        "AlarmClock" => (
            vec!["minutes_since_midnight", "day_of_week", "InputTrigger"],
            vec!["Q"],
            vec![],
        ),
        "PulseAt" => (
            vec!["minutes_since_midnight", "day_of_week"],
            vec!["Q"],
            vec![],
        ),
        "EdgeWipingRelay" => (vec!["InputTrigger"], vec!["Q"], vec!["Time"]),
        "PWM" => (vec!["Input"], vec!["Q"], vec!["Period"]),
        "RetOnDelay" => (vec!["InputTrigger"], vec!["Q"], vec!["Time"]),
        "SwitchingTimer" => (
            vec!["InputTrigger", "On", "Off"],
            vec!["Q"],
            vec!["OnTime", "OffTime"],
        ),
        "Memory" => (vec!["Input", "Trigger", "Reset"], vec!["Q"], vec![]),
        "SampleHold" => (vec!["Input", "Trigger"], vec!["AQ"], vec![]),
        "Shift" => (vec!["Input", "Trigger", "Reset"], vec!["Q1", "Q2"], vec![]),
        "State" => (vec!["InputTrigger"], vec!["Q", "AQ"], vec!["NumStates"]),
        "StateV" => (vec!["InputTrigger"], vec!["Q", "AQ"], vec![]),
        "HourCounter" => (vec!["Input", "Reset"], vec!["AQ"], vec![]),
        "UpDownCounter" => (
            vec!["Up", "Down", "Reset"],
            vec!["AQ", "Q"],
            vec!["Min", "Max"],
        ),
        "AMinmax" => (vec!["Input", "Reset"], vec!["Min", "Max"], vec![]),
        "Int" => (vec!["I1"], vec!["Q"], vec![]),
        "Minmax" => (vec!["Input1", "Input2"], vec!["Min", "Max"], vec![]),
        "Mod" => (vec!["Input1", "Input2"], vec!["AQ"], vec![]),
        "Alarm" => (
            vec!["InputTrigger", "InputArm", "InputDisable", "InputReset"],
            vec!["Q", "QArmed"],
            vec![],
        ),
        "Presence" => (vec!["InputTrigger", "InputDisable"], vec!["Q"], vec![]),
        "PresenceController" => (vec!["InputTrigger", "InputDisable"], vec!["Q"], vec![]),
        "AutoJalousie" => (vec!["Input", "InputDisable"], vec!["Q"], vec![]),
        "Energy" | "EnergyManager" | "EnergyManager2" => (vec!["Input"], vec!["AQ"], vec![]),
        "MeterAbsUni" | "MeterAbsBi" | "MeterPUni" | "MeterPBi" | "MeterDig" | "MeterAbsSt"
        | "MeterPSt" => (vec!["Input"], vec!["AQ"], vec![]),
        "AcControl" | "OvertempShutdown" => {
            (vec!["Temp", "Setpoint", "InputDisable"], vec!["AQ"], vec![])
        }
        "Heatcurve" => (
            vec!["Input", "Setpoint"],
            vec!["AQ"],
            vec!["BaseTemp", "Slope"],
        ),
        "Heatmixer" => (
            vec!["Input", "Setpoint"],
            vec!["AQ"],
            vec!["BaseTemp", "Slope"],
        ),
        "Heatmixer2" => (
            vec!["Supply", "Return", "Setpoint"],
            vec!["Position", "FlowTemp"],
            vec![],
        ),
        "Solarpumpcontrol" => (vec!["Input1", "Input2"], vec!["Q"], vec![]),
        "PoolController" => (vec!["Temp", "InputDisable"], vec!["Q"], vec![]),
        "Sauna" | "SaunaVapor" => (vec!["Temp", "InputTrigger"], vec!["Q"], vec![]),
        "2Point" => (vec!["Input", "Setpoint"], vec!["Q"], vec!["Hysteresis"]),
        "3Point" => (
            vec!["Input", "Setpoint"],
            vec!["QUp", "QDown"],
            vec!["Hysteresis"],
        ),
        "PID" | "PI" => (
            vec!["Input", "Setpoint"],
            vec!["AQ"],
            vec!["Kp", "Ki", "Kd"],
        ),
        "Wallbox" => (vec!["Input"], vec!["AQ"], vec![]),
        "InputRef" | "OutputRef" | "OutputRefLM" => (vec!["I1"], vec!["Q"], vec![]),
        "EIBPush" | "EIBsensor" | "EIBactor" | "EIBextactor" | "EibDimmer" => {
            (vec!["I1"], vec!["Q"], vec![])
        }
        "Average" | "Avg" => (vec!["I1", "I2", "I3", "I4"], vec!["AQ"], vec![]),
        "BinEncoder" => (vec!["I1", "I2", "I3", "I4"], vec!["AQ"], vec![]),
        "BinDecoder" => (vec!["Input"], vec!["Q1", "Q2", "Q3", "Q4"], vec![]),
        "DewPoint" => (vec!["Temp", "Humidity"], vec!["AQ"], vec![]),
        "Power" => (vec!["Input", "Exponent"], vec!["AQ"], vec![]),
        "Validator" => (vec!["Input"], vec!["AQ"], vec!["Min", "Max"]),
        "TimeMinmax" => (vec!["Input", "Reset"], vec!["Min", "Max"], vec!["Period"]),
        "Ramp" => (vec!["Input"], vec!["AQ"], vec!["Rate"]),
        "Rand" | "RandomGen" => (vec!["InputTrigger"], vec!["AQ"], vec!["Min", "Max"]),
        "PulseBy" => (vec!["InputTrigger"], vec!["Q"], vec!["Count"]),
        "LongClick" => (vec!["InputTrigger"], vec!["Q", "QLong"], vec!["Time"]),
        "MultiClick" => (vec!["InputTrigger"], vec!["Q1", "Q2", "Q3"], vec!["Time"]),
        "PushDimmer" => (vec!["InputTrigger"], vec!["AQ"], vec!["Min", "Max"]),
        "StepSel" => (vec!["InputTrigger", "Reset"], vec!["AQ"], vec!["Steps"]),
        "Sequencer" => (vec!["InputTrigger", "Reset"], vec!["AQ"], vec!["Steps"]),
        "LoadShed" => (vec!["I1"], vec!["Q"], vec!["Limit"]),
        "Fronius" | "PVProductionForecast" => (vec!["Input"], vec!["AQ"], vec![]),
        // All remaining types get a minimal I1 → Q passthrough schema.
        _ => (vec!["I1"], vec!["Q"], vec![]),
    }
}

/// Input test scenarios: (description, input values, dt, num_ticks).
///
/// For each block we run several scenarios to exercise stateless math,
/// stateful timers, edge detectors, and time-dependent controllers.
fn test_scenarios() -> Vec<(&'static str, &'static [f64], f64, usize)> {
    vec![
        ("zeros", &[0.0, 0.0, 0.0, 0.0], 0.1, 10),
        ("ones", &[1.0, 1.0, 1.0, 1.0], 0.1, 10),
        ("mixed", &[25.0, 10.0, 0.5, 3.0], 0.1, 10),
        ("negative", &[-5.0, -1.0, -0.5, -10.0], 0.1, 10),
        ("small", &[0.001, 0.002, 0.003, 0.004], 0.1, 10),
        ("large", &[1000.0, 500.0, 100.0, 50.0], 0.1, 10),
        ("dt_1s", &[1.0, 1.0, 1.0, 1.0], 1.0, 10),
    ]
}

/// Build a graph with a source VirtualIn wired to the block under test.
fn build_test_graph(
    block_type: &str,
    inputs: &[&str],
    outputs: &[&str],
    params: &[&str],
) -> SimGraph {
    let mut graph = SimGraph::new();

    // Source block (VirtualIn) — one output per target input.
    let source_outputs: Vec<String> = inputs.iter().map(|k| format!("Src_{k}")).collect();
    let source_out_refs: Vec<&str> = source_outputs.iter().map(|s| s.as_str()).collect();
    let source_id = graph.add_block(
        "Source",
        create_block("VirtualIn"),
        &[],
        &source_out_refs,
        &[],
    );

    // Target block under test.
    let target_id = graph.add_block("Target", create_block(block_type), inputs, outputs, params);

    // Wire each source output → target input.
    let source_outs = graph.block_info(source_id).outputs.clone();
    let target_ins = graph.block_info(target_id).inputs.clone();
    let n_wire = source_outs.len().min(target_ins.len());
    for i in 0..n_wire {
        graph.add_wire(source_outs[i], target_ins[i]).ok();
    }

    graph
}

/// Default parameter value for a given param key.
fn default_param_value(pkey: &str) -> f64 {
    match pkey {
        "Time" | "TimeHigh" | "TimeLow" | "OnDelay" | "OffDelay" | "Delay" | "OnTime"
        | "OffTime" | "PulseTime" | "TimeEnd" | "Timeout" | "TimeWarn" | "WarnTime" | "Period" => {
            2.0
        }
        "Kp" => 1.0,
        "Ki" | "Kd" => 0.0,
        "Value" => 42.0,
        "Factor" => 2.0,
        "On" => 10.0,
        "Off" => 5.0,
        "Hysteresis" => 1.0,
        "Min" => 0.0,
        "Max" => 100.0,
        "Step" | "Steps" => 1.0,
        "EndValue" => 10.0,
        "NumStates" => 4.0,
        "InMin" | "OutMin" => 0.0,
        "InMax" | "OutMax" => 100.0,
        "OnDiff" => 5.0,
        "OffDiff" => 2.0,
        "Rate" => 1.0,
        "Count" => 3.0,
        "BaseTemp" => 20.0,
        "Slope" => 1.5,
        "Exponent" => 1.0,
        "Limit" => 10000.0,
        "Mode" | "Manual" | "FadingTime" | "SceneMixTime" => 0.0,
        _ => 1.0,
    }
}

/// Apply parameter defaults to both engines.
fn apply_params(engine: &mut SimEngine, compiled: &mut CompiledGraph, param_refs: &[&str]) {
    for pkey in param_refs {
        let pval = default_param_value(pkey);
        engine.set_param("Target", pkey, pval);
        compiled.set_param("Target", pkey, pval);
    }
}

/// Verify a single block type across all test scenarios.
fn verify_block_equivalence(block_type: &str) -> Result<(), String> {
    let (input_keys, output_keys, param_keys) = block_connectors(block_type);
    let input_refs = input_keys.to_vec();
    let output_refs = output_keys.to_vec();
    let param_refs = param_keys.to_vec();

    // Skip Rand/RandomGen — they are intentionally non-deterministic.
    if matches!(block_type, "Rand" | "RandomGen") {
        return Ok(());
    }

    for (desc, values, dt, ticks) in test_scenarios() {
        let graph = build_test_graph(block_type, &input_refs, &output_refs, &param_refs);
        let n_conn = graph.connector_count();

        let mut engine = SimEngine::new(graph.clone());
        let mut compiled = CompiledGraph::from_graph(&graph);

        // Set source outputs (which feed into target inputs).
        let source_info = graph.block_info(0);
        for (i, &cid) in source_info.outputs.iter().enumerate() {
            let val = values.get(i).copied().unwrap_or(0.0);
            let key = &graph.connector(cid).key;
            let name = format!("Source.{key}");
            engine.set_input(&name, val);
            compiled.set_input(&name, val);
        }

        // Set param defaults (give timers non-zero durations, etc).
        apply_params(&mut engine, &mut compiled, &param_refs);

        // Tick both engines.
        for _ in 0..ticks {
            engine.tick(dt);
            compiled.tick(dt);
        }

        // Compare all connector signals.
        for cid in 0..n_conn {
            let interp = engine.signal(cid);
            let jit = compiled.signal(cid);

            if !interp.is_finite() && !jit.is_finite() {
                continue;
            }
            if !interp.is_finite() || !jit.is_finite() {
                return Err(format!(
                    "scenario '{desc}': connector {cid} finite mismatch (interp={interp}, jit={jit})"
                ));
            }
            if (interp - jit).abs() > EPSILON {
                let conn = graph.connector(cid);
                let block = graph.block_info(conn.block_id);
                return Err(format!(
                    "scenario '{desc}': {}.{} mismatch (interp={interp}, jit={jit}, diff={})",
                    block.name,
                    conn.key,
                    (interp - jit).abs()
                ));
            }
        }
    }

    // Rising-edge scenario: set inputs to 0, tick, then set to 1, tick.
    // This catches edge-sensitive blocks (flip-flops, edge detectors, counters).
    {
        let graph = build_test_graph(block_type, &input_refs, &output_refs, &param_refs);
        let n_conn = graph.connector_count();
        let mut engine = SimEngine::new(graph.clone());
        let mut compiled = CompiledGraph::from_graph(&graph);

        // Set params.
        apply_params(&mut engine, &mut compiled, &param_refs);

        let source_info = graph.block_info(0);

        // Phase 1: all zeros for 5 ticks.
        for &cid in &source_info.outputs {
            let name = format!("Source.{}", graph.connector(cid).key);
            engine.set_input(&name, 0.0);
            compiled.set_input(&name, 0.0);
        }
        for _ in 0..5 {
            engine.tick(0.1);
            compiled.tick(0.1);
        }

        // Phase 2: rising edge (all → 1) for 5 ticks.
        for &cid in &source_info.outputs {
            let name = format!("Source.{}", graph.connector(cid).key);
            engine.set_input(&name, 1.0);
            compiled.set_input(&name, 1.0);
        }
        for _ in 0..5 {
            engine.tick(0.1);
            compiled.tick(0.1);
        }

        // Phase 3: falling edge (all → 0) for 5 ticks.
        for &cid in &source_info.outputs {
            let name = format!("Source.{}", graph.connector(cid).key);
            engine.set_input(&name, 0.0);
            compiled.set_input(&name, 0.0);
        }
        for _ in 0..5 {
            engine.tick(0.1);
            compiled.tick(0.1);
        }

        for cid in 0..n_conn {
            let interp = engine.signal(cid);
            let jit = compiled.signal(cid);
            if !interp.is_finite() && !jit.is_finite() {
                continue;
            }
            if !interp.is_finite() || !jit.is_finite() {
                return Err(format!(
                    "edge scenario: connector {cid} finite mismatch (interp={interp}, jit={jit})"
                ));
            }
            if (interp - jit).abs() > EPSILON {
                let conn = graph.connector(cid);
                let block = graph.block_info(conn.block_id);
                return Err(format!(
                    "edge scenario: {}.{} mismatch (interp={interp}, jit={jit}, diff={})",
                    block.name,
                    conn.key,
                    (interp - jit).abs()
                ));
            }
        }
    }

    Ok(())
}

#[test]
fn all_blocks_jit_matches_interpreter() {
    let block_types = all_block_types();
    assert!(
        block_types.len() >= 212,
        "expected at least 212 block types, got {}",
        block_types.len()
    );

    let mut failures = Vec::new();
    let mut passed = 0usize;

    for block_type in &block_types {
        match verify_block_equivalence(block_type) {
            Ok(()) => passed += 1,
            Err(msg) => failures.push(format!("{block_type}: {msg}")),
        }
    }

    eprintln!(
        "JIT equivalence: {passed}/{} passed, {} failed",
        block_types.len(),
        failures.len()
    );

    assert!(
        failures.is_empty(),
        "{} blocks failed JIT/interpreter equivalence:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
