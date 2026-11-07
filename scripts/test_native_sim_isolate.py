import importlib.util
from pathlib import Path
import unittest


SPEC = importlib.util.spec_from_file_location(
    "native_sim_isolate", Path(__file__).with_name("native-sim-isolate.py"))
ISOLATE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ISOLATE)


class IsolationTests(unittest.TestCase):
    def output(self, **changes):
        lines = []
        for stage in ("ready", "attached", "triggered"):
            for role in ("main", "timer"):
                values = {
                    "thread": 12 if role == "main" else 13,
                    "teb": 0x10000, "array": 0x20000, "index": 0, "slot_read": 1,
                    "slot": 0x30000, "offset_known": 1, "offset": 16,
                    "marker_read": 1, "marker": 0x1122334455667788,
                    "dr_query": 1, "under_dr": 1, **changes,
                }
                lines.append(f"TLS1 stage={stage} role={role} " + " ".join(
                    f"{key}={value:x}" for key, value in values.items()))
        lines.append("fixture-v1 value=0a7fac99710cdf63 clock=0123456789abcdef calls=80 "
                     "return_rva=1234 fp_sentinels=1 tls=1 fp_calls=64")
        return ("\n".join(lines) + "\n").encode()

    def test_requires_real_tls_and_thread_control_not_successful_exit_alone(self):
        rows, returned = ISOLATE.evaluate_stdout(self.output(), "no-client")
        self.assertEqual(len(rows), 6)
        self.assertEqual(returned, 0x1234)
        for changes in ({"marker_read": 0}, {"marker": 0}, {"slot_read": 0}):
            with self.subTest(changes=changes), self.assertRaises(ValueError):
                ISOLATE.evaluate_stdout(self.output(**changes), "no-client")
        rows, _ = ISOLATE.evaluate_stdout(self.output(dr_query=0, under_dr=0), "no-client")
        self.assertFalse(ISOLATE.thread_control_witnessed(rows))
        self.assertTrue(ISOLATE.thread_control_witnessed(ISOLATE.tls_rows(self.output())))

    def test_rejects_partial_duplicate_and_wrong_caller_evidence(self):
        raw = self.output()
        for changed in (raw[:-1], raw + raw.splitlines(keepends=True)[0],
                        raw.replace(b"stage=attached role=timer", b"stage=ready role=timer")):
            with self.subTest(changed=changed[-30:]), self.assertRaises(ValueError):
                ISOLATE.evaluate_stdout(changed, "observer")
        with self.assertRaises(ValueError):
            ISOLATE.evaluate_stdout(raw, "observer", 0x5678)

    def test_layer_argv_keeps_owned_attach_distinct_from_launch(self):
        profile = {"drrun_path": r"C:\tools\drrun.exe"}
        clients = {"noop_path": r"C:\owned\noop.dll"}
        directory = Path("owned-directory")
        fixture = Path("observer_fixture.exe")
        argv = ISOLATE.layer_argv("noop", profile, clients, directory, fixture, pid=123)
        self.assertIn("-attach", argv)
        self.assertNotIn("--", argv)
        self.assertNotIn("-pidfile", argv)
        self.assertEqual(argv[-2:], ["-c", clients["noop_path"]])
        launch = ISOLATE.layer_argv("no-client", profile, clients, directory, fixture)
        self.assertIn("-pidfile", launch)
        self.assertNotIn("-c", launch)
        self.assertIn("--", launch)


if __name__ == "__main__":
    unittest.main()
