#!/usr/bin/env python3
"""Extract a lossless, static native identity sidecar; never execute the image.

Usage:
  python scripts\\element-native-map.py --inventory INVENTORY --native-catalog CATALOG --output OUTPUT

The four reviewed factory cases are deliberately bounded. All other exact
stripped-class-name matches remain candidates, not XML or execution proof.
--factory-proof optionally admits an explicitly reviewed, digest-pinned report.
Reviewed historical producer inputs live beside that report in sources\\<SHA256>;
all live application/corpus inputs must still match their original source pins.
Every UX source record remains in catalog_records, even without a join.
"""

import argparse
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import struct
import sys
from collections import Counter


ROOT = Path(__file__).resolve().parents[1]
VERSION = "17.2.10.21"
TARGET = "7b55e3ef18401574ab967c5c6a0140cd5a66dc728521ebba04ad53093577d8b3"
FACTORY = "0x140432790"
FACTORY_SHA256 = "ff55a98ab896be192c361b63eb28f61b22853b55e4a74f32e13b30920dfe8f83"
# Adding a report digest is an explicit static-identity review, not producer trust.
REVIEWED_FACTORY_PROOFS = {
    "28e47977b739ec151f67fdccae4c9a012d4adfe1da59395704a592f9779825e8": {
        "version": VERSION, "sha256": TARGET,
        "historical_sources": {
            ("scripts\\element-native-map.py",
             "bb60a1452ac7ddd09a757b43f8c6f21fec0ae7a18d8fdc6ea41619d2513b1096"),
            (".reverse-engineering\\element-atlas\\native-map.json",
             "44c73370477203133c4eb8a906ed455946dbfe745c2e4150db32251d4c761627"),
            ("scripts\\element-native-map.py",
             "3ef8019b714f27a3208f20b4029e4f25532fefef82368bc2a918dd9fed97baa1"),
        },
    },
    "9771b8a288f333d9e3c93f77e0af7e540de895387b90bcdc169d19436309ce97": {
        "version": VERSION, "sha256": TARGET,
        "historical_sources": {
            ("scripts\\element-native-map.py",
             "bb60a1452ac7ddd09a757b43f8c6f21fec0ae7a18d8fdc6ea41619d2513b1096"),
            (".reverse-engineering\\element-atlas\\native-map.json",
             "44c73370477203133c4eb8a906ed455946dbfe745c2e4150db32251d4c761627"),
            ("scripts\\element-native-map.py",
             "3ef8019b714f27a3208f20b4029e4f25532fefef82368bc2a918dd9fed97baa1"),
        },
    },
    "c6f1ec984727cfe373a54cf0efbb2079a886e988e7ba3928398a2ebbd2d32d7b": {
        "version": VERSION, "sha256": TARGET,
        "historical_sources": {
            ("scripts\\element-native-map.py",
             "bb60a1452ac7ddd09a757b43f8c6f21fec0ae7a18d8fdc6ea41619d2513b1096"),
            (".reverse-engineering\\element-atlas\\native-map.json",
             "44c73370477203133c4eb8a906ed455946dbfe745c2e4150db32251d4c761627"),
        },
    },
}
ANCHORS = (
    ("JalousieUpDown2", "349", "CControlObjectPush2Jalousie2",
     "0x1407fd870", None, "0x1407ff450", 0xf18, "14041dd00"),
    ("EnergyManager2", "515", "CControlObjectEnergyManager2",
     "0x140614600", "0x140613dd0", "0x140616320", 0x370, "14041d620"),
    ("LightController2", "484", "CControlObjectLightController2",
     "0x1405bb7b0", "0x1405b8ae0", "0x1405c4cc0", 0xf18, "14041dd00"),
    ("HeatIRoomController2", "469", "CControlObjectHeatIRoomController2",
     "0x14064b110", "0x140648c90", "0x140658900", 0xf18, "14041dd00"),
)


def fail(message):
    raise ValueError(message)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            fail(f"Duplicate JSON key: {key}")
        result[key] = value
    return result


def decode_json(data):
    return json.loads(data.decode("utf-8-sig"), object_pairs_hook=unique_object,
                      parse_constant=lambda value: fail(f"Invalid JSON number: {value}"))


def load_json(path):
    return decode_json(Path(path).read_bytes())


