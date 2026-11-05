#!/usr/bin/env python3
"""Local, version-bound native function review; never executes the target image."""

import argparse
from bisect import bisect_right
from collections import Counter
from contextlib import closing
import hashlib
import json
from pathlib import Path
import re
import sqlite3
import struct
import sys


ROOT = Path(__file__).resolve().parents[1]
PRIVATE = ROOT / ".reverse-engineering" / "native-simulator" / "scope"
CLASSIFICATIONS = {"candidate", "inferred", "confirmed", "rejected"}


def sha256(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def load(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def save(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, ensure_ascii=True) + "\n",
                    encoding="utf-8")


def integer(value):
    return int(value, 0) if isinstance(value, str) else int(value)


def canonical_address(value):
    return hex(integer(value)) if re.fullmatch(r"0x[0-9a-fA-F]+", value) else value


def evidence(path):
    path = Path(path).resolve()
    return {"path": str(path), "sha256": sha256(path), "bytes": path.stat().st_size}


def private_directory(path):
    path = Path(path).resolve()
    if not path.is_relative_to(PRIVATE.resolve()):
        raise ValueError(f"Recovered output must remain under {PRIVATE}")
    path.mkdir(parents=True, exist_ok=True)
    return path


def pseudocode(text):
    # Export headers also mention callees; those are not body references.
    return "\n".join(line for line in text.splitlines() if not line.startswith("//"))


def call_mentions(text):
    return sorted({hex(int(match, 16)) for match in
                   re.findall(r"\b(?:thunk_)?FUN_([0-9a-fA-F]+)\s*\(", pseudocode(text))})


class Corpus:
    def __init__(self, latest):
        self.latest_path = Path(latest)
        self.latest = load(latest)
        self.binary = self.latest["binaries"]["LoxoneConfig.exe"]
        self.index_path = Path(self.binary["export"]["index_path"])
        self.index = load(self.index_path)
        self.target = self.index["target_sha256"]
        if self.binary["export"]["target_sha256"] != self.target:
            raise ValueError("Latest/index target identity mismatch")
        self.functions = {}
        for item in self.index["functions"]:
            address = canonical_address(item["address"])
            if address in self.functions:
                raise ValueError(f"Duplicate canonical function {address}")
            self.functions[address] = item
        self.database = Path(self.binary["database"])

    def connect(self):
        return sqlite3.connect(self.database.resolve().as_uri() + "?mode=ro", uri=True)

    def body(self, address):
        item = self.functions[hex(integer(address))]
        if item["status"] != "written" or not item.get("body", {}).get("available"):
            raise ValueError(f"No canonical body for {address}")
        relative = Path(item["relative_path"].replace("/", "\\")
                        if sys.platform == "win32" else item["relative_path"].replace("\\", "/"))
        path = (self.index_path.parent / relative).resolve()
        if not path.is_relative_to(self.index_path.parent.resolve()):
            raise ValueError("Canonical body path escapes index directory")
        if sha256(path) != item["sha256"]:
            raise ValueError(f"Canonical body hash mismatch: {address}")
        return path, path.read_text(encoding="utf-8")

    def query(self, expression, limit=30, observed=None):
        with closing(self.connect()) as connection:
            rows = connection.execute(
                "SELECT address,name FROM functions WHERE functions MATCH ? ORDER BY address",
                (expression,))
            result = []
            for address, name in rows:
                address = canonical_address(address)
                if observed is not None and address not in observed:
                    continue
                result.append({"address": address, "name": name,
                               "observed": None if observed is None else True})
                if len(result) == limit:
                    break
        return result

    def callers(self, address):
        symbol = "FUN_" + hex(integer(address))[2:]
        with closing(self.connect()) as connection:
            rows = connection.execute(
                "SELECT address,body FROM functions WHERE functions MATCH ?", (symbol,))
            return sorted({canonical_address(caller) for caller, body in rows
                           if canonical_address(caller) != address and
                           address in call_mentions(body)})


def read_trace(corpus, path):
    path = Path(path)
    summary = load(path)
    if summary["application"]["sha256"] != corpus.target:
        raise ValueError("Trace target hash mismatch")
    if summary["inputs"]["decompile_index_sha256"] != sha256(corpus.index_path):
        raise ValueError("Trace uses a different canonical index")
    artifacts = [a for a in summary["artifacts"] if a["kind"] == "basic-block-function-map"]
    if len(artifacts) != 1:
        raise ValueError("Expected exactly one trace function map")
    mapping_path = (path.parent / summary["artifact_root"] / artifacts[0]["path"]).resolve()
    if sha256(mapping_path) != artifacts[0]["sha256"]:
        raise ValueError("Trace function map hash mismatch")
    mapping = load(mapping_path)
    if (mapping["application_sha256"] != corpus.target or
            mapping["raw_log_sha256"] != summary["measurement"]["raw_log_sha256"]):
        raise ValueError("Trace summary/map identity mismatch")
    functions = {}
    for block in mapping["blocks"]:
        if block["status"] != "confirmed":
            continue
        if len(block["functions"]) != 1:
            raise ValueError("Confirmed trace block has ambiguous ownership")
        address = hex(integer(block["functions"][0]))
        if address not in corpus.functions:
            raise ValueError(f"Trace function absent from canonical index: {address}")
        functions.setdefault(address, []).append(block)
    return {"summary": summary, "map": mapping, "functions": functions,
            "evidence": [evidence(path), evidence(mapping_path)]}


class PE:
    """Only on-disk, file-backed AMD64 PE bytes; no loader or vendor ABI."""

    def __init__(self, path):
        self.path = Path(path)
        self.data = self.path.read_bytes()
        d = self.data
        self.sha256 = hashlib.sha256(d).hexdigest()
        if d[:2] != b"MZ":
            raise ValueError("Not a PE image")
        nt = struct.unpack_from("<I", d, 60)[0]
        if d[nt:nt + 4] != b"PE\0\0":
            raise ValueError("Invalid PE signature")
        machine, count = struct.unpack_from("<HH", d, nt + 4)
        optional_size = struct.unpack_from("<H", d, nt + 20)[0]
        optional = nt + 24
        if machine != 0x8664 or struct.unpack_from("<H", d, optional)[0] != 0x20B:
            raise ValueError("Only AMD64 PE32+ is supported")
        self.base = struct.unpack_from("<Q", d, optional + 24)[0]
        directory_count = struct.unpack_from("<I", d, optional + 108)[0]
        if 112 + min(directory_count, 16) * 8 > optional_size:
            raise ValueError("Truncated PE data directories")
        self.directories = [struct.unpack_from("<II", d, optional + 112 + i * 8)
                            for i in range(min(directory_count, 16))]
        self._runtime_functions = None
        self._rtti_types = {}
        self._rtti_hierarchies = {}
        self.sections = []
        for i in range(count):
            entry = optional + optional_size + i * 40
            virtual_size, rva, size, offset = struct.unpack_from("<IIII", d, entry + 8)
            flags = struct.unpack_from("<I", d, entry + 36)[0]
            if offset + size > len(d):
                raise ValueError("Truncated PE section")
            self.sections.append({"va": self.base + rva, "offset": offset, "size": size,
                                  "virtual_size": virtual_size, "flags": flags,
                                  "name": d[entry:entry + 8].rstrip(b"\0").decode("ascii", "replace")})

    def read(self, address, size):
        for section in self.sections:
            relative = address - section["va"]
            if 0 <= relative and relative + size <= section["size"]:
                start = section["offset"] + relative
                return self.data[start:start + size]
        raise ValueError(f"Not file-backed section bytes: {hex(address)}+{size}")

    def address(self, offset):
        for section in self.sections:
            if section["offset"] <= offset < section["offset"] + section["size"]:
                return section["va"] + offset - section["offset"]
        return None

    def executable(self, address, size=1):
        return any(s["flags"] & 0x20000000 and
                   s["va"] <= address and address + size <= s["va"] + s["size"]
                   for s in self.sections)

    def runtime_functions(self):
        if self._runtime_functions is not None:
            return self._runtime_functions
        rva, size = self.directories[3] if len(self.directories) > 3 else (0, 0)
        if not rva and not size:
            self._runtime_functions = []
            return []
        if not rva or size % 12:
            raise ValueError("Invalid AMD64 exception directory extent")
        data = self.read(self.base + rva, size)
        records = []
        for i, (begin, end, unwind) in enumerate(struct.iter_unpack("<III", data)):
            if begin >= end:
                raise ValueError("Invalid RUNTIME_FUNCTION half-open range")
            records.append({"index": i, "entry_address": hex(self.base + rva + i * 12),
                            "begin": hex(self.base + begin), "end_exclusive": hex(self.base + end),
                            "unwind_rva": hex(unwind)})
        self._runtime_functions = records
        return records

    def unwind(self, record):
        address = self.base + integer(record["unwind_rva"])
        header = self.read(address, 4)
        version, flags, count = header[0] & 7, header[0] >> 3, header[2]
        size = 4 + ((count + 1) & ~1) * 2
        result = {**record, "unwind_address": hex(address), "version": version,
                  "flags": flags, "prolog_size": header[1], "unwind_code_slots": count,
                  "frame_register": header[3] & 15, "frame_offset_scaled": header[3] >> 4,
                  "authority": "PE_unwind_region_not_logical_function_ownership"}
        if version not in {1, 2} or flags & ~7 or (flags & 4 and flags & 3):
            result["limitation"] = "Unsupported unwind version/flag combination"
            return result
        if flags & 4:
            begin, end, unwind = struct.unpack("<III", self.read(address + size, 12))
            result["chained_runtime_function"] = {
                "begin": hex(self.base + begin), "end_exclusive": hex(self.base + end),
                "unwind_rva": hex(unwind)}
            size += 12
        elif flags & 3:
            result["handler_rva"] = hex(struct.unpack("<I", self.read(address + size, 4))[0])
            result["handler_specific_data"] = "not_parsed"
            size += 4
        result["unwind_bytes_sha256"] = hashlib.sha256(self.read(address, size)).hexdigest()
        return result

    def unwind_regions(self, start, end=None):
        end = start if end is None else end
        records = self.runtime_functions()
        rva, size = self.directories[3] if len(self.directories) > 3 else (0, 0)
        return {
            "directory_rva": hex(rva), "directory_bytes": size, "record_count": len(records),
            "directory_sha256": hashlib.sha256(self.read(self.base + rva, size)).hexdigest()
            if size else None,
            "query_range_inclusive": {"start": hex(start), "end": hex(end)},
            "matches": [self.unwind(r) for r in records
                        if integer(r["begin"]) <= end and start < integer(r["end_exclusive"])],
            "limitation": "No match does not imply no callback: leaf code may omit unwind entries. "
                          "No nearest-start ownership is inferred."}

    def matches(self, needle):
        start = 0
        while True:
            found = self.data.find(needle, start)
            if found < 0:
                return
            yield found
            start = found + 1

    def rtti(self, name, slots=(0x48, 0x50, 0x58)):
        """Validate MSVC x64 COL self-RVA, then report pointers, not ABI claims."""
        result = []
        for name_offset in self.matches((".?AV" + name + "@@\0").encode("ascii")):
            descriptor = self.address(name_offset - 16)
            if descriptor is None:
                continue
            for reference in self.matches(struct.pack("<I", descriptor - self.base)):
                col = self.address(reference - 12)
                if col is None:
                    continue
                try:
                    signature, offset, cd, typ, hierarchy, own = struct.unpack(
                        "<IIIIII", self.read(col, 24))
                except ValueError:
                    continue
                if signature != 1 or col - own != self.base or self.base + typ != descriptor:
                    continue
                for pointer in self.matches(struct.pack("<Q", col)):
                    table = self.address(pointer + 8)
                    if table is None or table % 8:
                        continue
                    try:
                        entries = [{"offset": hex(slot), "target": hex(struct.unpack(
                            "<Q", self.read(table + slot, 8))[0])} for slot in slots]
                    except ValueError:
                        continue
                    result.append({"class": name, "type_descriptor": hex(descriptor),
                                   "complete_object_locator": hex(col), "object_offset": hex(offset),
                                   "constructor_displacement": cd, "hierarchy_rva": hex(hierarchy),
                                   "vtable": hex(table), "slots": entries,
                                   "authority": "static_RTTI_and_pointer_linkage_not_runtime_object"})
        return result

    def type_descriptor(self, address):
        if address in self._rtti_types:
            return self._rtti_types[address]
        result = {"address": hex(address), "status": "unknown"}
        try:
            self.read(address, 16)
            section = next(s for s in self.sections
                           if s["va"] <= address + 16 < s["va"] + s["size"])
            raw = self.read(address + 16, min(1024, section["va"] + section["size"] - address - 16))
            if b"\0" not in raw:
                raise ValueError("Unterminated/bounded RTTI type name")
            name = raw.split(b"\0", 1)[0].decode("ascii")
            if not name.startswith((".?AV", ".?AU")) or not name.endswith("@@"):
                raise ValueError("Unsupported RTTI class type-name form")
            result.update(status="decoded", rtti_name=name, display_name=name[4:-2])
        except (ValueError, StopIteration, UnicodeError) as error:
            result["reason"] = str(error)
        self._rtti_types[address] = result
        return result

    def class_hierarchy(self, address):
        if address in self._rtti_hierarchies:
            return self._rtti_hierarchies[address]
        result = {"address": hex(address), "status": "unknown"}
        try:
            signature, attributes, count, array_rva = struct.unpack(
                "<IIII", self.read(address, 16))
            result.update(signature=signature, attributes=attributes, base_count=count,
                          base_array_address=hex(self.base + array_rva))
            if signature != 0 or not 1 <= count <= 4096:
                raise ValueError("Unsupported hierarchy signature/base count")
            descriptors = struct.unpack("<" + "I" * count,
                                        self.read(self.base + array_rva, count * 4))
            bases = []
            for i, descriptor_rva in enumerate(descriptors):
                descriptor = self.base + descriptor_rva
                typ, contained, mdisp, pdisp, vdisp, flags = struct.unpack(
                    "<IIiiiI", self.read(descriptor, 24))
                if contained > count - i - 1:
                    raise ValueError("Base descriptor subtree exceeds flattened hierarchy")
                typ = self.type_descriptor(self.base + typ)
                if typ["status"] != "decoded":
                    raise ValueError("Base type descriptor unavailable")
                base = {"descriptor_address": hex(descriptor), "type": typ,
                        "contained_bases": contained, "mdisp": mdisp, "pdisp": pdisp,
                        "vdisp": vdisp, "attributes": flags}
                if flags & 0x40:
                    base["hierarchy_address"] = hex(self.base + struct.unpack(
                        "<I", self.read(descriptor + 24, 4))[0])
                bases.append(base)
            if bases[0]["contained_bases"] != count - 1:
                raise ValueError("Self descriptor does not span flattened hierarchy")
            result.update(status="decoded", bases=bases)
        except (ValueError, struct.error) as error:
            result["reason"] = str(error)
        self._rtti_hierarchies[address] = result
        return result

    def rtti_loci(self):
        """Find self-RVA-validated x64 COLs, not class-name-prefix guesses."""
        loci, signature_sites = [], 0
        for offset in self.matches(b"\x01\0\0\0"):
            address = self.address(offset)
            if address is None or address % 4:
                continue
            signature_sites += 1
            try:
                signature, displacement, cd, typ, hierarchy, own = struct.unpack(
                    "<IIIIII", self.read(address, 24))
            except ValueError:
                continue
            if signature != 1 or self.base + own != address:
                continue
            typ = self.type_descriptor(self.base + typ)
            chd = self.class_hierarchy(self.base + hierarchy)
            self_matches = (chd["status"] == "decoded" and
                            chd["bases"][0]["type"]["address"] == typ["address"])
            loci.append({"address": hex(address), "object_offset": displacement,
                         "constructor_displacement": cd, "type": typ, "hierarchy": chd,
                         "hierarchy_self_matches": self_matches, "pointer_references": []})
        by_address = {integer(locus["address"]): locus for locus in loci}
        # A COL pointer immediately preceding an aligned table is evidence;
        # pointer storage/first slot are checked separately before promotion.
        for section in self.sections:
            start = (-section["va"]) % 8
            size = ((section["size"] - start) // 8) * 8
            data = self.data[section["offset"] + start:section["offset"] + start + size]
            for i, (pointer,) in enumerate(struct.iter_unpack("<Q", data)):
                if pointer in by_address:
                    by_address[pointer]["pointer_references"].append({
                        "vtable": hex(section["va"] + start + i * 8 + 8),
                        "section": section["name"], "section_flags": hex(section["flags"]),
                        "readonly_data": bool(section["flags"] & 0x40000000 and
                                              not section["flags"] & 0xA0000000)})
        return loci, {"aligned_signature_one_sites": signature_sites,
                      "self_rva_validated_COLs": len(loci),
                      "scan": "all_file_backed_sections; aligned_COLs_and_pointer_references"}


def capstone_module(dependency_dir=None):
    if dependency_dir:
        sys.path.insert(0, str(Path(dependency_dir).resolve()))
    try:
        import capstone
    except ImportError as error:
        raise ValueError("CFG export requires capstone==5.0.6; see native-simulator-scope.md") from error
    return capstone


def decode_window(image, start, end, capstone):
    decoder = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_64)
    decoder.detail = True
    if end < start or end - start > 4096 or not image.executable(start, end - start + 1):
        raise ValueError("Instruction proof window must be executable and at most 4097 bytes")
    return list(decoder.disasm(image.read(start, end - start + 1), start))


def instruction_evidence(instructions):
    return [{"address": hex(i.address), "bytes": i.bytes.hex(),
             "mnemonic": i.mnemonic, "operands": i.op_str} for i in instructions]


def exact_owners(corpus, address):
    return sorted(f["address"] for f in corpus.functions.values()
                  if f.get("body", {}).get("available") and any(
                      integer(r["start"]) <= address <= integer(r["end"])
                      for r in f["body"]["ranges"]))


class ExactRanges:
    def __init__(self, corpus):
        self.ranges = sorted(
            (integer(r["start"]), integer(r["end"]), f["address"])
            for f in corpus.functions.values() if f.get("body", {}).get("available")
            for r in f["body"]["ranges"])
        self.starts = [r[0] for r in self.ranges]
        self.prefix_end = []
        largest = -1
        for _, end, _ in self.ranges:
            largest = max(largest, end)
            self.prefix_end.append(largest)

    def owners(self, address):
        i, owners = bisect_right(self.starts, address) - 1, set()
        while i >= 0 and self.prefix_end[i] >= address:
            if self.ranges[i][1] >= address:
                owners.add(self.ranges[i][2])
            i -= 1
        return sorted(owners)


def build_handler_catalog(corpus, image, traces, root_name="CControlObject"):
    if image.sha256 != corpus.target or sha256(image.path) != corpus.target:
        raise ValueError("Catalog image/corpus identity mismatch")
    loci, scan = image.rtti_loci()
    root_names = {".?AV" + root_name + "@@", ".?AU" + root_name + "@@"}
    root_types = {l["type"]["address"] for l in loci if
                  l["type"].get("rtti_name") in root_names and l["hierarchy_self_matches"]}
    if not root_types:
        raise ValueError("No complete exact-name RTTI root; no lineage catalog inferred")
    ranges = ExactRanges(corpus)
    roots, classes, unclassified, excluded = {}, {}, [], []

    def observations(target, owners):
        result = []
        for trace in traces:
            blocks = [b for owner in owners for b in trace["functions"].get(owner, [])]
            result.append({
                "raw_log_sha256": trace["summary"]["measurement"]["raw_log_sha256"],
                "canonical_owner_body_observed": bool(blocks),
                "target_byte_observed": any(
                    image.base + integer(b["rva"]) <= target <
                    image.base + integer(b["rva"]) + b["size"] for b in blocks),
                "records": len(blocks),
                "distinct_blocks": len({(b["rva"], b["size"]) for b in blocks}),
                "meaning": "shared_function_presence_not_class_instance_or_simulator_support"})
        return result

    for locus in loci:
        typ, hierarchy = locus["type"], locus["hierarchy"]
        if not locus["hierarchy_self_matches"] or typ["status"] != "decoded":
            unclassified.append(locus)
            continue
        root_bases = [base for base in hierarchy["bases"] if
                      base["type"]["address"] in root_types]
        if not root_bases:
            excluded.append({"COL": locus["address"], "type": typ,
                             "reason": "decoded_hierarchy_has_no_exact_root_descriptor"})
            continue
        # A primary complete-object vtable is a dispatcher candidate only when
        # the CControlObject subobject is also the unique nonvirtual base at 0.
        compatible = (len(root_bases) == 1 and root_bases[0]["mdisp"] == 0 and
                      root_bases[0]["pdisp"] == -1 and root_bases[0]["vdisp"] == 0 and
                      not root_bases[0]["attributes"] & 0x12)
        destination = roots if typ["address"] in root_types else classes
        record = destination.setdefault(typ["address"], {
            "type": typ, "lineage": "verified_exact_root_in_flattened_RTTI_base_array",
            "base_types": [b["type"] for b in hierarchy["bases"][1:]],
            "primary_tables": [], "secondary_loci": [],
            "xml_type_name": None, "xml_type_id": None, "simulator_supported": None})
        if locus["object_offset"] != 0:
            record["secondary_loci"].append({
                **locus, "handler_slot_read": False,
                "reason": "secondary_COL_offset_nonzero; not_primary_dispatcher_table"})
            continue
        for reference in locus["pointer_references"] or [None]:
            table = {
                "COL": locus["address"], "object_offset": 0,
                "constructor_displacement": locus["constructor_displacement"],
                "hierarchy": hierarchy, "root_base_descriptors": root_bases,
                "primary_root_compatible": compatible,
                "vtable": None if reference is None else reference["vtable"],
                "storage": reference, "slot_offset": "0x50",
                "target": None, "exact_owners": [], "validated_candidate": False}
            reasons = []
            if not compatible:
                reasons.append("root_subobject_not_unique_nonvirtual_primary_at_zero")
            if locus["constructor_displacement"]:
                reasons.append("construction_displacement_nonzero")
            if reference is None:
                reasons.append("no_aligned_COL_pointer_reference")
            else:
                if not reference["readonly_data"]:
                    reasons.append("table_storage_not_readonly_nonexecuting_data")
                try:
                    va = integer(reference["vtable"])
                    first = struct.unpack("<Q", image.read(va, 8))[0]
                    target = struct.unpack("<Q", image.read(va + 0x50, 8))[0]
                    owners = ranges.owners(target)
                    table.update(target=hex(target), target_rva=hex(target - image.base),
                                 target_executable=image.executable(target),
                                 first_slot_executable=image.executable(first),
                                 exact_owners=owners,
                                 target_is_canonical_entry=len(owners) == 1 and target == integer(
                                     corpus.functions[owners[0]]["address"]),
                                 observations=observations(target, owners))
                    if not table["first_slot_executable"]:
                        reasons.append("first_vtable_slot_not_executable")
                    if not table["target_executable"]:
                        reasons.append("slot_target_not_executable")
                    if len(owners) != 1:
                        reasons.append("slot_target_without_unique_exact_owner")
                    elif not table["target_is_canonical_entry"]:
                        reasons.append("interior_target_not_canonical_entry")
                except ValueError as error:
                    reasons.append("slot_bytes_unavailable: " + str(error))
            table["reasons"] = reasons
            table["validated_candidate"] = not reasons
            table["slot_semantics"] = ("validated_primary_dispatcher_candidate" if not reasons else
                                       "unvalidated_raw_primary_plus_0x50_probe")
            record["primary_tables"].append(table)

    handlers = {}
    all_classes = {**roots, **classes}
    ancestor_targets = {
        address: {t["target"] for t in record["primary_tables"] if t["validated_candidate"]}
        for address, record in all_classes.items()}
    for record in classes.values():
        for table in record["primary_tables"]:
            if not table["validated_candidate"]:
                continue
            target = table["target"]
            handler = handlers.setdefault(target, {
                "address": target, "rva": table["target_rva"],
                "canonical_owner": table["exact_owners"][0],
                "canonical_name": corpus.functions[table["exact_owners"][0]]["name"],
                "canonical_body": corpus.functions[table["exact_owners"][0]]["body"],
                "canonical_body_sha256": corpus.functions[table["exact_owners"][0]].get("sha256"),
                "classes": [], "vtable_addresses": [], "observations": table["observations"],
                "role_status": "candidate_slot_target_not_behavior_confirmed",
                "same_target_ancestor_relationships": [],
                "sharing": "same_address; inheritance_or_identical_code_folding_not_disambiguated"})
            handler["classes"].append(record["type"])
            handler["vtable_addresses"].append(table["vtable"])
            for base in record["base_types"]:
                if target in ancestor_targets.get(base["address"], set()):
                    handler["same_target_ancestor_relationships"].append({
                        "derived_type": record["type"]["address"],
                        "ancestor_type": base["address"],
                        "authority": "RTTI_ancestry_and_equal_slot_pointer_not_source_inheritance"})
    for handler in handlers.values():
        handler["classes"] = sorted({c["address"]: c for c in handler["classes"]}.values(),
                                    key=lambda c: c["address"])
        handler["vtable_addresses"] = sorted(set(handler["vtable_addresses"]))
        handler["same_target_ancestor_relationships"] = list({
            (r["derived_type"], r["ancestor_type"]): r
            for r in handler["same_target_ancestor_relationships"]}.values())

    primary = [table for record in classes.values() for table in record["primary_tables"]]
    return {
        "schema_version": 1, "kind": "primary-control-object-vtable-handler-candidate-catalog",
        "target_sha256": corpus.target, "analysis_id": corpus.latest["analysis_id"],
        "installation_sha256": corpus.latest["installation_sha256"], "image_base": hex(image.base),
        "root_name": root_name, "root_type_descriptors": sorted(root_types), "scan": scan,
        "roots": sorted(roots.values(), key=lambda c: c["type"]["address"]),
        "classes": sorted(classes.values(), key=lambda c: c["type"]["address"]),
        "handlers": sorted(handlers.values(), key=lambda h: integer(h["address"])),
        "unclassified_loci": unclassified, "excluded_non_lineage_loci": excluded,
        "summary": {
            "class_count": len(classes),
            "classes_with_validated_primary_candidate": sum(
                any(t["validated_candidate"] for t in c["primary_tables"]) for c in classes.values()),
            "classes_without_validated_primary_candidate": sum(
                not any(t["validated_candidate"] for t in c["primary_tables"])
                for c in classes.values()),
            "classes_without_primary_table": sum(not c["primary_tables"] for c in classes.values()),
            "primary_table_candidates": len(primary),
            "located_primary_vtables": sum(t["vtable"] is not None for t in primary),
            "validated_primary_table_candidates": sum(t["validated_candidate"] for t in primary),
            "unique_handler_count": len(handlers),
            "all_primary_raw_values_with_unique_canonical_owner": len({
                t["target"] for t in primary if len(t["exact_owners"]) == 1}),
            "validated_unique_handlers_with_exact_canonical_entry": len(handlers),
            "unknown_or_invalid_primary_candidates": sum(not t["validated_candidate"] for t in primary),
            "secondary_loci_preserved": sum(len(c["secondary_loci"]) for c in classes.values()),
            "unknown_lineage_loci": len(unclassified),
            "shared_handler_addresses": sum(len(h["classes"]) > 1 for h in handlers.values()),
            "observed_unique_handlers": sum(any(
                o["canonical_owner_body_observed"] for o in h["observations"])
                for h in handlers.values()),
            "entry_byte_observed_unique_handlers": sum(any(
                o["target_byte_observed"] for o in h["observations"]) for h in handlers.values())},
        "source_traces": [{
            "raw_log_sha256": t["summary"]["measurement"]["raw_log_sha256"],
            "capture": t["summary"]["measurement"].get("capture"),
            "whole_executable_observed": t["summary"]["observed"],
            "observed_validated_unique_handlers": sum(any(
                o["raw_log_sha256"] == t["summary"]["measurement"]["raw_log_sha256"] and
                o["canonical_owner_body_observed"] for o in h["observations"])
                for h in handlers.values()),
            "mapping": t["summary"].get("mapping"),
            "identity": t["summary"].get("identity")} for t in traces],
        "limitations": [
            "COL-backed RTTI classes, not all source classes, XML block types or simulator-supported types.",
            "Only offset-zero primary tables with unique nonvirtual root at zero can be validated candidates.",
            "Secondary tables and unknown/invalid primary/lineage observations remain separate.",
            "RTTI does not encode vtable length; lineage, storage and slot pointer checks support candidates.",
            "A shared handler body observed in drcov does not identify which class instance dispatched it.",
            "Shared addresses may reflect inheritance, thunks or identical code folding; not source ownership.",
            "Default/no-op/guard stubs may be slot targets; unique addresses are not distinct block algorithms.",
            "XML type names/IDs and simulator support remain unknown; no fuzzy mapping is performed.",
            "Exact owner means frozen analyzed range containment, never nearest-start attribution.",
            "Original trace/maps, unmapped callback records and whole-engine denominators are unchanged."]}


def export_handler_catalog(corpus, image, traces, output):
    output = Path(output)
    if (output / "catalog.json").exists():
        raise ValueError("Existing catalogs are immutable; select a new output directory")
    catalog = build_handler_catalog(corpus, image, traces)
    output = private_directory(output)
    exporter = output / "exporter.py"
    exporter.write_bytes(Path(__file__).read_bytes())
    catalog["artifacts"] = [
        evidence(corpus.latest_path), evidence(corpus.index_path), evidence(image.path),
        evidence(exporter), *[a for trace in traces for a in trace["evidence"]]]
    path = output / "catalog.json"
    save(path, catalog)
    return path


def recover_switch(image, function, request, capstone, corpus):
    """Prove one bounded AMD64 byte-selector/u32-target switch idiom."""
    if image.sha256 != corpus.target:
        raise ValueError("Switch proof image/corpus identity mismatch")
    start, site = integer(request["guard_start"]), integer(request["site"])
    if not any(integer(r["start"]) <= start <= site + 1 <= integer(r["end"])
               for r in function["body"]["ranges"]):
        raise ValueError("Switch proof window is outside the selected exact body")
    insns = decode_window(image, start, site + 1, capstone)
    if (len(insns) != 10 or insns[-1].address != site or insns[-1].size != 2 or
            [i.mnemonic for i in insns] !=
            ["mov", "add", "cmp", "ja", "cdqe", "lea", "movzx", "mov", "add", "jmp"]):
        raise ValueError("Unsupported switch instruction sequence; no targets inferred")
    x = capstone.x86

    def reg(operand, name):
        return operand.type == x.X86_OP_REG and operand.reg == getattr(x, "X86_REG_" + name)

    def imm(operand):
        return operand.type == x.X86_OP_IMM

    def mem(operand, base, index, scale, size):
        return (operand.type == x.X86_OP_MEM and operand.size == size and
                operand.mem.base == getattr(x, "X86_REG_" + base) and
                operand.mem.index == getattr(x, "X86_REG_" + index) and
                operand.mem.scale == scale and operand.mem.segment == 0)

    ops = [i.operands for i in insns]
    if not (all(len(ops[i]) == 2 for i in (0, 1, 2, 5, 6, 7, 8)) and
            all(len(ops[i]) == 1 for i in (3, 9)) and
            reg(ops[0][0], "EAX") and ops[0][1].type == x.X86_OP_MEM and
            ops[0][1].size == 4 and ops[0][1].mem.base != 0 and
            ops[0][1].mem.index == 0 and ops[0][1].mem.segment == 0 and
            reg(ops[1][0], "EAX") and imm(ops[1][1]) and
            reg(ops[2][0], "EAX") and imm(ops[2][1]) and imm(ops[3][0]) and
            reg(ops[5][0], "RDX") and mem(ops[5][1], "RIP", "INVALID", 1, 8) and
            reg(ops[6][0], "EAX") and mem(ops[6][1], "RDX", "RAX", 1, 1) and
            reg(ops[7][0], "ECX") and mem(ops[7][1], "RDX", "RAX", 4, 4) and
            reg(ops[8][0], "RCX") and reg(ops[8][1], "RDX") and reg(ops[9][0], "RCX")):
        raise ValueError("Unsupported switch register/dataflow form; no targets inferred")
    bound, addend = ops[2][1].imm, ops[1][1].imm
    if not 0 <= bound < 65536:
        raise ValueError("Switch selector bound exceeds audited read limit")
    if start <= ops[3][0].imm <= site + 1:
        raise ValueError("Out-of-range switch branch does not leave the proof window")
    base = insns[5].address + insns[5].size + ops[5][1].mem.disp
    selector_address = base + ops[6][1].mem.disp
    table_address = base + ops[7][1].mem.disp
    selectors = image.read(selector_address, bound + 1)
    baseline = cfg(image, function, capstone)
    decoded = {byte: integer(i["address"]) for b in baseline["basic_blocks"]
               for i in b["instructions"] for byte in range(
                   integer(i["address"]), integer(i["address"]) + i["size"])}
    if decoded.get(site) != site or decoded.get(start) != start:
        raise ValueError("Switch guard/site is not entry-reachable at instruction boundaries")
    interior = {i.address for i in insns[1:]}
    unexpected = [edge for block in baseline["basic_blocks"] for edge in block["successors"]
                  if integer(edge["target"]) in interior and
                  not start <= integer(block["instructions"][-1]["address"]) <= site]
    if unexpected:
        raise ValueError("Direct control flow bypasses the proposed switch guard")
    entries, targets = [], []
    for selector in sorted(set(selectors)):
        raw = image.read(table_address + selector * 4, 4)
        value = struct.unpack("<I", raw)[0]  # MOV ECX zero-extends, not signed rel32.
        target = base + value
        owners = exact_owners(corpus, target)
        in_range = any(integer(r["start"]) <= target <= integer(r["end"])
                       for r in function["body"]["ranges"])
        executable = image.executable(target)
        first = []
        if executable:
            decoder = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_64)
            end = next((integer(r["end"]) for r in function["body"]["ranges"]
                        if integer(r["start"]) <= target <= integer(r["end"])), target)
            first = list(decoder.disasm(image.read(target, min(15, end - target + 1)),
                                       target, count=1))
        boundary_conflict = target in decoded and decoded[target] != target
        valid = (executable and in_range and owners == [function["address"]] and
                 bool(first) and not boundary_conflict)
        cases = [((index - addend) & 0xFFFFFFFF)
                 for index, value in enumerate(selectors) if value == selector]
        entries.append({"selector": selector, "table_entry_address": hex(table_address + selector * 4),
                        "encoded_u32": hex(value), "target": hex(target),
                        "executable": executable, "in_selected_exact_body": in_range,
                        "exact_owners": owners, "instruction_decodes": bool(first),
                        "known_instruction_boundary_conflict": boundary_conflict,
                        "validated_for_cfg": valid, "input_values": cases})
        if valid:
            targets.append(target)
    default = ops[3][0].imm
    default_valid = (image.executable(default) and decoded.get(default) == default and
                     exact_owners(corpus, default) == [function["address"]] and
                     any(integer(r["start"]) <= default <= integer(r["end"])
                         for r in function["body"]["ranges"]))
    complete = all(e["validated_for_cfg"] for e in entries) and default_valid
    return {
        "kind": "bounded_two_stage_switch_proof", "target_sha256": image.sha256,
        "site": hex(site), "status": "proven" if complete else "partial",
        "instructions": instruction_evidence(insns),
        "selector_expression": {
            "source": insns[0].op_str, "addition_modulo_2pow32": addend,
            "accepted_unsigned_index_min": 0, "accepted_unsigned_index_max": bound,
            "guard": hex(insns[3].address), "out_of_range_target": hex(default),
            "out_of_range_target_validated": default_valid},
        "base_address": hex(base), "selector_address": hex(selector_address),
        "selector_count": len(selectors), "selector_bytes_sha256": hashlib.sha256(selectors).hexdigest(),
        "target_table_address": hex(table_address), "target_encoding": "u32_zero_extended_plus_base",
        "entries": entries, "validated_targets": [hex(t) for t in sorted(set(targets))],
        "limitations": [
            "Pattern-specific proof, not a general symbolic evaluator or switch recognizer.",
            "No known direct edge bypasses the guard; arbitrary exception transfers are not modeled.",
            "Instruction decoding is checked; logical function ownership still inherits exact corpus ranges.",
            "Static destinations do not prove which inputs or branches executed at runtime."]}


