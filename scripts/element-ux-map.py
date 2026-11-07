#!/usr/bin/env python3
"""Extract shared UX mechanism leads from a saved, private metadata inventory.

Usage: python scripts\\element-ux-map.py --inventory PATH --output PATH
Optional --state, --qt-metadata, --native-dialogs and --advanced-blocks select
explicit artifacts; otherwise use the inventory's state and sibling artifacts.
No target execution, GUI, database search, or new decompilation is performed.

Schema 1 distinguishes catalog metadata membership from candidate native/name
associations. catalog_index identifies a record; ControlType is not unique.
Sources use repository-relative Windows paths and SHA-256 of exact input bytes.
Exit 0 means extraction completed, even with preserved unavailable-code leads,
not UX parity. Exit 2 means a required input/pin or requested scope is missing;
1 means invalid input or a contradictory identity/hash. Output remains private.
"""

import argparse
from collections import Counter, defaultdict
import csv
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import re
import struct
import sys


ROOT = Path(__file__).resolve().parents[1]
BOUNDS = {"anchor_bodies": 384, "dispatcher_bodies": 128,
          "shared_helper_bodies": 64, "body_bytes": 262144}
CONSTRAINTS = ("default", "min", "max", "step", "range", "unit", "condition")


class RequiredEvidenceError(ValueError):
    """A declared, required evidence artifact or digest is unavailable."""


def digest(data):
    return hashlib.sha256(data).hexdigest()


def canonical(value):
    if not isinstance(value, str) or not re.fullmatch(r"0x[0-9a-fA-F]+", value):
        raise ValueError(f"Invalid native address: {value!r}")
    return hex(int(value, 16))


def normalized(value):
    return re.sub(r"[^a-z0-9]", "", (value or "").lower())


def stable_id(prefix, key):
    return prefix + ":" + digest(key.encode("utf-8"))[:16]


class Sources:
    def __init__(self, root):
        self.root = Path(root).resolve()
        self.records = {}

    def path(self, path):
        path = Path(path)
        return path.resolve() if path.is_absolute() else (self.root / path).resolve()

    def relative(self, path):
        try:
            return str(self.path(path).relative_to(self.root)).replace("/", "\\")
        except ValueError as error:
            raise ValueError(f"Evidence must be within repository: {path}") from error

    def read(self, path, role, expected=None):
        path = self.path(path)
        relative = self.relative(path)
        data = path.read_bytes()
        checksum = digest(data)
        if expected is not None and checksum != expected:
            raise ValueError(f"Source hash mismatch ({role}): {relative}")
        self.records[(relative, role)] = {
            "path": relative, "sha256": checksum, "role": role,
        }
        return data

    def json(self, path, role, expected=None):
        return json.loads(self.read(path, role, expected).decode("utf-8-sig"))

    def evidence(self, path, pointer):
        relative = self.relative(path)
        checksums = {row["sha256"] for (name, _), row in self.records.items()
                     if name == relative}
        if len(checksums) != 1:
            raise ValueError(f"Unpinned or changing evidence: {relative}")
        return {"source": relative, "sha256": checksums.pop(), "pointer": pointer}

    def all(self):
        return sorted(self.records.values(), key=lambda row: (row["path"], row["role"]))


def pe_version(data):
    """Read RT_VERSION/VS_FIXEDFILEINFO, not an arbitrary matching byte string."""
    if len(data) < 64 or data[:2] != b"MZ":
        raise ValueError("Executable is not a PE image")
    pe = struct.unpack_from("<I", data, 60)[0]
    if data[pe:pe + 4] != b"PE\0\0":
        raise ValueError("Invalid PE signature")
    count = struct.unpack_from("<H", data, pe + 6)[0]
    optional_size = struct.unpack_from("<H", data, pe + 20)[0]
    optional = pe + 24
    magic = struct.unpack_from("<H", data, optional)[0]
    directories = optional + {0x10B: 96, 0x20B: 112}.get(magic, 0)
    if directories == optional or directories + 24 > optional + optional_size:
        raise ValueError("Unsupported PE resource directory")
    resource_rva, resource_size = struct.unpack_from("<II", data, directories + 16)
    sections = []
    for i in range(count):
        size, rva, raw_size, offset = struct.unpack_from(
            "<4I", data, optional + optional_size + i * 40 + 8)
        sections.append((rva, raw_size, offset))

    def file_offset(rva, size):
        matches = [offset + rva - start for start, length, offset in sections
                   if start <= rva and rva + size <= start + length]
        if len(matches) != 1 or matches[0] + size > len(data):
            raise ValueError("PE resource has no unique file-backed range")
        return matches[0]

    if not resource_rva or not resource_size:
        return None
    base = file_offset(resource_rva, resource_size)

    def entries(relative):
        if relative < 0 or relative + 16 > resource_size:
            raise ValueError("PE resource directory outside declared range")
        named, ids = struct.unpack_from("<HH", data, base + relative + 12)
        if relative + 16 + (named + ids) * 8 > resource_size:
            raise ValueError("Truncated PE resource directory entries")
        return [struct.unpack_from("<II", data, base + relative + 16 + i * 8)
                for i in range(named + ids)]

    payloads = []

    def visit(relative, depth):
        if depth > 3:
            raise ValueError("Unexpected RT_VERSION resource depth")
        for _, child in entries(relative):
            if child & 0x80000000:
                visit(child & 0x7FFFFFFF, depth + 1)
            else:
                if child + 16 > resource_size:
                    raise ValueError("PE resource data entry outside declared range")
                rva, size = struct.unpack_from("<II", data, base + child)
                offset = file_offset(rva, size)
                payloads.append(data[offset:offset + size])

    for name, child in entries(0):
        if name == 16:
            if not child & 0x80000000:
                raise ValueError("RT_VERSION must have a resource directory")
            visit(child & 0x7FFFFFFF, 1)
    versions = set()
    key = "VS_VERSION_INFO\0".encode("utf-16-le")
    for payload in payloads:
        length, value_length, kind = struct.unpack_from("<HHH", payload)
        offset = (6 + len(key) + 3) & ~3
        if (payload[6:6 + len(key)] != key or length > len(payload)
                or value_length < 52 or kind != 0 or offset + 52 > length):
            raise ValueError("Unsupported VS_VERSION_INFO")
        signature, revision, ms, ls = struct.unpack_from("<4I", payload, offset)
        if signature != 0xFEEF04BD or revision != 0x10000:
            raise ValueError("Invalid VS_FIXEDFILEINFO")
        versions.add(f"{ms >> 16}.{ms & 65535}.{ls >> 16}.{ls & 65535}")
    if len(versions) > 1:
        raise ValueError("Conflicting executable file versions")
    return next(iter(versions), None)


