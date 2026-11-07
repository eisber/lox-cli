import copy
import importlib.util
import json
from pathlib import Path
import shutil
import unittest
import uuid


SPEC = importlib.util.spec_from_file_location(
    "native_sim_observe", Path(__file__).with_name("native-sim-observe.py"))
OBSERVE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(OBSERVE)


class ObserverTests(unittest.TestCase):
    def setUp(self):
        self.root = Path(__file__).resolve().parents[1]
        self.directory = self.root / ".reverse-engineering" / "native-simulator" / \
            "observer" / "tests" / uuid.uuid4().hex
        self.directory.mkdir(parents=True)
        self.addCleanup(shutil.rmtree, self.directory)
        self.identity = {
            "name": "synthetic.exe", "sha256": "a" * 64, "size": 0x3000,
            "timestamp": 123, "sections": [(0x1000, 0x1000, 512, 512, 0x60000020)],
        }
        self.sites = [{"id": 7, "rva": 0x1000, "post": 1, "mask": 1}]

    def records(self):
        samples = [
            {"reg": 0, "address": "0x1", "status": 3, "size": 0, "bytes": "",
             "vtable_status": 0, "vtable_size": 0, "vtable_bytes": ""},
        ] + [
            {"reg": i, "address": "0x0", "status": 0, "size": 0, "bytes": "",
             "vtable_status": 0, "vtable_size": 0, "vtable_bytes": ""}
            for i in range(1, 4)
        ]
        return [
            {"type": "HEADER", "schema": 1, "kind": "passive-entry-observer",
             "target_sha256": "a" * 64, "client_sha256": "b" * 64,
             "config_sha256": "c" * 64, "module": "synthetic.exe",
             "module_base": "0x140000000", "module_size": 0x3000,
             "module_timestamp": 123, "module_seen": True, "module_mismatch": False,
             "max_events": 3, "max_objects": 2, "prefix_bytes": 32},
            {"type": "SITE", "id": 7, "rva": 0x1000, "post": 1, "mask": 1,
             "wrapped": True},
            {"type": "ENTRY", "seq": 0, "site": 7, "thread": 21, "context_ok": True,
             "pc": "0x140001000", "rcx": "0x1", "rdx": "0x2", "r8": "0x3",
             "r9": "0x4", "rsp": "0x100008", "return_ok": True,
             "return_address": "0x140002000", "samples": samples},
            {"type": "RETURN", "seq": 1, "site": 7, "thread": 21, "context_ok": True,
             "entry": 0, "rax": "0x1234"},
            {"type": "SUMMARY", "events": 2, "unique_objects": 1,
             "dropped_events": 0, "unreadable_reads": 1, "partial_reads": 0,
             "truncated_reads": 0, "object_limit_reads": 0, "context_failures": 0,
             "nudge_exit": False},
        ]

    def parse(self, records, expected=None):
        path = self.directory / (uuid.uuid4().hex + ".jsonl")
        path.write_text("".join(json.dumps(r) + "\n" for r in records), encoding="utf-8")
        return OBSERVE.parse_log(path, expected)

    def test_explicit_bad_read_and_associated_return(self):
        result = self.parse(self.records())
        self.assertEqual(result["events"][0]["samples"][0]["status"], 3)
        self.assertEqual(result["events"][1]["entry"], 0)
        self.assertEqual(result["summary"]["unreadable_reads"], 1)

    def test_empty_interval_preserves_no_observation_instead_of_inventing_events(self):
        records = self.records()
        records = records[:2] + records[-1:]
        records[-1].update(events=0, unique_objects=0, unreadable_reads=0)
        self.assertEqual(self.parse(records)["events"], [])

    def test_rejects_bad_read_with_fabricated_zero_prefix(self):
        records = self.records()
        records[2]["samples"][0].update(size=32, bytes="00" * 32)
        with self.assertRaisesRegex(ValueError, "status/length"):
            self.parse(records)

    def test_accepts_explicit_partial_prefix_without_vtable(self):
        records = self.records()
        records[2]["samples"][0].update(status=2, size=3, bytes="abcdef")
        self.assertEqual(self.parse(records)["events"][0]["samples"][0]["size"], 3)

    def test_rejects_truncation_missing_summary_and_duplicate_json_key(self):
        for raw in (
            b'{"type":"HEADER"}',
            ("\n".join(json.dumps(r) for r in self.records()[:-1]) + "\n").encode(),
            ("".join(json.dumps(r) + "\n" for r in self.records())).replace(
                '"schema": 1', '"schema": 1, "schema": 1').encode(),
        ):
            with self.subTest(raw=raw[:60]):
                path = self.directory / (uuid.uuid4().hex + ".jsonl")
                path.write_bytes(raw)
                with self.assertRaises(ValueError):
                    OBSERVE.parse_log(path)

    def test_rejects_identity_site_and_return_mismatches(self):
        for modify in (
            lambda r: r[0].update(module_mismatch=True),
            lambda r: r[1].update(wrapped=False),
            lambda r: r[2].update(pc="0x140001001"),
            lambda r: r[3].update(thread=22),
            lambda r: r[3].update(entry=1),
            lambda r: r[-1].update(events=3),
            lambda r: r[0].update(max_events=1),
            lambda r: r[0].update(max_objects=0),
        ):
            records = self.records()
            modify(records)
            with self.subTest(records=records):
                with self.assertRaises(ValueError):
                    self.parse(records)
        with self.assertRaisesRegex(ValueError, "identity mismatch"):
            self.parse(self.records(), {
                "target_sha256": "f" * 64, "client_sha256": "b" * 64,
                "config_sha256": "c" * 64})
        for extra in ({"module_timestamp": 124}, {"sites": []}):
            with self.subTest(extra=extra):
                with self.assertRaises(ValueError):
                    self.parse(self.records(), {
                        "target_sha256": "a" * 64, "client_sha256": "b" * 64,
                        "config_sha256": "c" * 64, **extra})

    def test_config_uses_exact_rvas_and_bounded_selected_registers(self):
        data = OBSERVE.config_bytes(self.identity, "b" * 64, self.sites, 5, 1, 32)
        self.assertTrue(data.endswith(b"\n7 1000 1 1\n"))
        self.assertIn(b"12288 123 5 1 32", data)
        for kwargs in ({"max_events": 4097}, {"max_objects": 513}, {"prefix": 257},
                       {"prefix": 7}, {"max_events": True}):
            with self.subTest(kwargs=kwargs):
                with self.assertRaises(ValueError):
                    OBSERVE.config_bytes(self.identity, "b" * 64, self.sites, **kwargs)

    def test_read_config_path_and_bytes_return_the_same_flat_bound_identity(self):
        payload = OBSERVE.config_bytes(self.identity, "b" * 64, self.sites, 5, 1, 256)
        path = self.directory / "observer.cfg"
        path.write_bytes(payload)
        expected = {
            "target_sha256": "a" * 64, "client_sha256": "b" * 64,
            "config_sha256": OBSERVE.sha256(path), "module": "synthetic.exe",
            "module_size": 0x3000, "module_timestamp": 123,
            "max_events": 5, "max_objects": 1, "prefix_bytes": 256, "sites": self.sites,
        }
        self.assertEqual(OBSERVE.read_config(path), expected)
        self.assertEqual(OBSERVE.read_config(payload), expected)

    def test_read_config_rejects_noncanonical_unbounded_and_ambiguous_recipes(self):
        payload = OBSERVE.config_bytes(self.identity, "b" * 64, self.sites, 5, 1, 32)
        for bad in (
            payload[:-1], payload.replace(b"\n", b"\r\n"), payload + b"7 1000 1 1\n",
            payload.replace(b" 5 1 32 ", b" 4097 1 32 "),
            payload.replace(b" 5 1 32 ", b" 5 513 32 "),
            payload.replace(b" 5 1 32 ", b" 5 1 257 "),
            payload.replace(b" 5 1 32 ", b" 05 1 32 "),
            payload.replace(b"a" * 64, b"x" * 64),
            payload.replace(b"7 1000 1 1", b"7 3000 1 1"),
            payload.replace(b"7 1000 1 1", b"7 1000 2 1"),
            payload.replace(b"7 1000 1 1", b"7 1000 1 16"),
            payload.replace(b"7 1000 1 1", b"4294967296 1000 1 1"),
            payload + b"\x80", b"", b"x" * 8191,
        ):
            with self.subTest(bad=bad[:100]):
                with self.assertRaises(ValueError):
                    OBSERVE.read_config(bad)

    def test_parser_checks_optional_read_config_budget_identity(self):
        for key in ("max_events", "max_objects", "prefix_bytes"):
            expected = {
                "target_sha256": "a" * 64, "client_sha256": "b" * 64,
                "config_sha256": "c" * 64, key: 999,
            }
            with self.subTest(key=key):
                with self.assertRaisesRegex(ValueError, "budget mismatch"):
                    self.parse(self.records(), expected)

    def test_schema_two_entry_backend_has_no_wrapped_or_return_claim(self):
        records = self.records()
        records = records[:3] + records[-1:]
        records[0].update(schema=2, backend="drmgr-entry", client_version=5, supports_return=False)
        records[1].pop("wrapped")
        records[1].update(post=0, selected=True, instrumented=True)
        records[-1].update(events=1)
        parsed = self.parse(records)
        self.assertEqual(parsed["header"]["backend"], "drmgr-entry")
        self.assertNotIn("wrapped", parsed["sites"][0])
        for change in ({"wrapped": True}, {"instrumented": False}, {"post": 1}):
            altered = copy.deepcopy(records)
            altered[1].update(change)
            with self.subTest(change=change), self.assertRaises(ValueError):
                self.parse(altered)
        empty = copy.deepcopy(records[:2] + records[-1:])
        empty[1]["instrumented"] = False
        empty[-1].update(events=0, unique_objects=0, unreadable_reads=0)
        self.assertEqual(self.parse(empty)["events"], [])
        wrong = copy.deepcopy(records)
        wrong[0]["backend"] = "drwrap"
        with self.assertRaisesRegex(ValueError, "backend"):
            self.parse(wrong)

    def test_rejects_noncode_duplicate_and_bad_masks(self):
        for sites in (
            self.sites * 2,
            [{"id": 7, "rva": 0x200, "post": 0, "mask": 1}],
            [{"id": 7, "rva": 0x1000, "post": 0, "mask": 16}],
        ):
            with self.subTest(sites=sites):
                with self.assertRaises(ValueError):
                    OBSERVE.config_bytes(self.identity, "b" * 64, sites)

    def test_late_attach_has_no_application_tail_or_pidfile(self):
        profile = {"drrun_path": r"C:\public tools\drrun.exe",
                   "client_path": r"C:\owned tools\observer.dll"}
        argv = OBSERVE.launch_argv(
            profile, "config", "events", "c" * 64, "pidfile", attach_pid=123)
        self.assertEqual(argv[argv.index("-attach") + 1], "123")
        self.assertNotIn("--", argv)
        self.assertNotIn("-pidfile", argv)
        self.assertEqual(argv[-5:], [
            "-c", profile["client_path"], "config", "events", "c" * 64])
        with self.assertRaises(ValueError):
            OBSERVE.launch_argv(profile, "cfg", "log", "c" * 64, "pid",
                                application="exe", attach_pid=123)


if __name__ == "__main__":
    unittest.main()
