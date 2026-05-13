"""
Trace-based evaluation: discover what an agent's circuit actually does
by probing fixture sensors and observing actuator responses, then
(optionally) asking an LLM judge whether the behavior matches the intent.

Usage from eval-agent.py:
    from trace_eval import evaluate_by_trace

    result = evaluate_by_trace(config_path, utterance, agent_backend="copilot")
    # result = {"pass": True/False, "behavior_map": {...}, "summary": "...", ...}
"""

import json
import shutil
import subprocess
from pathlib import Path
from typing import Any

SCRIPT_DIR = Path(__file__).parent
EVAL_DIR = SCRIPT_DIR.parent
REPO_ROOT = EVAL_DIR.parent.parent

# ── Fixture sensor/actuator catalog ──────────────────────────

# Probe sensors: (name, test_value, description)
# Boolean sensors get 1.0, analog sensors get a reasonable representative value.
PROBE_SENSORS: list[tuple[str, float, str]] = [
    # Weather SysVars
    ("Außentemperatur", 30.0, "outdoor temperature 30°C"),
    ("Sonnenschein", 1.0, "sunshine active"),
    ("Windgeschwindigkeit", 50.0, "wind speed 50 km/h"),
    ("Regen", 1.0, "rain active"),
    ("Luftfeuchtigkeit", 80.0, "humidity 80%"),
    ("Helligkeit", 80000.0, "brightness 80k lux"),
    # Virtual inputs — boolean switches
    ("Schalter 1", 1.0, "switch 1 on"),
    ("Schalter 2", 1.0, "switch 2 on"),
    ("Bewässerung Manuell", 1.0, "manual irrigation on"),
    ("Türklingel", 1.0, "doorbell pressed"),
    ("Garagentor Sensor", 1.0, "garage door sensor active"),
    # Virtual inputs — analog sensors
    ("Feuchtesensor Garten", 30.0, "garden soil moisture 30%"),
    ("CO2 Sensor", 1200.0, "CO₂ level 1200 ppm"),
    ("Raumtemperatur Wohnzimmer", 25.0, "living room temp 25°C"),
    ("Raumtemperatur Schlafzimmer", 25.0, "bedroom temp 25°C"),
    ("Pool Temperatur", 28.0, "pool temp 28°C"),
    # Per-room tree sensors — temperature
    ("Raumtemperatur Küche", 25.0, "kitchen temp 25°C"),
    ("Raumtemperatur Bad", 25.0, "bathroom temp 25°C"),
    ("Raumtemperatur Flur", 25.0, "hallway temp 25°C"),
    # Per-room tree sensors — humidity
    ("Raumfeuchtigkeit Küche", 70.0, "kitchen humidity 70%"),
    ("Raumfeuchtigkeit Bad", 85.0, "bathroom humidity 85%"),
    ("Raumfeuchtigkeit Flur", 60.0, "hallway humidity 60%"),
    # Window/door contacts (boolean)
    ("Fensterkontakt Wohnzimmer", 1.0, "living room window open"),
    ("Türkontakt Wohnzimmer", 1.0, "living room door open"),
    ("Fensterkontakt Schlafzimmer", 1.0, "bedroom window open"),
    ("Türkontakt Schlafzimmer", 1.0, "bedroom door open"),
    ("Fensterkontakt Küche", 1.0, "kitchen window open"),
    ("Türkontakt Küche", 1.0, "kitchen door open"),
    ("Fensterkontakt Bad", 1.0, "bathroom window open"),
    ("Türkontakt Bad", 1.0, "bathroom door open"),
    ("Fensterkontakt Flur", 1.0, "hallway window open"),
    ("Türkontakt Eingang", 1.0, "entrance door open"),
    # Presence / motion detectors
    ("Bewegungsmelder.InputTrigger", 1.0, "hallway motion detected"),
    ("Bewegungsmelder Garten.InputTrigger", 1.0, "garden motion detected"),
    # Push button
    ("PushButton.InputTrigger", 1.0, "push button pressed"),
    # Smoke detector
    ("Rauchmelder", 1.0, "smoke alarm active"),
    # Water sensor
    ("Wassersensor", 1.0, "water leak detected"),
    # Additional analog sensors
    ("Pegelsensor", 80.0, "water level 80%"),
    ("Vorlauftemperatur", 45.0, "flow temperature 45°C"),
    ("Warmwasserspeicher", 55.0, "hot water tank 55°C"),
    ("Puffertemperatur", 50.0, "buffer tank 50°C"),
    ("Kesseltemperatur", 60.0, "boiler temp 60°C"),
    ("Solarproduktion", 5.0, "solar production 5 kW"),
    ("Hausverbrauch", 3.0, "house consumption 3 kW"),
    ("Stromverbrauch WP", 2.0, "heat pump consumption 2 kW"),
    ("Wärmeleistung WP", 8.0, "heat pump output 8 kW"),
    ("KNX Schalter Garage", 1.0, "KNX garage switch on"),
]