def recover_callback(image, request, capstone):
    start, end = integer(request["entry"]), integer(request["window_end_inclusive"])
    insns = decode_window(image, start, end, capstone)
    insns = insns[:8]
    x = capstone.x86
    if (len(insns) != 8 or [i.mnemonic for i in insns] !=
            ["test", "je", "cmp", "jne", "mov", "mov", "jmp", "ret"]):
        raise ValueError("Unsupported callback thunk shape; no callback edge inferred")
    # This audited shape preserves the third argument and tail-dispatches it as
    # this. Do not turn the indirect jump into a fabricated direct-call edge.
    expected = [
        "r8, r8", None, None, None, "rax, qword ptr [r8]", "rcx, r8", None, ""]
    if any(want is not None and insn.op_str != want for insn, want in zip(insns, expected)):
        raise ValueError("Unsupported callback register flow")
    compare, jump = insns[2].operands, insns[6].operands
    if not (insns[1].operands[0].imm == insns[7].address ==
            insns[3].operands[0].imm and len(compare) == 2 and
            compare[0].type == x.X86_OP_MEM and compare[0].size == 4 and
            compare[0].mem.base == x.X86_REG_R8 and compare[0].mem.index == 0 and
            compare[0].mem.segment == 0 and
            compare[1].type == x.X86_OP_IMM and len(jump) == 1 and
            jump[0].type == x.X86_OP_MEM and jump[0].size == 8 and
            jump[0].mem.base == x.X86_REG_RAX and jump[0].mem.index == 0 and
            jump[0].mem.segment == 0):
        raise ValueError("Unsupported callback guard or virtual slot")
    return {"kind": "guarded_callback_virtual_tail_dispatch", "entry": hex(start),
            "decoded_end_inclusive": hex(insns[7].address + insns[7].size - 1),
            "target_sha256": image.sha256, "instructions": instruction_evidence(insns),
            "argument_register": "r8", "this_register": "rcx",
            "guard_field_offset": hex(compare[0].mem.disp),
            "guard_constant_u32": hex(compare[1].imm & 0xFFFFFFFF),
            "virtual_slot": hex(jump[0].mem.disp), "jump_site": hex(insns[6].address),
            "direct_callback_to_worker_call": False,
            "limitation": "Live vtable/object identity and the dynamic destination are not in drcov."}


