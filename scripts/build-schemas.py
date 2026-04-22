#!/usr/bin/env python3
"""Build structured element schemas from KB markdown articles.

Parses the Input/Output/Parameter/Property tables from the 165+ KB
articles that document Loxone function blocks, and outputs structured
JSON schemas for each type.

Usage:
    python3 scripts/build-schemas.py                 # build all
    python3 scripts/build-schemas.py --slug switch   # single article
"""

import argparse
import json
import os
import re
import sys

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
REPO_DIR = os.path.dirname(SCRIPT_DIR)

# Map KB slug → XML Type name (where they differ)
SLUG_TO_TYPE = {
    "2-position-controller": "PositionController2",
    "3-position-controller": "PositionController3",
    "aal-smart-alarm": "AalSmartAlarm",
    "ac-central-controller": "ACCentral",
    "ac-control": "ACControl",
    "access-controller": "AccessController",
    "add-2-way": "Add2",
    "add-4-way": "Add4",
    "alarm-chain": "AlarmChain",
    "alarm-clock": "AlarmClock",
    "analogue-memory": "Memory",
    "analogue-min-max-limiter": "AnalogLimiter",
    "analogue-multiplexer-2-way": "AnalogMultiplexer2",
    "analogue-multiplexer-4-way": "AnalogMultiplexer4",
    "analogue-watchdog": "AnalogWatchdog",
    "and": "And",
    "audio-central": "AudioCentral",
    "audio-player": "AudioPlayer",
    "audio-player-fixed-group": "AudioPlayerFixedGroup",
    "authentication-nfc-code-touch": "NFCCodeTouch",
    "automated-blinds": "Jalousie",
    "automated-blinds-integrated": "CentralJalousie",
    "automatic-blinds": "Jalousie",
    "automatic-blinds-integrated": "CentralJalousie",
    "automatic-rule": "AutoPilot",
    "average": "Average",
    "binary-decoder": "BinaryDecoder",
    "binary-decoder-2": "BinaryDecoder2",
    "binary-encoder": "BinaryEncoder",
    "brightness-control": "DaylightControl",
    "burglar-alarm": "Alarm",
    "call-generator": "CallGenerator",
    "climate-controller": "ClimateController",
    "combined-window-contact": "CombinedWindowContact",
    "command-recognition": "CommandRecognition",
    "comparator": "Comparator",
    "counter": "Counter",
    "custom-script-programming": "Script",
    "delayed-pulse": "DelayedPulse",
    "dewpoint-calculator": "DewpointCalculator",
    "differential-threshold-switch": "DiffThresholdSwitch",
    "dimmer": "Dimmer",
    "divide": "Divide",
    "door-controller": "DoorController",
    "door-window-monitor": "DoorWindowMonitor",
    "edge-detection": "EdgeDetection",
    "edge-triggered-wiping-relay": "WipingRelay",
    "eib-dimmer": "EIBDimmer",
    "eib-push-button": "EIBPushbutton",
    "eib-shading": "EIBJalousie",
    "emergency-alarm": "EmergencyAlarm",
    "energy-flow-monitor": "EnergyFlowMonitor",
    "energy-manager": "EnergyManager",
    "energy-manager-2": "EnergyManager2",
    "energy-monitor": "EnergyMonitor",
    "event-database-connector": "EventDBConnector",
    "exclusive-or": "ExclusiveOr",
    "fire-water-alarm": "FireWaterAlarm",
    "fixed-value-meter": "FixedValueMeter",
    "flipflop-rs": "FlipFlopRS",
    "flipflop-sr": "FlipFlopSR",
    "formula": "Formula",
    "fronius": "Fronius",
    "garagegate": "GateController",
    "gate": "Gate",
    "gate-overview": "GateCentral",
    "heating-curve": "HeatingCurve",
    "hotel-lighting-controller": "HotelLightController",
    "hvac-controller": "HVACController",
    "integer": "Integer",
    "intelligent-room-controller": "IRoomControllerV2",
    "intelligent-temperature-controller": "FlowTemperature",
    "intercom": "IntercomDevice",
    "intercom-2": "IntercomV2",
    "internorm-ventilation": "InternormVentilation",
    "ir-control": "IRController",
    "irrigation": "Irrigation",
    "leaf-ventilation": "LeafVentilation",
    "lighting-controller": "LightController2",
    "lighting-overview": "LightCentral",
    "load-manager": "LoadManager",
    "long-click": "LongClick",
    "mail-and-parcel-box": "MailBox",
    "mail-generator": "MailGenerator",
    "maintenance-counter": "RuntimeCounter",
    "media-controller": "MediaController",
    "memory-flags": "MemoryFlags",
    "meter": "Meter",
    "meter-bidirectional": "MeterBidirectional",
    "meter-storage": "MeterStorage",
    "minmax": "MinMax",
    "minmax-since-reset": "MinMaxReset",
    "mixing-valve-controller": "MixingValve",
    "modulo": "Modulo",
    "monoflop": "Monoflop",
    "moving-average": "MovingAverage",
    "mqtt": "Plugin",
    "multiclick": "MultiClick",
    "multifunction-switch": "ComfortSwitch",
    "multiply": "Multiply",
    "music-server-zone": "MusicServerZone",
    "not": "Not",
    "or": "Or",
    "pi-controller": "PIController",
    "pid-controller": "PIDController",
    "ping-function-block": "Ping",
    "pool-controller": "PoolController",
    "power-supply-backup-block": "PowerSupplyBackup",
    "presence": "PresenceDetector",
    "presence-detector": "PresenceDetector",
    "pulse-at": "PulseAt",
    "pulse-by": "PulseBy",
    "pulse-generator": "PulseGenerator",
    "pulse-meter": "PulseMeter",
    "pulse-meter-bidirectional": "PulseMeterBidirectional",
    "pulse-meter-storage": "PulseMeterStorage",
    "pulse-width-modulation": "PWM",
    "push-button": "Pushbutton",
    "push-switch": "PushSwitch",
    "pv-production-forecast": "PVForecast",
    "radio-buttons": "RadioButtons",
    "radio-buttons-16x": "RadioButtons16",
    "ramp-controller": "RampController",
    "random-controller": "RandomController",
    "random-number-generator": "RandomGenerator",
    "retractive-switch": "RetractiveSwitch",
    "rgb-scene-controller": "LightsceneRGB",
    "sauna": "Sauna",
    "sauna-controller": "Sauna",
    "sauna-controller-evaporator": "SaunaEvaporator",
    "saving-switch-on-delay": "SavingSwitchOnDelay",
    "scaler": "Scaler",
    "scene": "Scene",
    "security-overview": "SecurityCentral",
    "selection-switch-onoff": "SelectionSwitchOnOff",
    "selection-switch-plus": "SelectionSwitchPlus",
    "selection-switch-plus-minus": "SelectionSwitchPlusMinus",
    "sequencer": "Sequencer",
    "sequential-controller": "SequentialController",
    "session-database-connector": "SessionDBConnector",
    "shading-overview": "JalousieCentral",
    "shift-register": "ShiftRegister",
    "skylight": "SkyWindow",
    "skylight-blinds": "SkyWindowJalousie",
    "solar-pump-controller": "SolarPumpController",
    "spot-price-optimizer": "SpotPriceOptimizer",
    "stairwell-light-switch": "StairwellLight",
    "status": "Status",
    "status-indicator": "StatusIndicator",
    "status-monitor": "StatusMonitor",
    "stepper": "Stepper",
    "subtract": "Subtract",
    "switch": "Switch",
    "switch-on-and-off-delay": "SwitchOnOffDelay",
    "switch-on-delay": "SwitchOnDelay",
    "switching-timer": "SwitchingTimer",
    "tablet": "Tablet",
    "text-generator": "TextGenerator",
    "threshold-switch": "ThresholdSwitch",
    "timer": "Timer",
    "timerscheduler": "TimerScheduler",
    "toilet-ventilation-controller": "ToiletVentilation",
    "touch-and-grill-air": "TouchGrillAir",
    "touch-and-grill-control": "TouchGrillControl",
    "touch-nightlight-air": "TouchNightlight",
    "touch-pure-flex-controller": "TouchPureFlex",
    "tracker": "Tracker",
    "up-down-counter": "UpDownCounter",
    "utility-meter": "UtilityMeter",
    "value-validator": "ValueValidator",
    "ventilation": "Ventilation",
    "wallbox": "Wallbox",
    "wallbox-block": "WallboxBlock",
    "wallbox-manager": "WallboxManager",
    "weather-service": "WeatherServer",
    "wind-gauge": "WindGauge",
    "window-central": "WindowCentral",
}