def address(value):
    if not isinstance(value, str) or not re.fullmatch(r"0x[0-9a-fA-F]+", value):
        fail(f"Invalid native address: {value!r}")
    return hex(int(value, 16))


def control_type(value):
    if isinstance(value, bool) or not isinstance(value, (str, int)):
        fail(f"Invalid ControlType: {value!r}")
    if not re.fullmatch(r"[0-9]+", str(value)):
        fail(f"Invalid ControlType: {value!r}")
    return str(int(value))


def expect_object(value, label):
    if not isinstance(value, dict):
        fail(f"{label} must be an object")
    return value


def expect_list(value, label):
    if not isinstance(value, list):
        fail(f"{label} must be an array")
    return value


def check_count(summary, key, actual, label):
    if key in summary and (type(summary[key]) is not int or summary[key] != actual):
        fail(f"{label}.{key} disagrees with records: {summary[key]!r} != {actual}")


def source_path(value):
    if (not isinstance(value, str) or not value or "/" in value or ":" in value or
            value.startswith("\\") or ".." in value.split("\\")):
        fail("Every source path must be root-relative with Windows separators")
    return Path(*value.split("\\"))


def validate_inputs(inventory, native):
    expect_object(inventory, "inventory")
    expect_object(native, "native catalog")
    if (type(inventory.get("schema_version")) is not int or
            type(native.get("schema_version")) is not int or
            inventory["schema_version"] != 1 or native["schema_version"] != 1):
        fail("Expected inventory/native schema_version 1")
    source = expect_object(inventory.get("source"), "inventory.source")
    if source.get("executable_sha256") != TARGET or native.get("target_sha256") != TARGET:
        fail("Inventory/native executable SHA-256 mismatch or unsupported executable")
    blocks = expect_list(inventory.get("blocks"), "inventory.blocks")
    classes = expect_list(native.get("classes"), "native.classes")
    seen = set()
    for position, block in enumerate(blocks):
        expect_object(block, f"inventory.blocks[{position}]")
        index = block.get("catalog_index")
        if type(index) is not int or index < 0 or index in seen:
            fail(f"Invalid or duplicate catalog_index: {index!r}")
        seen.add(index)
        control_type(block.get("control_type"))
        xml = block.get("xml_type")
        if xml is not None and (not isinstance(xml, str) or not xml):
            fail(f"Invalid XML type at catalog_index {index}")
    rtti = set()
    for position, record in enumerate(classes):
        expect_object(record, f"native.classes[{position}]")
        typ = expect_object(record.get("type"), f"native.classes[{position}].type")
        descriptor = address(typ.get("address"))
        if descriptor in rtti:
            fail(f"Duplicate native RTTI identity: {descriptor}")
        rtti.add(descriptor)
        if not isinstance(typ.get("display_name"), str) or not typ["display_name"]:
            fail(f"Missing native class name at source_index {position}")
        for key in ("primary_tables", "secondary_loci"):
            expect_list(record.get(key), f"native.classes[{position}].{key}")
        for table in record["primary_tables"]:
            expect_object(table, "primary table")
            if type(table.get("validated_candidate")) is not bool:
                fail("Primary table must explicitly declare validated_candidate")
            if table["validated_candidate"]:
                target = address(table.get("target"))
                if (table.get("target_is_canonical_entry") is not True or
                        table.get("exact_owners") != [target]):
                    fail("Validated primary candidate lacks an exact canonical entry")
                address(table.get("vtable"))
    handlers = expect_list(native.get("handlers"), "native.handlers")
    handler_addresses = [address(h.get("address")) for h in handlers]
    if len(set(handler_addresses)) != len(handlers):
        fail("Duplicate native handler record")
    targets = {address(t["target"]) for c in classes for t in c["primary_tables"]
               if t["validated_candidate"]}
    if set(handler_addresses) != targets:
        fail("Handler records disagree with validated primary targets")
    summary = expect_object(native.get("summary"), "native.summary")
    counts = {
        "class_count": len(classes),
        "classes_with_validated_primary_candidate": sum(
            any(t["validated_candidate"] for t in c["primary_tables"]) for c in classes),
        "unique_handler_count": len(targets),
        "validated_unique_handlers_with_exact_canonical_entry": len(targets),
        "secondary_loci_preserved": sum(len(c["secondary_loci"]) for c in classes),
    }
    for key, actual in counts.items():
        check_count(summary, key, actual, "native.summary")
    ux_summary = expect_object(inventory.get("summary"), "inventory.summary")
    check_count(ux_summary, "catalog_records", len(blocks), "inventory.summary")
    check_count(ux_summary, "explicit_xml_types",
                len({b["xml_type"] for b in blocks if b["xml_type"] is not None}),
                "inventory.summary")
    return blocks, classes, counts


