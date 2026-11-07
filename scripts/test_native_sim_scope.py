"""Synthetic scope, PE/RTTI, and CFG evidence fixtures; no vendor input."""

import copy
from contextlib import closing
import importlib.util
import json
from pathlib import Path
import shutil
import sqlite3
import struct
import unittest
import uuid


SPEC = importlib.util.spec_from_file_location(
    "native_sim_scope", Path(__file__).with_name("native-sim-scope.py"))
SCOPE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(SCOPE)
BASE = 0x140000000
ADDRESS = hex(BASE + 0x1000)


def fixture_pe(code=b"\x85\xc0\x74\x05\xe8\x17\0\0\0\xff\xd0\xff\xe0\xc3"):
    data = bytearray(0xA00)
    data[:2] = b"MZ"
    struct.pack_into("<I", data, 60, 0x80)
    data[0x80:0x84] = b"PE\0\0"
    struct.pack_into("<HH", data, 0x84, 0x8664, 1)
    struct.pack_into("<H", data, 0x94, 240)
    struct.pack_into("<H", data, 0x98, 0x20B)
    struct.pack_into("<Q", data, 0x98 + 24, BASE)
    section = 0x98 + 240
    data[section:section + 8] = b".fixture"
    struct.pack_into("<IIII", data, section + 8, 0x1000, 0x1000, 0x800, 0x200)
    struct.pack_into("<I", data, section + 36, 0x60000020)
    data[0x200:0x200 + len(code)] = code
    name = b".?AVAuthoredFixture@@\0"
    data[0x610:0x610 + len(name)] = name
    struct.pack_into("<IIIIII", data, 0x700, 1, 0, 0, 0x1400, 0x1550, 0x1500)
    struct.pack_into("<Q", data, 0x7F8, BASE + 0x1500)
    for slot in (0x48, 0x50, 0x58):
        struct.pack_into("<Q", data, 0x800 + slot, BASE + 0x1000)
    return bytes(data)