# Fixture actuator output connectors to watch.
# Format: "BlockTitle [Room].Connector" or "BlockTitle.Connector"
# These are the outputs we care about — the things that physically happen.
ACTUATOR_OUTPUTS: list[str] = [
    # Wohnzimmer
    "Lichtsteuerung [Wohnzimmer].I1",
    "Lichtsteuerung [Wohnzimmer].Presence",
    "Jalousie 1 [Wohnzimmer].InputTriggerDown",
    "Jalousie 1 [Wohnzimmer].InputTriggerUp",
    "Jalousie 1 [Wohnzimmer].InputDisable",
    "Jalousie 2 [Wohnzimmer].InputTriggerDown",
    "Jalousie 2 [Wohnzimmer].InputTriggerUp",
    "Jalousie 2 [Wohnzimmer].InputDisable",
    "Raumregler [Wohnzimmer].Temp",
    "Klimaanlage [Wohnzimmer].toggle",
    "Leinwand [Wohnzimmer].InputTriggerDown",
    "Leinwand [Wohnzimmer].InputTriggerUp",
    # Schlafzimmer
    "Lichtsteuerung [Schlafzimmer].I1",
    "Lichtsteuerung [Schlafzimmer].Presence",
    "Jalousie 1 [Schlafzimmer].InputTriggerDown",
    "Jalousie 1 [Schlafzimmer].InputTriggerUp",
    "Jalousie 1 [Schlafzimmer].InputDisable",
    "Jalousie 2 [Schlafzimmer].InputTriggerDown",
    "Jalousie 2 [Schlafzimmer].InputTriggerUp",
    "Jalousie 2 [Schlafzimmer].InputDisable",
    # Küche
    "Lichtsteuerung [Küche].I1",
    "Lichtsteuerung [Küche].Presence",
    "Jalousie 1 [Küche].InputTriggerDown",
    "Jalousie 1 [Küche].InputTriggerUp",
    "Jalousie 1 [Küche].InputDisable",
    "Jalousie 2 [Küche].InputTriggerDown",
    "Jalousie 2 [Küche].InputTriggerUp",
    "Jalousie 2 [Küche].InputDisable",
    # Bad
    "Lichtsteuerung [Bad].I1",
    "Lichtsteuerung [Bad].Presence",
    "Jalousie 1 [Bad].InputTriggerDown",
    "Jalousie 1 [Bad].InputTriggerUp",
    "Jalousie 1 [Bad].InputDisable",
    "Jalousie 2 [Bad].InputTriggerDown",
    "Jalousie 2 [Bad].InputTriggerUp",
    "Jalousie 2 [Bad].InputDisable",
    "Lüfter Bad.I1",
    "Heizkörper Bad.I1",
    "Heizstab [Bad].I",
    # Flur
    "Lichtsteuerung [Flur].I1",
    "Lichtsteuerung [Flur].Presence",
    "Jalousie 1 [Flur].InputTriggerDown",
    "Jalousie 1 [Flur].InputTriggerUp",
    "Jalousie 1 [Flur].InputDisable",
    "Jalousie 2 [Flur].InputTriggerDown",
    "Jalousie 2 [Flur].InputTriggerUp",
    "Jalousie 2 [Flur].InputDisable",
    "Türöffner.I1",
    "Alarmanlage [Flur].I",
    "Türschloss [Flur].I",
    # Garten
    "Gartenbeleuchtung [Garten].I1",
    "Gartenbeleuchtung [Garten].Presence",
    "Bewässerungsventil.I1",
    "Poolpumpe.I1",
    # Garage
    "Garagenlicht [Garage].I1",
    "Garagenlicht [Garage].Presence",
    "Garagentor.I1",
    "Wallbox [Garage].I",
    "Batteriespeicher [Garage].I",
    # Wohnzimmer misc
    "Statusblock [Wohnzimmer].I",
    "Audiozone [Wohnzimmer].I",
    "Formel [Wohnzimmer].I",
    # Küche misc
    "Steckdose [Küche].I",
]

