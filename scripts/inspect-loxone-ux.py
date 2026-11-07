#!/usr/bin/env python3
"""Inventory advanced block settings and recover Qt UI metadata from a saved analysis."""

import argparse
from collections import Counter, defaultdict
import hashlib
import json
import os
from pathlib import Path
import re
import sqlite3
import subprocess
import sys

from qt_metadata import PeImage, recover_qt_metadata
from pe_dialogs import recover_dialog_resources


ROOT = Path(__file__).resolve().parents[1]
FAMILIES = {
    "lighting": {
        "types": ["LightController2", "LightController", "LightControllerH",
                  "LightsceneRGB", "LightsceneLearn", "Lightscene", "PushDimmer",
                  "DaylightControl", "DaylightController", "LightCentral"],
        "qt_terms": ["lighting", "lightcontroller", "lightscene", "mood", "color", "dimmer"],
        "surfaces": ["moods and output assignments", "output roles and color modes",
                     "presence, transitions and per-output settings"],
    },
    "energy": {
        "types": ["EnergyManager2", "EnergyManager", "EFM", "LoadShed", "SpotOpt",
                  "Wallbox", "WBEM", "PowerUnit", "Fronius", "Energy"],
        "qt_terms": ["energy", "wallbox", "utilitymeter", "loadmanager", "spot", "tariff"],
        "surfaces": ["consumer priorities and operating modes", "meter/group topology",
                     "tariffs, charging limits and relationships"],
    },
    "hvac": {
        "types": ["HeatIRoomController2", "IRoomcontrol", "HVACController",
                  "ClimateControllerUS", "ClimateController", "Heatmixer",
                  "Heatmixer2", "Ventilation", "Fan", "VentInternorm", "Leaf",
                  "Fancoil", "FancoilFreshAir", "CentralFancoil", "AcControl",
                  "Sauna", "SaunaVapor", "PoolController", "Solarpumpcontrol"],
        "qt_terms": ["climate", "heat", "roomcontrol", "fancoil", "vent", "sauna", "pool"],
        "surfaces": ["setpoints, modes and schedules", "equipment assignments",
                     "interlocks and cross-field limits"],
    },
    "shading": {
        "types": ["AutoJalousie", "Jalousiemotor", "JalousieUpDown2",
                  "ShadeRoof", "RoofWindow", "CentralJalousie"],
        "qt_terms": ["jalousie", "shade", "blind", "roofwindow"],
        "surfaces": ["travel/slat calibration", "orientation and shading geometry",
                     "automatic operation, safety and locking"],
    },
    "audio": {
        "types": ["MusicPlayer", "MediaClient", "AudioCentral", "MediaController"],
        "qt_terms": ["music", "audio", "favorite", "playlist", "equalizer"],
        "surfaces": ["zones, players and favorites", "event sounds and announcements",
                     "mode-dependent volume and source selection"],
    },
    "schedules": {
        "types": ["DayTimer", "AlarmClock", "Calendar", "PulseAt", "PulseBy"],
        "qt_terms": ["daytimer", "alarmclock", "schedule", "calendar"],
        "surfaces": ["schedule/exception records", "operating modes and time handling",
                     "event, sound and action assignments"],
    },
    "logic-programming": {
        "types": ["State", "StateV", "SequenceController", "Formula", "Script",
                  "StatusMonitor", "TextGenerator", "Scene"],
        "qt_terms": ["state", "sequence", "condition", "formula", "script"],
        "surfaces": ["rule/expression tables", "program text and expressions",
                     "text, status and action mappings"],
    },
    "security-access": {
        "types": ["Alarm", "AlarmChain", "SmokeAlarm", "NfcCodeTouch",
                  "Doorcontroller", "Door", "Intercom", "Intercom2",
                  "WindowsMonitor", "AalEmergency", "AalSmartAlarm"],
        "qt_terms": ["alarm", "access", "nfccode", "intercom", "doorcontroller"],
        "surfaces": ["zones, authorizations and actions", "delays and schedules",
                     "device assignments and dependencies"],
    },
    "io-integrations": {
        "types": ["Plugin", "VirtualIn", "VirtualOut", "VirtualOutCmd",
                  "ModbusServer", "ModbusDevice", "EIBSensor", "EIBActor",
                  "Mail", "MailGenerator", "Irrigation", "TpfController", "TPDC"],
        "qt_terms": ["modbus", "mqtt", "knx", "virtual", "http", "touchpure"],
        "surfaces": ["protocol-specific nested records", "templates, assignments and mappings",
                     "command payloads, addresses and device options"],
    },
}
SPECIALIZED_COMMANDS = {
    "LightController2": ["moods", "add-mood", "set-mood-color", "splice-actor"],
    "DayTimer": ["timer-schedule"],
    "SequenceController": ["get-program", "set-program"],
}