def cfg(image, function, capstone, indirect=None):
    """Recover direct-flow reachable basic blocks without guessing switch targets."""
    ranges = [(integer(r["start"]), integer(r["end"]))
              for r in function["body"]["ranges"]]
    entry = integer(function["address"])
    decoder = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_64)
    decoder.detail = True
    instructions, edges, calls, omissions, owners = {}, {}, [], [], {}
    pending, leaders = [entry], {entry}
    indirect = indirect or {}

    def inside(address):
        return any(start <= address <= end for start, end in ranges)

    while pending:
        address = pending.pop()
        if address in instructions:
            continue
        if not inside(address):
            omissions.append({"kind": "flow_outside_exact_body", "address": hex(address)})
            continue
        if address in owners:
            omissions.append({"kind": "overlapping_instruction_target", "address": hex(address),
                              "instruction": hex(owners[address])})
            continue
        end = next(end for start, end in ranges if start <= address <= end)
        try:
            raw = image.read(address, min(15, end - address + 1))
        except ValueError:
            omissions.append({"kind": "bytes_unavailable", "address": hex(address)})
            continue
        insn = next(decoder.disasm(raw, address, count=1), None)
        if insn is None:
            omissions.append({"kind": "decode_failed", "address": hex(address)})
            continue
        if any(byte in owners for byte in range(address, address + insn.size)):
            omissions.append({"kind": "overlapping_decode", "address": hex(address)})
            continue
        for byte in range(address, address + insn.size):
            owners[byte] = address
        instructions[address] = {"address": hex(address), "size": insn.size,
                                 "bytes": insn.bytes.hex(), "mnemonic": insn.mnemonic,
                                 "operands": insn.op_str}
        fallthrough = address + insn.size
        target = (insn.operands[0].imm if insn.operands and
                  insn.operands[0].type == capstone.x86.X86_OP_IMM else None)
        following = []
        if insn.group(capstone.CS_GRP_CALL):
            calls.append({"site": hex(address), "target": None if target is None else hex(target),
                          "kind": "indirect" if target is None else "direct",
                          "operand": insn.op_str, "return_behavior": "unknown_assume_fallthrough"})
            following.append((fallthrough, "call_return_assumed"))
            leaders.add(fallthrough)
        elif insn.group(capstone.CS_GRP_JUMP):
            if target is None:
                proof = indirect.get(address)
                if proof:
                    for destination in proof["validated_targets"]:
                        destination = integer(destination)
                        following.append((destination, "proven_indirect_branch"))
                        leaders.add(destination)
                if not proof or proof["status"] != "proven":
                    omissions.append({"kind": "unresolved_indirect_jump", "address": hex(address),
                                      "operand": insn.op_str})
            else:
                following.append((target, "branch"))
                leaders.add(target)
            if insn.mnemonic not in {"jmp", "ljmp"}:
                following.append((fallthrough, "fallthrough"))
                leaders.add(fallthrough)
        elif (insn.group(capstone.CS_GRP_RET) or insn.group(capstone.CS_GRP_IRET) or
              insn.group(capstone.CS_GRP_INT) or insn.mnemonic in {"hlt", "ud2"}):
            pass
        else:
            following.append((fallthrough, "fallthrough"))
        edges[address] = following
        pending.extend(target for target, _ in following if target not in instructions)

    blocks, assigned = [], set()
    # All entry roots and branch/call successors begin blocks; a missing successor
    # remains a boundary edge rather than a fabricated decoded block.
    for start in sorted(leaders & instructions.keys()):
        if start in assigned:
            continue
        members, current = [], start
        while current in instructions and current not in assigned:
            members.append(instructions[current])
            assigned.add(current)
            outgoing = edges[current]
            if len(outgoing) != 1:
                break
            target, kind = outgoing[0]
            if kind != "fallthrough" or target in leaders or target not in instructions:
                break
            current = target
        last = integer(members[-1]["address"])
        blocks.append({"start": hex(start), "end_inclusive": hex(last + members[-1]["size"] - 1),
                       "instructions": members,
                       "successors": [{"target": hex(target), "kind": kind,
                                       "decoded": target in instructions}
                                      for target, kind in edges[last]]})
    if assigned != instructions.keys():
        raise ValueError("Internal CFG block partition omitted decoded instructions")
    return {"schema_version": 1, "provider": {"id": "capstone", "version": capstone.__version__},
            "target_sha256": image.sha256,
            "address": hex(entry), "ranges": function["body"]["ranges"],
            "range_authority": function["body"].get("authority", "frozen_exact_Ghidra_body"),
            "root_policy": "entry_and_proven_indirect_targets" if indirect else
                           "canonical_entry_only_direct_control_flow",
            "indirect_target_proofs": list(indirect.values()),
            "basic_blocks": blocks, "calls": calls, "omissions": omissions,
            "decoded_bytes": len(owners),
            "body_bytes_not_decoded": sum(end - start + 1 for start, end in ranges) - len(owners),
            "complete_engine_denominator": False,
            "limitations": [
                "Independent image-byte CFG, not a retained Ghidra analyze_function export.",
                "Only explicitly proven indirect destinations are reconstructed; exception edges are not.",
                "Call return is assumed, including unresolved calls; no no-return oracle.",
                "Range authority is explicit: frozen Ghidra body or bounded noncanonical decode window.",
                "Undecoded body bytes may be unreachable code, padding, or missing switch targets.",
                "drcov blocks use different boundaries; no block or branch percentage is inferred."]}