def reconcile_source_catalog(inventory, resource_catalog):
    originals = expect_list(resource_catalog.get("blocks"), "resource catalog.blocks")
    blocks = inventory["blocks"]
    if len(originals) != len(blocks):
        fail("Inventory dropped or added source catalog records")
    if {b["catalog_index"] for b in blocks} != set(range(len(originals))):
        fail("Inventory catalog indices do not cover every source catalog record")
    for block in blocks:
        attrs = expect_object(originals[block["catalog_index"]].get("attributes"),
                              "source catalog attributes")
        if (control_type(attrs.get("ControlType")) != control_type(block["control_type"]) or
                attrs.get("LxType") != block["xml_type"] or
                attrs.get("Name") != block.get("name")):
            fail(f"Inventory/source record identity mismatch at {block['catalog_index']}")


def factory_cases(text):
    """Parse only this reviewed factory's top-level two-space case layout."""
    clean = "\n".join(line for line in text.splitlines() if not line.startswith("//"))
    if len(re.findall(r"switch\s*\(param_1\)", clean)) != 1:
        fail("Factory must have one reviewed ControlType selector")
    if not re.search(r"\breturn plVar5\s*;", clean):
        fail("Factory result variable changed")
    start = clean.index("switch(param_1)")
    end = clean.find("\n  }\n", start)
    if end == -1:
        fail("Factory selector boundary changed")
    region = clean[start:end]
    matches = list(re.finditer(r"^  (?:case (0x[0-9a-fA-F]+|[0-9]+)|default):",
                               region, re.MULTILINE))
    result, pending = {}, []
    for i, match in enumerate(matches):
        if match[1] is not None:
            pending.append(str(int(match[1], 0)) if match[1].startswith("0x")
                           else str(int(match[1])))
        body = region[match.end():matches[i + 1].start() if i + 1 < len(matches)
                      else len(region)]
        if not body.strip():
            continue
        for key in pending:
            if key in result:
                fail(f"Duplicate factory case {key}")
            result[key] = body
        pending = []
    if pending:
        fail("Unterminated factory case group")
    return result


def prove_factory_class(case_body, native_class, constructor_body=None, constructor=None):
    if re.search(r"\bswitch\s*\(|\bgoto\b|\belse\b", case_body):
        fail("Bounded factory case has unsupported alternate control flow")
    assignments = re.findall(r"\bplVar5\s*=\s*([^;]+);", case_body)
    if len(assignments) != 1:
        fail("Bounded factory case has ambiguous result assignments")
    if constructor is None:
        if not re.fullmatch(r"plVar3", assignments[0].strip()):
            fail("Expected reviewed inline factory allocation")
        classes = re.findall(
            r"\*plVar3\s*=\s*(?:\(longlong\))?([A-Za-z_][A-Za-z_0-9]*)::vftable\s*;",
            case_body)
        if classes != [native_class]:
            fail("Inline factory primary class mismatch")
        if len(re.findall(r"\bplVar3\s*=\s*operator_new\s*\(", case_body)) != 1:
            fail("Inline factory allocation identity changed")
    else:
        call = r"\(longlong\s*\*\)FUN_" + re.escape(address(constructor)[2:]) + r"\(pvVar4\)"
        if not re.fullmatch(call, assignments[0].strip()):
            fail("Factory constructor call does not match reviewed allocation")
        if len(re.findall(r"\bpvVar4\s*=\s*operator_new\s*\(", case_body)) != 1:
            fail("Factory constructor allocation identity changed")
        classes = re.findall(
            r"\*param_1\s*=\s*([A-Za-z_][A-Za-z_0-9]*)::vftable\s*;",
            constructor_body or "")
        if classes != [native_class]:
            fail("Constructor primary vtable class mismatch")