def load_json(path):
    with path.open(encoding="utf-8") as stream:
        return json.load(stream)


def save_json(path, value):
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(json.dumps(value, indent=2, ensure_ascii=True) + "\n", encoding="utf-8")
    temporary.replace(path)


def fixture_storage(paths, lox):
    observed = defaultdict(lambda: {"attributes": set(), "paths": set(), "fixtures": set()})
    omissions = []
    environment = os.environ.copy()
    environment["LOX_TELEMETRY"] = "0"
    for path in paths:
        result = subprocess.run(
            [str(lox), "--quiet", "--output", "json", "config", "surfaces", str(path)],
            text=True, encoding="utf-8", capture_output=True, timeout=120,
            env=environment, check=False,
        )
        if result.returncode:
            omissions.append({
                "path": str(path), "exit_code": result.returncode,
                "reason": result.stderr.strip() or result.stdout.strip(),
            })
            continue
        response = json.loads(result.stdout)
        if response.get("ok") is not True:
            raise ValueError(f"Native surface inspection did not succeed: {path}")
        for surface in response["surfaces"]:
            xml_type = surface["element_type"]
            entry = observed[xml_type]
            entry["attributes"].update(surface["attributes"])
            entry["paths"].update(surface["storage_paths"])
            entry["fixtures"].add(str(path))
    return {
        key: {name: sorted(values) for name, values in entry.items()}
        for key, entry in observed.items()
    }, omissions


def msvc_dialog_types(path):
    data = path.read_bytes()
    image = PeImage(data)
    result = []
    for match in re.finditer(rb"\.\?AV[^\0]{1,500}@@\0", data):
        decorated = match.group()[:-1].decode("ascii", errors="replace")
        if not any(term in decorated.lower() for term in ("dlg", "dialog", "editor")):
            continue
        addresses = [
            hex(image.base + section["rva"] + match.start() - section["offset"])
            for section in image.sections
            if section["offset"] <= match.start() < section["offset"] + section["size"]
        ]
        result.append({
            "decorated_rtti_name": decorated, "file_offset": match.start(),
            "analysis_address": addresses[0] if len(addresses) == 1 else None,
            "status": "rtti_name_only_lead",
        })
    return result


def classify_block(xml_type, counts, patterns, conditional, nested, minimum_parameters=8):
    families = [
        name for name, definition in FAMILIES.items()
        if xml_type in definition["types"]
    ]
    reasons = []
    if counts["Parameter"] >= minimum_parameters:
        reasons.append("many_parameter_entries")
    if counts["Input"] + counts["Output"] >= 24:
        reasons.append("many_io_entries")
    if patterns:
        reasons.append("repeated_connector_patterns")
    if conditional:
        reasons.append("conditional_metadata")
    if nested:
        reasons.append("nested_storage_observed")
    if families:
        reasons.append("specialized_configuration_family")
    return families, reasons


def parameter_rows(block, schema):
    same_identity = (
        schema is not None
        and str(schema.get("control_type")) == block["attributes"].get("ControlType")
    )
    mappings = defaultdict(set)
    if same_identity:
        for row in schema.get("connectors", []):
            if row.get("io_type") == "Parameter":
                mappings[row.get("name_pattern")].update(row.get("k_values", []))
    result = []
    for group in block["groups"]:
        if group["attributes"].get("Type") != "Parameter":
            continue
        for row in group["io"]:
            name = row.get("Name")
            repeated = "%d" in (name or "") or "%e" in (name or "")
            result.append({
                "id": row.get("Id"), "name_pattern": name,
                "short_name": row.get("ShortName"),
                "default": row.get("Default"), "min": row.get("Min"),
                "max": row.get("Max"), "unit": row.get("Unit"),
                "step": row.get("Step"), "range": row.get("Range"),
                "condition": row.get("Condition"), "template_id": row.get("TemplateId"),
                "known_xml_keys": sorted(mappings.get(name, [])),
                "mapping_status": (
                    "sampled_pattern_keys" if repeated and mappings.get(name)
                    else "name_and_control_id_match" if mappings.get(name)
                    else "unmapped"
                ),
            })
    return result