class Fixture(unittest.TestCase):
    def module(self):
        try:
            return SCOPE.capstone_module(SCOPE.PRIVATE / "dependencies")
        except ValueError:
            self.skipTest("Optional capstone dependency unavailable")

    def setUp(self):
        self.directory = SCOPE.PRIVATE / ("test-" + uuid.uuid4().hex)
        self.directory.mkdir(parents=True)
        self.addCleanup(shutil.rmtree, self.directory)
        self.pe_path = self.directory / "fixture.exe"
        self.pe_path.write_bytes(fixture_pe())
        self.target = SCOPE.sha256(self.pe_path)
        self.image = SCOPE.PE(self.pe_path)
        self.function = {
            "address": ADDRESS, "name": "authored_entry", "status": "written",
            "relative_path": "functions/entry.c",
            "body": {"available": True, "non_contiguous": False,
                     "ranges": [{"start": ADDRESS, "end": hex(BASE + 0x100D)}]}}
        self.body_path = self.directory / "code" / "functions" / "entry.c"
        self.body_path.parent.mkdir(parents=True)
        self.body_path.write_text(
            "// metadata FUN_140001099()\n"
            "void FUN_140001000() { FUN_140001050(); }\n", encoding="utf-8")
        self.function["sha256"] = SCOPE.sha256(self.body_path)
        self.index = {"target_sha256": self.target, "functions": [
            self.function, {"address": "EXTERNAL:0x1", "status": "external"}]}
        self.index_path = self.directory / "code" / "decompile-index.json"
        SCOPE.save(self.index_path, self.index)
        self.database_path = self.directory / "research.sqlite3"
        with closing(sqlite3.connect(self.database_path)) as connection:
            connection.execute(
                "CREATE VIRTUAL TABLE functions USING fts5(address UNINDEXED,name,body)")
            connection.executemany("INSERT INTO functions VALUES (?,?,?)", [
                (ADDRESS, "entry", self.body_path.read_text(encoding="utf-8")),
                (hex(BASE + 0x1050), "callee", "void FUN_140001050() {}"),
                ("EXTERNAL:0x1", "external", "external symbol")])
            connection.commit()
        self.latest_path = self.directory / "latest.json"
        SCOPE.save(self.latest_path, {
            "analysis_id": "authored-analysis", "installation_sha256": "b" * 64,
            "binaries": {"LoxoneConfig.exe": {
                "database": str(self.database_path), "export": {
                    "index_path": str(self.index_path), "target_sha256": self.target}}}})
        self.corpus = SCOPE.Corpus(self.latest_path)
        self.review_path = self.directory / "review.json"
        self.review = {
            "target_sha256": self.target, "analysis_id": "authored-analysis",
            "image_base": hex(BASE), "functions": [
                {"address": ADDRESS, "classification": "candidate", "roles": ["authored-role"],
                 "facts": ["Synthetic branch and call instructions"],
                 "uncertainties": ["Synthetic unresolved indirect jump"],
                 "rtti_names": ["AuthoredFixture"]}]}
        SCOPE.save(self.review_path, self.review)

    def replace_image(self, data):
        self.pe_path.write_bytes(data)
        self.image = SCOPE.PE(self.pe_path)
        self.target = self.image.sha256
        self.index["target_sha256"] = self.target
        SCOPE.save(self.index_path, self.index)
        latest = SCOPE.load(self.latest_path)
        latest["binaries"]["LoxoneConfig.exe"]["export"]["target_sha256"] = self.target
        SCOPE.save(self.latest_path, latest)
        self.corpus = SCOPE.Corpus(self.latest_path)
        self.review["target_sha256"] = self.target
        SCOPE.save(self.review_path, self.review)

    def trace(self):
        mapping_path = self.directory / "map.json"
        SCOPE.save(mapping_path, {
            "application_sha256": self.target, "raw_log_sha256": "c" * 64,
            "blocks": [
                {"record": 1, "rva": "0x1000", "size": 2, "status": "confirmed",
                 "functions": [ADDRESS]},
                {"record": 2, "rva": "0x1000", "size": 2, "status": "confirmed",
                 "functions": [ADDRESS]},
                {"record": 3, "rva": "0x1002", "size": 2, "status": "unmapped",
                 "functions": []}]})
        path = self.directory / "trace.json"
        SCOPE.save(path, {
            "application": {"sha256": self.target},
            "measurement": {"raw_log_sha256": "c" * 64,
                            "capture": {"startup_coverage": False}},
            "observed": {"target_bb_count": 3, "executed_function_count": 1,
                         "unknown_module_bb_count": 0},
            "mapping": {"unmapped": 1},
            "inputs": {"decompile_index_sha256": SCOPE.sha256(self.index_path)},
            "artifact_root": ".", "artifacts": [{
                "kind": "basic-block-function-map", "path": "map.json",
                "sha256": SCOPE.sha256(mapping_path)}]})
        return path


class CorpusTests(Fixture):
    def test_external_spaces_and_observed_query_do_not_infer_roles(self):
        self.assertIn("EXTERNAL:0x1", self.corpus.functions)
        self.assertEqual(self.corpus.query("external")[0]["address"], "EXTERNAL:0x1")
        self.assertEqual(self.corpus.query("entry", observed=set()), [])
        self.assertEqual(self.corpus.query("entry", observed={ADDRESS})[0]["address"], ADDRESS)

    def test_body_hash_mismatch_fails_closed(self):
        self.body_path.write_text("changed", encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "hash mismatch"):
            self.corpus.body(ADDRESS)

    def test_body_cannot_escape_index_directory(self):
        self.function["relative_path"] = "..\\review.json"
        self.corpus.functions[ADDRESS] = self.function
        with self.assertRaisesRegex(ValueError, "escapes"):
            self.corpus.body(ADDRESS)

    def test_lexical_references_ignore_headers_and_self_declaration(self):
        self.assertEqual(SCOPE.call_mentions(self.body_path.read_text(encoding="utf-8")),
                         [ADDRESS, hex(BASE + 0x1050)])
        self.assertEqual(self.corpus.callers(hex(BASE + 0x1050)), [ADDRESS])
        self.assertEqual(self.corpus.callers(hex(BASE + 0x1099)), [])


