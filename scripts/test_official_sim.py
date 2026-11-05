"""Offline tests of native evidence admission and safe launch construction."""

import copy
import importlib.util
import json
from pathlib import Path
import os
import shutil
import sys
import unittest
from unittest.mock import patch
import uuid


sys.path.insert(0, str(Path(__file__).parent))


def load(name, file):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(file))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


REPLAY = load("official_sim", "official-sim.py")
LAUNCH = load("official_sim_launcher", "official_sim_runner.py")


class EvidenceTests(unittest.TestCase):
    def setUp(self):
        self.case = {"wait_seconds": 70, "expected_outputs": {"after_first_calculation": {"OL1": 1}}}
        self.remote = {
            "native_running_verified": True, "error": None, "cleanup": {"status": "closed"},
            "observations": [{
                "phase": "after_first_calculation", "elapsed_ms": 70000, "outputs": {"OL1": 1},
                "source": {"kind": "ocr", "artifacts": ["actual.png", "actual.ocr.json"]},
            }],
        }

    def test_positive_actual_passes_but_not_default_off(self):
        self.assertEqual(REPLAY.judge(self.case, self.remote), ("pass", None))
        self.remote["observations"][0]["outputs"] = {}
        self.assertEqual(REPLAY.judge(self.case, self.remote)[0], "blocked")

    def test_actual_mismatch_is_failure_not_missing_readback(self):
        self.remote["observations"][0]["outputs"]["OL1"] = 0
        self.assertEqual(REPLAY.judge(self.case, self.remote)[0], "fail")

    def test_output_is_not_execution_proof(self):
        for value in (False, None, "true", 1):
            with self.subTest(value=value):
                self.remote["native_running_verified"] = value
                self.assertEqual(REPLAY.judge(self.case, self.remote)[0], "blocked")

    def test_missing_source_and_boolean_pseudovalues_are_blocked(self):
        for change in (
            {"source": {"kind": "mock", "artifacts": ["actual.png"]}},
            {"source": {"kind": "ocr", "artifacts": []}},
            {"outputs": {"OL1": True}},
        ):
            record = copy.deepcopy(self.remote)
            record["observations"][0].update(change)
            self.assertEqual(REPLAY.judge(self.case, record)[0], "blocked")

    def test_duplicate_phase_and_missing_cleanup_are_blocked(self):
        record = copy.deepcopy(self.remote)
        record["observations"].append(copy.deepcopy(record["observations"][0]))
        self.assertEqual(REPLAY.judge(self.case, record)[0], "blocked")
        self.remote["cleanup"]["status"] = "blocked"
        self.assertEqual(REPLAY.judge(self.case, self.remote)[0], "blocked")

    def test_positive_state_before_calculation_interval_is_not_a_pass(self):
        self.remote["observations"][0]["elapsed_ms"] = 1000
        self.assertEqual(REPLAY.judge(self.case, self.remote)[0], "blocked")

    def test_operator_blocker_is_not_hidden_by_missing_native_progress(self):
        self.remote["native_running_verified"] = False
        self.remote["error"] = {"message": "Higher-integrity input owner; operator action required."}
        self.assertEqual(REPLAY.judge(self.case, self.remote),
                         ("blocked", "Higher-integrity input owner; operator action required."))