def audit_catalog(catalog, schemas, storage, native, minimum_parameters=8):
    result = []
    for index, block in enumerate(catalog["blocks"]):
        attrs = block["attributes"]
        xml_type = attrs.get("LxType")
        counts = Counter()
        patterns = []
        conditional = []
        template_refs = 0
        constraints = Counter()
        for group in block["groups"]:
            kind = group["attributes"].get("Type")
            counts[kind] += len(group["io"])
            for io in group["io"]:
                if "%d" in io.get("Name", "") or "%e" in io.get("Name", ""):
                    patterns.append({"kind": kind, "id": io.get("Id"), "name": io.get("Name")})
                if "Condition" in io:
                    conditional.append({"kind": kind, "id": io.get("Id"), "condition": io["Condition"]})
                template_refs += "TemplateId" in io
                if kind == "Parameter":
                    constraints.update(key for key in ("Default", "Min", "Max", "Step", "Range")
                                       if key in io)
        observation = storage.get(xml_type, {})
        nested = [path for path in observation.get("paths", []) if path != "SET"]
        families, reasons = classify_block(
            xml_type, counts, patterns, conditional, nested, minimum_parameters
        )
        parameters = parameter_rows(block, schemas.get(xml_type))
        anchors = []
        if xml_type and native is not None:
            query = '"' + xml_type.replace('"', '""') + '"'
            for address, name, body in native.execute(
                "SELECT address,name,body FROM functions WHERE functions MATCH ?", (query,)
            ):
                if '"' + xml_type + '"' in body:
                    anchors.append({"address": address, "name": name, "status": "string_literal_lead"})
        result.append({
            "catalog_index": index, "xml_type": xml_type, "control_type": attrs.get("ControlType"),
            "name": attrs.get("Name"),
            "identity_status": "explicit_LxType" if xml_type else "xml_type_unknown",
            "families": families, "advanced_candidate": bool(reasons), "advanced_reasons": reasons,
            "entry_counts": dict(counts), "parameters": parameters,
            "parameter_constraint_counts": dict(constraints),
            "repeated_connector_patterns": patterns, "conditional_entries": conditional,
            "documentation_template_references": template_refs,
            "storage_observations": observation,
            "native_anchors": anchors,
            "cli": {
                "known_parameter_keys": sorted({
                    key for row in parameters for key in row["known_xml_keys"]
                }),
                "specialized_commands": SPECIALIZED_COMMANDS.get(xml_type, []),
                "generic_connector_metadata": "present" if xml_type in schemas else "not_assessed",
                "scalar_range_enforcement": "unverified",
            },
            "ux_parity": {
                "total_gui_field_count": None,
                "full_parameter_domain": "unverified",
                "cross_field_validation": "unverified",
                "feedback_equivalence": "unverified",
                "gui_roundtrip": "unverified",
                "overall": "unverified",
            },
        })
    return result