class TraceTests(Fixture):
    def test_records_are_not_hit_counts_or_role_confirmation(self):
        trace = SCOPE.read_trace(self.corpus, self.trace())
        self.assertEqual(len(trace["functions"][ADDRESS]), 2)
        self.assertEqual(len(trace["functions"]), 1)

    def test_rejects_target_index_map_and_raw_identity_mismatch(self):
        for field in ("target", "index", "map", "raw"):
            with self.subTest(field=field):
                path = self.trace()
                summary = SCOPE.load(path)
                if field == "target":
                    summary["application"]["sha256"] = "0" * 64
                elif field == "index":
                    summary["inputs"]["decompile_index_sha256"] = "0" * 64
                elif field == "map":
                    summary["artifacts"][0]["sha256"] = "0" * 64
                else:
                    summary["measurement"]["raw_log_sha256"] = "0" * 64
                SCOPE.save(path, summary)
                with self.assertRaises(ValueError):
                    SCOPE.read_trace(self.corpus, path)


class PETests(Fixture):
    def test_rtti_recovers_pointer_linkage_not_runtime_ownership(self):
        links = self.image.rtti("AuthoredFixture")
        self.assertEqual(len(links), 1)
        self.assertEqual(links[0]["vtable"], hex(BASE + 0x1600))
        self.assertEqual(links[0]["slots"][1], {"offset": "0x50", "target": ADDRESS})
        self.assertIn("not_runtime_object", links[0]["authority"])

    def test_rtti_rejects_self_rva_mismatch(self):
        data = bytearray(fixture_pe())
        struct.pack_into("<I", data, 0x714, 0x1501)
        self.pe_path.write_bytes(data)
        self.assertEqual(SCOPE.PE(self.pe_path).rtti("AuthoredFixture"), [])

    def test_virtual_tail_and_truncation_are_not_file_backed(self):
        with self.assertRaisesRegex(ValueError, "Not file-backed"):
            self.image.read(BASE + 0x1800, 1)
        self.pe_path.write_bytes(fixture_pe()[:-1])
        with self.assertRaisesRegex(ValueError, "Truncated"):
            SCOPE.PE(self.pe_path)


class ExportTests(Fixture):
    def test_private_export_binds_hashes_ranges_and_observations(self):
        trace = SCOPE.read_trace(self.corpus, self.trace())
        path = SCOPE.export(self.corpus, self.review_path, [trace],
                            self.directory / "output", self.image)
        ledger = SCOPE.load(path)
        function = ledger["functions"][0]
        self.assertEqual(ledger["denominator"]["whole_simulator_functions"], None)
        self.assertEqual(function["rva_ranges_inclusive"], [{"start": "0x1000", "end": "0x100d"}])
        self.assertEqual(function["observations"][0]["records"], 2)
        self.assertEqual(function["observations"][0]["distinct_blocks"], 1)
        self.assertEqual(function["classification"], "candidate")
        self.assertIsNone(function["cfg"])
        for artifact in ledger["artifacts"]:
            self.assertEqual(SCOPE.sha256(artifact["path"]), artifact["sha256"])

    def test_export_rejects_mismatched_review_target_analysis_and_image(self):
        for field in ("target_sha256", "analysis_id", "image_base"):
            with self.subTest(field=field):
                review = copy.deepcopy(self.review)
                review[field] = "0x100" if field == "image_base" else "mismatched"
                SCOPE.save(self.review_path, review)
                with self.assertRaises(ValueError):
                    SCOPE.export(self.corpus, self.review_path, [], self.directory / field, self.image)

    def test_recovered_output_cannot_escape_private_storage(self):
        with self.assertRaisesRegex(ValueError, "must remain"):
            SCOPE.export(self.corpus, self.review_path, [], SCOPE.ROOT / "docs" / "forbidden")
        self.assertFalse((SCOPE.ROOT / "docs" / "forbidden").exists())

    def test_review_requires_explicit_uncertainty(self):
        del self.review["functions"][0]["uncertainties"]
        SCOPE.save(self.review_path, self.review)
        with self.assertRaisesRegex(ValueError, "uncertainty"):
            SCOPE.export(self.corpus, self.review_path, [], self.directory / "invalid")

    def test_existing_ledger_is_not_overwritten(self):
        output = self.directory / "immutable"
        path = SCOPE.export(self.corpus, self.review_path, [], output)
        original = path.read_bytes()
        with self.assertRaisesRegex(ValueError, "immutable"):
            SCOPE.export(self.corpus, self.review_path, [], output)
        self.assertEqual(path.read_bytes(), original)

    def test_reviewed_byte_fact_mismatch_is_rejected(self):
        self.review["functions"][0]["byte_facts"] = [
            {"address": ADDRESS, "bytes": "0000", "meaning": "Wrong authored bytes"}]
        SCOPE.save(self.review_path, self.review)
        with self.assertRaisesRegex(ValueError, "byte fact"):
            SCOPE.export(self.corpus, self.review_path, [], self.directory / "bad-bytes", self.image)