# Human-readable action descriptions for actuator connectors
_ACTUATOR_DESCRIPTIONS: dict[str, str] = {
    "InputTriggerDown": "closes",
    "InputTriggerUp": "opens",
    "InputDisable": "is disabled",
    "I1": "turns on",
    "I": "activates",
    "Presence": "receives presence signal",
    "Temp": "receives temperature",
    "toggle": "toggles",
}


def _find_lox_binary() -> str:
    """Locate the lox CLI binary."""
    lox = shutil.which("lox")
    if lox:
        return lox
    candidate = REPO_ROOT / "target" / "release" / "lox"
    if candidate.exists():
        return str(candidate)
    return "lox"


def _run_sim_probe(
    config_path: str,
    inputs: dict[str, float],
    lox_bin: str,
    ticks: int = 10,
    dt: float = 0.1,
) -> tuple[dict[str, float], list[dict[str, Any]]]:
    """
    Run a single sim probe with trace=true and expected_outputs for all
    known actuator inputs.  Returns (actuator_values, trace_entries).

    actuator_values: {"Jalousie 1 [Wohnzimmer].InputTriggerDown": 1.0, ...}
    trace_entries:   [{"output": "...", "value": ..., ...}, ...]
    """
    # Build expected_outputs that always pass — just to read actual values
    expected_outputs = {name: {">=": -999999} for name in ACTUATOR_OUTPUTS}

    sim_spec = json.dumps({
        "inputs": inputs,
        "ticks": ticks,
        "dt": dt,
        "trace": True,
        "expected_outputs": expected_outputs,
    }, ensure_ascii=False)

    try:
        result = subprocess.run(
            [lox_bin, "sim", "run", config_path, "--sim", sim_spec],
            capture_output=True,
            text=True,
            timeout=30,
        )
    except (subprocess.TimeoutExpired, FileNotFoundError) as exc:
        return {}, [{"error": str(exc)}]

    try:
        data = json.loads(result.stdout)
    except (json.JSONDecodeError, ValueError):
        return {}, []

    scenarios = data.get("scenarios", [])
    if not scenarios:
        return {}, []

    scenario = scenarios[0]
    trace_entries = scenario.get("trace", [])

    # Extract actual values from checks
    actuator_values: dict[str, float] = {}
    for check in scenario.get("checks", []):
        output_name = check.get("output", "")
        actual = check.get("actual", 0.0)
        actuator_values[output_name] = actual

    return actuator_values, trace_entries


def _sensor_key(sensor_name: str, value: float) -> str:
    """Build a human-readable key for a sensor probe."""
    if value == 1.0:
        return f"{sensor_name}=1"
    return f"{sensor_name}={value:g}"


def _describe_behavior(
    sensor_name: str, sensor_desc: str, value: float,
    actuator_responses: dict[str, float],
) -> str:
    """Generate a natural-language sentence for one probe result."""
    if not actuator_responses:
        return ""

    parts = []
    for output_name, out_val in sorted(actuator_responses.items()):
        # Parse "Block [Room].Connector"
        if "." in output_name:
            block_part, connector = output_name.rsplit(".", 1)
        else:
            block_part, connector = output_name, ""

        action = _ACTUATOR_DESCRIPTIONS.get(connector, f"receives {out_val:g}")
        if out_val < 0.01:
            action = "turns off" if connector in ("I1", "I") else f"receives 0"

        parts.append(f"{block_part} {action}")

    trigger = sensor_desc if sensor_desc else f"{sensor_name} is {value:g}"
    return f"When {trigger}: {', '.join(parts)}."