def build_map(inventory, native, sources, proofs=()):
    blocks, originals, counts = validate_inputs(inventory, native)
    source_roles = {}
    for source in sources:
        path = source.get("path")
        source_path(path)
        if not re.fullmatch(r"[0-9a-f]{64}", source.get("sha256", "")):
            fail("Every source needs an exact SHA-256 digest")
        source_roles.setdefault(source["role"], []).append(source)
    for role in ("inventory", "native_catalog"):
        if len(source_roles.get(role, [])) != 1:
            fail(f"Expected exactly one {role} source")
    classes, by_name, by_descriptor = [], {}, {}
    for i, original in enumerate(originals):
        record = copy.deepcopy(original)
        descriptor = address(record["type"]["address"])
        record.update(class_id=f"native:{TARGET}:{i}:{descriptor}", source_index=i,
                      native_class=record["type"]["display_name"],
                      source_reference={"role": "native_catalog", "pointer": f"/classes/{i}"})
        classes.append(record)
        by_name.setdefault(record["native_class"], []).append(record)
        by_descriptor[descriptor] = record
    proven = {}
    for proof in proofs:
        key = (proof["xml_type"], control_type(proof["control_type"]))
        descriptor = address(proof["type_descriptor"])
        if descriptor not in by_descriptor:
            fail("Proof references an absent native RTTI identity")
        if key in proven and proven[key] != proof:
            fail(f"Conflicting static proofs for {key}")
        if not proof.get("evidence") or proof.get("basis") != "factory_constructor_and_xml_registration":
            fail("Proven joins require factory/constructor AND XML registration evidence")
        proven[key] = proof
    joins, records, joined, status_counts = [], [], set(), Counter()
    for i, block in enumerate(blocks):
        xml = block["xml_type"]
        proof = proven.get((xml, control_type(block["control_type"])))
        candidates = by_name.get("CControlObject" + xml, []) if xml is not None else []
        if proof is not None:
            candidates = [by_descriptor[address(proof["type_descriptor"])]]
            basis = proof["basis"]
            status = ("exact" if candidates[0]["native_class"] == "CControlObject" + xml
                      else "alias")
        else:
            basis = "exact_stripped_class_name_only_no_factory_or_xml_registration_proof"
            status = "ambiguous" if len(candidates) > 1 else "candidate"
        indices = []
        for candidate in candidates:
            indices.append(len(joins))
            joined.add(candidate["class_id"])
            joins.append({
                "catalog_index": block["catalog_index"], "xml_type": xml,
                "control_type": block["control_type"], "class_id": candidate["class_id"],
                "status": status, "basis": basis,
                "evidence": copy.deepcopy(proof["evidence"]) if proof else [
                    {"role": "inventory", "pointer": f"/blocks/{i}/xml_type"},
                    candidate["source_reference"]],
            })
        record_status = status if candidates else "unresolved"
        status_counts[record_status] += 1
        records.append({
            "source_index": i, "catalog_index": block["catalog_index"],
            "xml_type": xml, "control_type": block["control_type"], "name": block.get("name"),
            "status": record_status, "join_indices": indices,
        })
    if len(records) != len(blocks) or len(classes) != len(originals):
        fail("Internal error: source records were dropped")
    unjoined_classes = [c["class_id"] for c in classes if c["class_id"] not in joined]
    return {
        "schema_version": 1, "application": {"version": VERSION, "sha256": TARGET},
        "sources": sorted(copy.deepcopy(sources), key=lambda s: (s["role"], s["path"])),
        "classes": classes, "handlers": copy.deepcopy(native["handlers"]),
        "roots": copy.deepcopy(native.get("roots", [])),
        "catalog_records": records, "joins": joins,
        "unjoined_catalog_indices": [r["catalog_index"] for r in records if not r["join_indices"]],
        "unjoined_class_ids": unjoined_classes,
        "summary": {
            "native_classes": counts["class_count"],
            "classes_with_validated_primary_candidates": counts["classes_with_validated_primary_candidate"],
            "unique_exact_canonical_handler_entries": counts["unique_handler_count"],
            "secondary_loci": counts["secondary_loci_preserved"],
            "catalog_records": len(blocks),
            "distinct_control_type_ids": len({control_type(b["control_type"]) for b in blocks}),
            "distinct_non_null_xml_types": len({b["xml_type"] for b in blocks if b["xml_type"] is not None}),
            "null_xml_records": sum(b["xml_type"] is None for b in blocks),
            "join_records": len(joins),
            "catalog_records_by_status": {k: status_counts[k] for k in
                                          ("exact", "alias", "candidate", "ambiguous", "unresolved")},
            "unjoined_classes": len(unjoined_classes),
        },
        "limitations": [
            "Static metadata only; no code execution, simulator support or behavioral semantics established.",
            "Primary vtable validation confirms candidate pointers, not handler semantics or class dispatch.",
            "Only four reviewed factory/constructor/XML-registration joins are promoted beyond name candidates.",
            "ControlType is not a record identity; catalog_index and source_index preserve duplicate IDs.",
            "Missing XML/native mappings do not imply internal, hidden, unsupported or absent elements.",
            "Factory/XML joins do not recover properties, connector semantics, runtime instances or an engine.",
        ],
    }