class CFGTests(Fixture):
    def test_real_branches_calls_and_indirect_omissions(self):
        graph = SCOPE.cfg(self.image, self.function, self.module())
        self.assertEqual([b["start"] for b in graph["basic_blocks"]],
                         [hex(BASE + offset) for offset in (0x1000, 0x1004, 0x1009, 0x100B)])
        self.assertEqual([c["kind"] for c in graph["calls"]], ["direct", "indirect"])
        self.assertEqual(graph["calls"][0]["target"], hex(BASE + 0x1020))
        self.assertEqual(graph["omissions"], [{
            "kind": "unresolved_indirect_jump", "address": hex(BASE + 0x100B), "operand": "rax"}])
        self.assertEqual(graph["body_bytes_not_decoded"], 1)
        self.assertFalse(graph["complete_engine_denominator"])

    def test_range_holes_are_not_replaced_with_envelopes(self):
        self.pe_path.write_bytes(fixture_pe(b"\xeb\x00\x90\xc3"))
        self.function["body"]["ranges"] = [
            {"start": ADDRESS, "end": hex(BASE + 0x1001)},
            {"start": hex(BASE + 0x1003), "end": hex(BASE + 0x1003)}]
        graph = SCOPE.cfg(SCOPE.PE(self.pe_path), self.function, self.module())
        self.assertEqual(graph["omissions"][0]["kind"], "flow_outside_exact_body")
        self.assertEqual(graph["omissions"][0]["address"], hex(BASE + 0x1002))
        self.assertEqual(len(graph["basic_blocks"]), 1)

    def test_selected_cfg_export_persists_hashed_graph(self):
        path = SCOPE.export(self.corpus, self.review_path, [], self.directory / "cfg-output",
                            self.image, self.module())
        function = SCOPE.load(path)["functions"][0]
        self.assertEqual(function["cfg"]["sha256"], SCOPE.sha256(function["cfg"]["path"]))
        self.assertEqual(function["cfg_summary"]["blocks"], 4)
        self.assertEqual(function["cfg_summary"]["unresolved_call_sites"], 1)


def switch_pe():
    """Authored two-stage selector: inputs 2/4 -> return 1, input 3 -> return 0."""
    data = bytearray(fixture_pe())
    code = bytearray(b"\x90" * 0x80)
    prefix = b"\x8b\x83\x88\0\0\0\x83\xc0\xfe\x3d\x02\0\0\0"
    prefix += b"\x0f\x87" + struct.pack("<i", 0x1050 - (0x1000 + len(prefix) + 6))
    prefix += b"\x48\x98"
    prefix += b"\x48\x8d\x15" + struct.pack("<i", -(0x1000 + len(prefix) + 7))
    prefix += b"\x0f\xb6\x84\x02" + struct.pack("<i", 0x1700)
    prefix += b"\x8b\x8c\x82" + struct.pack("<i", 0x1740)
    prefix += b"\x48\x03\xca\xff\xe1"
    code[:len(prefix)] = prefix
    code[0x40:0x46] = b"\xb8\x01\0\0\0\xc3"
    code[0x50:0x53] = b"\x31\xc0\xc3"
    data[0x200:0x280] = code
    data[0x900:0x903] = b"\x00\x01\x00"
    struct.pack_into("<II", data, 0x940, 0x1040, 0x1050)
    return data