def _diff_from_baseline(
    actuator_values: dict[str, float],
    baseline_values: dict[str, float],
) -> dict[str, float]:
    """Return only actuator values that differ from baseline."""
    responses: dict[str, float] = {}
    for name, val in actuator_values.items():
        baseline_val = baseline_values.get(name, 0.0)
        if abs(val - baseline_val) > 1e-6:
            responses[name] = val
    return responses


def probe_circuit(
    config_path: str,
    verbose: bool = False,
) -> tuple[dict[str, dict[str, float]], str]:
    """
    Probe fixture sensors and record actuator responses.

    Phase 1: Single-sensor probes (one sensor active, all others at 0).
    Phase 2: All-sensors-active probe (catches AND logic).
    Phase 3: Pairwise combinations of sensors whose single probes activated
             intermediate blocks but no actuators.

    Returns:
        (behavior_map, behavior_summary)
        - behavior_map: {"SensorName=value": {"Actuator.Connector": value, ...}}
        - behavior_summary: multi-line natural-language description
    """
    lox_bin = _find_lox_binary()
    behavior_map: dict[str, dict[str, float]] = {}
    summary_lines: list[str] = []

    # Run a baseline probe with all inputs at 0 to capture default state
    baseline_values, _ = _run_sim_probe(config_path, {}, lox_bin)

    # Track sensors that caused intermediate block activity but no actuator change
    sensors_with_intermediate_only: list[tuple[str, float, str]] = []

    # Phase 1: Single-sensor probes
    for sensor_name, test_value, description in PROBE_SENSORS:
        inputs = {sensor_name: test_value}
        if verbose:
            print(f"  [probe] {sensor_name}={test_value:g}", flush=True)

        actuator_values, trace = _run_sim_probe(config_path, inputs, lox_bin)
        actuator_responses = _diff_from_baseline(actuator_values, baseline_values)

        if actuator_responses:
            key = _sensor_key(sensor_name, test_value)
            behavior_map[key] = actuator_responses
            line = _describe_behavior(
                sensor_name, description, test_value, actuator_responses
            )
            if line:
                summary_lines.append(line)
        else:
            # Check if this sensor caused any non-sensor intermediate output
            sensor_outputs = {f"{sensor_name}.AQ", f"{sensor_name}.Q",
                              sensor_name, f"{sensor_name}.OutputPresence"}
            has_intermediate = any(
                isinstance(e, dict) and e.get("output", "") not in sensor_outputs
                for e in trace
                if isinstance(e, dict) and "output" in e
            )
            if has_intermediate:
                sensors_with_intermediate_only.append(
                    (sensor_name, test_value, description)
                )

    # Phase 2: All-sensors-active probe (catches multi-condition AND logic)
    all_inputs = {name: val for name, val, _ in PROBE_SENSORS}
    if verbose:
        print("  [probe] ALL sensors active", flush=True)
    all_values, _ = _run_sim_probe(config_path, all_inputs, lox_bin)
    all_responses = _diff_from_baseline(all_values, baseline_values)
    if all_responses:
        behavior_map["ALL_ACTIVE"] = all_responses
        parts = []
        for out_name, out_val in sorted(all_responses.items()):
            if "." in out_name:
                block_part, connector = out_name.rsplit(".", 1)
            else:
                block_part, connector = out_name, ""
            action = _ACTUATOR_DESCRIPTIONS.get(connector, f"receives {out_val:g}")
            if out_val < 0.01:
                action = "turns off" if connector in ("I1", "I") else "receives 0"
            parts.append(f"{block_part} {action}")
        summary_lines.append(
            f"When all sensors are active simultaneously: {', '.join(parts)}."
        )

    # Phase 3: Pairwise probes for sensors that had intermediate-only activity.
    # This discovers which sensor pairs trigger actuators (AND logic patterns).
    if len(sensors_with_intermediate_only) >= 2 and all_responses:
        # Only probe pairs if the all-active probe found actuator responses,
        # meaning there IS multi-condition logic to discover.
        pairs_tested = 0
        max_pairs = 50  # cap to avoid combinatorial explosion
        for i, (s1_name, s1_val, s1_desc) in enumerate(sensors_with_intermediate_only):
            if pairs_tested >= max_pairs:
                break
            for s2_name, s2_val, s2_desc in sensors_with_intermediate_only[i + 1:]:
                if pairs_tested >= max_pairs:
                    break
                pair_inputs = {s1_name: s1_val, s2_name: s2_val}
                if verbose:
                    print(f"  [probe] {s1_name} + {s2_name}", flush=True)
                pair_values, _ = _run_sim_probe(config_path, pair_inputs, lox_bin)
                pair_responses = _diff_from_baseline(pair_values, baseline_values)
                if pair_responses:
                    key = f"{_sensor_key(s1_name, s1_val)} + {_sensor_key(s2_name, s2_val)}"
                    behavior_map[key] = pair_responses
                    parts = []
                    for out_name, out_val in sorted(pair_responses.items()):
                        if "." in out_name:
                            block_part, connector = out_name.rsplit(".", 1)
                        else:
                            block_part, connector = out_name, ""
                        action = _ACTUATOR_DESCRIPTIONS.get(connector, f"receives {out_val:g}")
                        if out_val < 0.01:
                            action = "turns off" if connector in ("I1", "I") else "receives 0"
                        parts.append(f"{block_part} {action}")
                    summary_lines.append(
                        f"When {s1_desc} AND {s2_desc}: {', '.join(parts)}."
                    )
                pairs_tested += 1

    summary = "\n".join(summary_lines) if summary_lines else "No actuator responses detected for any sensor probe."
    return behavior_map, summary