def validate_blocks(inventory):
    blocks = inventory.get("blocks")
    if inventory.get("schema_version") != 1 or not isinstance(blocks, list):
        raise ValueError("Expected schema-1 block-ux-coverage inventory")
    indices = [row.get("catalog_index") for row in blocks]
    if any(type(index) is not int or index < 0 for index in indices):
        raise ValueError("Every record requires a nonnegative catalog_index")
    if len(set(indices)) != len(indices):
        raise ValueError("Duplicate catalog_index")
    return sorted(blocks, key=lambda row: row["catalog_index"])


def make_mechanism(identifier, kind, status, indices, method, reason, evidence,
                   blocks, parameter_refs=None, **extra):
    indices = sorted(set(indices))
    by_index = {row["catalog_index"]: row for row in blocks}
    if any(index not in by_index for index in indices):
        raise ValueError("Mechanism references an absent catalog_index")
    refs = sorted(parameter_refs or [], key=lambda row: (
        row["catalog_index"], row["parameter_index"]))
    membership = "metadata_membership" if status == "metadata_lead" else (
        "candidate" if indices else "unjoined_lead")
    return {
        "mechanism_id": identifier, "kind": kind, "status": status,
        "catalog_indices": indices,
        "membership_basis": {"status": membership, "method": method,
                             "uncertainty": reason, "execution_verified": False},
        "native_addresses": [],
        "evidence": evidence,
        "potential_impact": {
            "catalog_records": len(indices) if indices else None,
            "catalog_parameter_entries": sum(len(by_index[i].get("parameters", []))
                                             for i in indices) if indices else None,
            "selected_parameter_entries": len(refs) if parameter_refs is not None else None,
            "is_ux_coverage": False,
        },
        "parameter_refs": refs,
        **extra,
    }


def catalog_mechanisms(blocks, evidence):
    grouped = defaultdict(list)
    nested = defaultdict(list)
    for block in blocks:
        index = block["catalog_index"]
        for i, parameter in enumerate(block.get("parameters", [])):
            ref = {"catalog_index": index, "parameter_index": i,
                   "name_pattern": parameter.get("name_pattern")}
            shape = [key for key in CONSTRAINTS if parameter.get(key) is not None]
            grouped[("parameter_declaration_shape", "+".join(shape) or "no_constraints")].append(ref)
            grouped[("parameter_xml_mapping", parameter.get("mapping_status", "unspecified"))].append(ref)
            if re.search(r"%[de]", parameter.get("name_pattern") or ""):
                grouped[("parameter_pattern", "sampled_not_exhaustive")].append(ref)
        paths = block.get("storage_observations", {}).get("paths", [])
        roots = sorted({path.split("/")[0] for path in paths
                        if path and path != "SET" and not path.startswith("SET/")})
        for root in roots:
            nested[root].append({
                "catalog_index": index,
                "paths": sorted({path for path in paths
                                 if path == root or path.startswith(root + "/")}),
            })
    result = []
    for (kind, key), refs in sorted(grouped.items()):
        result.append(make_mechanism(
            stable_id("catalog", kind + ":" + key), kind, "metadata_lead",
            [ref["catalog_index"] for ref in refs], "inventory_parameter_entries",
            "Catalog declarations/mappings are not native editor bindings, enforcement, "
            "or a GUI-field denominator; sampled pattern keys are not channel limits.",
            [evidence], blocks, refs, pattern=key))
    for root, observations in sorted(nested.items()):
        result.append(make_mechanism(
            stable_id("nested-storage", root), "nested_storage_exception", "metadata_lead",
            [row["catalog_index"] for row in observations], "fixture_storage_shape_in_inventory",
            "Observed nested storage must not inherit scalar-editor behavior; CRUD, order, "
            "reference validity and roundtrip behavior remain unverified.",
            [evidence], blocks, storage_root=root, observations=observations))
    return result