class SwitchTests(Fixture):
    def prepare_switch(self, data=None):
        self.function["body"]["ranges"][0]["end"] = hex(BASE + 0x1052)
        self.replace_image(switch_pe() if data is None else data)
        return {"guard_start": ADDRESS, "site": hex(BASE + 0x102F)}

    def test_bounded_switch_recovers_only_dataflow_proven_targets(self):
        request = self.prepare_switch()
        proof = SCOPE.recover_switch(
            self.image, self.function, request, self.module(), self.corpus)
        self.assertEqual(proof["status"], "proven")
        self.assertEqual([(e["input_values"], e["target"]) for e in proof["entries"]],
                         [([2, 4], hex(BASE + 0x1040)), ([3], hex(BASE + 0x1050))])
        before = SCOPE.cfg(self.image, self.function, self.module())
        after = SCOPE.cfg(self.image, self.function, self.module(), {BASE + 0x102F: proof})
        self.assertGreater(after["decoded_bytes"], before["decoded_bytes"])
        self.assertFalse(any(o["kind"] == "unresolved_indirect_jump" for o in after["omissions"]))
        self.assertEqual(proof["target_encoding"], "u32_zero_extended_plus_base")

    def test_changed_register_flow_is_rejected_not_guessed(self):
        data = switch_pe()
        data[0x22C:0x22F] = b"\x48\x03\xcb"  # RCX += RBX, not table base RDX.
        request = self.prepare_switch(data)
        with self.assertRaisesRegex(ValueError, "dataflow"):
            SCOPE.recover_switch(self.image, self.function, request, self.module(), self.corpus)

    def test_out_of_range_branch_cannot_bypass_the_bound(self):
        data = switch_pe()
        struct.pack_into("<i", data, 0x210, 0)  # JA falls into the table dataflow.
        request = self.prepare_switch(data)
        with self.assertRaisesRegex(ValueError, "does not leave"):
            SCOPE.recover_switch(self.image, self.function, request, self.module(), self.corpus)

    def test_unsigned_target_outside_image_remains_unresolved(self):
        data = switch_pe()
        struct.pack_into("<I", data, 0x940, 0xFFFF1040)
        request = self.prepare_switch(data)
        proof = SCOPE.recover_switch(
            self.image, self.function, request, self.module(), self.corpus)
        self.assertEqual(proof["status"], "partial")
        self.assertFalse(proof["entries"][0]["validated_for_cfg"])
        graph = SCOPE.cfg(self.image, self.function, self.module(), {BASE + 0x102F: proof})
        self.assertTrue(any(o["kind"] == "unresolved_indirect_jump" for o in graph["omissions"]))

    def test_exact_owner_overlap_cannot_be_promoted(self):
        request = self.prepare_switch()
        self.corpus.functions["0x140001040"] = {
            "address": "0x140001040", "body": {"available": True, "ranges": [
                {"start": "0x140001040", "end": "0x140001045"}]}}
        proof = SCOPE.recover_switch(
            self.image, self.function, request, self.module(), self.corpus)
        self.assertFalse(proof["entries"][0]["validated_for_cfg"])

    def test_table_target_inside_known_instruction_is_not_a_valid_boundary(self):
        data = switch_pe()
        struct.pack_into("<I", data, 0x940, 0x1001)
        request = self.prepare_switch(data)
        proof = SCOPE.recover_switch(
            self.image, self.function, request, self.module(), self.corpus)
        self.assertTrue(proof["entries"][0]["known_instruction_boundary_conflict"])
        self.assertFalse(proof["entries"][0]["validated_for_cfg"])