def judge_with_llm(
    utterance: str,
    behavior_summary: str,
    agent_backend: str = "copilot",
) -> dict[str, Any]:
    """
    Ask an LLM judge whether the circuit behavior matches the utterance intent.

    This is a pluggable stub — the actual LLM call will be added in a follow-up.
    For now, returns a placeholder result with the prompt that would be sent.

    Returns:
        {"pass": None, "verdict": "pending", "prompt": "...", "explanation": ""}
    """
    prompt = (
        f'The user asked: "{utterance}"\n'
        f"\n"
        f"The agent built a circuit that behaves like this:\n"
        f"{behavior_summary}\n"
        f"\n"
        f"Does this circuit correctly implement what the user asked for? "
        f"Answer YES or NO with a brief explanation."
    )

    # TODO: implement actual LLM judge call using agent_backend
    # For now, return the structured prompt so callers can inspect it.
    return {
        "pass": None,
        "verdict": "pending",
        "prompt": prompt,
        "explanation": "LLM judge not yet implemented — see behavior_map for manual review.",
    }


def evaluate_by_trace(
    config_path: str,
    utterance: str,
    agent_backend: str = "copilot",
    verbose: bool = False,
) -> dict[str, Any]:
    """
    Trace-based evaluation: probe the circuit, build behavior map, optionally judge.

    Args:
        config_path: path to the .Loxone config file the agent produced
        utterance: the original user request
        agent_backend: which LLM backend to use for judging (future)
        verbose: print probe progress

    Returns:
        {
            "pass": bool | None,
            "behavior_map": {...},
            "summary": "...",
            "sensor_probes": int,
            "responding_sensors": int,
            "trace_judge": {...},
        }
    """
    behavior_map, summary = probe_circuit(config_path, verbose=verbose)

    judge_result = judge_with_llm(utterance, summary, agent_backend)

    # For now, pass is based on whether any actuator responded at all.
    # A circuit that produces zero actuator changes is almost certainly wrong.
    has_responses = len(behavior_map) > 0
    # judge_result["pass"] will be None until LLM judge is implemented;
    # use has_responses as a weak heuristic.
    effective_pass = judge_result["pass"] if judge_result["pass"] is not None else has_responses

    return {
        "pass": effective_pass,
        "behavior_map": behavior_map,
        "summary": summary,
        "sensor_probes": len(PROBE_SENSORS),
        "responding_sensors": len(behavior_map),
        "trace_judge": judge_result,
    }