def name_candidates(names, blocks):
    keys = {normalized(name) for name in names if len(normalized(name)) >= 4}
    return sorted({row["catalog_index"] for row in blocks
                   if keys.intersection({normalized(row.get("xml_type")),
                                         normalized(row.get("name"))})})


def class_core(name):
    name = name.split("::")[-1]
    if re.match(r"C[A-Z]", name):
        name = name[1:]
    return re.sub(r"(Configuration|Config|Dialog|Dlg|Editor|Widget|Model|View)", "", name)


def qt_mechanisms(qt, inventory, blocks, evidence):
    classes = []
    groups = defaultdict(list)
    for source_index, row in enumerate(qt.get("classes", [])):
        name = row["class_name"]
        record = {"class_id": stable_id("qt-class", name + ":" + str(row.get("metaobject_address"))),
                  "source_index": source_index, "class_name": name,
                  "dispatcher_address": row.get("dispatcher_address"),
                  "dispatcher_export_status": row.get("dispatcher_export", {}).get("status"),
                  "metaobject_address": row.get("metaobject_address"),
                  "methods": row.get("methods", []), "properties": row.get("properties", [])}
        classes.append(record)
        match = re.search(r"((?:Setting|Property|Attribute)Editor)(.*)", name)
        if match:
            key = "shared-" + match[1] + ":" + (re.sub(r"Fancy$", "", match[2]) or "base")
        elif any(term in name.lower() for term in
                 ("setting", "property", "attribute", "validat", "serial")):
            key = "settings-infrastructure"
        elif any(term in name.lower() for term in ("dlg", "dialog", "editor", "configuration")):
            key = "editor:" + normalized(class_core(name))
        else:
            continue
        groups[key].append(record)
    result = []
    for key, records in sorted(groups.items()):
        names = {row["class_name"] for row in records}
        direct = name_candidates([class_core(name) for name in names], blocks)
        families = []
        candidates = set(direct)
        for family, definition in sorted(inventory.get("families", {}).items()):
            leads = {row["class_name"] for row in definition.get("qt_name_leads", [])}
            if names.intersection(leads):
                families.append(family)
                candidates.update(row["catalog_index"] for row in blocks
                                  if family in row.get("families", []))
        result.append(make_mechanism(
            stable_id("qt", key), "shared_editor_lead" if key.startswith("shared-")
            else "qt_editor_or_settings_lead", "candidate", candidates,
            "normalized_class_name_or_existing_family_name_lead",
            "Class/member/property declarations and family names do not prove a block uses "
            "this editor. Shared dispatchers are not individual method implementations; "
            "a declared Qt property may only control styling.",
            [evidence], blocks, group=key, class_ids=sorted(row["class_id"] for row in records),
            class_names=sorted(names), family_name_candidates=families,
            exact_core_name_candidate_indices=direct,
            native_addresses=sorted({row["dispatcher_address"] for row in records
                                     if row["dispatcher_address"]}),
            address_role="qt_static_metacall_dispatcher_not_per_method"))
    operations = {
        "create-record": r"^(?:On|on)?(?:Add|Create|Insert)",
        "delete-record": r"^(?:On|on)?(?:Delete|Remove)",
        "reorder-record": r"^(?:On|on)?(?:SortUp|SortDown|MoveUp|MoveDown)",
        "validate-content": r"(?:CheckCode|Validat|Verify|Unfulfilled)",
        "serialize-content": r"^(?:On|on)?(?:Save|Load|Read|Write)(?:Text|Code|Settings|Data)",
        "reset-setting": r"(?:ResetSetting|ResetButton|SettingNeedRefresh|SettingWritten)",
    }
    for operation, pattern in operations.items():
        declarations = []
        operation_indices = set()
        for row in classes:
            members = [method for method in row["methods"]
                       if re.search(pattern, method["name"])]
            if not members:
                continue
            declarations.append({"class_id": row["class_id"],
                                 "method_indices": [method["index"] for method in members],
                                 "method_names": [method["name"] for method in members]})
            operation_indices.update(name_candidates([class_core(row["class_name"])], blocks))
            for family, definition in inventory.get("families", {}).items():
                if row["class_name"] in {
                        lead["class_name"] for lead in definition.get("qt_name_leads", [])}:
                    operation_indices.update(block["catalog_index"] for block in blocks
                                             if family in block.get("families", []))
        if len(declarations) >= 2:
            result.append(make_mechanism(
                "qt-shared-operation:" + operation, "shared_editor_operation_lead", "candidate",
                operation_indices, "repeated_declared_member_name_pattern",
                "Declared method names suggest reusable editing/validation operations, not "
                "their implementation, reachability, acceptance rules or catalog ownership.",
                [evidence], blocks, operation=operation, declarations=declarations,
                class_ids=sorted(row["class_id"] for row in declarations)))
    return classes, result