class UnwindTests(Fixture):
    def data(self):
        data = bytearray(fixture_pe())
        struct.pack_into("<I", data, 0x98 + 108, 16)
        struct.pack_into("<II", data, 0x98 + 112 + 24, 0x16A0, 24)
        struct.pack_into("<IIIIII", data, 0x8A0,
                         0x1000, 0x1004, 0x16D0, 0x1004, 0x100C, 0x16E0)
        data[0x8D0:0x8D4] = b"\x01\0\0\0"
        data[0x8E0:0x8E4] = b"\x21\0\0\0"
        struct.pack_into("<III", data, 0x8E4, 0x1000, 0x1004, 0x16D0)
        return data

    def test_runtime_ranges_are_half_open_and_chains_not_logical_ownership(self):
        self.replace_image(self.data())
        regions = self.image.unwind_regions(BASE + 0x1004, BASE + 0x100B)["matches"]
        self.assertEqual(len(regions), 1)
        self.assertEqual(regions[0]["begin"], hex(BASE + 0x1004))
        self.assertEqual(regions[0]["chained_runtime_function"]["begin"], ADDRESS)
        self.assertIn("not_logical", regions[0]["authority"])
        self.assertEqual(self.image.unwind_regions(BASE + 0x100C)["matches"], [])

    def test_unindexed_leaf_does_not_inherit_the_previous_unwind_entry(self):
        self.replace_image(self.data())
        result = self.image.unwind_regions(BASE + 0x1080)
        self.assertEqual(result["matches"], [])
        self.assertIn("leaf", result["limitation"])
        self.assertIn("No nearest-start", result["limitation"])

    def test_malformed_exception_extent_is_rejected(self):
        data = self.data()
        struct.pack_into("<I", data, 0x98 + 112 + 28, 13)
        self.replace_image(data)
        with self.assertRaisesRegex(ValueError, "exception directory"):
            self.image.runtime_functions()


class CallbackTests(Fixture):
    def callback(self):
        data = bytearray(fixture_pe())
        code = (b"\x4d\x85\xc0\x74\x14\x41\x81\x78\x28\x78\x56\x34\x12\x75\x0a"
                b"\x49\x8b\x00\x49\x8b\xc8\x48\xff\x60\x18\xc3")
        data[0x280:0x280 + len(code)] = code
        struct.pack_into("<Q", data, 0x818, BASE + 0x1000)
        self.replace_image(data)
        return {"entry": hex(BASE + 0x1080), "window_end_inclusive": hex(BASE + 0x109F),
                "classification": "candidate", "roles": ["authored-callback"],
                "facts": ["Synthetic null/sentinel guarded virtual tail dispatch"],
                "uncertainties": ["No runtime object identity"],
                "rtti_names": ["AuthoredFixture"], "rtti_slots": ["0x18"]}

    def test_virtual_tail_dispatch_is_not_fabricated_as_a_direct_call(self):
        request = self.callback()
        proof = SCOPE.recover_callback(self.image, request, self.module())
        self.assertFalse(proof["direct_callback_to_worker_call"])
        self.assertEqual(proof["guard_constant_u32"], "0x12345678")
        self.assertEqual(proof["virtual_slot"], "0x18")
        self.assertEqual(proof["decoded_end_inclusive"], hex(BASE + 0x1099))

    def test_noncanonical_region_export_keeps_unmapped_source_status(self):
        request = self.callback()
        self.review["regions"] = [request]
        SCOPE.save(self.review_path, self.review)
        trace_path = self.trace()
        map_path = self.directory / "map.json"
        mapping = SCOPE.load(map_path)
        mapping["blocks"].append({"record": 4, "rva": "0x1080", "size": 5,
                                  "status": "unmapped", "functions": []})
        SCOPE.save(map_path, mapping)
        summary = SCOPE.load(trace_path)
        summary["artifacts"][0]["sha256"] = SCOPE.sha256(map_path)
        SCOPE.save(trace_path, summary)
        path = SCOPE.export(self.corpus, self.review_path,
                            [SCOPE.read_trace(self.corpus, trace_path)],
                            self.directory / "callback-output", self.image, self.module())
        region = SCOPE.load(path)["noncanonical_regions"][0]
        self.assertIsNone(region["canonical_function"])
        self.assertEqual(region["observations"][0]["source_statuses"], {"unmapped": 1})
        self.assertFalse(region["static_virtual_destination_candidates"][0]["runtime_edge_confirmed"])
        self.assertIn("not_function", SCOPE.load(region["cfg"]["path"])["range_authority"])


