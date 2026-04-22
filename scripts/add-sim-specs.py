#!/usr/bin/env python3
"""Add simulation specs to eval cases that are missing them.

Reads each case file in tests/eval/cases/, generates appropriate simulation
specs based on the utterance, expected blocks, wiring, and params, then
saves the updated case files.

Also adds specs to progressive/challenges.json stages.
"""

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CASES_DIR = ROOT / "tests" / "eval" / "cases"
PROGRESSIVE_DIR = ROOT / "tests" / "eval" / "progressive"

# ---------------------------------------------------------------------------
# Fixture sensors available (from tests/eval/fixture.Loxone)
# ---------------------------------------------------------------------------
FIXTURE_SENSORS = {
    "Außentemperatur": "AQ",
    "Sonnenschein": "AQ",
    "Regen": "AQ",
    "Wind": "AQ",
    "Luftfeuchtigkeit": "AQ",
    "Bewegungsmelder": "OutputPresence",
    "Türkontakt Eingang": "Q",
    "Türklingel": "Q",
    "Pool Temperatur": "AQ",
}

# Map non-fixture sensor titles that appear in wiring to fixture equivalents
SENSOR_ALIAS_MAP = {
    "Temperature": ("Außentemperatur", "AQ"),
    "Temperatur": ("Außentemperatur", "AQ"),
    "Temp": ("Außentemperatur", "AQ"),
    "Humidity": ("Luftfeuchtigkeit", "AQ"),
    "Feuchtesensor": ("Luftfeuchtigkeit", "AQ"),
    "Feuchtesensor Garten": ("Luftfeuchtigkeit", "AQ"),
    "Feuchte": ("Luftfeuchtigkeit", "AQ"),
    "Helligkeit": ("Sonnenschein", "AQ"),
    "Brightness": ("Sonnenschein", "AQ"),
    "Motion": ("Bewegungsmelder", "OutputPresence"),
    "Motion Ceiling": ("Bewegungsmelder", "OutputPresence"),
    "Motion Ceiling Lights": ("Bewegungsmelder", "OutputPresence"),
    "Presence": ("Bewegungsmelder", "OutputPresence"),
    "Contact": ("Türkontakt Eingang", "Q"),
    "Window": ("Türkontakt Eingang", "Q"),
    "Türkontakt": ("Türkontakt Eingang", "Q"),
    "Garagentor Sensor": ("Türkontakt Eingang", "Q"),
    "Bewegungsmelder Garten": ("Bewegungsmelder", "OutputPresence"),
    "Windgeschwindigkeit": ("Wind", "AQ"),
    "CO2 Sensor": ("Luftfeuchtigkeit", "AQ"),  # closest analog
    "Pool Temp": ("Pool Temperatur", "AQ"),
    "Regensensor": ("Regen", "AQ"),
    # Switches and controllers — map to closest binary fixture sensor
    "Schalter 1": ("Türkontakt Eingang", "Q"),
    "Schalter 2": ("Türkontakt Eingang", "Q"),
    "Schalter": ("Türkontakt Eingang", "Q"),
    "T5": ("Türkontakt Eingang", "Q"),
    "Dial key 1": ("Türkontakt Eingang", "Q"),
    "Input 2": ("Türkontakt Eingang", "Q"),
    # Light controller output used as trigger (e.g. fan after light off)
    "Lichtsteuerung": ("Bewegungsmelder", "OutputPresence"),
    # Night/Away/Leaving modes — binary triggers
    "Night": ("Türkontakt Eingang", "Q"),
    "Away": ("Türkontakt Eingang", "Q"),
    "Leaving House": ("Türkontakt Eingang", "Q"),
    "Goodnight": ("Türkontakt Eingang", "Q"),
    "Authentication Status": ("Türkontakt Eingang", "Q"),
    # Bewässerung Manuell (manual irrigation switch)
    "Bewässerung Manuell": ("Türkontakt Eingang", "Q"),
    "No Cleaning": ("Türkontakt Eingang", "Q"),
    # Presence variants
    "Präsenz": ("Bewegungsmelder", "OutputPresence"),
    "Presense": ("Bewegungsmelder", "OutputPresence"),
}