class LaunchTests(unittest.TestCase):
    def request(self):
        return {
            "mode": "drcov", "application": {"path": "owned.exe", "sha256": "a" * 64},
            "fixture_sha256": "b" * 64, "timeout_seconds": 300,
            "instrumentation": {
                **{prefix + "_path": prefix for prefix in ("drrun", "client", "runtime", "tool_config")},
                **{prefix + "_sha256": "c" * 64 for prefix in ("drrun", "client", "runtime", "tool_config")},
                "args": ["-t", "drcov", "-dump_binary"], "logdir_flag": "-logdir",
                "pidfile_flag": "-pidfile", "time_limit_flag": "-s",
            },
        }

    def test_verified_drcov_interface_order_and_pins(self):
        with patch.object(LAUNCH, "pinned", side_effect=lambda path, sha: str(path)) as pins:
            argv = LAUNCH.launcher_argv(self.request(), Path("owned-run"))
        self.assertEqual(pins.call_count, 6)
        self.assertLess(argv.index("-pidfile"), argv.index("-t"))
        self.assertLess(argv.index("-s"), argv.index("-t"))
        self.assertLess(argv.index("-logdir"), argv.index("--"))
        self.assertEqual(argv[-2:], ["owned.exe", str(Path("owned-run") / "input.Loxone")])

    def test_rejects_unreviewed_launcher_options(self):
        request = self.request()
        request["instrumentation"]["args"].append("-arbitrary")
        with patch.object(LAUNCH, "pinned", return_value="owned"):
            with self.assertRaises(ValueError):
                LAUNCH.launcher_argv(request, Path("owned-run"))

    def test_new_runs_launch_the_unique_byte_pinned_fixture_basename(self):
        request = self.request()
        request.update(run_id="native-run-a", fixture_filename="input-native-run-a.Loxone", mode="plain")
        with patch.object(LAUNCH, "pinned", side_effect=lambda path, sha: str(path)) as pins:
            argv = LAUNCH.launcher_argv(request, Path("owned-run"))
        self.assertEqual(argv[-1], str(Path("owned-run") / "input-native-run-a.Loxone"))
        self.assertEqual(pins.call_args_list[-1].args[1], request["fixture_sha256"])
        for value in ("input.Loxone", r"..\escape.Loxone", r"C:\escape.Loxone",
                      "input:stream.Loxone", "CON.Loxone", "file?.Loxone", "../escape.Loxone", ""):
            with self.subTest(value=value):
                with self.assertRaises(ValueError):
                    LAUNCH.fixture_basename({"fixture_filename": value})
        with self.assertRaises(ValueError):
            LAUNCH.fixture_basename({})
        self.assertEqual(LAUNCH.fixture_basename({}, allow_legacy=True), "input.Loxone")

    def test_child_environment_is_whitelisted_not_secret_filtered(self):
        root = Path(__file__).resolve().parents[1] / ".reverse-engineering" / "native-simulator" / ("launch-test-" + uuid.uuid4().hex)
        root.mkdir(parents=True)
        request = self.request()
        request.update(mode="plain", run_dir=str(root))
        environment = {"SystemRoot": "Windows", "GH_TOKEN": "secret", "WINDOWS_MCP_AUTH_KEY": "secret", "NEW_API_KEY": "secret"}
        try:
            with patch.dict(os.environ, environment, clear=True), \
                    patch.object(LAUNCH, "pinned", side_effect=lambda path, sha: str(path)), \
                    patch.object(LAUNCH, "image_path", return_value="owned.exe"), \
                    patch.object(LAUNCH, "creation_ticks", return_value=12345), \
                    patch.object(LAUNCH.subprocess, "Popen") as process:
                process.return_value.pid = 123
                LAUNCH.launch(request)
            self.assertEqual(process.call_count, 2)
            self.assertEqual(process.call_args_list[0].kwargs["creationflags"], 0)
            for call in process.call_args_list:
                self.assertEqual({key.upper(): value for key, value in call.kwargs["env"].items()}, {"SYSTEMROOT": "Windows"})
                self.assertTrue(call.kwargs["close_fds"])
        finally:
            shutil.rmtree(root)

    def test_transfer_refuses_option_injection(self):
        profile = {"host": "-evil", "port": 2222, "user": "owned", "identity_file": "key", "known_hosts_file": "hosts", "desktop_host": "owned-notebook"}
        with self.assertRaises(ValueError):
            REPLAY.Transfer(profile)

    def test_post_init_owns_existing_pid_not_a_second_vendor_launch(self):
        request = self.request()
        request["instrumentation_start"] = "post-init"
        with patch.object(LAUNCH, "pinned", side_effect=lambda path, sha: str(path)):
            bootstrap = LAUNCH.launcher_argv(request, Path("owned-run"))
            attach = LAUNCH.attach_argv(request, Path("owned-run"), 4321)
        self.assertEqual(bootstrap, ["owned.exe", str(Path("owned-run") / "input.Loxone")])
        self.assertEqual(attach[attach.index("-attach") + 1], "4321")
        self.assertNotIn("--", attach)
        self.assertNotIn("owned.exe", attach)
        self.assertLess(attach.index("-attach"), attach.index("-t"))

    def test_exit_dump_uses_only_the_verified_pid_specific_method(self):
        request = self.request()
        request["exit_dump_method"] = {
            "schema_version": 1, "kind": "dynamorio-drcov-exit-dump-profile",
            "console_runtime_verified": True, "independent_timeout_seconds": 8,
            "effect": "process_exit_and_drcov_dump_not_live_snapshot",
            "drconfig_path": "pinned-drconfig.exe", "drconfig_sha256": "a" * 64,
            "args_template": ["-nudge_timeout", "5000", "-nudge_pid", "{verified_owned_target_pid}", "00000000", "1"],
        }
        with patch.object(LAUNCH, "pinned", side_effect=lambda path, sha: str(path)):
            self.assertEqual(LAUNCH.exit_dump_argv(request, 4321),
                             ["pinned-drconfig.exe", "-nudge_timeout", "5000", "-nudge_pid", "4321", "00000000", "1"])
            request["exit_dump_method"]["args_template"][-1] = "2"
            with self.assertRaises(ValueError):
                LAUNCH.exit_dump_argv(request, 4321)

    def test_attach_deadline_terminates_only_its_owned_attacher(self):
        root = Path(__file__).resolve().parents[1] / ".reverse-engineering" / "native-simulator" / ("attach-test-" + uuid.uuid4().hex)
        root.mkdir(parents=True)
        request = self.request()
        request.update(instrumentation_start="post-init", run_dir=str(root))
        (root / "launched.json").write_text(json.dumps({
            "target_pid": 4321, "target_creation_ticks": 111, "launch_epoch": 100.0,
        }))
        try:
            with patch.object(LAUNCH, "pinned", side_effect=lambda path, sha: str(path)), \
                    patch.object(LAUNCH, "image_path", return_value="owned.exe"), \
                    patch.object(LAUNCH, "creation_ticks", side_effect=lambda pid: 111 if pid == 4321 else 222), \
                    patch.object(LAUNCH, "interactive_session", return_value=1), \
                    patch.object(LAUNCH, "start_watchdog") as watchdog, \
                    patch.object(LAUNCH.subprocess, "Popen") as process, \
                    patch.object(LAUNCH.time, "time", return_value=100.0), \
                    patch.object(LAUNCH.time, "monotonic", side_effect=(100.0, 131.0)):
                process.return_value.pid = 123
                process.return_value.poll.return_value = None
                actual = LAUNCH.attach_owned(request)
            self.assertEqual(actual["status"], "blocked")
            self.assertIn("deadline", actual["reason"])
            process.return_value.terminate.assert_called_once_with()
            watchdog.assert_called_once()
            self.assertEqual(json.loads((root / "attached.json").read_text())["target_pid"], 4321)
        finally:
            shutil.rmtree(root)