def inspect(state_path, output, repo, lox, rea_root=None, qt_layout_profiles=None):
    if (output / "qt-metadata.json").exists():
        raise ValueError("Refusing to overwrite an existing Qt snapshot; select a new --output directory")
    state = load_json(state_path)
    if state["status"] != "complete":
        raise ValueError("A completed bulk analysis is required; do not interrupt an active run")
    analysis = Path(state["analysis_directory"])
    version = Path(state["version_directory"])
    source = version / "source" / "LoxoneConfig.exe"
    qt = recover_qt_metadata(
        source, layout_profiles=load_json(qt_layout_profiles) if qt_layout_profiles else None,
    )
    rtti = msvc_dialog_types(source)
    export = state["binaries"]["LoxoneConfig.exe"]
    if qt["source"]["sha256"] != export["export"]["target_sha256"]:
        raise ValueError("Qt metadata and native corpus identify different executable bytes")
    native_index = load_json(analysis / "LoxoneConfig.exe" / "code" / "decompile-index.json")
    if native_index["target_sha256"] != qt["source"]["sha256"]:
        raise ValueError("Native export index identifies a different executable")
    native_by_address = {row["address"]: row for row in native_index["functions"]}
    for cls in qt["classes"]:
        record = native_by_address.get(cls["dispatcher_address"])
        cls["dispatcher_export"] = (
            {"status": record["status"], "relative_path": record.get("relative_path")}
            if record else {"status": "not_in_export_inventory"}
        )
    output.mkdir(parents=True, exist_ok=True)
    pe_dialogs = None
    if rea_root is not None:
        resource_source = version / "source" / "LoxoneConfigres_ENG.dll"
        resource_evidence = output / "english-resources.evidence.json"
        with resource_evidence.open("w", encoding="utf-8") as stdout:
            with (output / "pe-resource-inspection.log").open("w", encoding="utf-8") as stderr:
                subprocess.run(
                    ["node", str(rea_root / "scripts" / "rea.mjs"), "inspect-pe-resources",
                     str(resource_source), "--format", "json"],
                    stdout=stdout, stderr=stderr, check=True, timeout=120,
                )
        pe_dialogs = recover_dialog_resources(resource_source, load_json(resource_evidence))
        save_json(output / "native-dialogs.json", pe_dialogs)
    catalog_path = analysis / "resources" / "block-catalog.json"
    catalog = load_json(catalog_path)
    schema_path = repo / "docs" / "schemas" / "loxone-xml-schema.json"
    schemas = load_json(schema_path)
    fixture_paths = sorted((repo / "tests" / "eval").rglob("*.Loxone"))
    storage, omissions = fixture_storage(fixture_paths, lox)
    connection = sqlite3.connect(Path(export["database"]).as_uri() + "?mode=ro", uri=True)
    try:
        blocks = audit_catalog(catalog, schemas, storage, connection)
    finally:
        connection.close()
    by_family = {}
    for name, definition in FAMILIES.items():
        by_family[name] = {
            "surfaces_to_characterize": definition["surfaces"],
            "catalog_types": [row["xml_type"] for row in blocks if name in row["families"]],
            "qt_name_leads": [
                {"class_name": cls["class_name"], "dispatcher_address": cls["dispatcher_address"],
                 "methods": [method["name"] for method in cls["methods"]],
                 "match_status": "name_only_lead"}
                for cls in qt["classes"]
                if any(term in cls["class_name"].lower() for term in definition["qt_terms"])
            ],
        }
    advanced = [row for row in blocks if row["advanced_candidate"]]
    advanced.sort(key=lambda row: (
        -row["entry_counts"].get("Parameter", 0), -(len(row["storage_observations"].get("paths", []))),
        row["xml_type"] or "", row["control_type"] or "",
    ))
    report = {
        "schema_version": 1,
        "source": {
            "state": str(state_path.resolve()), "executable_sha256": qt["source"]["sha256"],
            "catalog": str(catalog_path), "schema": str(schema_path),
            "schema_sha256": hashlib.sha256(schema_path.read_bytes()).hexdigest(),
            "decoder_sha256": hashlib.sha256(Path(__file__).with_name("qt_metadata.py").read_bytes()).hexdigest(),
            "resource_decoder_sha256": hashlib.sha256(Path(__file__).with_name("pe_dialogs.py").read_bytes()).hexdigest(),
            "inventory_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
            "surface_reader": str(lox),
            "surface_reader_sha256": hashlib.sha256(lox.read_bytes()).hexdigest(),
        },
        "criteria": {
            "minimum_parameter_entries": 8, "minimum_combined_io_entries": 24,
            "also_include": ["named specialized families", "repeated connector patterns",
                             "conditional metadata", "nested storage observed in fixtures"],
        },
        "summary": {
            "catalog_records": len(blocks),
            "explicit_xml_types": sum(row["xml_type"] is not None for row in blocks),
            "advanced_candidates": len(advanced),
            "advanced_explicit_xml_types": sum(row["xml_type"] is not None for row in advanced),
            "parameter_entries": sum(len(row["parameters"]) for row in blocks),
            "mapped_parameter_entries": sum(
                bool(param["known_xml_keys"]) for row in blocks for param in row["parameters"]
            ),
            "gui_parity_verified": 0,
            "qt": qt["coverage"],
            "qt_decoding_issues": len(qt["issues"]),
            "qt_unsupported_objects": len(qt["unsupported_objects"]),
            "msvc_dialog_type_leads": len(rtti),
            "native_dialogs": pe_dialogs["coverage"] if pe_dialogs else None,
            "native_resource_issues": len(pe_dialogs["issues"]) if pe_dialogs else None,
        },
        "families": by_family,
        "qt_dialogs": [
            {
                "class_name": cls["class_name"], "dispatcher_address": cls["dispatcher_address"],
                "dispatcher_export": cls["dispatcher_export"],
                "methods": [method["name"] for method in cls["methods"]],
                "properties": [prop["name"] for prop in cls["properties"]],
                "status": "declared_qt_metadata_not_behavior_verified",
            }
            for cls in qt["classes"]
            if any(term in cls["class_name"].lower() for term in ("dlg", "dialog", "editor", "configuration"))
        ],
        "msvc_dialog_types": rtti,
        "blocks": blocks,
        "fixture_omissions": omissions,
        "limitations": [
            "A catalog connector entry is not a GUI field; a numeric parameter count does not measure UX parity.",
            "TemplateId is retained as a documentation reference, not interpreted as a repeated UI row.",
            "Pattern expansions in the repository schema are observed examples, not exhaustive channel limits.",
            "Parameter keys require exact name and control-type identity; unknown types and unmapped fields remain unknown.",
            "Fixture storage proves serialized shapes only; parsing a fixture does not establish GUI acceptance or runtime behavior.",
            "Qt names and literal references are navigation leads, not verified links from a block to a dialog.",
            "Documented ranges and existing commands do not establish validation parity; this read-only report does not validate or edit parameter values.",
            "No total UX denominator or percentage is claimed. No live Miniserver, alarm, or saved user configuration is modified.",
            "PE dialog templates are inspected only when --rea-root is supplied; their decoded controls do not establish which legacy dialogs remain reachable.",
        ],
    }
    output.mkdir(parents=True, exist_ok=True)
    save_json(output / "qt-metadata.json", qt)
    save_json(output / "block-ux-coverage.json", report)
    lines = ["TYPE\tCONTROL_ID\tPARAMETERS\tINPUTS\tOUTPUTS\tFAMILIES\tREASONS"]
    for row in advanced:
        counts = row["entry_counts"]
        lines.append("\t".join([
            row["xml_type"] or "(unmapped)", row["control_type"] or "",
            str(counts.get("Parameter", 0)), str(counts.get("Input", 0)),
            str(counts.get("Output", 0)), ",".join(row["families"]),
            ",".join(row["advanced_reasons"]),
        ]))
    (output / "advanced-blocks.tsv").write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(json.dumps(report["summary"], indent=2))
    print(f"Reports: {output}")
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--state", type=Path, default=ROOT / ".reverse-engineering" / "latest.json")
    parser.add_argument("--output", type=Path, default=ROOT / ".reverse-engineering" / "ux")
    parser.add_argument("--lox", type=Path, default=ROOT / "target" / "release" / "lox.exe",
                        help="Built CLI providing the read-only config surfaces command")
    parser.add_argument("--rea-root", type=Path,
                        help="Also inspect native dialog resources using this local REA checkout")
    parser.add_argument("--qt-layout-profiles", type=Path,
                        help="Source-pinned reviewed per-object Qt header/base profiles; unprofiled objects remain issues")
    args = parser.parse_args()
    try:
        report = inspect(
            args.state.resolve(), args.output.resolve(), ROOT, args.lox.resolve(strict=True),
            args.rea_root.resolve(strict=True) if args.rea_root else None,
            args.qt_layout_profiles.resolve(strict=True) if args.qt_layout_profiles else None,
        )
        summary = report["summary"]
        if (report["fixture_omissions"] or summary["qt_decoding_issues"]
                or summary["qt_unsupported_objects"] or summary["native_resource_issues"]):
            sys.exit(2)
    except (OSError, ValueError, KeyError, sqlite3.Error, subprocess.SubprocessError) as error:
        print(f"UX inventory failed: {error}", file=sys.stderr)
        sys.exit(1)