# Utterance keyword → fixture sensor mapping
UTTERANCE_SENSOR_MAP = {
    "temperatur": ("Außentemperatur", "AQ"),
    "temperature": ("Außentemperatur", "AQ"),
    "temp ": ("Außentemperatur", "AQ"),
    "frost": ("Außentemperatur", "AQ"),
    "kalt": ("Außentemperatur", "AQ"),
    "cold": ("Außentemperatur", "AQ"),
    "warm": ("Außentemperatur", "AQ"),
    "hot": ("Außentemperatur", "AQ"),
    "heiß": ("Außentemperatur", "AQ"),
    "heating": ("Außentemperatur", "AQ"),
    "heiz": ("Außentemperatur", "AQ"),
    "°c": ("Außentemperatur", "AQ"),
    "degree": ("Außentemperatur", "AQ"),
    "grad": ("Außentemperatur", "AQ"),
    "sonn": ("Sonnenschein", "AQ"),
    "sun": ("Sonnenschein", "AQ"),
    "bright": ("Sonnenschein", "AQ"),
    "hell": ("Sonnenschein", "AQ"),
    "dunkel": ("Sonnenschein", "AQ"),
    "dark": ("Sonnenschein", "AQ"),
    "dämmer": ("Sonnenschein", "AQ"),
    "dawn": ("Sonnenschein", "AQ"),
    "dusk": ("Sonnenschein", "AQ"),
    "sunset": ("Sonnenschein", "AQ"),
    "sunrise": ("Sonnenschein", "AQ"),
    "regen": ("Regen", "AQ"),
    "rain": ("Regen", "AQ"),
    "nass": ("Regen", "AQ"),
    "wet": ("Regen", "AQ"),
    "wind": ("Wind", "AQ"),
    "sturm": ("Wind", "AQ"),
    "storm": ("Wind", "AQ"),
    "humid": ("Luftfeuchtigkeit", "AQ"),
    "feucht": ("Luftfeuchtigkeit", "AQ"),
    "moisture": ("Luftfeuchtigkeit", "AQ"),
    "motion": ("Bewegungsmelder", "OutputPresence"),
    "bewegung": ("Bewegungsmelder", "OutputPresence"),
    "presence": ("Bewegungsmelder", "OutputPresence"),
    "anwesenheit": ("Bewegungsmelder", "OutputPresence"),
    "präsenz": ("Bewegungsmelder", "OutputPresence"),
    "person": ("Bewegungsmelder", "OutputPresence"),
    "sitting": ("Bewegungsmelder", "OutputPresence"),
    "someone": ("Bewegungsmelder", "OutputPresence"),
    "nobody": ("Bewegungsmelder", "OutputPresence"),
    "jemand": ("Bewegungsmelder", "OutputPresence"),
    "doorbell": ("Türklingel", "Q"),
    "klingel": ("Türklingel", "Q"),
    "door open": ("Türkontakt Eingang", "Q"),
    "door close": ("Türkontakt Eingang", "Q"),
    "tür of": ("Türkontakt Eingang", "Q"),
    "tür ge": ("Türkontakt Eingang", "Q"),
    "garage": ("Türkontakt Eingang", "Q"),
    "pool": ("Pool Temperatur", "AQ"),
    "switch": ("Türkontakt Eingang", "Q"),
    "schalter": ("Türkontakt Eingang", "Q"),
    "taster": ("Türkontakt Eingang", "Q"),
    "button": ("Türkontakt Eingang", "Q"),
    "press": ("Türkontakt Eingang", "Q"),
    "nfc": ("Türkontakt Eingang", "Q"),
    "smoke": ("Türkontakt Eingang", "Q"),
    "rauch": ("Türkontakt Eingang", "Q"),
    "co2": ("Luftfeuchtigkeit", "AQ"),
    "co₂": ("Luftfeuchtigkeit", "AQ"),
    "lüfter": ("Luftfeuchtigkeit", "AQ"),
    "ventilat": ("Luftfeuchtigkeit", "AQ"),
    "steamy": ("Luftfeuchtigkeit", "AQ"),
    "dampf": ("Luftfeuchtigkeit", "AQ"),
    "blind": ("Sonnenschein", "AQ"),
    "jalousie": ("Sonnenschein", "AQ"),
    "beschattung": ("Sonnenschein", "AQ"),
    "shade": ("Sonnenschein", "AQ"),
    "window": ("Sonnenschein", "AQ"),
    "fenster": ("Sonnenschein", "AQ"),
    "night light": ("Sonnenschein", "AQ"),
    "nachtlicht": ("Sonnenschein", "AQ"),
}

# Common actuator titles that appear as output targets
COMMON_ACTUATORS = {
    "Lichtsteuerung", "Jalousie 1", "Jalousie 2", "Jalousie 3",
    "Bewässerungsventil", "Gartenbeleuchtung", "Lüfter Bad",
    "Klimaanlage", "Raumregler", "Heizkörper Bad", "Poolpumpe",
    "Garagenlicht", "Garagentor", "Leinwand", "Türöffner",
    "Smart Actuator RGBW", "Relay", "Ventilator",
}

# Block types that are logic/intermediate (not actuators)
LOGIC_BLOCK_TYPES = {
    "And", "Or", "Not", "GreaterEqual", "Less", "Greater",
    "Mult", "Memory", "FlipFlop", "AMemory", "StairwayLS", "Monoflop",
    "OnPulseDelay", "OffDelay", "OnDelay", "PulseGen", "Counter",
    "State", "EdgeDetection", "Sub", "Add", "Div", "Mod",
    "AnalogComparator", "AnalogMultiplexer2", "Formula",
    "PulseAt", "PushButton", "Mode", "StepSel",
    "Validator", "Average", "Avg", "Minmax", "PulseBy",
    "StatusMonitor", "TextGenerator", "MailGen",
}


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def extract_threshold(utterance: str) -> float | None:
    """Try to extract a numeric threshold from the utterance."""
    patterns = [
        r'(?:above|over|exceed[s]?|greater\s+than|mehr\s+als|höher\s+als|über|oberhalb)\s+(\d+(?:\.\d+)?)',
        r'(?:below|under|less\s+than|weniger\s+als|niedriger\s+als|unter|unterhalb)\s+(\d+(?:\.\d+)?)',
        r'(?:reaches?|hits?|drops?\s+to|steigt\s+auf|sinkt\s+auf)\s+(\d+(?:\.\d+)?)',
        r'>\s*(\d+(?:\.\d+)?)',
        r'<\s*(\d+(?:\.\d+)?)',
        r'(\d+(?:\.\d+)?)\s*(?:°[CcFf]|degrees?|Grad|km/?h|%|percent|Prozent)',
        r'(?:threshold|Schwellwert|Grenzwert)\s+(?:of|von)?\s*(\d+(?:\.\d+)?)',
        r'(\d+(?:\.\d+)?)\s*(?:lux|Lux)',
    ]
    for pat in patterns:
        m = re.search(pat, utterance, re.IGNORECASE)
        if m:
            return float(m.group(1))
    return None


def extract_duration_seconds(utterance: str) -> int | None:
    """Try to extract a duration in seconds from the utterance."""
    m = re.search(r'(\d+)\s*(?:minute[ns]?|min\.?|Minuten?)', utterance, re.IGNORECASE)
    if m:
        return int(m.group(1)) * 60
    m = re.search(r'(\d+)\s*(?:second[s]?|sec\.?|Sekunden?)', utterance, re.IGNORECASE)
    if m:
        return int(m.group(1))
    m = re.search(r'(\d+)\s*(?:hour[s]?|Stunden?|hr)', utterance, re.IGNORECASE)
    if m:
        return int(m.group(1)) * 3600
    return None


def extract_percentage(utterance: str) -> float | None:
    """Extract a percentage from the utterance."""
    m = re.search(r'(\d+(?:\.\d+)?)\s*%', utterance)
    if m:
        return float(m.group(1))
    return None


def get_block_types(case: dict) -> list[str]:
    """Return list of block types from expected.new_blocks."""
    return [b.get("type", "") for b in case.get("expected", {}).get("new_blocks", [])]