def dialog_mechanisms(dialogs, blocks, evidence):
    groups = defaultdict(list)
    for source_index, dialog in enumerate(dialogs.get("dialogs", [])):
        counts = Counter(json.dumps(control.get("window_class", {}), sort_keys=True)
                         for control in dialog.get("controls", []))
        for key, count in sorted(counts.items()):
            caption = dialog.get("caption", {}).get("value")
            groups[key].append({
                "source_index": source_index, "resource_index": dialog.get("resource_index"),
                "resource_name": dialog.get("name"), "language": dialog.get("language"),
                "caption": caption, "static_control_count": count,
                "caption_candidate_indices": name_candidates([caption], blocks),
            })
    result = []
    for key, observations in sorted(groups.items()):
        result.append(make_mechanism(
            stable_id("dialog-control-class", key), "static_dialog_control_class", "candidate",
            [index for row in observations for index in row["caption_candidate_indices"]],
            "static_template_control_class_and_exact_normalized_caption",
            "A template control class is not a reachable or verified widget. Captions are "
            "name candidates only; dynamic controls and active dialog versions are unknown.",
            [evidence], blocks, window_class=json.loads(key), templates=observations,
            static_control_count=sum(row["static_control_count"] for row in observations)))
    diagnostics = []
    for table_index, table in enumerate(dialogs.get("string_tables", [])):
        for entry_index, entry in enumerate(table.get("entries", [])):
            text = entry.get("text", "")
            cues = sorted(set(re.findall(
                r"\b(?:invalid|must|cannot|out of range|at least|maximum|minimum)\b",
                text, re.IGNORECASE)))
            if cues:
                diagnostics.append({"string_id": entry["id"], "table_source_index": table_index,
                                    "entry_source_index": entry_index, "name_cues": cues,
                                    "text_sha256": digest(text.encode("utf-8"))})
    if diagnostics:
        result.append(make_mechanism(
            "resources:validation-message-candidates", "validation_message_lead", "candidate", [],
            "diagnostic_word_candidates",
            "Resource text may be help, logging or an unreachable message; no validation "
            "rule or catalog binding is established.", [evidence], blocks, messages=diagnostics))
    return result


def body_code(text):
    tokens = r'"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\'|/\*.*?\*/|//[^\n]*'
    text = re.sub(tokens, lambda match: "\n" * match[0].count("\n")
                  if match[0].startswith(("//", "/*")) else match[0], text, flags=re.DOTALL)
    # Exclude the exported declaration as well as export-header callee mentions.
    return text.partition("{")[2]


def direct_calls(text):
    code = re.sub(r'"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\'', "", body_code(text))
    return Counter(canonical("0x" + match) for match in
                   re.findall(r"\b(?:thunk_)?FUN_([0-9a-fA-F]+)\s*\(", code))


def helper_kind(text):
    code = body_code(text)
    names = sorted(set(re.findall(r"\b([A-Za-z_]\w*(?:::\w+)*)::vftable\b", code)))
    if any(re.search(r"(?:Setting|Property|Attribute|IODescriptor)", name) for name in names):
        return "native_property_registration_lead", names
    if re.search(r'"[^"\n]*(?:XmlItem|[Ss]erializ|[Dd]eserializ|Read Xml|Write Xml)', code):
        return "native_serializer_lead", names
    if any(re.search(r"(?:Validat|Verify)", name) for name in names):
        return "native_validation_lead", names
    return None, names


