#!/usr/bin/env python3
"""Extract a lossless, static native identity sidecar; never execute the image.

Usage:
  python scripts\\element-native-map.py --inventory INVENTORY --native-catalog CATALOG --output OUTPUT

The four reviewed factory cases are deliberately bounded. All other exact
stripped-class-name matches remain candidates, not XML or execution proof.
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
        if (not isinstance(path, str) or not path or "/" in path or
                ":" in path or path.startswith("\\") or ".." in path.split("\\")):
            fail("Every source path must be root-relative with Windows separators")
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


def extract(inventory_path, native_path, latest_path):
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
    return build_map(inventory, native, list(sources.items.values()), proofs)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--inventory", type=Path, required=True)
    parser.add_argument("--native-catalog", type=Path, required=True)
    parser.add_argument("--latest", type=Path, default=ROOT / ".reverse-engineering" / "latest.json")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args(argv)
    try:
        output = args.output.resolve()
        private = (ROOT / ".reverse-engineering").resolve()
        if not output.is_relative_to(private):
            fail("Generated native metadata must remain under .reverse-engineering")
        if output in {args.inventory.resolve(), args.native_catalog.resolve(), args.latest.resolve()}:
            fail("Output must not overwrite an input")
        result = extract(args.inventory, args.native_catalog, args.latest)
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
