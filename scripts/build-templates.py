#!/usr/bin/env python3
"""Build element templates from a Loxone config XML file.

Analyzes all element types and extracts their connector schemas,
property schemas, and default attributes. Outputs templates.json
for use by the CLI.

Usage:
    python3 scripts/build-templates.py config.Loxone
    python3 scripts/build-templates.py config.Loxone --out templates.json
"""

import argparse
import json
import os
import sys
import xml.etree.ElementTree as ET
from collections import defaultdict

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
REPO_DIR = os.path.dirname(SCRIPT_DIR)

# Map control type → doc slug
DOC_SLUGS = {
    "LightController2": "lighting-controller",
    "Switch": "switch",
    "Dimmer": "dimmer",
    "Jalousie": "automated-blinds",
    "CentralJalousie": "automated-blinds",
    "Gate": "gate",
    "CentralGate": "gate",
    "AlarmClock": "alarm-clock",
    "PresenceDetector": "presence-detector",
    "IRoomControllerV2": "intelligent-room-controller",
    "Alarm": "alarm-system",
    "WeatherData": "weather-service",
    "WeatherServer": "weather-service",
    "Plugin": "mqtt",
    "GenTSensor": "mqtt",
    "GenTActor": "mqtt",
    "Sauna": "sauna",
    "AudioZone": "audioserver",
}


def analyze_config(xml_path: str) -> dict:
    """Analyze a .Loxone XML and extract type templates."""
    tree = ET.parse(xml_path)
    root = tree.getroot()

    # Collect per-type information
    types = defaultdict(lambda: {
        "count": 0,
        "connectors": defaultdict(lambda: {"count": 0, "direction": "unknown"}),
        "properties": defaultdict(lambda: {"count": 0, "type_codes": set(), "sample_values": []}),
        "attributes": defaultdict(lambda: {"count": 0, "samples": set()}),
        "has_io_data": False,
        "parent_types": set(),
        "child_types": set(),
        "sample_titles": [],
    })

    # Build parent map
    parent_map = {}
    for parent in root.iter():
        for child in parent:
            parent_map[child] = parent

    for elem in root.iter("C"):
        t = elem.get("Type")
        if not t:
            continue

        info = types[t]
        info["count"] += 1

        # Sample titles (up to 3)
        title = elem.get("Title", "")
        if title and len(info["sample_titles"]) < 3:
            info["sample_titles"].append(title)

        # Attributes
        for attr, val in elem.attrib.items():
            info["attributes"][attr]["count"] += 1
            if len(info["attributes"][attr]["samples"]) < 3:
                info["attributes"][attr]["samples"].add(val)

        # Connectors
        for co in elem.findall("Co"):
            k = co.get("K", "?")
            info["connectors"][k]["count"] += 1
            # Classify direction
            if k.startswith("I") or k.startswith("AI") or k == "Input":
                info["connectors"][k]["direction"] = "input"
            elif k.startswith("Q") or k.startswith("AQ") or k.startswith("Output"):
                info["connectors"][k]["direction"] = "output"
            elif k.startswith("Par") or k.endswith("Time") or k.endswith("Period"):
                info["connectors"][k]["direction"] = "parameter"

        # Properties (SET children)
        set_elem = elem.find("SET")
        if set_elem is not None:
            for prop in set_elem:
                pname = prop.tag
                ptype = prop.get("t", "")
                pval = prop.get("v", "")
                info["properties"][pname]["count"] += 1
                info["properties"][pname]["type_codes"].add(ptype)
                if pval and len(info["properties"][pname]["sample_values"]) < 3:
                    info["properties"][pname]["sample_values"].append(pval)

        # IoData
        if elem.find("IoData") is not None:
            info["has_io_data"] = True

        # Parent type
        parent = parent_map.get(elem)
        if parent is not None:
            pt = parent.get("Type", parent.tag)
            info["parent_types"].add(pt)

        # Child types
        for child in elem.findall("C"):
            ct = child.get("Type", "")
            if ct:
                info["child_types"].add(ct)

    # Convert to serializable format
    result = {}
    for t, info in sorted(types.items()):
        connectors = {}
        for k, v in sorted(info["connectors"].items()):
            connectors[k] = {"direction": v["direction"]}

        properties = {}
        for k, v in sorted(info["properties"].items()):
            properties[k] = {
                "type_codes": sorted(v["type_codes"]),
                "sample_values": v["sample_values"],
            }

        attributes = {}
        for k, v in sorted(info["attributes"].items()):
            if k in ("Type", "U"):  # skip obvious ones
                continue
            attributes[k] = {
                "count": v["count"],
                "samples": sorted(v["samples"])[:3],
            }

        result[t] = {
            "count": info["count"],
            "sample_titles": info["sample_titles"],
            "has_io_data": info["has_io_data"],
            "connectors": connectors,
            "properties": properties,
            "attributes": attributes,
            "parent_types": sorted(info["parent_types"]),
            "child_types": sorted(info["child_types"]),
        }

        # Add doc slug if known
        if t in DOC_SLUGS:
            result[t]["doc_slug"] = DOC_SLUGS[t]

    return result


def main():
    parser = argparse.ArgumentParser(description="Build element templates from Loxone config")
    parser.add_argument("config", help="Path to .Loxone XML file")
    parser.add_argument("--out", help="Output path (default: stdout)")
    args = parser.parse_args()

    templates = analyze_config(args.config)

    output = json.dumps(templates, indent=2, ensure_ascii=False)

    if args.out:
        with open(args.out, "w") as f:
            f.write(output)
        print(f"✓ {len(templates)} type templates → {args.out}", file=sys.stderr)
    else:
        print(output)


if __name__ == "__main__":
    main()
