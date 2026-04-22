#!/usr/bin/env python3
"""Extract eval cases from Loxone use case golden configs.

Reads use-cases.json metadata and parses golden config XML files
to produce structured eval cases with blocks, wiring, and params.

Usage:
    python3 scripts/extract-usecase-evals.py
"""

import json
import os
import re
import sys
import xml.etree.ElementTree as ET
from collections import defaultdict
from pathlib import Path
from unicodedata import normalize

REPO = Path(__file__).resolve().parent.parent
USE_CASES = REPO / "tests" / "eval" / "use-cases.json"
GOLDEN_DIR = REPO / "tests" / "eval" / "golden-configs"
CONNECTOR_MAP = REPO / "docs" / "schemas" / "connector-map.json"
OUTPUT = REPO / "tests" / "eval" / "cases" / "use_cases.json"

# German function block name → English XML type name
DE_TO_EN_TYPE = {
    "Addierer": "Add",
    "Alarmanlage": "Alarm",
    "Alarmierungskette": "AlarmChain",
    "Analogwahlschalter": "AnalogMultiplexer2",
    "Analogwahlschalter 4-fach": "AnalogMultiplexer",
    "Anlagenschema": None,  # visual-only, no logic block
    "Audio Player": "MusicPlayer",
    "Audio Player Gruppe fix": "MPGroup",
    "Audio Zentral": "CentralMusic",
    "Ausschaltverzögerung": "OffDelay",
    "Automatik-Regel": "AutopilotRule",
    "Automatikbeschattung": "AutoJalousie",
    "Berechtigung NFC Code Touch": "NfcCodeTouch",
    "Binärdekoder": "BinDecoder",
    "Brand- und Wassermeldezentrale": "SmokeAlarm",
    "Composite-Fensterkontakt": "JoinWindowSensor",
    "Ein- und Ausschaltverzögerung": "OnOffDelay",
    "Einschaltverzögerung": "OnDelay",
    "Energieflussmonitor": "EFM",
    "Energiemanager": "EnergyManager2",
    "Fenster - und Türüberwachung": "WindowsMonitor",
    "Flankenerkennung": "EdgeDetection",
    "Formel": "Formula",
    "Gleich": "Equal",
    "Größer": "Greater",
    "Größer oder gleich": "GreaterEqual",
    "Heiz- und Kühlsteuerung": "HVACController",
    "Impuls bei": "PulseBy",
    "Impuls um": "PulseAt",
    "Impulsgeber": "PulseGen",
    "Intelligente Raumregelung": "HeatIRoomController2",
    "Kleiner": "Less",
    "Klimaanlagen Zentralsteuerung": "CentralFancoil",
    "Klimaanlagensteuerung": "AcControl",
    "Langzeitklick": "LongClick",
    "Lastmanager": "LoadShed",
    "Licht Zentral": "CentralLight",
    "Lichtsteuerung": "LightController2",
    "MinMax": "Minmax",
    "Monoflop": "Monoflop",
    "Multiplizierer": "Mult",
    "NICHT": "Not",
    "ODER": "Or",
    "Ping": "Ping",
    "Präsenz": "Presence",
    "Radiotasten": "Radio2",
    "Schalter": "PushButton",
    "Schaltuhr": "DayTimer",
    "Schwellwertschalter": "AnalogThresholdTrigger",
    "Session Database Connector": "DbConS",
    "Spotpreis-Optimierer": "SpotOpt",
    "Status": "State",
    "Status Monitor": "StatusMonitor",
    "Stufenauswahl": "StepSel",
    "Szene": "AutomaticScene",
    "Taster": "PButtonT",
    "Text Generator": "TextGenerator",
    "UND": "And",
    "Verzögerter Impuls": "OnPulseDelay",
    "Virtueller Status": "StateV",
    "Wallbox": "Wallbox",
    "Wallbox Manager": None,  # not a standard block type
    "Wecker": "AlarmClock",
    "Zähler": "Counter",
    "Zähler Bidirektional": "MeterAbsBi",
    "Zähler für Speicher": "MeterAbsSt",
}