def parse_markdown_table(text: str, section_name: str) -> list[dict]:
    """Parse a markdown table following a ## heading into a list of dicts."""
    # Find the section
    pattern = rf'## {re.escape(section_name)}[^\n]*\n\n?((?:\|.*\n)+)'
    m = re.search(pattern, text, re.IGNORECASE)
    if not m:
        return []

    table_text = m.group(1).strip()
    lines = [l.strip() for l in table_text.split('\n') if l.strip().startswith('|')]

    if len(lines) < 3:  # need header + separator + at least one row
        return []

    # Parse header
    header = [c.strip() for c in lines[0].split('|')[1:-1]]
    # Skip separator (line 1)

    rows = []
    for line in lines[2:]:
        cells = [c.strip() for c in line.split('|')[1:-1]]
        if len(cells) >= len(header):
            row = {}
            for i, h in enumerate(header):
                key = h.lower().replace(' ', '_')
                row[key] = cells[i] if i < len(cells) else ''
            rows.append(row)

    return rows


def extract_description(text: str) -> str:
    """Extract the first paragraph of content (before ToC or first heading)."""
    # Skip the header block (# Title, Source, ---)
    m = re.search(r'^---\n\n(.+?)(?=\n##|\n\n## )', text, re.DOTALL | re.MULTILINE)
    if m:
        desc = m.group(1).strip()
        # Take first paragraph only
        desc = desc.split('\n\n')[0]
        # Clean up
        desc = re.sub(r'\[([^\]]+)\]\([^)]+\)', r'\1', desc)  # strip links
        desc = re.sub(r'\*\*([^*]+)\*\*', r'\1', desc)  # strip bold
        return desc.strip()
    return ""


