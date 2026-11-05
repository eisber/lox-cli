"""Independently authored drcov/PE fixtures; no vendor binaries or logs."""

import contextlib
import copy
import importlib.util
import io
import json
from pathlib import Path
import shutil
import struct
import sys
import unittest
import uuid


SPEC = importlib.util.spec_from_file_location(
    "native_sim_coverage", Path(__file__).with_name("native-sim-coverage.py"))
COVERAGE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = COVERAGE
SPEC.loader.exec_module(COVERAGE)

BASE = 0x140000000
TARGET_SHA = "a" * 64
MODULE_PATH = r"C:\Coverage Fixtures\owned,console.exe"


def fixture_log(blocks=(), *, start=0x7FF600000000, size=0x3000, path=MODULE_PATH,
                preferred=None, checksum=0x1234, timestamp=0x5678):
    preferred = start if preferred is None else preferred
    header = (
        "DRCOV VERSION: 3\n"
        "DRCOV FLAVOR: drcov\n"
        "Module Table: version 5, count 1\n"
        f"{COVERAGE.COLUMNS}\n"
        f"  0,   0, 0x{start:016x}, 0x{start + size:016x}, "
        f"0x{start + 0x1000:016x}, 0000000000000000, "
        f"0x{preferred:016x}, 0x{checksum:08x}, 0x{timestamp:08x},  {path}\n"
        f"BB Table: {len(blocks)} bbs\n"
    )
    return header.encode() + b"".join(struct.pack("<IHH", *block) for block in blocks)


def function(address=0x1000, ranges=((0x1000, 0x101F),), name="authored_function"):
    return {
        "address": hex(BASE + address), "name": name, "status": "written",
        "body": {"available": True, "non_contiguous": len(ranges) > 1,
                 "ranges": [{"start": hex(BASE + start), "end": hex(BASE + end)}
                            for start, end in ranges]},
    }


def index(functions=None, target_sha=TARGET_SHA):
    return {"target_sha256": target_sha, "provider": {"id": "authored_fixture"},
            "functions": [function()] if functions is None else functions}


def manifest(raw, target_sha=TARGET_SHA, path=MODULE_PATH):
    return {"schema_version": 1, "raw_sha256": COVERAGE.digest(raw),
            "modules": [{"module_id": 0, "path": path, "sha256": target_sha}]}


def fixture_pe():
    data = bytearray(1024)
    data[:2] = b"MZ"
    struct.pack_into("<I", data, 60, 0x80)
    data[0x80:0x84] = b"PE\0\0"
    struct.pack_into("<HHI", data, 0x84, 0x8664, 1, 0x5678)
    struct.pack_into("<H", data, 0x94, 240)
    optional = 0x98
    struct.pack_into("<H", data, optional, 0x20B)
    struct.pack_into("<I", data, optional + 16, 0x1000)
    struct.pack_into("<Q", data, optional + 24, BASE)
    struct.pack_into("<I", data, optional + 56, 0x3000)
    struct.pack_into("<I", data, optional + 64, 0x1234)
    return bytes(data)