# Infrastructure types to skip when extracting blocks
INFRA_TYPES = {
    "Document", "LoxCaption", "CategoryCaption", "Category",
    "PlaceCaption", "Place", "UserCaption", "User", "UserGroupCaption",
    "UserGroup", "Permission", "RightGroup", "ConstantCaption",
    "MemoryCaption", "ModeCaption", "Mode", "CalendarCaption", "Calendar",
    "CalendarEntry", "RemoteControls", "TimeCaption", "WeatherServer",
    "WeatherData", "WeatherCaption", "GlobalStates", "SysVar",
    "MessageCenter", "Devicemonitor", "AutoPilot", "LoxLIVE", "LoxTree",
    "Online", "VirtualInCaption", "VirtualOutCaption", "InputCaption",
    "OutputCaption", "AnalogInputCaption", "TaskCaption", "LoggerOutCaption",
    "Logger", "Notification", "Mailer", "SwitchingTimer", "Program", "Page",
    "PageCaption",
    # Time function blocks (built-in)
    "DateTime", "Hour", "Minute", "Second", "Day", "Day2009", "DayOfWeek",
    "Week", "Month", "Year", "Sunrise", "Sunset", "Daylight", "Daylight2",
    "Morningtwilight", "Eveningtwilight", "NightTime", "SunAltitude",
    "SunAzimuth", "Time", "SecondsBoot", "StartPulse",
    "ImpulseDay", "ImpulseHour", "ImpulseMinute", "ImpulseMonth",
    "ImpulseSecond", "ImpulseSunrise", "ImpulseSunset", "ImpulseYear",
    "ImpulseMorningtwilight", "ImpulseEveningtwilight",
    # Hardware / system
    "LanInt", "VoltageIn", "OvertempShutdown", "DigitalIn", "Actor",
    # Visual-only
    "Text", "Line", "Display",
}

# Types that are reference/passthrough blocks (not logic the user configures)
REF_TYPES = {"InputRef", "OutputRef", "OutputRefLM"}

# Category → difficulty mapping
CATEGORY_DIFFICULTY = {
    "access": "medium",
    "audio": "medium",
    "climate": "hard",
    "energy": "hard",
    "lighting": "medium",
    "security": "hard",
    "shading": "medium",
    "other": "medium",
}

# Category → pattern tags
CATEGORY_PATTERNS = {
    "access": ["access-control"],
    "audio": ["audio", "notification"],
    "climate": ["climate", "hvac"],
    "energy": ["energy", "metering"],
    "lighting": ["lighting"],
    "security": ["security", "alarm"],
    "shading": ["shading"],
    "other": ["automation"],
}


def safe_parse_xml(path):
    """Parse Loxone XML handling BOM and non-standard attribute names."""
    with open(path, "rb") as f:
        content = f.read()
    if content.startswith(b"\xef\xbb\xbf"):
        content = content[3:]
    text = content.decode("utf-8")
    # Fix attributes starting with digits (invalid XML but used by Loxone)
    text = re.sub(r" (\d+\w+)=", r" _\1=", text)
    return ET.fromstring(text)


def build_room_map(root):
    """Build UUID → room name mapping from PlaceCaption."""
    rooms = {}
    for elem in root.iter("C"):
        if elem.get("Type") == "Place":
            u = elem.get("U", "")
            title = elem.get("Title", "")
            if u and title:
                rooms[u] = title
    return rooms