def get_block_titles(case: dict) -> list[str]:
    """Return list of block titles from expected.new_blocks."""
    titles = []
    for b in case.get("expected", {}).get("new_blocks", []):
        t = b.get("title", b.get("title_contains", ""))
        if t:
            titles.append(t)
    return titles


def get_wiring(case: dict) -> list[dict]:
    """Return wiring list."""
    return case.get("expected", {}).get("wiring", [])


def get_params(case: dict) -> list[dict]:
    """Return params list."""
    p = case.get("expected", {}).get("params", [])
    if isinstance(p, dict):
        return [p] if p else []
    return p


def find_param_value(params: list[dict], param_name: str) -> float | None:
    """Find a specific parameter value."""
    for p in params:
        if p.get("param") == param_name:
            try:
                return float(p.get("value", 0))
            except (ValueError, TypeError):
                return None
    return None


def find_threshold_param(params: list[dict]) -> float | None:
    """Find a threshold/reference parameter."""
    for name in ["Input2", "Ref", "Threshold", "TH", "Input 2", "Reference",
                 "RefValue", "T", "ReferenceValue"]:
        val = find_param_value(params, name)
        if val is not None:
            return val
    return None


def is_above_threshold(utterance: str) -> bool:
    """Determine if the condition is 'above threshold' vs 'below threshold'."""
    utt_lower = utterance.lower()
    below_words = ["below", "under", "less than", "drop", "frost", "kalt", "cold",
                   "unter ", "niedriger", "weniger", "fällt", "sinkt", "kühl",
                   "cool", "niedrig"]
    for w in below_words:
        if w in utt_lower:
            return False
    return True


def get_default_threshold_for_sensor(sensor_key: str, above: bool) -> float:
    """Return a sensible default threshold based on the sensor type."""
    if "Außentemperatur" in sensor_key or "Pool" in sensor_key:
        return 20.0
    elif "Wind" in sensor_key:
        return 40.0
    elif "Luftfeuchtigkeit" in sensor_key:
        return 60.0
    elif "Sonnenschein" in sensor_key:
        return 0.5  # binary
    elif "Regen" in sensor_key:
        return 0.5
    elif "Bewegungsmelder" in sensor_key:
        return 0.5
    elif "Türkontakt" in sensor_key:
        return 0.5
    elif "Türklingel" in sensor_key:
        return 0.5
    return 10.0


def is_binary_sensor(sensor_key: str) -> bool:
    """Check if a sensor is binary (0/1)."""
    binary = ["Sonnenschein", "Regen", "Bewegungsmelder", "Türkontakt", "Türklingel"]
    return any(b in sensor_key for b in binary)


def _get_to_conn(w: dict) -> str:
    """Get to_connector, falling back to a default for known actuators."""
    conn = w.get("to_connector", w.get("to_conn", ""))
    if conn:
        return conn
    # If missing, infer from the target title
    to_title = w.get("to_title", "")
    defaults = {
        "Lichtsteuerung": "I1", "Jalousie 1": "InputTriggerDown",
        "Jalousie 2": "InputTriggerDown", "Jalousie 3": "InputTriggerDown",
        "Bewässerungsventil": "I1", "Gartenbeleuchtung": "I1",
        "Lüfter Bad": "I1", "Klimaanlage": "I1", "Raumregler": "Temp",
        "Heizkörper Bad": "I1", "Poolpumpe": "I1", "Garagenlicht": "I1",
        "Türöffner": "I1", "Leinwand": "InputTriggerDown",
    }
    return defaults.get(to_title, "I1") if to_title else ""


def _get_from_conn(w: dict) -> str:
    """Get from_connector, falling back to a default for known sensors."""
    conn = w.get("from_connector", w.get("from_conn", ""))
    if conn:
        return conn
    from_title = w.get("from_title", "")
    if from_title in FIXTURE_SENSORS:
        return FIXTURE_SENSORS[from_title]
    if from_title in SENSOR_ALIAS_MAP:
        return SENSOR_ALIAS_MAP[from_title][1]
    return ""


def find_output_target(wiring: list[dict], block_types: list[str],

                       block_titles: list[str],
                       case_blocks: list[dict] = None) -> tuple[str, str] | None:
    """Find the best output target from wiring.

    Priority: named actuators > named targets > type-based targets with connectors.
    """
    # Pass 1: wiring from a logic type to a named actuator
    for w in wiring:
        to_title = w.get("to_title", "")
        to_conn = _get_to_conn(w)
        from_type = w.get("from_type", "")
        if to_title and to_conn and from_type:
            if to_title not in LOGIC_BLOCK_TYPES:
                return (to_title, to_conn)

    # Pass 2: wiring from a named block to a named actuator
    for w in wiring:
        to_title = w.get("to_title", "")
        to_conn = _get_to_conn(w)
        from_title = w.get("from_title", "")
        if to_title and to_conn and from_title:
            if to_title not in LOGIC_BLOCK_TYPES and to_title not in FIXTURE_SENSORS \
               and to_title not in SENSOR_ALIAS_MAP:
                return (to_title, to_conn)

    # Pass 3: any wiring with a to_title that looks like an actuator
    for w in wiring:
        to_title = w.get("to_title", "")
        to_conn = _get_to_conn(w)
        if to_title and to_conn and to_title in COMMON_ACTUATORS:
            return (to_title, to_conn)

    # Pass 4: find the "terminal" block — a type that receives but doesn't send
    receiving_types = set()
    sending_types = set()
    for w in wiring:
        ft = w.get("from_type", "")
        tt = w.get("to_type", "")
        if ft:
            sending_types.add(ft)
        if tt:
            receiving_types.add(tt)

    terminal_types = receiving_types - sending_types
    for w in wiring:
        to_title = w.get("to_title", "")
        to_type = w.get("to_type", "")
        to_conn = _get_to_conn(w)
        if to_type in terminal_types and to_conn:
            if to_title:
                return (to_title, to_conn)
            for t in block_titles:
                if t:
                    return (t, to_conn)

    # Pass 5: any to_title with to_connector (excluding known sensors/aliases)
    for w in wiring:
        to_title = w.get("to_title", "")
        to_conn = _get_to_conn(w)
        if to_title and to_conn:
            if to_title not in FIXTURE_SENSORS and to_title not in SENSOR_ALIAS_MAP:
                return (to_title, to_conn)

    # Pass 6: last resort — from_type to to_title
    for w in reversed(wiring):
        to_title = w.get("to_title", "")
        to_conn = _get_to_conn(w)
        if to_title and to_conn:
            return (to_title, to_conn)

    # Pass 7: find output from type-only wiring with wildcard connectors
    for w in wiring:
        to_type = w.get("to_type", "")
        to_conn = w.get("to_connector", "")
        if to_type and to_conn and to_type not in LOGIC_BLOCK_TYPES:
            # Use the type name as the title (actuator types like MusicPlayer, etc.)
            for b in (case_blocks or []):
                bt = b.get("type", "")
                btitle = b.get("title", b.get("title_contains", ""))
                if bt == to_type and btitle:
                    return (btitle, to_conn)

    return None