class Sources:
    def __init__(self, root=ROOT):
        self.root = Path(root).resolve()
        self.items = {}

    def add(self, path, role, expected=None):
        path = Path(path).resolve()
        if not path.is_relative_to(self.root):
            fail(f"Source must remain under the repository root: {path.name}")
        data = path.read_bytes()
        actual = digest(data)
        if expected is not None and actual != expected:
            fail(f"Source hash mismatch: {path.relative_to(self.root)}")
        relative = str(path.relative_to(self.root)).replace("/", "\\")
        self.items[(role, relative)] = {"path": relative, "sha256": actual, "role": role}
        return data


def factory_proof_joins(report, inventory, native, baseline):
    blocks, classes, _ = validate_inputs(inventory, native)
    if (type(report.get("schema_version")) is not int or report["schema_version"] != 1 or
            report.get("kind") != "private-static-bulk-factory-identity-proof" or
            report.get("application") != {
                "version": VERSION, "sha256": TARGET, "image_base": "0x140000000"}):
        fail("Reviewed factory proof schema/application mismatch")
    by_descriptor = {address(c["type"]["address"]): (i, c) for i, c in enumerate(classes)}
    by_index = {b["catalog_index"]: b for b in blocks}
    associations, proofs = {}, []
    successes = expect_list(report.get("successes"), "factory successes")
    for i, item in enumerate(successes):
        expect_object(item, "factory association")
        ct = control_type(item.get("control_type"))
        descriptor = address(item.get("type_descriptor"))
        if ct in associations:
            fail(f"Duplicate or contradictory factory ControlType: {ct}")
        if descriptor not in by_descriptor:
            fail("Factory proof references an absent native RTTI identity")
        index, record = by_descriptor[descriptor]
        if (type(item.get("native_source_index")) is not int or
                item["native_source_index"] != index or
                item.get("native_class") != record["type"]["display_name"] or
                item.get("basis") != "factory_constructor_and_xml_registration" or
                item.get("factory") != FACTORY or
                not isinstance(item.get("xml_type"), str) or not item["xml_type"]):
            fail("Factory association/native class identity mismatch")
        if not any(address(t["vtable"]) == address(item.get("primary_vtable")) and
                   t.get("object_offset") == 0 and t.get("primary_root_compatible") is True
                   for t in record["primary_tables"]):
            fail("Factory association primary vtable mismatch")
        registrations = expect_list(item.get("registrations"), "factory XML registrations")
        if not registrations or any(r.get("slot_offset") != "0xf18" or
                                    r.get("xml_type") != item["xml_type"] for r in registrations):
            fail("Factory association lacks its reviewed XML registration")
        associations[ct] = item
        proofs.append({
            "xml_type": item["xml_type"], "control_type": ct, "type_descriptor": descriptor,
            "basis": item["basis"],
            "evidence": [{"role": "reviewed_factory_proof", "pointer": f"/successes/{i}",
                          "fact": "reviewed_static_factory_primary_vtable_and_XML_registration"}],
        })
    records = expect_list(report.get("catalog_records"), "factory catalog records")
    seen = set()
    for item in records:
        index = item.get("catalog_index")
        if type(index) is not int or index not in by_index or index in seen:
            fail("Factory catalog record membership mismatch")
        block = by_index[index]
        if (item.get("source_record") != block or
                control_type(item.get("control_type")) != control_type(block["control_type"]) or
                item.get("xml_type") != block["xml_type"]):
            fail("Factory catalog record identity mismatch")
        seen.add(index)
    if seen != set(by_index):
        fail("Factory report dropped catalog records")
    ledger = expect_list(report.get("native_class_ledger"), "factory native class ledger")
    if len(ledger) != len(classes):
        fail("Factory report dropped native classes")
    for i, (item, record) in enumerate(zip(ledger, classes)):
        if (type(item.get("source_index")) is not int or item["source_index"] != i or
                address(item.get("type_descriptor")) != address(record["type"]["address"]) or
                item.get("native_class") != record["type"]["display_name"]):
            fail("Factory native class ledger identity mismatch")
    expected = {}
    rejected = set()
    for block in blocks:
        item = associations.get(control_type(block["control_type"]))
        if item is None:
            continue
        if block["xml_type"] is None or block["xml_type"] != item["xml_type"]:
            rejected.add(block["catalog_index"])
        else:
            expected[block["catalog_index"]] = item
    seen = set()
    joins = expect_list(report.get("catalog_joins"), "factory catalog joins")
    for item in joins:
        index = item.get("catalog_index")
        if type(index) is not int or index not in expected or index in seen:
            fail("Factory catalog join membership mismatch")
        association = expected[index]
        status = ("exact" if association["native_class"] == "CControlObject" + association["xml_type"]
                  else "alias")
        if (control_type(item.get("control_type")) != control_type(association["control_type"]) or
                item.get("xml_type") != association["xml_type"] or
                address(item.get("type_descriptor")) != address(association["type_descriptor"]) or
                item.get("native_class") != association["native_class"] or
                item.get("status") != status or
                type(item.get("proof_index")) is not int or
                not 0 <= item["proof_index"] < len(successes) or
                successes[item["proof_index"]] != association):
            fail("Factory catalog join identity mismatch")
        seen.add(index)
    if seen != set(expected):
        fail("Factory report dropped admissible catalog joins")
    rejections = expect_list(report.get("catalog_join_rejections"), "factory catalog rejections")
    rejection_indices = [r.get("catalog_index") for r in rejections]
    if (any(type(i) is not int for i in rejection_indices) or
            len(set(rejection_indices)) != len(rejection_indices) or
            set(rejection_indices) != rejected):
        fail("Factory report dropped nonadmitted catalog records")
    merged = list(baseline)
    baseline_by_key = {(p["xml_type"], control_type(p["control_type"])): p for p in baseline}
    for item in baseline:
        match = associations.get(control_type(item["control_type"]))
        if (match is None or match["xml_type"] != item["xml_type"] or
                address(match["type_descriptor"]) != address(item["type_descriptor"])):
            fail("Factory report contradicts or omits a baseline proof")
    merged.extend(p for p in proofs if (p["xml_type"], p["control_type"]) not in baseline_by_key)
    summary = expect_object(report.get("summary"), "factory summary")
    for key, actual in (
            ("catalog_records", len(blocks)), ("native_classes", len(classes)),
            ("proven_factory_CT_class_XML_associations", len(successes)),
            ("proven_catalog_records", len(joins)),
            ("distinct_native_classes_proven_joined_to_catalog",
             len({item["type_descriptor"] for item in expected.values()})),
            ("catalog_rows_with_factory_proof_not_admitted_as_XML_join", len(rejections))):
        if key not in summary:
            fail(f"Factory summary missing {key}")
        check_count(summary, key, actual, "factory summary")
    return merged