class ParserTests(unittest.TestCase):
    def test_validated_format_binary_layout_and_comma_in_path(self):
        raw = fixture_log([(0x1010, 5, 0), (0x1020, 7, 0)])
        parsed = COVERAGE.parse_drcov(raw)
        self.assertEqual(parsed.modules[0].path, MODULE_PATH)
        self.assertEqual(parsed.bb_count, 2)
        self.assertEqual(list(parsed.entries()), [(0x1010, 5, 0), (0x1020, 7, 0)])
        self.assertEqual(parsed.sha256, COVERAGE.digest(raw))

    def test_rejects_truncation_trailing_bytes_and_bad_counts(self):
        raw = fixture_log([(0x1000, 1, 0)])
        cases = [raw[:-1], raw + b"\n",
                 raw.replace(b"BB Table: 1 bbs", b"BB Table: 2 bbs"),
                 raw.replace(b"count 1", b"count 2"),
                 raw.split(b"BB Table:")[0]]
        for case in cases:
            with self.subTest(case=case[-24:]), self.assertRaises(COVERAGE.CoverageError):
                COVERAGE.parse_drcov(case)

    def test_rejects_unvalidated_formats_columns_and_text_tables(self):
        raw = fixture_log([(0x1000, 1, 0)])
        cases = [
            raw.replace(b"VERSION: 3", b"VERSION: 2"),
            raw.replace(b"VERSION: 3", b"VERSION: 4"),
            raw.replace(b"FLAVOR: drcov", b"FLAVOR: drcov-64"),
            raw.replace(b"version 5", b"version 2"),
            raw.replace(b", path\n", b", (custom fields), path\n"),
            raw.replace(b"BB Table: 1 bbs\n", b"BB Table: 1 bbs\nmodule id, start, size:\n"),
        ]
        for case in cases:
            with self.subTest(case=case[:80]), self.assertRaises(COVERAGE.CoverageError):
                COVERAGE.parse_drcov(case)

    def test_rejects_undeclared_modules_zero_sizes_and_out_of_bounds(self):
        for block in [(0x1000, 1, 1), (0x1000, 0, 0), (0x2FFF, 2, 0),
                      (0x3000, 1, 0)]:
            with self.subTest(block=block), self.assertRaises(COVERAGE.CoverageError):
                COVERAGE.parse_drcov(fixture_log([block]))
        self.assertEqual(COVERAGE.parse_drcov(fixture_log([(0x2FFF, 1, 0)])).bb_count, 1)

    def test_explicit_unknown_module_sentinel_is_preserved_not_mapped(self):
        parsed = COVERAGE.parse_drcov(fixture_log([(0x12345678, 4, 65535)]))
        report, full_map = COVERAGE.build_report(
            parsed, COVERAGE.RangeMapper(index(), BASE), {0}, TARGET_SHA, BASE)
        self.assertEqual(report["observed"]["unknown_module_bb_count"], 1)
        self.assertEqual(report["observed"]["target_bb_count"], 0)
        self.assertEqual(full_map["blocks"], [])

    def test_rejects_nonsequential_segmented_or_invalid_module_geometry(self):
        raw = fixture_log()
        cases = [raw.replace(b"  0,   0,", b"  1,   1,"),
                 raw.replace(b"  0,   0,", b"  0,   1,"),
                 raw.replace(b"0000000000000000,", b"0000000000000001,"),
                 raw.replace(b"0x00007ff600003000,", b"0x00007ff600000000,")]
        for case in cases:
            with self.subTest(case=case), self.assertRaises(COVERAGE.CoverageError):
                COVERAGE.parse_drcov(case)

    def test_declared_limits_and_invalid_header_encoding_fail_closed(self):
        raw = fixture_log()
        cases = [
            raw.replace(b"count 1", b"count 65536"),
            raw.replace(b"BB Table: 0 bbs", b"BB Table: 4000001 bbs"),
            raw.replace(MODULE_PATH.encode(), b"C:\\fixture\\\xff.exe"),
            b"x" * 65_537 + b"\n",
        ]
        for case in cases:
            with self.subTest(length=len(case)), self.assertRaises(COVERAGE.CoverageError):
                COVERAGE.parse_drcov(case)