def determine_input_sensors(wiring: list[dict], utterance: str,
                            block_types: list[str] = None) -> dict[str, float]:
    """Determine which fixture sensor(s) to use. Returns {sensor_key: None}."""
    utt_lower = utterance.lower()
    inputs = {}

    # 1. Check wiring for direct fixture sensor connections
    for w in wiring:
        from_title = w.get("from_title", "")
        from_conn = _get_from_conn(w)
        if from_title in FIXTURE_SENSORS and from_conn:
            sensor_key = f"{from_title}.{from_conn}"
            inputs[sensor_key] = None

    # 2. Check wiring for aliased sensors
    if not inputs:
        for w in wiring:
            from_title = w.get("from_title", "")
            if from_title in SENSOR_ALIAS_MAP:
                fixture_name, fixture_conn = SENSOR_ALIAS_MAP[from_title]
                sensor_key = f"{fixture_name}.{fixture_conn}"
                inputs[sensor_key] = None

    # 3. Infer from utterance
    if not inputs:
        for keyword, (sensor, conn) in UTTERANCE_SENSOR_MAP.items():
            if keyword in utt_lower:
                key = f"{sensor}.{conn}"
                if key not in inputs:
                    inputs[key] = None

    # 4. For combined conditions, try to get multiple sensors
    if len(inputs) == 1 and block_types:
        has_and = "And" in block_types
        has_or = "Or" in block_types
        if has_and or has_or:
            # Try to find a second sensor from utterance
            first_sensor = list(inputs.keys())[0]
            for keyword, (sensor, conn) in UTTERANCE_SENSOR_MAP.items():
                if keyword in utt_lower:
                    key = f"{sensor}.{conn}"
                    if key != first_sensor and key not in inputs:
                        inputs[key] = None
                        break

    return inputs


def fill_sensor_values(inputs: dict, utterance: str, threshold: float | None,
                       above: bool, positive: bool) -> dict:
    """Fill sensor input dict with appropriate values for pos/neg test."""
    filled = {}
    for sensor_key in inputs:
        if is_binary_sensor(sensor_key):
            filled[sensor_key] = 1.0 if positive else 0.0
        else:
            t = threshold if threshold else get_default_threshold_for_sensor(sensor_key, above)
            if positive:
                filled[sensor_key] = t + 5.0 if above else t - 5.0
            else:
                filled[sensor_key] = t - 5.0 if above else t + 5.0
    return filled


# ---------------------------------------------------------------------------
# Sim spec generators by pattern
# ---------------------------------------------------------------------------

def gen_threshold_sim(case, block_types, wiring, params, block_titles) -> list[dict] | None:
    """Generate sim for threshold-based cases."""
    utterance = case.get("utterance", "")
    above = is_above_threshold(utterance)

    threshold = extract_threshold(utterance)
    if threshold is None:
        threshold = find_threshold_param(params)

    inputs = determine_input_sensors(wiring, utterance, block_types)
    if not inputs:
        return None

    output = find_output_target(wiring, block_types, block_titles)
    if not output:
        return None

    out_key = f"{output[0]}.{output[1]}"
    pos = fill_sensor_values(inputs, utterance, threshold, above, True)
    neg = fill_sensor_values(inputs, utterance, threshold, above, False)

    return [
        {
            "name": "should trigger when condition met",
            "inputs": pos,
            "ticks": 10, "dt": 0.1,
            "expected_outputs": {out_key: {">": 0.5}}
        },
        {
            "name": "should NOT trigger when condition not met",
            "inputs": neg,
            "ticks": 10, "dt": 0.1,
            "expected_outputs": {out_key: {"<": 0.5}}
        }
    ]


def gen_combined_sim(case, block_types, wiring, params, block_titles) -> list[dict] | None:
    """Generate sim for AND/OR combined conditions."""
    utterance = case.get("utterance", "")
    above = is_above_threshold(utterance)
    threshold = extract_threshold(utterance)

    inputs = determine_input_sensors(wiring, utterance, block_types)
    if not inputs:
        return None

    output = find_output_target(wiring, block_types, block_titles)
    if not output:
        return None

    out_key = f"{output[0]}.{output[1]}"
    is_and = "And" in block_types
    sensor_keys = list(inputs.keys())

    pos = fill_sensor_values(inputs, utterance, threshold, above, True)

    sims = [
        {
            "name": "should trigger when all conditions met",
            "inputs": pos,
            "ticks": 10, "dt": 0.1,
            "expected_outputs": {out_key: {">": 0.5}}
        }
    ]

    if is_and and len(sensor_keys) > 1:
        # Only first sensor fails
        partial = {}
        for i, sk in enumerate(sensor_keys):
            if i == 0:
                partial[sk] = 0.0 if is_binary_sensor(sk) else (
                    (threshold or get_default_threshold_for_sensor(sk, above)) - 5.0
                    if above else
                    (threshold or get_default_threshold_for_sensor(sk, above)) + 5.0
                )
            else:
                partial[sk] = pos[sk]
        sims.append({
            "name": "should NOT trigger when only partial conditions met",
            "inputs": partial,
            "ticks": 10, "dt": 0.1,
            "expected_outputs": {out_key: {"<": 0.5}}
        })
    else:
        neg = fill_sensor_values(inputs, utterance, threshold, above, False)
        sims.append({
            "name": "should NOT trigger when condition not met",
            "inputs": neg,
            "ticks": 10, "dt": 0.1,
            "expected_outputs": {out_key: {"<": 0.5}}
        })

    return sims