def native_mechanisms(corpus, blocks, classes, sources, bounds=None,
                      image=None, index_evidence=None):
    bounds = bounds or BOUNDS
    seeds = defaultdict(lambda: {"catalog_indices": set(), "class_ids": set()})
    anchor_types = defaultdict(lambda: defaultdict(set))
    for block in blocks:
        if not block.get("xml_type"):
            continue
        for anchor in block.get("native_anchors", []):
            address = canonical(anchor["address"])
            anchor_types[address][block["xml_type"]].add(block["catalog_index"])
    dispatcher_classes = defaultdict(set)
    for row in classes:
        name = row["class_name"].lower()
        if any(term in name for term in ("setting", "property", "attribute", "validat", "serial")):
            if row.get("dispatcher_address") and (row["methods"] or row["properties"]):
                dispatcher_classes[canonical(row["dispatcher_address"])].add(row["class_id"])
    omissions = []
    diagnostics = []
    cache = {}

    def unavailable(address, item):
        containing = []
        number = int(address, 16)
        for owner, record in corpus.functions.items():
            for region in record.get("body", {}).get("ranges", []):
                if int(region["start"], 16) <= number <= int(region["end"], 16):
                    containing.append({"address": owner, "range": region})
        if item is None:
            reason = "canonical_entry_absent"
            body_input = "no_path_or_digest_declared_for_this_pointer"
        elif item.get("status") == "external":
            reason = "canonical_entry_external"
            body_input = "external_entry_has_no_required_body"
        else:
            reason = "canonical_entry_without_available_body"
            body_input = "index_does_not_declare_available_body"
        pointer = {"status": "not_inspected"}
        evidence = [index_evidence] if index_evidence is not None else []
        if image is not None:
            evidence.append(sources.evidence(image.path, "virtual-address:" + address))
            sections = [section for section in image.sections
                        if section["va"] <= number
                        and number + 16 <= section["va"] + section["size"]]
            pointer = {
                "status": "file_backed" if sections else "not_file_backed",
                "executable": image.executable(number, 16),
                "section_names": sorted(section["name"] for section in sections),
                "pe_unwind_regions": [
                    record for record in image.runtime_functions()
                    if int(record["begin"], 16) <= number < int(record["end_exclusive"], 16)],
                "authority": "pointer_location_not_function_ownership_or_behavior",
            }
            if sections:
                data = image.read(number, 16)
                pointer["byte_range"] = {"address": address, "bytes": len(data),
                                         "sha256": digest(data)}
        diagnostics.append({
            "address": address, "status": "preserved_incomplete_evidence",
            "reason": reason, "canonical_entry_status": item.get("status") if item else "absent",
            "containing_canonical_ranges": sorted(containing, key=lambda row: row["address"]),
            "body_input_status": body_input,
            "not_a_stale_path_or_hash_failure": True,
            "classes": [
                {key: row.get(key) for key in ("class_id", "source_index", "class_name",
                                               "dispatcher_export_status")}
                for row in classes if row.get("dispatcher_address")
                and canonical(row["dispatcher_address"]) == address],
            "catalog_indices": sorted({index for indices in anchor_types.get(address, {}).values()
                                       for index in indices}),
            "binary_pointer": pointer,
            "evidence": evidence,
            "limitation": "No replacement function or import target is inferred. "
            "Why the original analyzer did not export a function here is unknown; "
            "executable bytes and absent unwind metadata do not establish its implementation.",
        })

    def read(address):
        if address in cache:
            return cache[address]
        item = corpus.functions.get(address)
        if not item or item.get("status") != "written" or not item.get("body", {}).get("available"):
            unavailable(address, item)
            cache[address] = None
            return None
        if not item.get("sha256"):
            raise RequiredEvidenceError(f"Required canonical body digest absent: {address}")
        if not item.get("relative_path"):
            raise RequiredEvidenceError(f"Required canonical body path absent: {address}")
        path, text = corpus.body(address)
        data = sources.read(path, "canonical-native-body", item["sha256"])
        if len(data) > bounds["body_bytes"]:
            omissions.append({"address": address, "reason": "body_size_bound",
                              "bytes": len(data)})
            cache[address] = None
            return None
        cache[address] = (text, sources.evidence(path, "pseudocode"))
        return cache[address]

    for i, address in enumerate(sorted(anchor_types, key=lambda a: int(a, 16))):
        if i >= bounds["anchor_bodies"]:
            omissions.append({"address": address, "reason": "anchor_count_bound"})
            continue
        body = read(address)
        if body:
            code = body_code(body[0])
            literals = set(re.findall(r'"((?:\\.|[^"\\])*)"', code))
            for xml_type, indices in anchor_types[address].items():
                if xml_type in literals:
                    seeds[address]["catalog_indices"].update(indices)
                else:
                    omissions.append({"address": address, "xml_type": xml_type,
                                      "reason": "inventory_literal_not_in_canonical_body"})
    for i, address in enumerate(sorted(dispatcher_classes, key=lambda a: int(a, 16))):
        if i >= bounds["dispatcher_bodies"]:
            omissions.append({"address": address, "reason": "dispatcher_count_bound"})
            continue
        if read(address):
            seeds[address]["class_ids"].update(dispatcher_classes[address])
    callers = defaultdict(dict)
    for address, memberships in sorted(seeds.items()):
        for callee, count in sorted(direct_calls(cache[address][0]).items()):
            if callee != address:
                callers[callee][address] = {
                    "caller_address": address, "direct_call_mentions": count,
                    "catalog_indices": sorted(memberships["catalog_indices"]),
                    "class_ids": sorted(memberships["class_ids"]),
                    "evidence": cache[address][1],
                }
    shared = [(address, rows) for address, rows in callers.items() if len(rows) >= 2]
    shared.sort(key=lambda pair: (-len(pair[1]), int(pair[0], 16)))
    result = []
    unclassified = []
    for i, (address, rows) in enumerate(shared):
        if i >= bounds["shared_helper_bodies"]:
            omissions.append({"address": address, "reason": "shared_helper_count_bound"})
            continue
        body = read(address)
        if not body:
            continue
        kind, names = helper_kind(body[0])
        indices = sorted({index for row in rows.values() for index in row["catalog_indices"]})
        if kind is None:
            unclassified.append({
                "address": address, "caller_count": len(rows),
                "candidate_catalog_indices": indices, "observed_vftable_names": names,
                "evidence": body[1],
            })
            continue
        result.append(make_mechanism(
            "native-helper:" + address, kind, "candidate", indices,
            "canonical_direct_calls_from_exact_xml_literal_or_settings_dispatcher",
            "Direct calls and named-vftable/literal cues are static evidence only. A catalog "
            "XML-name occurrence is not class ownership or execution proof; IO descriptors "
            "can include inputs/outputs, not only parameters. Helper semantics are leads.",
            [body[1]], blocks, native_addresses=[address],
            observed_vftable_names=names, callers=list(rows.values()),
            class_ids=sorted({value for row in rows.values() for value in row["class_ids"]}),
            address_role="canonical_callee"))
    if unclassified:
        result.append(make_mechanism(
            "native:unclassified-shared-callees", "unclassified_shared_call_leads", "candidate", [],
            "shared_direct_calls_without_property_or_serializer_cues",
            "Includes ordinary strings, containers and allocation helpers; do not treat "
            "common call frequency as UX support.", [], blocks, helpers=unclassified))
    return result, {
        "bounds": bounds, "anchor_addresses": len(anchor_types),
        "settings_dispatcher_addresses": len(dispatcher_classes),
        "seed_bodies_joined": len(seeds), "shared_direct_callees": len(shared),
        "canonical_bodies_read": sum(value is not None for value in cache.values()),
        "classified_helper_groups": sum(row["kind"] != "unclassified_shared_call_leads"
                                       for row in result),
        "unclassified_helpers": len(unclassified), "omissions": omissions,
        "diagnostics": diagnostics, "diagnostic_count": len(diagnostics),
        "diagnostics_by_reason": dict(sorted(Counter(
            row["reason"] for row in diagnostics).items())),
        "evidence_status": "incomplete" if diagnostics or omissions else "complete_for_bounded_inputs",
        "whole_corpus_scan": False,
    }