class MappingTests(unittest.TestCase):
    def test_aslr_does_not_change_rva_to_static_function_mapping(self):
        mapper = COVERAGE.RangeMapper(index(), BASE)
        results = []
        for base in (BASE, 0x7FF600000000):
            parsed = COVERAGE.parse_drcov(fixture_log([(0x1000, 4, 0)], start=base))
            report, full_map = COVERAGE.build_report(parsed, mapper, {0}, TARGET_SHA, BASE)
            results.append(full_map["blocks"][0])
            self.assertEqual(report["observed"]["executed_function_count"], 1)
        self.assertEqual(results[0], results[1])
        self.assertEqual(results[0]["functions"], [hex(BASE + 0x1000)])

    def test_inclusive_boundaries_noncontiguous_ranges_and_gaps(self):
        mapper = COVERAGE.RangeMapper(
            index([function(ranges=((0x1000, 0x100F), (0x1100, 0x110F)))]), BASE)
        for rva, size, status in [(0x1000, 16, "confirmed"), (0x100F, 1, "confirmed"),
                                  (0x100F, 2, "crosses_range"), (0x1080, 4, "unmapped"),
                                  (0x1100, 16, "confirmed"), (0x110F, 2, "crosses_range")]:
            with self.subTest(rva=rva, size=size):
                self.assertEqual(mapper.map(rva, size)["status"], status)

    def test_overlapping_functions_even_partial_overlap_are_ambiguous(self):
        mapper = COVERAGE.RangeMapper(index([
            function(ranges=((0x1000, 0x102F),)),
            function(address=0x1010, ranges=((0x1010, 0x103F),), name="other"),
        ]), BASE)
        self.assertEqual(mapper.map(0x1000, 0x14)["status"], "ambiguous")
        self.assertEqual(mapper.map(0x1010, 2)["status"], "ambiguous")
        self.assertEqual(mapper.map(0x1000, 4)["status"], "confirmed")
        report, _ = COVERAGE.build_report(
            COVERAGE.parse_drcov(fixture_log([(0x1010, 2, 0)])),
            mapper, {0}, TARGET_SHA, BASE)
        self.assertEqual(report["observed"]["executed_function_count"], 0)
        self.assertEqual(report["mapping"]["ambiguous"], 1)

    def test_whole_bb_must_fit_one_exact_range_not_only_one_function_union(self):
        mapper = COVERAGE.RangeMapper(
            index([function(ranges=((0x1000, 0x100F), (0x1010, 0x101F)))]), BASE)
        self.assertEqual(mapper.map(0x100E, 4)["status"], "crosses_range")

    def test_duplicate_records_are_not_hit_counts_or_a_coverage_denominator(self):
        parsed = COVERAGE.parse_drcov(fixture_log([(0x1000, 2, 0), (0x1000, 2, 0),
                                                  (0x1002, 2, 0)]))
        report, _ = COVERAGE.build_report(
            parsed, COVERAGE.RangeMapper(index(), BASE), {0}, TARGET_SHA, BASE)
        self.assertEqual(report["observed"]["target_bb_count"], 3)
        self.assertEqual(report["observed"]["target_unique_bb_count"], 2)
        self.assertEqual(report["observed"]["executed_function_count"], 1)
        self.assertEqual(report["mapping"]["confirmed"], 3)
        self.assertIsNone(report["coverage"]["basic_block_percentage"])
        self.assertIsNone(report["coverage"]["branch_percentage"])
        self.assertFalse(report["coverage"]["full_engine_coverage"])
        self.assertEqual(report["scope"]["status"], "unknown")

    def test_empty_coverage_never_becomes_complete_or_100_percent(self):
        report, _ = COVERAGE.build_report(
            COVERAGE.parse_drcov(fixture_log()), COVERAGE.RangeMapper(index(), BASE),
            {0}, TARGET_SHA, BASE)
        self.assertEqual(report["observed"]["executed_function_count"], 0)
        self.assertEqual(report["coverage"]["completion_status"], "blocked")
        self.assertIsNone(report["coverage"]["basic_block_percentage"])

    def test_external_functions_and_unavailable_bodies_are_not_guessed(self):
        external = {"address": "EXTERNAL:0x1", "status": "external",
                    "body": {"available": True, "ranges": [], "non_contiguous": False}}
        unavailable = function(address=0x1100)
        unavailable["body"] = {"available": False, "ranges": []}
        mapper = COVERAGE.RangeMapper(index([external, unavailable]), BASE)
        self.assertEqual(mapper.map(0x1000, 1)["status"], "unmapped")
        self.assertEqual(mapper.external_count, 1)
        self.assertEqual(mapper.unavailable_count, 1)

    def test_invalid_ranges_and_duplicate_functions_fail_closed(self):
        backward = function(ranges=((0x1010, 0x1000),))
        outside = function(ranges=((0x3000, 0x3001),))
        for functions in ([backward], [outside], [function(), function()]):
            with self.subTest(functions=functions), self.assertRaises(COVERAGE.CoverageError):
                COVERAGE.RangeMapper(index(functions), BASE, 0x3000)