def catalog_pe():
    """Authored RTTI graph with sharing, missing ownership and a secondary table."""
    data = bytearray(fixture_pe())
    data.extend(b"\0" * (0x2200 - len(data)))
    data[0x600:0xA00] = b"\0" * 0x400
    struct.pack_into("<H", data, 0x86, 2)
    section = 0x98 + 240 + 40
    data[section:section + 8] = b".rdata\0\0"
    struct.pack_into("<IIII", data, section + 8, 0x1800, 0x3000, 0x1800, 0xA00)
    struct.pack_into("<I", data, section + 36, 0x40000040)
    data[0x210] = 0xC3
    data[0x320] = 0xC3
    cursor, classes = 0x3000, {}

    def allocate(size, alignment=8):
        nonlocal cursor
        cursor = (cursor + alignment - 1) & -alignment
        result = cursor
        cursor += size
        return result

    def write(rva, fmt, *values):
        struct.pack_into(fmt, data, 0xA00 + rva - 0x3000, *values)

    def col_and_table(typ, hierarchy, target, offset):
        col = allocate(24)
        write(col, "<IIIIII", 1, offset, 0, typ, hierarchy, col)
        pointer = allocate(8 + 0x58)
        write(pointer, "<Q", BASE + col)
        for slot in range(0, 0x58, 8):
            write(pointer + 8 + slot, "<Q", BASE + target)
        return col, pointer + 8

    def add_class(name, target, parent=None, mdisp=0, unknown=False):
        typ = allocate(96)
        raw_name = (".?AV" + name + "@@\0").encode()
        start = 0xA00 + typ - 0x3000 + 16
        data[start:start + len(raw_name)] = raw_name
        hierarchy = allocate(16)
        count = 2 if parent else 1
        array = allocate(count * 4)
        own = allocate(28, 4)
        write(own, "<IIiiiII", typ, count - 1, 0, -1, 0, 0x40, hierarchy)
        write(array, "<I", own)
        if parent:
            base = allocate(28, 4)
            write(base, "<IIiiiII", parent["typ"], 0, mdisp, -1, 0, 0x40, parent["hierarchy"])
            write(array + 4, "<I", base)
        write(hierarchy, "<IIII", 0, 0, 8193 if unknown else count, array)
        col, table = col_and_table(typ, hierarchy, target, 0)
        result = {"typ": typ, "hierarchy": hierarchy, "COL": col, "vtable": table}
        classes[name] = result
        return result

    root = add_class("CControlObject", 0x1000)
    shared = add_class("AuthoredSharedA", 0x1000, root)
    add_class("AuthoredSharedB", 0x1000, root)
    add_class("AuthoredOverride", 0x1010, root)
    add_class("AuthoredNoOwner", 0x1120, root)
    add_class("AuthoredInvalid", 0x3000, root)
    add_class("AuthoredNonprimary", 0x1000, root, mdisp=16)
    add_class("CControlObjectUnproven", 0x1000, root, unknown=True)
    add_class("CControlObjectLookalike", 0x1000)
    col_and_table(shared["typ"], shared["hierarchy"], 0x3000, 32)
    return data, classes