def extract_blocks_and_wiring(root, room_map, connector_schema, target_types):
    """Extract function blocks and wiring from the Program section.

    Returns (blocks, wiring, params) where:
    - blocks: list of {type, title, room, page}
    - wiring: list of {from_type, from_title, from_connector,
                       to_type, to_title, to_connector}
    - params: list of {block_type, block_title, param, value}
    """
    # Phase 1: Collect all blocks in Program pages and their connectors
    blocks = []
    co_map = {}  # connector UUID → {block_type, block_title, connector_name}

    def process_program(program_elem):
        for page in program_elem:
            if page.tag != "C" or page.get("Type") != "Page":
                continue
            page_title = page.get("Title", "")
            for block in page:
                if block.tag != "C":
                    continue
                btype = block.get("Type", "")
                if btype in INFRA_TYPES:
                    continue
                btitle = block.get("Title", "")
                broom_uuid = block.get("R", "")
                broom = room_map.get(broom_uuid, "")

                block_info = {
                    "type": btype,
                    "title": btitle,
                    "room": broom,
                    "page": page_title,
                    "uuid": block.get("U", ""),
                    "is_ref": btype in REF_TYPES,
                }
                blocks.append(block_info)

                # Collect connectors
                co_index = 0
                for child in block:
                    if child.tag == "Co":
                        co_uuid = child.get("U", "")
                        co_name = child.get("K", "")
                        if co_uuid:
                            co_map[co_uuid] = {
                                "block_type": btype,
                                "block_title": btitle,
                                "connector_name": co_name,
                                "block_uuid": block.get("U", ""),
                                "co_elem": child,
                            }
                        co_index += 1
                    elif child.tag == "C":
                        # Sub-blocks (e.g. nested inside a control)
                        sub_type = child.get("Type", "")
                        if sub_type and sub_type not in INFRA_TYPES:
                            sub_info = {
                                "type": sub_type,
                                "title": child.get("Title", ""),
                                "room": room_map.get(child.get("R", ""), broom),
                                "page": page_title,
                                "uuid": child.get("U", ""),
                                "is_ref": sub_type in REF_TYPES,
                            }
                            blocks.append(sub_info)
                            for co in child:
                                if co.tag == "Co":
                                    co_uuid = co.get("U", "")
                                    co_name = co.get("K", "")
                                    if co_uuid:
                                        co_map[co_uuid] = {
                                            "block_type": sub_type,
                                            "block_title": child.get("Title", ""),
                                            "connector_name": co_name,
                                            "block_uuid": child.get("U", ""),
                                            "co_elem": co,
                                        }

    # Also collect connectors from non-Program blocks (system vars, modes, etc.)
    # since wiring can reference them
    def collect_all_connectors(elem, parent_type="", parent_title=""):
        for child in elem:
            if child.tag == "Co":
                co_uuid = child.get("U", "")
                co_name = child.get("K", "")
                if co_uuid and co_uuid not in co_map:
                    co_map[co_uuid] = {
                        "block_type": parent_type,
                        "block_title": parent_title,
                        "connector_name": co_name,
                        "block_uuid": elem.get("U", ""),
                        "co_elem": child,
                    }
            elif child.tag == "C":
                ct = child.get("Type", "")
                ctitle = child.get("Title", "")
                collect_all_connectors(child, ct, ctitle)

    collect_all_connectors(root)

    # Find Program elements and process them
    for elem in root.iter("C"):
        if elem.get("Type") == "Program":
            process_program(elem)

    # Phase 2: Extract wiring from In elements
    wiring = []
    for co_uuid, co_info in co_map.items():
        co_elem = co_info.get("co_elem")
        if co_elem is None:
            continue
        for child in co_elem:
            if child.tag == "In":
                src_uuid = child.get("Input", "")
                if src_uuid and src_uuid in co_map:
                    src = co_map[src_uuid]
                    wire = {
                        "from_type": src["block_type"],
                        "from_title": src["block_title"],
                        "from_connector": src["connector_name"],
                        "to_type": co_info["block_type"],
                        "to_title": co_info["block_title"],
                        "to_connector": co_info["connector_name"],
                    }
                    wiring.append(wire)

    # Phase 3: Extract params (connectors with Def attribute and type P)
    params = []
    for co_uuid, co_info in co_map.items():
        co_elem = co_info.get("co_elem")
        if co_elem is None:
            continue
        # Loxone stores param values in the "Def" attribute on Co elements
        val = co_elem.get("Def", "")
        co_name = co_info["connector_name"]
        btype = co_info["block_type"]
        if not val or not co_name or btype in INFRA_TYPES or btype in REF_TYPES:
            continue
        # Check if this is a parameter connector (type P in schema)
        if btype in connector_schema:
            type_info = connector_schema[btype].get("t", {})
            if type_info.get(co_name) == "P":
                # Check if value differs from connector-map default
                defaults = connector_schema[btype].get("d", {})
                default_val = defaults.get(co_name, "")
                if val != default_val:
                    params.append({
                        "block_type": btype,
                        "block_title": co_info["block_title"],
                        "param": co_name,
                        "value": val,
                    })

    return blocks, wiring, params


def filter_to_usecase_blocks(blocks, wiring, params, target_types):
    """Filter blocks/wiring/params to only those relevant to the use case.

    Keeps blocks whose type matches target_types (from function_blocks),
    plus any block that wires to/from those blocks.
    """
    # Find the primary block UUIDs (matching target types)
    primary_types = set(target_types)
    primary_uuids = set()
    for b in blocks:
        if b["type"] in primary_types and not b["is_ref"]:
            primary_uuids.add(b["uuid"])

    # Also include blocks that wire to/from primary blocks
    connected_titles = set()
    for b in blocks:
        if b["type"] in primary_types:
            connected_titles.add(b["title"])

    # Filter blocks: keep non-ref, non-infra blocks matching target types
    filtered_blocks = []
    for b in blocks:
        if b["is_ref"]:
            continue
        if b["type"] in primary_types:
            filtered_blocks.append(b)

    # Filter wiring: keep wires involving target type blocks
    filtered_wiring = []
    for w in wiring:
        involves_target = (
            w["from_type"] in primary_types or w["to_type"] in primary_types
        )
        if involves_target:
            filtered_wiring.append(w)

    # Filter params: keep params for target type blocks
    filtered_params = []
    for p in params:
        if p["block_type"] in primary_types:
            filtered_params.append(p)

    return filtered_blocks, filtered_wiring, filtered_params