class ArtifactTests(unittest.TestCase):
    def setUp(self):
        self.root = (Path(__file__).resolve().parents[1] / ".reverse-engineering" /
                     "native-simulator" / "coverage" / "tests" / uuid.uuid4().hex)
        self.root.mkdir(parents=True)
        self.addCleanup(shutil.rmtree, self.root)
        self.raw = fixture_log([(0x1000, 4, 0)])
        self.raw_path = self.root / "fixture.log"
        self.raw_path.write_bytes(self.raw)
        self.index_path = self.root / "index.json"
        self.pin_path = self.root / "module-manifest.json"
        self.output = self.root / "report.json"
        self.write_json(self.index_path, index())
        self.write_json(self.pin_path, manifest(self.raw))

    def write_json(self, path, value):
        path.write_text(json.dumps(value), encoding="utf-8")

    def invoke(self, extra=(), *, pe_only=False):
        args = ["import", str(self.raw_path), "--artifact-root", str(self.root),
                "--output", str(self.output), "--decompile-index", str(self.index_path)]
        if not pe_only:
            args += ["--module-manifest", str(self.pin_path), "--image-base", hex(BASE)]
        out, err = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            status = COVERAGE.main(args + list(extra))
        return status, out.getvalue(), err.getvalue()

    def test_import_pins_raw_and_target_hash_and_writes_relative_hashed_artifacts(self):
        status, stdout, stderr = self.invoke()
        self.assertEqual(status, 0, stderr)
        report = json.loads(self.output.read_text())
        self.assertEqual(report["measurement"]["raw_log_sha256"], COVERAGE.digest(self.raw))
        self.assertEqual(report["application"]["sha256"], TARGET_SHA)
        self.assertEqual(report["artifact_root"], ".")
        self.assertEqual(report["observed"]["target_bb_count"], 1)
        for item in report["artifacts"]:
            self.assertFalse(Path(item["path"]).is_absolute())
            self.assertEqual(COVERAGE.file_digest(self.root / item["path"]), item["sha256"])
        self.assertEqual(json.loads(stdout)["kind"], "native-basic-block-coverage")
        self.assertNotIn("blocks", json.loads(stdout))

    def test_stale_log_binary_path_or_module_pin_is_rejected_without_output(self):
        for field in ("raw", "binary", "path", "id", "duplicate"):
            pin = manifest(self.raw)
            if field == "raw":
                pin["raw_sha256"] = "b" * 64
            elif field == "binary":
                pin["modules"][0]["sha256"] = "b" * 64
            elif field == "path":
                pin["modules"][0]["path"] = r"C:\Elsewhere\owned,console.exe"
            elif field == "id":
                pin["modules"][0]["module_id"] = 1
            else:
                pin["modules"].append(copy.deepcopy(pin["modules"][0]))
            self.write_json(self.pin_path, pin)
            with self.subTest(field=field):
                status, _, _ = self.invoke()
                self.assertEqual(status, 2)
                self.assertFalse(self.output.exists())
                self.assertFalse(self.output.with_suffix(".bb-map.json").exists())

    def test_windows_path_matching_is_case_insensitive_and_separator_normalized(self):
        pin = manifest(self.raw, path=MODULE_PATH.upper().replace("\\", "/"))
        self.write_json(self.pin_path, pin)
        status, _, stderr = self.invoke()
        self.assertEqual(status, 0, stderr)

    def test_refuses_to_overwrite_either_output(self):
        for path in (self.output, self.root / "report.bb-map.json"):
            path.write_text("keep", encoding="utf-8")
            status, _, _ = self.invoke()
            self.assertEqual(status, 2)
            self.assertEqual(path.read_text(), "keep")
            path.unlink()

    def test_artifacts_cannot_escape_declared_root(self):
        outside = self.root.parent / (self.root.name + ".json")
        status, _, stderr = self.invoke(["--bb-map", str(outside)])
        self.assertEqual(status, 2)
        self.assertIn("artifact-root", stderr)
        self.assertFalse(outside.exists())
        self.assertFalse(self.output.exists())

    def test_duplicate_json_keys_and_stale_tool_receipt_fail_closed(self):
        self.pin_path.write_text('{"schema_version":1,"schema_version":1}')
        self.assertEqual(self.invoke()[0], 2)
        self.write_json(self.pin_path, manifest(self.raw))
        receipt = self.root / "receipt.json"
        self.write_json(receipt, {"schema_version": 1, "repository": "DynamoRIO/dynamorio",
                                 "release": "authored_fixture", "sha256": "b" * 64,
                                 "github_digest": "sha256:" + "c" * 64})
        self.assertEqual(self.invoke(["--tool-receipt", str(receipt)])[0], 2)
        self.assertFalse(self.output.exists())

    def test_tool_receipt_digest_and_release_are_bound_without_inventing_a_version(self):
        receipt = self.root / "receipt.json"
        self.write_json(receipt, {"schema_version": 1, "repository": "DynamoRIO/dynamorio",
                                 "release": "authored_fixture", "sha256": "b" * 64,
                                 "github_digest": "sha256:" + "b" * 64})
        status, _, stderr = self.invoke(["--tool-receipt", str(receipt)])
        self.assertEqual(status, 0, stderr)
        report = json.loads(self.output.read_text())
        self.assertEqual(report["measurement"]["tool_version"], "authored_fixture")
        self.assertEqual(report["measurement"]["tool_receipt_sha256"],
                         COVERAGE.file_digest(receipt))

    def test_boolean_schema_and_non_json_constants_are_not_accepted(self):
        pin = manifest(self.raw)
        pin["schema_version"] = True
        self.write_json(self.pin_path, pin)
        self.assertEqual(self.invoke()[0], 2)
        self.write_json(self.pin_path, manifest(self.raw))
        self.index_path.write_text('{"target_sha256":NaN,"functions":[]}')
        self.assertEqual(self.invoke()[0], 2)
        self.assertFalse(self.output.exists())

    def captured_manifest(self):
        source = self.root / "native-runtime.json"
        self.write_json(source, {"status": "blocked", "post_attach_running": False})
        pin = manifest(self.raw)
        pin["capture"] = {
            "instrumentation_start": "post-start",
            "startup_coverage": False, "simulation_start_coverage": False,
            "interval_utc": {"start": "2026-01-01T10:00:00Z",
                             "end": "2026-01-01T10:00:04Z"},
            "native_case_status": "blocked", "native_running_verified": False,
            "run_id": "authored-run", "cleanup_kind": "client-requested-exit-and-dump",
            "source_artifacts": [{"path": source.name, "sha256": COVERAGE.file_digest(source)}],
        }
        return pin

    def test_post_start_context_remains_blocked_despite_confirmed_function_execution(self):
        self.write_json(self.pin_path, self.captured_manifest())
        status, _, stderr = self.invoke()
        self.assertEqual(status, 0, stderr)
        report = json.loads(self.output.read_text())
        self.assertEqual(report["observed"]["executed_function_count"], 1)
        capture = report["measurement"]["capture"]
        self.assertEqual(capture["native_case_status"], "blocked")
        self.assertFalse(capture["native_running_verified"])
        self.assertFalse(capture["startup_coverage"])
        self.assertFalse(capture["simulation_start_coverage"])
        source = capture["source_artifacts"][0]
        self.assertIn(source, report["artifacts"])

    def test_capture_context_refuses_stale_sources_and_invalid_intervals(self):
        for problem in ("hash", "naive_time", "reversed_time", "outside_root"):
            pin = self.captured_manifest()
            capture = pin["capture"]
            if problem == "hash":
                capture["source_artifacts"][0]["sha256"] = "b" * 64
            elif problem == "naive_time":
                capture["interval_utc"]["start"] = "2026-01-01T10:00:00"
            elif problem == "reversed_time":
                capture["interval_utc"]["start"] = "2026-01-01T11:00:00Z"
            else:
                capture["source_artifacts"][0]["path"] = "..\\outside.json"
            self.write_json(self.pin_path, pin)
            with self.subTest(problem=problem):
                self.assertEqual(self.invoke()[0], 2)
                self.assertFalse(self.output.exists())

    def test_capture_context_cannot_claim_excluded_startup_phases(self):
        for key in ("startup_coverage", "simulation_start_coverage"):
            pin = self.captured_manifest()
            pin["capture"][key] = True
            self.write_json(self.pin_path, pin)
            with self.subTest(key=key):
                self.assertEqual(self.invoke()[0], 2)
                self.assertFalse(self.output.exists())

    def test_native_pe_identity_aslr_and_runtime_expansion(self):
        pe_path = self.root / "owned,console.exe"
        pe_data = fixture_pe()
        pe_path.write_bytes(pe_data)
        self.write_json(self.index_path, index(target_sha=COVERAGE.digest(pe_data)))
        self.raw_path.write_bytes(fixture_log([(0x1000, 4, 0)], size=0x4000))
        status, _, stderr = self.invoke(["--target-pe", str(pe_path)], pe_only=True)
        self.assertEqual(status, 0, stderr)
        report = json.loads(self.output.read_text())
        self.assertEqual(report["mapping"]["image_base"], hex(BASE))
        self.assertEqual(report["observed"]["executed_function_count"], 1)
        self.assertEqual(report["identity"]["runtime_geometry"], [
            {"module_id": 0, "module_span_matches_pe": False,
             "logged_preferred_base_matches_pe": False}])

    def test_pe_hash_header_and_image_base_mismatches_are_rejected(self):
        pe_path = self.root / "owned,console.exe"
        pe_data = fixture_pe()
        pe_path.write_bytes(pe_data)
        self.assertEqual(self.invoke(["--target-pe", str(pe_path)], pe_only=True)[0], 2)
        self.write_json(self.index_path, index(target_sha=COVERAGE.digest(pe_data)))
        self.raw_path.write_bytes(fixture_log([(0x1000, 4, 0)], timestamp=0x9999))
        self.assertEqual(self.invoke(["--target-pe", str(pe_path)], pe_only=True)[0], 2)
        self.raw_path.write_bytes(self.raw)
        self.assertEqual(self.invoke(["--target-pe", str(pe_path), "--image-base",
                                      hex(BASE + 1)], pe_only=True)[0], 2)
        pe_path.write_bytes(pe_data[:100])
        self.assertEqual(self.invoke(["--target-pe", str(pe_path)], pe_only=True)[0], 2)
        self.assertFalse(self.output.exists())


if __name__ == "__main__":
    unittest.main()