def export(corpus, review_path, traces, output, image=None, capstone=None):
    review = load(review_path)
    if (review["target_sha256"] != corpus.target or
            review["analysis_id"] != corpus.latest["analysis_id"]):
        raise ValueError("Review does not match frozen target/analysis")
    if image and (image.sha256 != corpus.target or sha256(image.path) != corpus.target):
        raise ValueError("PE bytes do not match canonical target")
    if image and image.base != integer(review["image_base"]):
        raise ValueError("Review/PE image base mismatch")
    if capstone and image is None:
        raise ValueError("CFG requires pinned PE bytes")
    if any(f.get("switches") for f in review["functions"]) and not capstone:
        raise ValueError("Switch refinements require --cfg and pinned PE bytes")
    if review.get("regions") and not capstone:
        raise ValueError("Noncanonical region refinements require --cfg and pinned PE bytes")
    if (Path(output) / "ledger.json").exists():
        raise ValueError("Existing ledgers are immutable; select a new output directory")
    supporting = []
    for artifact in review.get("supporting_artifacts", []):
        actual = evidence(artifact["path"])
        if actual["sha256"] != artifact["sha256"]:
            raise ValueError("Supporting evidence hash mismatch")
        supporting.append({**actual, "kind": artifact["kind"]})
    requested = review["functions"]
    seen = set()
    # Validate all identities and review/body availability before writing dossiers.
    for item in requested:
        address = hex(integer(item["address"]))
        if address in seen:
            raise ValueError(f"Duplicate review entry: {address}")
        seen.add(address)
        if item["classification"] not in CLASSIFICATIONS or not item["roles"]:
            raise ValueError("Explicit role and review classification required")
        if not item.get("facts") or not item.get("uncertainties"):
            raise ValueError("Each review needs facts and uncertainty, including confirmed roles")
        corpus.body(address)
    output = private_directory(output)
    exporter_copy = output / "exporter.py"
    exporter_copy.write_bytes(Path(__file__).read_bytes())
    artifacts = [evidence(corpus.latest_path), evidence(corpus.index_path),
                 evidence(corpus.database), evidence(review_path), evidence(exporter_copy)]
    artifacts.extend(supporting)
    if image:
        artifacts.append(evidence(image.path))
    for trace in traces:
        artifacts.extend(trace["evidence"])
    functions = []
    for item in requested:
        address = hex(integer(item["address"]))
        canonical = corpus.functions[address]
        source, text = corpus.body(address)
        dossier = output / "functions" / address
        dossier.mkdir(parents=True, exist_ok=True)
        copy = dossier / "canonical.c"
        copy.write_bytes(source.read_bytes())
        observations = []
        for trace in traces:
            blocks = trace["functions"].get(address, [])
            observations.append({
                "raw_log_sha256": trace["summary"]["measurement"]["raw_log_sha256"],
                "records": len(blocks),
                "distinct_blocks": len({(b["rva"], b["size"]) for b in blocks}),
                "observed": bool(blocks),
                "capture": trace["summary"]["measurement"].get("capture"),
                "meaning": "whole_BB_containment_not_role_or_behavior_confirmation"})
        record = dict(item)
        record.update({
            "address": address, "name": canonical["name"],
            "rva": hex(integer(address) - (image.base if image else
                        integer(review["image_base"]))),
            "exact_body": canonical["body"],
            "rva_ranges_inclusive": [
                {"start": hex(integer(r["start"]) - integer(review["image_base"])),
                 "end": hex(integer(r["end"]) - integer(review["image_base"]))}
                for r in canonical["body"]["ranges"]],
            "canonical": evidence(source),
            "canonical_limitations": canonical.get("limitations", []),
            "warning_lines": [line.strip() for line in pseudocode(text).splitlines()
                              if "WARNING" in line],
            "decompiler_call_mentions": [a for a in call_mentions(text) if a != address],
            "decompiler_callers": corpus.callers(address),
            "call_reference_authority": "lexical_decompiler_mentions_not_canonical_call_graph",
            "observations": observations})
        if image:
            record["verified_byte_facts"] = []
            for fact in item.get("byte_facts", []):
                raw = bytes.fromhex(fact["bytes"])
                if not raw or len(raw) > 256 or image.read(integer(fact["address"]), len(raw)) != raw:
                    raise ValueError("Reviewed byte fact differs from pinned image")
                record["verified_byte_facts"].append({
                    **fact, "verified": True, "target_sha256": image.sha256})
            record["unwind_regions"] = [
                image.unwind_regions(integer(r["start"]), integer(r["end"]))
                for r in canonical["body"]["ranges"]]
            slots = tuple(integer(s) for s in item.get("rtti_slots", ["0x48", "0x50", "0x58"]))
            record["rtti"] = [link for name in item.get("rtti_names", [])
                              for link in image.rtti(name, slots)]
        if capstone:
            proofs = [recover_switch(image, canonical, request, capstone, corpus)
                      for request in item.get("switches", [])]
            record["switch_proofs"] = proofs
            graph = cfg(image, canonical, capstone,
                        {integer(p["site"]): p for p in proofs})
            graph_path = dossier / "cfg.json"
            save(graph_path, graph)
            record["cfg"] = evidence(graph_path)
            record["cfg_summary"] = {
                "blocks": len(graph["basic_blocks"]), "decoded_bytes": graph["decoded_bytes"],
                "body_bytes_not_decoded": graph["body_bytes_not_decoded"],
                "omissions": dict(Counter(o["kind"] for o in graph["omissions"])),
                "unresolved_call_sites": sum(c["kind"] == "indirect" for c in graph["calls"])}
            decoded = {byte for block in graph["basic_blocks"] for insn in block["instructions"]
                       for byte in range(integer(insn["address"]),
                                         integer(insn["address"]) + insn["size"])}
            record["cfg_observation_comparison"] = []
            for trace in traces:
                blocks = trace["functions"].get(address, [])
                outside = [b for b in blocks if not all(
                    image.base + integer(b["rva"]) + offset in decoded
                    for offset in range(b["size"]))]
                record["cfg_observation_comparison"].append({
                    "raw_log_sha256": trace["summary"]["measurement"]["raw_log_sha256"],
                    "records": len(blocks), "wholly_decoded_records": len(blocks) - len(outside),
                    "partially_or_not_decoded_records": len(outside),
                    "distinct_blocks_outside_direct_cfg": len({
                        (b["rva"], b["size"]) for b in outside}),
                    "meaning": "byte_containment_only_not_equal_BB_boundaries_or_coverage"})
        else:
            record["cfg"] = None
        save(dossier / "dossier.json", record)
        artifacts.extend([evidence(copy), evidence(dossier / "dossier.json")])
        if record["cfg"]:
            artifacts.append(record["cfg"])
        functions.append(record)
    regions = []
    for item in review.get("regions", []):
        entry = integer(item["entry"])
        if (item["classification"] not in CLASSIFICATIONS or not item.get("roles") or
                not item.get("facts") or not item.get("uncertainties")):
            raise ValueError("Noncanonical regions require explicit reviewed roles/facts/uncertainty")
        owners = exact_owners(corpus, entry)
        if owners:
            raise ValueError("Region entry already has an exact canonical owner")
        proof = recover_callback(image, item, capstone)
        window_end = integer(item["window_end_inclusive"])
        model = {"address": hex(entry), "body": {
            "ranges": [{"start": hex(entry), "end": hex(window_end)}],
            "authority": "explicit_decode_window_not_function_or_unwind_bounds"}}
        graph = cfg(image, model, capstone)
        dossier = output / "regions" / hex(entry)
        save(dossier / "cfg.json", graph)
        record = {**item, "entry": hex(entry), "rva": hex(entry - image.base),
                  "canonical_function": None, "exact_owners": owners,
                  "callback_proof": proof,
                  "unwind_regions": image.unwind_regions(entry, window_end),
                  "cfg": evidence(dossier / "cfg.json"), "observations": []}
        slots = tuple(integer(s) for s in item.get("rtti_slots", ["0x10"]))
        record["rtti"] = [link for name in item.get("rtti_names", [])
                          for link in image.rtti(name, slots)]
        record["static_virtual_destination_candidates"] = [
            {"class": link["class"], "vtable": link["vtable"], "slot": slot["offset"],
             "target": slot["target"], "runtime_edge_confirmed": False}
            for link in record["rtti"] if integer(link["object_offset"]) == 0
            for slot in link["slots"] if slot["offset"] == proof["virtual_slot"]]
        for trace in traces:
            blocks = [b for b in trace["map"]["blocks"] if
                      entry <= image.base + integer(b["rva"]) and
                      image.base + integer(b["rva"]) + b["size"] - 1 <=
                      integer(proof["decoded_end_inclusive"])]
            record["observations"].append({
                "raw_log_sha256": trace["summary"]["measurement"]["raw_log_sha256"],
                "records": len(blocks), "distinct_blocks": len({
                    (b["rva"], b["size"]) for b in blocks}),
                "source_statuses": dict(Counter(b["status"] for b in blocks)),
                "blocks": blocks, "meaning": "bounded_code_region_presence_not_function_ownership"})
        save(dossier / "dossier.json", record)
        artifacts.extend([record["cfg"], evidence(dossier / "dossier.json")])
        regions.append(record)
    ledger = {
        "schema_version": 1, "kind": "reviewed-native-function-subscope",
        "target_sha256": corpus.target, "analysis_id": corpus.latest["analysis_id"],
        "installation_sha256": corpus.latest["installation_sha256"],
        "image_base": review["image_base"], "review_context": review.get("context"),
        "functions": functions, "noncanonical_regions": regions, "artifacts": artifacts,
        "denominator": {"whole_simulator_functions": None, "whole_simulator_basic_blocks": None,
                        "reviewed_functions": len(functions), "coverage_percentage": None},
        "boundaries": review.get("boundaries", []),
        "limitations": [
            "Human-reviewed role hypotheses; execution or a string hit alone proves no ownership.",
            "Post-attach traces exclude application startup and SimulationStart.",
            "Trace comparison is a lead only; captures are not equivalent behavioral experiments.",
            "Indirect calls, unknown modules, function-boundary errors and omitted CFG edges remain.",
            "Selected local CFGs are not a whole-engine static denominator."]}
    path = output / "ledger.json"
    save(path, ledger)
    return path


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--latest", type=Path, default=ROOT / ".reverse-engineering" / "latest.json")
    commands = parser.add_subparsers(dest="command", required=True)
    query = commands.add_parser("query", help="Targeted read-only FTS lookup; no bodies printed")
    query.add_argument("--fts", required=True)
    query.add_argument("--limit", type=int, default=30)
    query.add_argument("--observed", type=Path, help="Pinned analysis.json, not a raw map")
    rtti = commands.add_parser("rtti", help="Static MSVC RTTI/vtable pointer linkage")
    rtti.add_argument("--pe", type=Path, required=True)
    rtti.add_argument("--type", dest="type_name", required=True)
    rtti.add_argument("--slot", type=integer, action="append",
                      help="Explicit virtual slot offset; repeat to select multiple slots")
    catalog = commands.add_parser("catalog", help="Primary CControlObject RTTI/vtable handler candidates")
    catalog.add_argument("--pe", type=Path, required=True)
    catalog.add_argument("--trace", type=Path, action="append", default=[])
    catalog.add_argument("--output", type=Path, required=True)
    write = commands.add_parser("export", help="Export reviewed dossiers into ignored scope storage")
    write.add_argument("--review", type=Path, required=True)
    write.add_argument("--trace", type=Path, action="append", default=[])
    write.add_argument("--output", type=Path, required=True)
    write.add_argument("--pe", type=Path)
    write.add_argument("--cfg", action="store_true")
    write.add_argument("--dependency-dir", type=Path)
    args = parser.parse_args(argv)
    try:
        corpus = Corpus(args.latest)
        if args.command == "query":
            if args.limit < 1:
                raise ValueError("--limit must be positive")
            observed = None
            if args.observed:
                observed = read_trace(corpus, args.observed)["functions"]
            print(json.dumps({"target_sha256": corpus.target,
                              "results": corpus.query(args.fts, args.limit, observed),
                              "limit": args.limit, "roles_inferred": False}, indent=2))
        elif args.command == "rtti":
            image = PE(args.pe)
            if image.sha256 != corpus.target or sha256(args.pe) != corpus.target:
                raise ValueError("PE bytes do not match canonical target")
            print(json.dumps(image.rtti(args.type_name, args.slot or (0x48, 0x50, 0x58)), indent=2))
        elif args.command == "catalog":
            traces = [read_trace(corpus, path) for path in args.trace]
            path = export_handler_catalog(corpus, PE(args.pe), traces, args.output)
            print(json.dumps({"catalog": str(path), "sha256": sha256(path),
                              "summary": load(path)["summary"]}, indent=2))
        else:
            traces = [read_trace(corpus, path) for path in args.trace]
            image = PE(args.pe) if args.pe else None
            module = capstone_module(args.dependency_dir) if args.cfg else None
            path = export(corpus, args.review, traces, args.output, image, module)
            print(json.dumps({"ledger": str(path), "sha256": sha256(path)}, indent=2))
    except (ValueError, KeyError, OSError, sqlite3.Error, struct.error) as error:
        parser.exit(1, f"scope: {error}\n")


if __name__ == "__main__":
    main()