def extract(inventory_path, state_path=None, qt_path=None, dialogs_path=None,
            advanced_path=None, root=ROOT, skip_native=False):
    sources = Sources(root)
    if sources.path(Path("scripts") / "element-ux-map.py").is_file():
        sources.read(Path("scripts") / "element-ux-map.py", "extractor")
    inventory_path = sources.path(inventory_path)
    inventory = sources.json(inventory_path, "inventory")
    blocks = validate_blocks(inventory)
    target = inventory.get("source", {}).get("executable_sha256")
    if target is None:
        raise RequiredEvidenceError("Inventory executable SHA-256 pin is missing")
    if not isinstance(target, str) or not re.fullmatch(r"[a-f0-9]{64}", target):
        raise ValueError("Inventory requires executable SHA-256")
    omissions = []
    artifacts = {}

    def optional(path, role, expected=None, is_json=True, require_hash=False):
        if not path or not sources.path(path).is_file():
            omissions.append({"role": role, "path": sources.relative(path) if path else None,
                              "reason": "artifact_absent"})
            artifacts[role] = {"status": "absent"}
            return None
        if require_hash and not expected:
            omissions.append({"role": role, "path": sources.relative(path),
                              "reason": "required_source_pin_absent"})
            artifacts[role] = {"status": "required_pin_absent", "path": sources.relative(path)}
            return None
        value = (sources.json(path, role, expected) if is_json
                 else sources.read(path, role, expected))
        artifacts[role] = {"status": "present", "path": sources.relative(path)}
        return value

    state_path = sources.path(state_path or inventory.get("source", {}).get("state")
                              or Path(".reverse-engineering") / "latest.json")
    state = optional(state_path, "analysis-state")
    version = None
    if state:
        if state.get("status") != "complete":
            raise ValueError("A completed frozen analysis state is required")
        state_target = state["binaries"]["LoxoneConfig.exe"]["export"].get("target_sha256")
        if state_target is None:
            raise RequiredEvidenceError("State executable SHA-256 pin is missing")
        if state_target != target:
            raise ValueError("State/inventory executable identity mismatch")
        executable = sources.path(Path(state["version_directory"]) / "source" / "LoxoneConfig.exe")
        binary = optional(executable, "application-executable", target, is_json=False)
        if binary is not None:
            version = pe_version(binary)
            if version is None:
                omissions.append({"role": "application-version", "reason": "RT_VERSION_absent"})
    raw_path = inventory.get("source", {}).get("catalog")
    raw = optional(raw_path, "raw-catalog")
    if raw is not None:
        for row in blocks:
            index = row["catalog_index"]
            if index >= len(raw["blocks"]):
                raise ValueError("Inventory catalog_index outside raw catalog")
            attrs = raw["blocks"][index]["attributes"]
            if (attrs.get("ControlType") != row.get("control_type")
                    or attrs.get("LxType") != row.get("xml_type")
                    or attrs.get("Name") != row.get("name")):
                raise ValueError(f"Raw catalog identity mismatch at catalog_index {index}")
        if len(raw["blocks"]) != len(blocks):
            raise ValueError("Raw catalog and inventory populations differ")
        catalog_source = raw.get("source", {})
        optional(catalog_source.get("xml"), "catalog-xml",
                 catalog_source.get("xml_sha256"), is_json=False, require_hash=True)
    inventory_evidence = sources.evidence(inventory_path, "/blocks")
    mechanisms = catalog_mechanisms(blocks, inventory_evidence)
    qt_path = sources.path(qt_path or inventory_path.with_name("qt-metadata.json"))
    qt = optional(qt_path, "qt-metadata")
    classes = []
    if qt is not None:
        if qt.get("schema_version") != 1 or not isinstance(qt.get("classes"), list):
            raise ValueError("Unsupported Qt metadata schema")
        if qt.get("source", {}).get("sha256") is None:
            raise RequiredEvidenceError("Qt executable SHA-256 pin is missing")
        if qt["source"]["sha256"] != target:
            raise ValueError("Qt/inventory executable identity mismatch")
        optional(qt["source"].get("path"), "qt-source-executable", target, is_json=False)
        classes, groups = qt_mechanisms(
            qt, inventory, blocks, sources.evidence(qt_path, "/classes"))
        mechanisms.extend(groups)
        if qt.get("issues") or qt.get("unsupported_objects"):
            omissions.append({"role": "qt-metadata", "reason": "decoder_issues_or_unsupported_objects",
                              "issues": len(qt.get("issues", [])),
                              "unsupported_objects": len(qt.get("unsupported_objects", []))})
    dialogs_path = sources.path(dialogs_path or inventory_path.with_name("native-dialogs.json"))
    dialogs = optional(dialogs_path, "native-dialogs")
    if dialogs is not None:
        if (dialogs.get("schema_version") != 1 or not isinstance(dialogs.get("dialogs"), list)
                or not isinstance(dialogs.get("source"), dict)):
            raise ValueError("Unsupported native dialog metadata")
        resource_hash = dialogs["source"].get("sha256")
        if resource_hash is not None and not re.fullmatch(r"[a-f0-9]{64}", resource_hash):
            raise ValueError("Malformed native dialog resource pin")
        optional(dialogs.get("source", {}).get("path"), "dialog-resource-dll",
                 resource_hash, is_json=False, require_hash=True)
        mechanisms.extend(dialog_mechanisms(
            dialogs, blocks, sources.evidence(dialogs_path, "/dialogs,/string_tables")))
        if dialogs.get("issues"):
            omissions.append({"role": "native-dialogs", "reason": "resource_decoder_issues",
                              "issues": len(dialogs["issues"])})
    advanced_path = sources.path(advanced_path or inventory_path.with_name("advanced-blocks.tsv"))
    advanced = optional(advanced_path, "advanced-candidates", is_json=False)
    advanced_count = None
    if advanced is not None:
        reader = csv.DictReader(io.StringIO(advanced.decode("utf-8-sig")), delimiter="\t")
        if not {"TYPE", "CONTROL_ID"}.issubset(reader.fieldnames or []):
            raise ValueError("Unrecognized advanced-blocks.tsv header")
        advanced_count = sum(1 for _ in reader)
    native = {"status": "not_inspected", "bounds": BOUNDS}
    if skip_native:
        omissions.append({"role": "native-static-joins", "reason": "explicitly_skipped"})
    elif state is not None:
        export = state["binaries"]["LoxoneConfig.exe"]["export"]
        index = optional(export.get("index_path"), "decompile-index")
        if index is not None:
            if index.get("target_sha256") is None:
                raise RequiredEvidenceError("Canonical index executable SHA-256 pin is missing")
            if index.get("target_sha256") != target:
                raise ValueError("Canonical index/inventory executable identity mismatch")
            scope_path = ROOT / "scripts" / "native-sim-scope.py"
            sources.read(scope_path, "native-body-reader")
            spec = importlib.util.spec_from_file_location("element_ux_native_scope", scope_path)
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            corpus = module.Corpus(state_path)
            image = None
            if binary is not None:
                image = module.PE(executable)
                if image.sha256 != target:
                    raise ValueError("Native pointer inspection executable hash mismatch")
            groups, native = native_mechanisms(
                corpus, blocks, classes, sources, image=image,
                index_evidence=sources.evidence(export["index_path"], "/functions"))
            native["status"] = "bounded_canonical_static_joins"
            mechanisms.extend(groups)
            if native["omissions"]:
                omissions.append({"role": "native-static-joins",
                                  "reason": "bounded_join_omissions",
                                  "count": len(native["omissions"])})
    if native["status"] == "not_inspected" and not skip_native:
        omissions.append({"role": "native-static-joins", "reason": "analysis_state_or_index_absent"})
    ids = [row["mechanism_id"] for row in mechanisms]
    if len(set(ids)) != len(ids):
        raise ValueError("Mechanism ID collision")
    kinds = Counter(row["kind"] for row in mechanisms)
    parameter_count = sum(len(row.get("parameters", [])) for row in blocks)
    mapped = sum(bool(parameter.get("known_xml_keys"))
                 for row in blocks for parameter in row.get("parameters", []))
    candidate_indices = {index for row in mechanisms
                         if row["membership_basis"]["status"] == "candidate"
                         for index in row["catalog_indices"]}
    return {
        "schema_version": 1, "application": {"version": version, "sha256": target},
        "sources": sources.all(), "artifacts": artifacts,
        "mechanisms": sorted(mechanisms, key=lambda row: row["mechanism_id"]),
        "classes": classes,
        "unjoined_leads": [{
            "population": "msvc_dialog_rtti_name_leads",
            "count": len(inventory["msvc_dialog_types"]) if "msvc_dialog_types" in inventory else None,
            "evidence": sources.evidence(inventory_path, "/msvc_dialog_types"),
            "status": "name_leads_only_no_factory_or_rtti_class_census_repeated",
        }],
        "native_static_joins": native, "omissions": omissions,
        "summary": {
            "status": "partial_metadata_extraction" if omissions else "complete_metadata_extraction",
            "evidence_status": "incomplete" if omissions or native.get("diagnostics")
            else "complete_for_bounded_inputs",
            "incomplete_evidence_diagnostics": native.get("diagnostic_count", 0),
            "input_or_scope_omissions": len(omissions),
            "source_populations": {
                "catalog_records": len(blocks),
                "distinct_control_type_ids": len({row.get("control_type") for row in blocks
                                                 if row.get("control_type") is not None}),
                "explicit_xml_records": sum(row.get("xml_type") is not None for row in blocks),
                "distinct_explicit_xml_names": len({row["xml_type"] for row in blocks
                                                    if row.get("xml_type") is not None}),
                "null_xml_records": sum(row.get("xml_type") is None for row in blocks),
                "catalog_parameter_entries": parameter_count,
                "parameters_with_xml_keys": mapped, "parameters_without_xml_keys": parameter_count - mapped,
                "advanced_tsv_rows": advanced_count,
                "qt_classes": len(classes) if qt is not None else None,
                "qt_methods": sum(len(row["methods"]) for row in classes) if qt is not None else None,
                "qt_properties": sum(len(row["properties"]) for row in classes) if qt is not None else None,
                "qt_editor_name_candidates": len(inventory["qt_dialogs"]) if "qt_dialogs" in inventory else None,
                "msvc_dialog_rtti_name_leads": len(inventory["msvc_dialog_types"])
                if "msvc_dialog_types" in inventory else None,
                "native_dialog_templates": len(dialogs.get("dialogs", [])) if dialogs is not None else None,
                "native_static_controls": sum(len(row.get("controls", [])) for row in dialogs.get("dialogs", []))
                if dialogs is not None else None,
                "resource_string_tables": len(dialogs.get("string_tables", [])) if dialogs is not None else None,
            },
            "mechanisms": len(mechanisms), "mechanisms_by_kind": dict(sorted(kinds.items())),
            "candidate_catalog_records": len(candidate_indices),
            "candidate_catalog_parameter_entries": sum(len(row.get("parameters", []))
                                                       for row in blocks
                                                       if row["catalog_index"] in candidate_indices),
            "total_gui_fields": None, "total_native_property_registrations": None,
            "verified_widget_bindings": 0, "verified_execution_memberships": 0,
            "verified_validation_rules": 0,
        },
        "limitations": [
            "Populations have different denominators and are never summed into UX coverage.",
            "Null XML identities include non-block catalog records, not only unknown blocks.",
            "Metadata membership proves only the stated declaration or observed storage shape.",
            "Native XML literals, captions and family/class names remain candidate associations.",
            "Potential parameter impact counts catalog entries, not supported or verified fields.",
            "Unjoined common editors/family leads are retained; their catalog denominator is null.",
            "Nested editors remain exceptions; no scalar-editor parity is inherited.",
            "Native reads use pinned existing anchors/dispatchers and one bounded callee hop only.",
            "Unavailable canonical entries remain diagnosed leads, not missing declared input "
            "files, fabricated imports, replacement functions, or hash failures.",
            "No factory census, whole-program rebuild, GUI pass, target execution or runtime proof.",
        ],
    }


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--inventory", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--state", type=Path)
    parser.add_argument("--qt-metadata", type=Path)
    parser.add_argument("--native-dialogs", type=Path)
    parser.add_argument("--advanced-blocks", type=Path)
    parser.add_argument("--skip-native", action="store_true",
                        help="Explicit partial metadata-only extraction")
    args = parser.parse_args(argv)
    try:
        output = args.output.resolve()
        if not output.is_relative_to((ROOT / ".reverse-engineering").resolve()):
            raise ValueError("Recovered metadata output must remain under .reverse-engineering")
        report = extract(args.inventory, args.state, args.qt_metadata, args.native_dialogs,
                         args.advanced_blocks, skip_native=args.skip_native)
        output.parent.mkdir(parents=True, exist_ok=True)
        data = (json.dumps(report, indent=2, ensure_ascii=True) + "\n").encode("utf-8")
        output.write_bytes(data)
        print(json.dumps({"output": str(output), "sha256": digest(data),
                          "application": report["application"], "summary": report["summary"]},
                         indent=2))
        return 2 if report["omissions"] else 0
    except (FileNotFoundError, RequiredEvidenceError) as error:
        print(f"UX mechanism extraction incomplete: {error}", file=sys.stderr)
        return 2
    except (OSError, ValueError, KeyError, TypeError, struct.error) as error:
        print(f"UX mechanism extraction failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