class HandlerCatalogTests(Fixture):
    def prepare(self):
        data, types = catalog_pe()
        body = self.directory / "code" / "functions" / "second.c"
        body.write_text("void authored_second() {}\n", encoding="utf-8")
        self.index["functions"].append({
            "address": hex(BASE + 0x1010), "name": "authored_second", "status": "written",
            "relative_path": "functions/second.c", "sha256": SCOPE.sha256(body),
            "body": {"available": True, "ranges": [
                {"start": hex(BASE + 0x1010), "end": hex(BASE + 0x1010)}]}})
        self.replace_image(data)
        return types

    def test_exact_lineage_and_shared_addresses_are_deduplicated(self):
        self.prepare()
        catalog = SCOPE.build_handler_catalog(self.corpus, self.image, [])
        self.assertEqual(catalog["summary"]["class_count"], 6)
        self.assertEqual(catalog["summary"]["classes_with_validated_primary_candidate"], 3)
        self.assertEqual(catalog["summary"]["unique_handler_count"], 2)
        shared = next(h for h in catalog["handlers"] if h["address"] == ADDRESS)
        self.assertEqual(len(shared["classes"]), 2)
        self.assertEqual(len(shared["same_target_ancestor_relationships"]), 2)
        self.assertTrue(all(c["xml_type_name"] is None and c["xml_type_id"] is None and
                            c["simulator_supported"] is None for c in catalog["classes"]))

    def test_invalid_nonprimary_and_unknown_lineage_evidence_is_preserved(self):
        self.prepare()
        catalog = SCOPE.build_handler_catalog(self.corpus, self.image, [])
        by_name = {c["type"]["display_name"]: c for c in catalog["classes"]}
        secondary = by_name["AuthoredSharedA"]["secondary_loci"]
        self.assertEqual(secondary[0]["object_offset"], 32)
        self.assertFalse(secondary[0]["handler_slot_read"])
        self.assertFalse(by_name["AuthoredNonprimary"]["primary_tables"][0]["validated_candidate"])
        self.assertFalse(by_name["AuthoredNoOwner"]["primary_tables"][0]["validated_candidate"])
        self.assertFalse(by_name["AuthoredInvalid"]["primary_tables"][0]["target_executable"])
        self.assertEqual(catalog["summary"]["unknown_lineage_loci"], 1)
        self.assertEqual(catalog["unclassified_loci"][0]["type"]["display_name"],
                         "CControlObjectUnproven")
        self.assertNotIn("CControlObjectLookalike", by_name)

    def test_observed_shared_code_does_not_confirm_class_instances_or_change_maps(self):
        self.prepare()
        path = self.trace()
        original = (self.directory / "map.json").read_bytes()
        trace = SCOPE.read_trace(self.corpus, path)
        catalog = SCOPE.build_handler_catalog(self.corpus, self.image, [trace])
        self.assertEqual(catalog["summary"]["observed_unique_handlers"], 1)
        self.assertEqual(catalog["source_traces"][0]["mapping"]["unmapped"], 1)
        self.assertEqual((self.directory / "map.json").read_bytes(), original)
        observed = next(h for h in catalog["handlers"] if h["address"] == ADDRESS)
        self.assertIn("not_class_instance", observed["observations"][0]["meaning"])

    def test_catalog_export_is_version_bound_and_immutable(self):
        self.prepare()
        path = SCOPE.export_handler_catalog(
            self.corpus, self.image, [], self.directory / "catalog-output")
        catalog = SCOPE.load(path)
        self.assertEqual(catalog["target_sha256"], self.target)
        for artifact in catalog["artifacts"]:
            self.assertEqual(SCOPE.sha256(artifact["path"]), artifact["sha256"])
        with self.assertRaisesRegex(ValueError, "immutable"):
            SCOPE.export_handler_catalog(self.corpus, self.image, [], path.parent)
        self.corpus.target = "0" * 64
        with self.assertRaisesRegex(ValueError, "identity"):
            SCOPE.build_handler_catalog(self.corpus, self.image, [])

    def test_exact_range_lookup_preserves_gaps_and_overlaps(self):
        self.corpus.functions = {
            "a": {"address": "a", "body": {"available": True, "ranges": [
                {"start": "0x1000", "end": "0x1003"}, {"start": "0x1008", "end": "0x1009"}]}},
            "b": {"address": "b", "body": {"available": True, "ranges": [
                {"start": "0x1002", "end": "0x1004"}]}}}
        ranges = SCOPE.ExactRanges(self.corpus)
        self.assertEqual(ranges.owners(0x1002), ["a", "b"])
        self.assertEqual(ranges.owners(0x1005), [])
        self.assertEqual(ranges.owners(0x1008), ["a"])


if __name__ == "__main__":
    unittest.main()