class ReceiptTests(unittest.TestCase):
    def test_post_attach_clock_proof_requires_fresh_separated_actual_reads(self):
        samples = [
            {"observer_mode": "native-win32", "native_identity_geometry_verified": True,
             "readiness_mode": "direct-post-attach-clock-ocr", "clock_capture_unix_ms": 100000},
            {"observer_mode": "native-win32", "native_identity_geometry_verified": True,
             "readiness_mode": "direct-post-attach-clock-ocr", "clock_capture_unix_ms": 104000},
        ]
        readings = [
            {"text": "12:00:00", "confidence": 95, "seconds_since_midnight": 43200},
            {"text": "12:00:04", "confidence": 96, "seconds_since_midnight": 43204},
        ]
        self.assertEqual(REPLAY.validate_post_attach_clock_pair(samples, readings, 99)["native_clock_delta_seconds"], 4)
        for sample_change, reading_change in (
            ({"clock_capture_unix_ms": 99000}, {}),
            ({"clock_capture_unix_ms": 102000}, {}),
            ({}, {"text": "12:00:00", "seconds_since_midnight": 43200}),
            ({}, {"confidence": 86}),
        ):
            actual_samples, actual_readings = copy.deepcopy(samples), copy.deepcopy(readings)
            actual_samples[1].update(sample_change)
            actual_readings[1].update(reading_change)
            with self.subTest(sample=sample_change, reading=reading_change):
                with self.assertRaises(ValueError):
                    REPLAY.validate_post_attach_clock_pair(actual_samples, actual_readings, 99)

    def test_failed_attachment_without_record_stays_structured_blocker(self):
        original = {"execution": {"observer_mode": "native-win32", "instrumentation_start": "post-start"}}
        report = {"pre_attach_running_verified": True, "post_attach_clock_proof_pending": True, "attachment": None}
        with self.assertRaisesRegex(ValueError, "attachment was not initialized"):
            REPLAY.prove_native_clock_progress(original, report, Path("unused"), {}, {}, Path("unused"), Path("unused"))

    def test_native_observer_readiness_has_no_wm_null_or_uia_proxy(self):
        source = Path(__file__).with_name("official-sim.ps1").read_text()
        observer = source.split("function Assert-OracleNativeObserver {", 1)[1].split("\nfunction ", 1)[0]
        self.assertNotIn("::Ping(", observer)
        self.assertNotIn("AutomationElement", observer)
        self.assertNotIn("Get-OracleControl", observer)

    def test_missing_native_report_is_persisted_as_blocked_not_lost(self):
        root = Path(__file__).resolve().parents[1]
        run = root / ".reverse-engineering" / "native-simulator" / "replay" / ("test-" + uuid.uuid4().hex)
        (run / "remote").mkdir(parents=True)
        try:
            definition = {"id": "authored-case", "application": {"sha256": "a" * 64}, "fixture_sha256": "b" * 64}
            REPLAY.write_json(run / "case.definition.json", definition)
            (run / "input.Loxone").write_bytes(b"authored fixture bytes")
            output, code = REPLAY.finalize(run, root, definition, {"mode": "drcov", "run_dir": "REMOTE"})
            receipt = json.loads((root / output["receipt"]).read_text())
            self.assertEqual(code, 2)
            self.assertEqual(receipt["execution"]["status"], "blocked")
            self.assertFalse(receipt["execution"]["native_running_verified"])
            self.assertEqual(receipt["observations"], [])
            self.assertEqual(receipt["case"]["sha256"], receipt["case"]["definition"]["sha256"])
            self.assertTrue(all(not Path(item["path"]).is_absolute() for item in receipt["artifacts"]))
        finally:
            shutil.rmtree(run)

    def test_output_rectangles_follow_actual_header_not_desktop_coordinates(self):
        header = {"header": {"text": "Energy Manager", "words": [
            {"rect": [1000, 300, 44, 14]}, {"rect": [1049, 301, 58, 13]},
        ]}}
        with patch.object(REPLAY, "read_json", return_value=header):
            boxes = REPLAY.native_state_boxes(Path("unused"))
        self.assertEqual(boxes["pin"], (1256, 338, 1281, 355))
        self.assertEqual(boxes["wired_reference"], (1299, 338, 1326, 355))
        with patch.object(REPLAY, "read_json", return_value=header):
            with self.assertRaises(ValueError):
                REPLAY.native_state_boxes(Path("unused"), (1300, 400))
        header["header"]["words"][1]["rect"][2] += 1
        with patch.object(REPLAY, "read_json", return_value=header):
            with self.assertRaises(ValueError):
                REPLAY.native_state_boxes(Path("unused"))

    def test_canvas_origin_is_bound_to_inset_owned_and_virtual_bounds(self):
        capture = {
            "scope": "owned-native-programming-canvas", "rectangle": [60, 290, 1848, 676],
            "origin_screen": [60, 290], "root_handle": 123, "inset_pixels": 12,
            "virtual_screen_bounds": [0, 0, 1920, 1080],
            "owned_canvas": {
                "root_handle": 123, "native_handle": 456, "bounds": [48, 278, 1920, 978],
                "main_bounds": [0, 0, 1920, 1008],
            },
        }
        REPLAY.validate_canvas_capture(capture)
        for change in (
            {"origin_screen": [0, 0]}, {"rectangle": [60, 290, 1870, 676]},
            {"virtual_screen_bounds": [0, 0, 1900, 1080]}, {"root_handle": 999},
        ):
            with self.subTest(change=change):
                invalid = copy.deepcopy(capture)
                invalid.update(change)
                with self.assertRaises(ValueError):
                    REPLAY.validate_canvas_capture(invalid)

    def test_header_proposals_retain_uncorrected_geometry_not_an_admitted_caption(self):
        line = {"text": "Energy Mar-eger", "words": [
            {"text": "Energy", "rect": [391, 349, 44, 14]},
            {"text": "Mar-eger", "rect": [440, 349, 58, 14]},
        ]}
        proposals = REPLAY.native_header_proposals({"lines": [line]})
        self.assertEqual(proposals, [{"raw_windows_ocr": line, "box": [391, 349, 498, 363]}])
        self.assertEqual(proposals[0]["raw_windows_ocr"]["words"][1]["text"], "Mar-eger")
        self.assertNotIn("recognized_header", proposals[0])

    def test_header_anchor_maps_actual_tsv_boxes_not_the_fuzzy_proposal(self):
        recognized = {
            "text": "Energy Manager", "source": {"box": [10, 20, 125, 42], "image_size": [200, 100]},
            "words": [
                {"text": "Energy", "confidence": 95.6, "box": [24, 24, 180, 56]},
                {"text": "Manager", "confidence": 96.6, "box": [224, 24, 228, 56]},
            ],
        }
        anchor = REPLAY.header_anchor_from_recognition(recognized)
        self.assertEqual([word["rect"] for word in anchor["words"]], [[10, 20, 45, 14], [60, 20, 57, 14]])
        recognized["words"][1]["confidence"] = 86
        with self.assertRaises(ValueError):
            REPLAY.header_anchor_from_recognition(recognized)


if __name__ == "__main__":
    unittest.main()