def make_eval_id(uc_id, name_en):
    """Create a kebab-case eval ID from use case metadata."""
    if not name_en:
        name_en = f"usecase-{uc_id}"
    # Normalize unicode, remove non-alphanumeric, kebab-case
    name = normalize("NFKD", name_en).encode("ascii", "ignore").decode()
    name = re.sub(r"[^a-zA-Z0-9\s-]", "", name)
    name = re.sub(r"\s+", "-", name.strip()).lower()
    name = re.sub(r"-+", "-", name)
    # Truncate and prefix
    name = name[:50].rstrip("-")
    return f"uc-{uc_id}-{name}"


def make_utterance(uc):
    """Create a natural homeowner utterance from the use case description."""
    # Use description (German) as the primary utterance
    desc = uc.get("description", "")
    impl = uc.get("implementation", "")

    if desc:
        # Clean up HTML entities
        utterance = desc.replace("&amp;", "&")
        utterance = re.sub(r"<[^>]+>", "", utterance)  # strip HTML tags
        utterance = utterance.strip()
        if utterance:
            return utterance

    if impl:
        # Take the first meaningful sentence from implementation
        impl = impl.replace("&amp;", "&")
        impl = re.sub(r"<[^>]+>", "", impl)
        sentences = re.split(r"[.!?]\s+", impl)
        if sentences:
            return sentences[0].strip() + "."

    return uc.get("name", uc.get("name_en", ""))


def format_wiring_for_eval(wiring, filtered_blocks):
    """Format wiring specs for eval case output.

    Uses from_type/to_type for new blocks and from_title/to_title
    for reference blocks or existing infrastructure.
    """
    block_types_in_case = {b["type"] for b in filtered_blocks}
    block_titles = {b["title"] for b in filtered_blocks}

    # Count how many of each type — if unique, use type only
    type_counts = defaultdict(int)
    for b in filtered_blocks:
        type_counts[b["type"]] += 1

    result = []
    for w in wiring:
        entry = {}

        # Source side
        if w["from_type"] in block_types_in_case:
            if type_counts[w["from_type"]] == 1:
                entry["from_type"] = w["from_type"]
            else:
                entry["from_title"] = w["from_title"]
        elif w["from_type"] in REF_TYPES:
            entry["from_title"] = w["from_title"]
        else:
            entry["from_title"] = w["from_title"]

        if w["from_connector"]:
            entry["from_connector"] = w["from_connector"]

        # Destination side
        if w["to_type"] in block_types_in_case:
            if type_counts[w["to_type"]] == 1:
                entry["to_type"] = w["to_type"]
            else:
                entry["to_title"] = w["to_title"]
        elif w["to_type"] in REF_TYPES:
            entry["to_title"] = w["to_title"]
        else:
            entry["to_title"] = w["to_title"]

        if w["to_connector"]:
            entry["to_connector"] = w["to_connector"]

        result.append(entry)

    return result


def format_params_for_eval(params, filtered_blocks):
    """Format param specs for eval case output."""
    type_counts = defaultdict(int)
    for b in filtered_blocks:
        type_counts[b["type"]] += 1

    result = []
    for p in params:
        entry = {"block_type": p["block_type"], "param": p["param"], "value": p["value"]}
        if type_counts.get(p["block_type"], 0) > 1:
            entry["block_title"] = p["block_title"]
        result.append(entry)

    return result