def reviewed_factory_proofs(path, inventory, native, sources, baseline):
    data = sources.add(path, "reviewed_factory_proof")
    policy = REVIEWED_FACTORY_PROOFS.get(digest(data))
    if policy is None:
        fail("Unreviewed factory proof SHA-256; explicit static-identity review is required")
    report = decode_json(data)
    application = expect_object(report.get("application"), "factory application")
    if any(application.get(key) != policy[key] for key in ("version", "sha256")):
        fail("Reviewed factory proof application mismatch")
    current = {s["role"]: s for s in sources.items.values()
               if s["role"] in {"inventory", "native_catalog", "executable"}}
    pins = {}
    for item in expect_list(report.get("sources"), "factory sources"):
        expect_object(item, "factory source")
        relative = source_path(item.get("path"))
        checksum = item.get("sha256")
        if (not isinstance(checksum, str) or not re.fullmatch(r"[0-9a-f]{64}", checksum) or
                type(item.get("bytes")) is not int or item["bytes"] < 0):
            fail("Invalid reviewed factory source digest/byte count")
        if (item["path"], checksum) in policy.get("historical_sources", set()):
            # Historical producers are not live inputs or an output-overwrite dependency.
            content = sources.add(Path(path).resolve().parent / "sources" / checksum,
                                  "reviewed_factory_source", checksum)
        else:
            content = sources.add(sources.root / relative, "reviewed_factory_source", checksum)
        if len(content) != item["bytes"]:
            fail("Reviewed factory source byte count mismatch")
        pins.setdefault(item.get("role"), []).append(item)
    for role, input_role in (("inventory", "inventory"), ("native-catalog", "native_catalog"),
                             ("baseline-executable", "executable")):
        if (len(pins.get(role, [])) != 1 or input_role not in current or
                pins[role][0]["sha256"] != current[input_role]["sha256"]):
            fail(f"Reviewed factory proof/input source mismatch: {input_role}")
    merged = factory_proof_joins(report, inventory, native, baseline)
    trial = build_map(inventory, native, list(sources.items.values()), merged)
    actual = {k: v for k, v in trial["summary"]["catalog_records_by_status"].items() if v}
    if report["summary"].get("catalog_record_status") != actual:
        fail("Reviewed factory catalog status counts mismatch")
    return merged