def gen_timer_sim(case, block_types, wiring, params, block_titles) -> list[dict] | None:
    """Generate sim for timer-based cases."""
    utterance = case.get("utterance", "")
    above = is_above_threshold(utterance)
    threshold = extract_threshold(utterance)
    duration = extract_duration_seconds(utterance)

    inputs = determine_input_sensors(wiring, utterance, block_types)
    if not inputs:
        return None

    output = find_output_target(wiring, block_types, block_titles)
    if not output:
        return None

    out_key = f"{output[0]}.{output[1]}"
    pos = fill_sensor_values(inputs, utterance, threshold, above, True)

    ticks = 10
    if duration and duration > 10:
        ticks = min(50, max(10, duration // 2))

    sims = [
        {
            "name": "should activate on trigger",
            "inputs": pos,
            "ticks": ticks, "dt": 0.1,
            "expected_outputs": {out_key: {">": 0.5}}
        },
        {
            "name": "should NOT activate without trigger",
            "inputs": {k: 0.0 for k in pos},
            "ticks": 10, "dt": 0.1,
            "expected_outputs": {out_key: {"<": 0.5}}
        }
    ]
    return sims


def gen_memory_sim(case, block_types, wiring, params, block_titles) -> list[dict] | None:
    """Generate sim for Memory/FlipFlop latching cases."""
    utterance = case.get("utterance", "")
    above = is_above_threshold(utterance)
    threshold = extract_threshold(utterance)

    inputs = determine_input_sensors(wiring, utterance, block_types)
    if not inputs:
        return None

    output = find_output_target(wiring, block_types, block_titles)
    if not output:
        return None

    out_key = f"{output[0]}.{output[1]}"
    pos = fill_sensor_values(inputs, utterance, threshold, above, True)
    neg = fill_sensor_values(inputs, utterance, threshold, above, False)

    return [
        {
            "name": "should latch on when set",
            "inputs": pos,
            "ticks": 10, "dt": 0.1,
            "expected_outputs": {out_key: {">": 0.5}}
        },
        {
            "name": "should NOT latch without trigger",
            "inputs": neg,
            "ticks": 10, "dt": 0.1,
            "expected_outputs": {out_key: {"<": 0.5}}
        }
    ]


def gen_mult_sim(case, block_types, wiring, params, block_titles) -> list[dict] | None:
    """Generate sim for multiplication/scaling cases."""
    utterance = case.get("utterance", "")
    inputs = determine_input_sensors(wiring, utterance, block_types)
    if not inputs:
        return None

    output = find_output_target(wiring, block_types, block_titles)
    if not output:
        return None

    out_key = f"{output[0]}.{output[1]}"
    pos = {}
    for sk in inputs:
        pos[sk] = 1.0 if is_binary_sensor(sk) else 20.0

    return [
        {
            "name": "should produce scaled output when input active",
            "inputs": pos,
            "ticks": 10, "dt": 0.1,
            "expected_outputs": {out_key: {">": 0.0}}
        },
        {
            "name": "should produce no output with zero input",
            "inputs": {k: 0.0 for k in pos},
            "ticks": 10, "dt": 0.1,
            "expected_outputs": {out_key: {"<": 0.5}}
        }
    ]


def gen_daytimer_sim(case, block_types, wiring, params, block_titles) -> list[dict] | None:
    """Generate sim for DayTimer/PulseAt-based cases. Time-based, ticks=0."""
    output = find_output_target(wiring, block_types, block_titles)
    if not output:
        output = infer_output_from_utterance(
            case.get("utterance", ""), block_types, block_titles)
    if not output:
        return None

    out_key = f"{output[0]}.{output[1]}"
    return [
        {
            "name": "should activate during scheduled period",
            "inputs": {},
            "ticks": 0, "dt": 0.1,
            "expected_outputs": {out_key: {">": 0.5}}
        }
    ]


def gen_not_sim(case, block_types, wiring, params, block_titles) -> list[dict] | None:
    """Generate sim for NOT/inversion cases."""
    utterance = case.get("utterance", "")
    threshold = extract_threshold(utterance)

    inputs = determine_input_sensors(wiring, utterance, block_types)
    if not inputs:
        return None

    output = find_output_target(wiring, block_types, block_titles)
    if not output:
        return None

    out_key = f"{output[0]}.{output[1]}"
    high = {}
    low = {}
    for sk in inputs:
        if is_binary_sensor(sk):
            high[sk] = 1.0
            low[sk] = 0.0
        else:
            t = threshold or get_default_threshold_for_sensor(sk, True)
            high[sk] = t + 5.0
            low[sk] = t - 5.0

    return [
        {
            "name": "should output low when input high (NOT)",
            "inputs": high,
            "ticks": 10, "dt": 0.1,
            "expected_outputs": {out_key: {"<": 0.5}}
        },
        {
            "name": "should output high when input low (NOT)",
            "inputs": low,
            "ticks": 10, "dt": 0.1,
            "expected_outputs": {out_key: {">": 0.5}}
        }
    ]


def gen_presence_sim(case, block_types, wiring, params, block_titles) -> list[dict] | None:
    """Generate sim for presence/motion-based cases."""
    output = find_output_target(wiring, block_types, block_titles)
    if not output:
        return None
    out_key = f"{output[0]}.{output[1]}"
    return [
        {
            "name": "should activate on presence",
            "inputs": {"Bewegungsmelder.OutputPresence": 1.0},
            "ticks": 10, "dt": 0.1,
            "expected_outputs": {out_key: {">": 0.5}}
        },
        {
            "name": "should NOT activate without presence",
            "inputs": {"Bewegungsmelder.OutputPresence": 0.0},
            "ticks": 10, "dt": 0.1,
            "expected_outputs": {out_key: {"<": 0.5}}
        }
    ]


def gen_doorbell_sim(case, block_types, wiring, params, block_titles) -> list[dict] | None:
    """Generate sim for doorbell-triggered cases."""
    output = find_output_target(wiring, block_types, block_titles)
    if not output:
        return None
    out_key = f"{output[0]}.{output[1]}"
    return [
        {
            "name": "should activate on doorbell",
            "inputs": {"Türklingel.Q": 1.0},
            "ticks": 10, "dt": 0.1,
            "expected_outputs": {out_key: {">": 0.5}}
        },
        {
            "name": "should NOT activate without doorbell",
            "inputs": {"Türklingel.Q": 0.0},
            "ticks": 10, "dt": 0.1,
            "expected_outputs": {out_key: {"<": 0.5}}
        }
    ]


def gen_door_sim(case, block_types, wiring, params, block_titles) -> list[dict] | None:
    """Generate sim for door contact-triggered cases."""
    output = find_output_target(wiring, block_types, block_titles)
    if not output:
        return None
    out_key = f"{output[0]}.{output[1]}"
    duration = extract_duration_seconds(case.get("utterance", ""))
    ticks = min(duration * 10, 6000) if duration and duration > 10 else 10
    return [
        {
            "name": "should activate when door opens",
            "inputs": {"Türkontakt Eingang.Q": 1.0},
            "ticks": ticks, "dt": 0.1,
            "expected_outputs": {out_key: {">": 0.5}}
        },
        {
            "name": "should NOT activate when door closed",
            "inputs": {"Türkontakt Eingang.Q": 0.0},
            "ticks": 10, "dt": 0.1,
            "expected_outputs": {out_key: {"<": 0.5}}
        }
    ]


# ---------------------------------------------------------------------------
# Generators for cases without wiring (utterance + block types only)
# ---------------------------------------------------------------------------

def infer_output_from_utterance(utterance: str, block_types: list[str],
                                block_titles: list[str]) -> tuple[str, str] | None:
    """For cases without wiring, infer the output target from the utterance."""
    utt_lower = utterance.lower()

    # Map utterance keywords to actuator (title, connector)
    actuator_map = [
        (["blind", "jalousie", "beschattung", "shading", "rollo", "shade",
          "window", "fenster"],
         ("Jalousie 1", "InputTriggerDown")),
        (["light", "licht", "beleuchtung", "lampe", "dim", "hell", "night light",
          "nachtlicht", "bright"],
         ("Lichtsteuerung", "I1")),
        (["fan", "lüfter", "ventilat", "entfeuchter"],
         ("Lüfter Bad", "I1")),
        (["heat", "heiz", "radiator", "raumregl", "thermostat"],
         ("Raumregler", "Temp")),
        (["irrig", "bewässer", "water", "garten"],
         ("Bewässerungsventil", "I1")),
        (["ac", "klima", "air condition", "cooling", "kühl"],
         ("Klimaanlage", "I1")),
        (["garage", "tor"],
         ("Garagenlicht", "I1")),
        (["pool", "pump"],
         ("Poolpumpe", "I1")),
        (["door", "tür", "schloss", "lock"],
         ("Türöffner", "I1")),
        (["screen", "leinwand", "projector"],
         ("Leinwand", "I1")),
        (["music", "musik", "audio", "speaker"],
         ("Lichtsteuerung", "I1")),
    ]
    for keywords, target in actuator_map:
        for kw in keywords:
            if kw in utt_lower:
                return target

    # Try block titles
    for t in block_titles:
        if t:
            return (t, "I1")

    return None


def gen_no_wiring_sim(case, block_types, wiring, params, block_titles) -> list[dict] | None:
    """Generate sim for cases with no wiring — infer everything from utterance."""
    utterance = case.get("utterance", "")
    above = is_above_threshold(utterance)
    threshold = extract_threshold(utterance)

    inputs = determine_input_sensors([], utterance, block_types)
    if not inputs:
        return None

    output = infer_output_from_utterance(utterance, block_types, block_titles)
    if not output:
        return None

    out_key = f"{output[0]}.{output[1]}"
    pos = fill_sensor_values(inputs, utterance, threshold, above, True)
    neg = fill_sensor_values(inputs, utterance, threshold, above, False)

    return [
        {
            "name": "should trigger when condition met",
            "inputs": pos,
            "ticks": 10, "dt": 0.1,
            "expected_outputs": {out_key: {">": 0.5}}
        },
        {
            "name": "should NOT trigger when condition not met",
            "inputs": neg,
            "ticks": 10, "dt": 0.1,
            "expected_outputs": {out_key: {"<": 0.5}}
        }
    ]


# ---------------------------------------------------------------------------
# Pattern detection and sim spec dispatch
# ---------------------------------------------------------------------------

def detect_pattern(case: dict) -> str:
    """Detect the primary pattern of a case for sim spec generation."""
    utterance = case.get("utterance", "").lower()
    block_types = get_block_types(case)
    patterns = case.get("pattern", [])

    if isinstance(patterns, list):
        if "threshold" in patterns and "combined-condition" in patterns:
            return "combined-threshold"
        if "threshold" in patterns:
            return "threshold"
        if "combined-condition" in patterns:
            return "combined"

    has_and = "And" in block_types
    has_or = "Or" in block_types
    has_not = "Not" in block_types
    has_ge = "GreaterEqual" in block_types
    has_less = "Less" in block_types
    has_greater = "Greater" in block_types
    has_timer = any(t in block_types for t in
                    ["StairwayLS", "Monoflop", "OnPulseDelay", "OffDelay", "OnDelay",
                     "OnOffDelay"])
    has_memory = any(t in block_types for t in ["Memory", "FlipFlop", "AMemory",
                                                 "SRFlipFlop"])
    has_mult = "Mult" in block_types
    has_daytimer = "DayTimer" in block_types or "PulseAt" in block_types
    has_threshold = has_ge or has_less or has_greater
    has_analog_cmp = "AnalogComparator" in block_types

    # Utterance keywords
    is_presence = any(w in utterance for w in
                      ["motion", "presence", "bewegung", "anwesenheit", "präsenz",
                       "person", "sitting", "someone", "nobody", "jemand"])
    is_doorbell = any(w in utterance for w in ["doorbell", "klingel"])
    is_door = any(w in utterance for w in
                  ["door open", "tür of", "tür ge", "türkontakt", "door contact",
                   "garage"])
    is_switch = any(w in utterance for w in
                    ["switch", "schalter", "taster", "button", "press"])
    is_schedule = any(w in utterance for w in
                      ["every day", "schedule", "zeitplan", "um ", "at ",
                       "between", "zwischen", "täglich", "morgens", "abends",
                       "am ", "pm ", "uhr", "o'clock", "weekday", "weekend",
                       "wochenende"])

    if has_threshold and (has_and or has_or):
        return "combined-threshold"
    if has_analog_cmp and (has_and or has_or):
        return "combined-threshold"
    if has_not and has_threshold:
        return "not-threshold"
    if has_not and not has_threshold and not has_and:
        return "not"
    if has_threshold:
        return "threshold"
    if has_analog_cmp:
        return "threshold"
    if has_and or has_or:
        return "combined"
    if has_timer and has_mult:
        return "timer"  # timer with dimming
    if has_timer:
        return "timer"
    if has_memory:
        return "memory"
    if has_daytimer:
        return "daytimer"
    if has_mult:
        return "mult"
    if is_presence:
        return "presence"
    if is_doorbell:
        return "doorbell"
    if is_door:
        return "door"
    if is_schedule:
        return "daytimer"
    if is_switch:
        return "timer"  # switch press → output

    # Fallback: utterance-based patterns
    temp_words = ["temperatur", "temperature", "temp ", "°c", "degree", "grad",
                  "frost", "heating", "heiz", "cooling", "kühl", "warm", "cold",
                  "kalt", "heiß", "hot"]
    if any(w in utterance for w in temp_words):
        return "threshold"
    if any(w in utterance for w in ["wind", "sturm", "storm"]):
        return "threshold"
    if any(w in utterance for w in ["humid", "feucht", "moisture"]):
        return "threshold"
    if any(w in utterance for w in ["rain", "regen", "nass", "wet"]):
        return "threshold"
    if any(w in utterance for w in ["sun", "sonn", "bright", "hell", "dunkel",
                                     "dark", "dämmer", "dawn", "dusk",
                                     "sunset", "sunrise"]):
        return "threshold"
    if any(w in utterance for w in ["pool"]):
        return "threshold"
    if any(w in utterance for w in ["blind", "jalousie", "beschattung", "rollo"]):
        return "threshold"
    if any(w in utterance for w in ["light", "licht", "beleuchtung", "lampe"]):
        return "threshold"
    if any(w in utterance for w in ["alarm", "sicher", "security"]):
        return "memory"
    if any(w in utterance for w in ["disable", "override", "sperre"]):
        return "threshold"
    if any(w in utterance for w in ["co2", "co₂"]):
        return "threshold"

    return "unknown"


GENERATORS = {
    "threshold": gen_threshold_sim,
    "combined-threshold": gen_combined_sim,
    "combined": gen_combined_sim,
    "not-threshold": gen_not_sim,
    "not": gen_not_sim,
    "timer": gen_timer_sim,
    "memory": gen_memory_sim,
    "mult": gen_mult_sim,
    "daytimer": gen_daytimer_sim,
    "presence": gen_presence_sim,
    "doorbell": gen_doorbell_sim,
    "door": gen_door_sim,
}


def generate_sim_spec(case: dict) -> list[dict] | None:
    """Generate simulation spec(s) for a case."""
    if case.get("expected", {}).get("simulation"):
        return None

    block_types = get_block_types(case)
    block_titles = get_block_titles(case)
    wiring = get_wiring(case)
    params = get_params(case)
    pattern = detect_pattern(case)

    gen = GENERATORS.get(pattern)
    if gen:
        result = gen(case, block_types, wiring, params, block_titles)
        if result:
            return result

    # Fallback 1: generic with wiring
    if wiring:
        result = gen_fallback_with_wiring(case, block_types, wiring, params, block_titles)
        if result:
            return result

    # Fallback 2: no wiring — infer from utterance + blocks
    result = gen_no_wiring_sim(case, block_types, wiring, params, block_titles)
    if result:
        return result

    # Fallback 3: if we have wiring with output but couldn't find inputs,
    # use utterance-based inputs with wiring-based output
    utterance = case.get("utterance", "")
    if wiring:
        output = find_output_target(wiring, block_types, block_titles)
        if not output:
            output = infer_output_from_utterance(utterance, block_types, block_titles)
        inputs = determine_input_sensors(wiring, utterance, block_types)
        if not inputs:
            inputs = determine_input_sensors([], utterance, block_types)
        if output and inputs:
            above = is_above_threshold(utterance)
            threshold = extract_threshold(utterance)
            out_key = f"{output[0]}.{output[1]}"
            pos = fill_sensor_values(inputs, utterance, threshold, above, True)
            neg = fill_sensor_values(inputs, utterance, threshold, above, False)
            return [
                {
                    "name": "should activate when condition met",
                    "inputs": pos,
                    "ticks": 10, "dt": 0.1,
                    "expected_outputs": {out_key: {">": 0.5}}
                },
                {
                    "name": "should NOT activate when condition not met",
                    "inputs": neg,
                    "ticks": 10, "dt": 0.1,
                    "expected_outputs": {out_key: {"<": 0.5}}
                }
            ]
        # If have output but truly no inputs → time-based or Mult
        if output and not inputs:
            out_key = f"{output[0]}.{output[1]}"
            # For Mult blocks, use a generic analog input
            if "Mult" in block_types:
                return [
                    {
                        "name": "should scale output when active",
                        "inputs": {"Bewegungsmelder.OutputPresence": 1.0},
                        "ticks": 10, "dt": 0.1,
                        "expected_outputs": {out_key: {">": 0.0}}
                    },
                    {
                        "name": "should produce no output when inactive",
                        "inputs": {"Bewegungsmelder.OutputPresence": 0.0},
                        "ticks": 10, "dt": 0.1,
                        "expected_outputs": {out_key: {"<": 0.5}}
                    }
                ]
            # For time-window cases (GreaterEqual+Less+Or), treat as schedule
            if ("GreaterEqual" in block_types and "Less" in block_types) or \
               "DayTimer" in block_types:
                return [
                    {
                        "name": "should activate during scheduled period",
                        "inputs": {},
                        "ticks": 0, "dt": 0.1,
                        "expected_outputs": {out_key: {">": 0.5}}
                    }
                ]
            # For State blocks or other non-input blocks, use generic trigger
            return [
                {
                    "name": "should produce output when triggered",
                    "inputs": {"Türkontakt Eingang.Q": 1.0},
                    "ticks": 10, "dt": 0.1,
                    "expected_outputs": {out_key: {">": 0.0}}
                },
                {
                    "name": "should be inactive without trigger",
                    "inputs": {"Türkontakt Eingang.Q": 0.0},
                    "ticks": 10, "dt": 0.1,
                    "expected_outputs": {out_key: {"<": 0.5}}
                }
            ]

    return None


def gen_fallback_with_wiring(case, block_types, wiring, params, block_titles) -> list[dict] | None:
    """Generic fallback using wiring analysis."""
    utterance = case.get("utterance", "")
    above = is_above_threshold(utterance)
    threshold = extract_threshold(utterance)

    inputs = determine_input_sensors(wiring, utterance, block_types)
    output = find_output_target(wiring, block_types, block_titles)

    if not inputs or not output:
        return None

    out_key = f"{output[0]}.{output[1]}"
    pos = fill_sensor_values(inputs, utterance, threshold, above, True)
    neg = fill_sensor_values(inputs, utterance, threshold, above, False)

    return [
        {
            "name": "should activate when condition met",
            "inputs": pos,
            "ticks": 10, "dt": 0.1,
            "expected_outputs": {out_key: {">": 0.5}}
        },
        {
            "name": "should NOT activate when condition not met",
            "inputs": neg,
            "ticks": 10, "dt": 0.1,
            "expected_outputs": {out_key: {"<": 0.5}}
        }
    ]


# ---------------------------------------------------------------------------
# Progressive challenges handler
# ---------------------------------------------------------------------------

def generate_challenge_stage_sim(stage: dict) -> list[dict] | None:
    """Generate sim specs for a challenge stage."""
    if stage.get("expected", {}).get("simulation"):
        return None

    expected = stage.get("expected", {})
    params_raw = expected.get("params", {})
    params = []
    if isinstance(params_raw, dict):
        for title, pdict in params_raw.items():
            if isinstance(pdict, dict):
                for k, v in pdict.items():
                    params.append({"title_contains": title, "param": k, "value": v})
    elif isinstance(params_raw, list):
        params = params_raw

    pseudo = {
        "utterance": stage.get("utterance", stage.get("name", "")),
        "expected": {
            "new_blocks": expected.get("new_blocks", []),
            "wiring": expected.get("wiring", []),
            "params": params,
        }
    }
    return generate_sim_spec(pseudo)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def process_case_files():
    """Process all case files and add simulation specs."""
    total_added = 0
    total_cases = 0
    total_already = 0
    total_skipped = 0
    total_converted = 0

    for filepath in sorted(CASES_DIR.glob("*.json")):
        with open(filepath) as f:
            cases = json.load(f)

        modified = False
        file_added = 0
        for case in cases:
            total_cases += 1
            sim = case.get("expected", {}).get("simulation")
            if sim:
                # Convert legacy single-dict format to array with negative test
                if isinstance(sim, dict):
                    new_sims = [dict(sim, name="should trigger when condition met")]
                    # Generate a negative test from the positive
                    neg_inputs = {}
                    for k, v in sim.get("inputs", {}).items():
                        if is_binary_sensor(k):
                            neg_inputs[k] = 0.0 if v > 0.5 else 1.0
                        else:
                            neg_inputs[k] = v - 10.0 if v > 10 else v + 10.0
                    neg_outputs = {}
                    for k, comps in sim.get("expected_outputs", {}).items():
                        if ">" in comps:
                            neg_outputs[k] = {"<": 0.5}
                        elif ">=" in comps:
                            neg_outputs[k] = {"<": comps.get(">=", 0.5)}
                        else:
                            neg_outputs[k] = {">": 0.5}
                    if neg_inputs:
                        new_sims.append({
                            "name": "should NOT trigger when condition not met",
                            "inputs": neg_inputs,
                            "ticks": sim.get("ticks", 10),
                            "dt": sim.get("dt", 0.1),
                            "expected_outputs": neg_outputs
                        })
                    case["expected"]["simulation"] = new_sims
                    modified = True
                    total_converted += 1
                total_already += 1
                continue

            sim = generate_sim_spec(case)
            if sim:
                case.setdefault("expected", {})["simulation"] = sim
                modified = True
                file_added += 1
                total_added += 1
            else:
                total_skipped += 1

        if modified:
            with open(filepath, "w") as f:
                json.dump(cases, f, indent=2, ensure_ascii=False)
                f.write("\n")
            if file_added:
                print(f"  {filepath.name}: added {file_added} sim specs")

    return total_added, total_cases, total_already, total_skipped, total_converted


def process_progressive_challenges():
    """Process progressive/challenges.json stages."""
    filepath = PROGRESSIVE_DIR / "challenges.json"
    if not filepath.exists():
        print("  progressive/challenges.json not found, skipping")
        return 0

    with open(filepath) as f:
        challenges = json.load(f)

    total_added = 0
    for challenge in challenges:
        for stage in challenge.get("stages", []):
            sim = generate_challenge_stage_sim(stage)
            if sim:
                stage.setdefault("expected", {})["simulation"] = sim
                total_added += 1

    if total_added > 0:
        with open(filepath, "w") as f:
            json.dump(challenges, f, indent=2, ensure_ascii=False)
            f.write("\n")
        print(f"  challenges.json: added {total_added} stage sim specs")

    return total_added


def main():
    print("Adding simulation specs to eval cases...\n")

    print("Processing case files:")
    added, total, already, skipped, converted = process_case_files()

    print(f"\nProcessing progressive challenges:")
    challenge_added = process_progressive_challenges()

    print(f"\n--- Summary ---")
    print(f"Case files: {total} total cases")
    print(f"  Already had sim: {already}")
    print(f"  Converted to array format: {converted}")
    print(f"  Added sim specs: {added}")
    print(f"  Skipped (no sim possible): {skipped}")
    print(f"  Now with sim: {already + added}/{total}")
    print(f"Progressive challenges: {challenge_added} stages got sim specs")


if __name__ == "__main__":
    main()