def parse_schema(slug: str, kb_dir: str) -> dict | None:
    """Parse a KB article into a structured schema."""
    md_path = os.path.join(kb_dir, f"{slug}.md")
    if not os.path.exists(md_path):
        return None

    text = open(md_path).read()

    # Check if this article has I/O documentation
    has_inputs = bool(re.search(r'## Inputs', text))
    has_outputs = bool(re.search(r'## Outputs', text))
    if not has_inputs and not has_outputs:
        return None

    # Extract title
    title_m = re.search(r'^# (.+)', text)
    title = title_m.group(1) if title_m else slug

    # Extract description
    description = extract_description(text)

    # Parse tables
    inputs = parse_markdown_table(text, "Inputs")
    outputs = parse_markdown_table(text, "Outputs")
    parameters = parse_markdown_table(text, "Parameters")
    properties = parse_markdown_table(text, "Properties")

    # Also try "Actuators" and "Diagnostic Inputs" sections (some articles use these)
    if not inputs:
        inputs = parse_markdown_table(text, "Actuators")
    if not outputs:
        diagnostic = parse_markdown_table(text, "Diagnostic Inputs")
        if diagnostic:
            outputs.extend(diagnostic)

    if not inputs and not outputs:
        return None

    # Determine XML type
    xml_type = SLUG_TO_TYPE.get(slug, "")

    # Extract source URL
    url_m = re.search(r'Source: (https://[^\n]+)', text)
    source_url = url_m.group(1) if url_m else ""

    return {
        "title": title,
        "xml_type": xml_type,
        "kb_slug": slug,
        "description": description,
        "inputs": inputs,
        "outputs": outputs,
        "parameters": parameters,
        "properties": properties,
        "source_url": source_url,
        "input_count": len(inputs),
        "output_count": len(outputs),
        "parameter_count": len(parameters),
        "property_count": len(properties),
    }


def main():
    parser = argparse.ArgumentParser(description="Build schemas from KB articles")
    parser.add_argument("--slug", help="Process a single article")
    parser.add_argument("--kb-dir", default=os.path.join(REPO_DIR, "docs", "kb"))
    parser.add_argument("--out", default=os.path.join(REPO_DIR, "docs", "schemas"))
    args = parser.parse_args()

    os.makedirs(args.out, exist_ok=True)

    if args.slug:
        schema = parse_schema(args.slug, args.kb_dir)
        if schema:
            out_path = os.path.join(args.out, f"{args.slug}.json")
            with open(out_path, "w") as f:
                json.dump(schema, f, indent=2)
            print(f"✓ {schema['title']}: {schema['input_count']}I {schema['output_count']}O {schema['parameter_count']}P → {out_path}")
        else:
            print(f"No schema data in {args.slug}", file=sys.stderr)
        return

    # Process all KB articles
    index = {}
    count = 0

    for fname in sorted(os.listdir(args.kb_dir)):
        if not fname.endswith(".md"):
            continue
        slug = fname[:-3]
        schema = parse_schema(slug, args.kb_dir)
        if not schema:
            continue

        out_path = os.path.join(args.out, f"{slug}.json")
        with open(out_path, "w") as f:
            json.dump(schema, f, indent=2)

        index[slug] = {
            "title": schema["title"],
            "xml_type": schema["xml_type"],
            "inputs": schema["input_count"],
            "outputs": schema["output_count"],
            "parameters": schema["parameter_count"],
            "properties": schema["property_count"],
        }
        count += 1

    # Write index
    index_path = os.path.join(args.out, "index.json")
    with open(index_path, "w") as f:
        json.dump(index, f, indent=2, sort_keys=True)

    # Stats
    total_inputs = sum(v["inputs"] for v in index.values())
    total_outputs = sum(v["outputs"] for v in index.values())
    total_params = sum(v["parameters"] for v in index.values())
    total_props = sum(v["properties"] for v in index.values())

    print(f"✓ {count} schemas built → {args.out}/")
    print(f"  {total_inputs} inputs, {total_outputs} outputs, {total_params} parameters, {total_props} properties")


if __name__ == "__main__":
    main()