def bounded_proofs(corpus, image, native, sources, helper):
    factory_path, factory_text = corpus.body(FACTORY)
    sources.add(factory_path, "canonical_factory_body", FACTORY_SHA256)
    cases = factory_cases(helper.pseudocode(factory_text))
    proofs = []
    for xml, ct, name, handler, ctor, registration, slot, register_helper in ANCHORS:
        matches = [r for r in native["classes"] if r["type"]["display_name"] == name]
        if len(matches) != 1:
            fail(f"Reviewed native class is missing or ambiguous: {name}")
        record = matches[0]
        tables = [t for t in record["primary_tables"] if t["validated_candidate"]]
        if len(tables) != 1 or tables[0]["target"] != handler:
            fail(f"Reviewed primary handler changed: {name}")
        table = tables[0]
        if (table.get("object_offset") != 0 or
                table.get("primary_root_compatible") is not True):
            fail(f"Reviewed primary lineage changed: {name}")
        ctor_text = None
        evidence = [{"role": "canonical_factory_body", "address": FACTORY,
                     "sha256": FACTORY_SHA256, "selector": "param_1",
                     "case": ct, "fact": "allocation_on_ControlType_case"}]
        if ctor:
            path, ctor_text = corpus.body(ctor)
            sources.add(path, "canonical_constructor_body")
            evidence.append({"role": "canonical_constructor_body", "address": ctor,
                             "sha256": helper.sha256(path),
                             "fact": "primary_object_vtable_assignment"})
        if ct not in cases:
            fail(f"Reviewed factory case absent: {ct}")
        prove_factory_class(cases[ct], name, ctor_text, ctor)
        vt = int(address(table["vtable"]), 16)
        for offset, target in ((0x50, handler), (slot, registration)):
            raw = image.read(vt + offset, 8)
            if struct.unpack("<Q", raw)[0] != int(target, 16):
                fail(f"Reviewed vtable pointer changed: {name}+{hex(offset)}")
            evidence.append({"role": "executable", "address": hex(vt + offset),
                             "bytes": 8, "sha256": digest(raw), "target": target,
                             "slot_offset": hex(offset), "fact": "static_slot_pointer"})
        path, text = corpus.body(registration)
        sources.add(path, "canonical_xml_registration_body")
        expression = (r"\bFUN_" + register_helper +
                      r"\(\s*param_2(?:\s*\+\s*0x18)?\s*,\s*\"" +
                      re.escape(xml) + r"\"\s*\)")
        if len(re.findall(expression, helper.pseudocode(text))) != 1:
            fail(f"Reviewed XML registration changed: {xml}")
        evidence.append({"role": "canonical_xml_registration_body", "address": registration,
                         "sha256": helper.sha256(path), "xml_type": xml,
                         "fact": "literal_XML_registration_on_same_primary_vtable"})
        proofs.append({"xml_type": xml, "control_type": ct,
                       "type_descriptor": record["type"]["address"],
                       "basis": "factory_constructor_and_xml_registration", "evidence": evidence})
    return proofs