def main():
    with open(USE_CASES) as f:
        use_cases = json.load(f)

    with open(CONNECTOR_MAP) as f:
        connector_schema = json.load(f)

    eval_cases = []
    skipped = []

    for uc in use_cases:
        uc_id = uc["id"]
        name_en = uc.get("name_en", "")
        category = uc.get("category", "other")
        config_files = uc.get("config_files", [])
        function_blocks = uc.get("function_blocks", [])

        if not config_files:
            skipped.append((uc_id, "no config file"))
            continue

        config_name = config_files[0]["name"]
        config_path = GOLDEN_DIR / config_name
        if not config_path.exists():
            skipped.append((uc_id, f"missing config: {config_name}"))
            continue

        # Map German function block names to English types
        target_types = set()
        for fb in function_blocks:
            en_type = DE_TO_EN_TYPE.get(fb["name"])
            if en_type:
                target_types.add(en_type)

        if not target_types:
            skipped.append((uc_id, "no mappable function blocks"))
            continue

        # Parse XML
        try:
            root = safe_parse_xml(config_path)
        except ET.ParseError as e:
            skipped.append((uc_id, f"XML parse error: {e}"))
            continue

        room_map = build_room_map(root)
        blocks, wiring, params = extract_blocks_and_wiring(
            root, room_map, connector_schema, target_types
        )

        # Filter to use case-relevant blocks
        filtered_blocks, filtered_wiring, filtered_params = filter_to_usecase_blocks(
            blocks, wiring, params, target_types
        )

        if not filtered_blocks:
            skipped.append((uc_id, "no matching blocks found in config"))
            continue

        # Deduplicate blocks by (type, title)
        seen = set()
        deduped_blocks = []
        for b in filtered_blocks:
            key = (b["type"], b["title"])
            if key not in seen:
                seen.add(key)
                deduped_blocks.append(b)

        # Build eval case
        eval_id = make_eval_id(uc_id, name_en)
        utterance = make_utterance(uc)
        difficulty = CATEGORY_DIFFICULTY.get(category, "medium")
        patterns = CATEGORY_PATTERNS.get(category, ["automation"])

        # Add type-specific patterns
        for b in deduped_blocks:
            if b["type"] in ("Alarm", "AlarmChain", "SmokeAlarm"):
                if "alarm" not in patterns:
                    patterns.append("alarm")
            elif b["type"] in ("LightController2", "CentralLight"):
                if "lighting" not in patterns:
                    patterns.append("lighting")
            elif b["type"] in ("AcControl", "CentralFancoil", "HVACController"):
                if "hvac" not in patterns:
                    patterns.append("hvac")
            elif b["type"] in ("SpotOpt",):
                if "spot-price" not in patterns:
                    patterns.append("spot-price")
            elif b["type"] in ("EnergyManager2", "EFM", "LoadShed"):
                if "energy-management" not in patterns:
                    patterns.append("energy-management")
            elif b["type"] in ("Presence", "PresenceController"):
                if "presence" not in patterns:
                    patterns.append("presence")

        # Format new_blocks
        new_blocks = []
        for b in deduped_blocks:
            block_entry = {"type": b["type"]}
            if b["title"]:
                block_entry["title_contains"] = b["title"]
            if b["room"]:
                block_entry["room"] = b["room"]
            if b["page"]:
                block_entry["page"] = b["page"]
            new_blocks.append(block_entry)

        # Format wiring
        eval_wiring = format_wiring_for_eval(filtered_wiring, deduped_blocks)

        # Format params
        eval_params = format_params_for_eval(filtered_params, deduped_blocks)

        expected = {"new_blocks": new_blocks}
        if eval_wiring:
            expected["wiring"] = eval_wiring
        if eval_params:
            expected["params"] = eval_params

        eval_case = {
            "id": eval_id,
            "source": f"loxone-usecase-{uc_id}",
            "pattern": patterns,
            "utterance": utterance,
            "difficulty": difficulty,
            "expected": expected,
        }

        eval_cases.append(eval_case)

    # Sort by ID for stable output
    eval_cases.sort(key=lambda c: c["id"])

    # Write output
    os.makedirs(OUTPUT.parent, exist_ok=True)
    with open(OUTPUT, "w", encoding="utf-8") as f:
        json.dump(eval_cases, f, indent=2, ensure_ascii=False)

    # Summary
    print(f"Generated {len(eval_cases)} eval cases → {OUTPUT}")
    print(f"Skipped {len(skipped)} use cases:")
    for uc_id, reason in skipped:
        print(f"  {uc_id}: {reason}")

    # Stats
    total_blocks = sum(len(c["expected"]["new_blocks"]) for c in eval_cases)
    total_wiring = sum(len(c["expected"].get("wiring", [])) for c in eval_cases)
    total_params = sum(len(c["expected"].get("params", [])) for c in eval_cases)
    types_used = set()
    for c in eval_cases:
        for b in c["expected"]["new_blocks"]:
            types_used.add(b["type"])
    print(f"\nStats: {total_blocks} blocks, {total_wiring} wiring specs, "
          f"{total_params} param specs, {len(types_used)} unique types")
    print(f"Types: {sorted(types_used)}")


if __name__ == "__main__":
    main()