def extract(inventory_path, native_path, latest_path, factory_proof=None):
    sources = Sources()
    inventory = decode_json(sources.add(inventory_path, "inventory"))
    native = decode_json(sources.add(native_path, "native_catalog"))
    validate_inputs(inventory, native)
    latest = decode_json(sources.add(latest_path, "latest"))
    if (native.get("analysis_id") != latest.get("analysis_id") or
            native.get("installation_sha256") != latest.get("installation_sha256")):
        fail("Latest/native analysis or installation identity mismatch")
    binary = latest["binaries"]["LoxoneConfig.exe"]
    if binary["export"]["target_sha256"] != TARGET:
        fail("Latest executable SHA-256 mismatch")
    state = Path(inventory["source"]["state"]).resolve()
    if state != Path(latest_path).resolve():
        fail("Inventory references a different latest manifest")
    for artifact in native.get("artifacts", []):
        sources.add(artifact["path"], "native_catalog_evidence", artifact["sha256"])
    resource_path = Path(inventory["source"]["catalog"])
    resources = decode_json(sources.add(resource_path, "source_catalog"))
    reconcile_source_catalog(inventory, resources)
    helper_path = ROOT / "scripts" / "native-sim-scope.py"
    sources.add(helper_path, "corpus_helper")
    sources.add(__file__, "extractor")
    spec = importlib.util.spec_from_file_location("element_native_scope", helper_path)
    helper = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(helper)
    helper.load = load_json
    corpus = helper.Corpus(latest_path)
    if corpus.target != TARGET:
        fail("Canonical index executable SHA-256 mismatch")
    sources.add(corpus.index_path, "canonical_index")
    executables = [Path(a["path"]) for a in native.get("artifacts", [])
                   if Path(a["path"]).name == "LoxoneConfig.exe" and a["sha256"] == TARGET]
    if len(executables) != 1:
        fail("Expected exactly one hash-pinned source executable")
    sources.add(executables[0], "executable", TARGET)
    image = helper.PE(executables[0])
    if image.sha256 != TARGET:
        fail("Executable image SHA-256 mismatch")
    proofs = bounded_proofs(corpus, image, native, sources, helper)
    if factory_proof is not None:
        proofs = reviewed_factory_proofs(factory_proof, inventory, native, sources, proofs)
    result = build_map(inventory, native, list(sources.items.values()), proofs)
    if factory_proof is not None:
        result["limitations"][2] = (
            "Only baseline proofs and the explicitly reviewed digest-pinned factory report "
            "are promoted beyond name candidates; this is static identity, not runtime proof.")
    return result


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--inventory", type=Path, required=True)
    parser.add_argument("--native-catalog", type=Path, required=True)
    parser.add_argument("--latest", type=Path, default=ROOT / ".reverse-engineering" / "latest.json")
    parser.add_argument("--factory-proof", type=Path,
                        help="Optional explicitly reviewed, digest-pinned static factory report")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args(argv)
    try:
        output = args.output.resolve()
        private = (ROOT / ".reverse-engineering").resolve()
        if not output.is_relative_to(private):
            fail("Generated native metadata must remain under .reverse-engineering")
        inputs = {args.inventory.resolve(), args.native_catalog.resolve(), args.latest.resolve()}
        if args.factory_proof is not None:
            inputs.add(args.factory_proof.resolve())
        if output in inputs:
            fail("Output must not overwrite an input")
        result = extract(args.inventory, args.native_catalog, args.latest, args.factory_proof)
        if output in {ROOT / source["path"] for source in result["sources"]}:
            fail("Output must not overwrite any source evidence")
        data = (json.dumps(result, indent=2, ensure_ascii=True, allow_nan=False) + "\n").encode("utf-8")
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_bytes(data)
        print(json.dumps({"output": str(output.relative_to(ROOT)).replace("/", "\\"),
                          "sha256": digest(data), "summary": result["summary"]}, sort_keys=True))
    except (ValueError, KeyError, TypeError, OSError) as error:
        print(f"element-native-map: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
