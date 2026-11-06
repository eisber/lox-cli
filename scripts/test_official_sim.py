"""Offline tests of native evidence admission and safe launch construction."""

import copy
import importlib.util
import json
from pathlib import Path
import os
import shutil
import struct
import subprocess
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

    def test_named_multichannel_windows_require_every_actual_channel(self):
        case = {
            "channels": ["OL1", "OL2", "OL3"],
            "sampling_phases": [
                {"name": "before_deadline", "at_seconds": 20, "latest_seconds": 40,
                 "simulator_time_window": ["10:58:20", "10:58:40"]},
                {"name": "after_deadline", "at_seconds": 150, "latest_seconds": 175},
            ],
            "expected_outputs": {
                "before_deadline": {"OL1": 0, "OL2": 1, "OL3": 0},
                "after_deadline": {"OL1": 1, "OL2": 0, "OL3": 0},
            },
        }
        remote = copy.deepcopy(self.remote)
        remote["observations"] = [
            {"phase": phase, "elapsed_ms": elapsed, "simulator_time": clock,
             "capture_interval_ms": [elapsed - 100, elapsed + 100],
             "outputs": outputs, "source": {"kind": "ocr", "artifacts": ["actual.png"]}}
            for phase, elapsed, clock, outputs in (
                ("before_deadline", 25000, "10:58:25", {"OL1": 0, "OL2": 1, "OL3": 0}),
                ("after_deadline", 155000, "11:00:35", {"OL1": 1, "OL2": 0, "OL3": 0}),
            )
        ]
        self.assertEqual(REPLAY.judge(case, remote), ("pass", None))
        for change in (
            {"elapsed_ms": 41000}, {"capture_interval_ms": [24000, 41000]},
            {"capture_interval_ms": None},
            {"simulator_time": None}, {"simulator_time": "10:59:25"},
            {"simulator_time_after_capture": "10:59:00"},
            {"outputs": {"OL1": 0, "OL2": 1}}, {"outputs": {"OL1": 0, "OL2": True, "OL3": 0}},
        ):
            with self.subTest(change=change):
                invalid = copy.deepcopy(remote)
                invalid["observations"][0].update(change)
                self.assertEqual(REPLAY.judge(case, invalid)[0], "blocked")
        remote["observations"][1]["outputs"]["OL2"] = 1
        self.assertEqual(REPLAY.judge(case, remote)[0], "fail")

    def test_exploratory_actuals_are_retained_without_becoming_a_pass(self):
        case = copy.deepcopy(self.case)
        case.update(purpose="exploratory", expected_outputs={})
        status, reason = REPLAY.judge(case, self.remote)
        self.assertEqual(status, "blocked")
        self.assertIn("Exploratory", reason)
        self.assertEqual(self.remote["observations"][0]["outputs"], {"OL1": 1})


class CaseContractTests(unittest.TestCase):
    def test_public_reviewed_protocols_refuse_missing_private_calibration(self):
        root = Path(__file__).resolve().parents[1]
        empty = root / ".reverse-engineering" / "native-simulator" / ("portable-test-" + uuid.uuid4().hex)
        empty.mkdir(parents=True)
        try:
            for name in ("energy-multi-original-reviewed.json", "energy-multi-third-first-reviewed.json",
                         "energy-daily-observer-baseline.json"):
                with self.subTest(protocol=name):
                    case = REPLAY.read_json(root / "tests" / "ux" / "native-simulator" / name)
                    with patch.object(REPLAY, "digest", return_value=case["fixture_sha256"]):
                        REPLAY.validate_case(case, Path("separately-provided-fixture"))
                    with self.assertRaises(FileNotFoundError):
                        REPLAY.validate_readback_calibration(case, empty)
        finally:
            shutil.rmtree(empty)

    @unittest.skipUnless(shutil.which("powershell"), "Windows PowerShell is unavailable")
    def test_clock_section_navigation_requires_actual_field_specific_readback(self):
        source = str(Path(__file__).with_name("official-sim.ps1")).replace("'", "''")
        command = f". '{source}';" + r"""
$script:OracleRoot=(Get-Location).Path
$script:parts=@(23,59,59);$script:section=1;$script:position=3;$script:wrongHome=$false
function Click-Ux($Clock) {}
function Assert-OracleClockFocus($Clock) {}
function Write-OracleJson($Path,$Value) {}
function Get-UxValue($Clock) { '{0:D2}:{1:D2}:{2:D2}' -f $script:parts[0],$script:parts[1],$script:parts[2] }
function Send-UxKeys([string]$Value) {
    if($Value -ceq '{HOME}'){$script:position=$(if($script:wrongHome){3}else{0});$script:section=[int][Math]::Floor($script:position/3);return}
    if($Value -match '^(?:\{RIGHT\})+$'){
        $script:position += [regex]::Matches($Value,'\{RIGHT\}').Count
        $script:section=[int][Math]::Floor($script:position/3);return
    }
    foreach($match in [regex]::Matches($Value,'\{(UP|DOWN)\}')){
        $mod=if($script:section -eq 0){24}else{60}
        $delta=if($match.Groups[1].Value -ceq 'UP'){1}else{-1}
        $script:parts[$script:section]=($script:parts[$script:section]+$delta+$mod)%$mod
    }
}
$actual=Set-OracleClockSections $null '10:58:00'
if($actual -cne '10:58:00'){throw 'Verified section editing failed'}
$script:parts=@(23,59,59);$script:wrongHome=$true
try { Set-OracleClockSections $null '10:58:00';throw 'Wrong section was not rejected' }
catch {if($_.Exception.Message -cne 'Native clock keyboard section probe did not change exactly the intended field.'){throw}}
exit 0
"""
        result = subprocess.run(["powershell", "-NoProfile", "-NonInteractive", "-Command", command],
                                capture_output=True, timeout=30)
        self.assertEqual(result.returncode, 0, (result.stderr + result.stdout).decode(errors="replace"))

    @unittest.skipUnless(shutil.which("powershell"), "Windows PowerShell is unavailable")
    def test_remote_sampling_preserves_single_phase_array_and_legacy_fallback(self):
        source = str(Path(__file__).with_name("official-sim.ps1")).replace("'", "''")
        command = (
            f". '{source}';"
            "$a=Get-OracleSamplingPlan ('{\"sampling_phases\":[{\"name\":\"settled\",\"at_seconds\":70,\"latest_seconds\":95}]}'|ConvertFrom-Json);"
            "$b=Get-OracleSamplingPlan ('{\"wait_seconds\":70}'|ConvertFrom-Json);"
            "if($a.Count -ne 1 -or $a[0].name -cne 'settled' -or "
            "$b.Count -ne 1 -or $b[0].name -cne 'after_first_calculation'){throw 'Single native phase was unwrapped'}"
        )
        result = subprocess.run(["powershell", "-NoProfile", "-NonInteractive", "-Command", command],
                                capture_output=True, timeout=30)
        self.assertEqual(result.returncode, 0, result.stderr.decode(errors="replace"))

    @unittest.skipUnless(shutil.which("powershell"), "Windows PowerShell is unavailable")
    def test_remote_extended_window_never_exceeds_two_hundred_ten_seconds(self):
        source = str(Path(__file__).with_name("official-sim.ps1")).replace("'", "''")
        command = f". '{source}';" + r"""
$valid=Get-OracleSamplingPlan ('{"sampling_phases":[{"name":"post_minrun","at_seconds":200,"latest_seconds":210}]}'|ConvertFrom-Json)
if($valid.Count -ne 1 -or $valid[0].at_seconds -ne 200){throw 'Valid bounded extension was rejected'}
foreach($json in @(
'{"sampling_phases":[{"name":"late","at_seconds":201,"latest_seconds":210}]}',
'{"sampling_phases":[{"name":"late","at_seconds":200,"latest_seconds":211}]}',
'{"sampling_phases":[{"name":"late","at_seconds":200,"latest_seconds":200}]}',
'{"sampling_phases":[{"name":"late","at_seconds":199.5,"latest_seconds":210}]}',
'{"sampling_phases":[],"wait_seconds":70}'
)){
    $rejected=$false
    try { [void](Get-OracleSamplingPlan ($json|ConvertFrom-Json)) } catch {$rejected=$true}
    if(!$rejected){throw 'Unbounded/invalid phase was accepted'}
}
exit 0
"""
        result = subprocess.run(["powershell", "-NoProfile", "-NonInteractive", "-Command", command],
                                capture_output=True, timeout=30)
        self.assertEqual(result.returncode, 0, (result.stderr + result.stdout).decode(errors="replace"))

    def test_extended_window_keeps_phase_count_end_and_order_bounded(self):
        phases = [
            {"name": "late_minrun", "at_seconds": 180, "latest_seconds": 200},
            {"name": "post_minrun", "at_seconds": 200, "latest_seconds": 210},
        ]
        self.assertEqual(LAUNCH.sampling_plan({"sampling_phases": phases}), phases)
        for change in (
            {"at_seconds": 201}, {"latest_seconds": 211}, {"latest_seconds": 200},
            {"at_seconds": 199.5}, {"at_seconds": True},
        ):
            invalid = copy.deepcopy(phases)
            invalid[-1].update(change)
            with self.subTest(change=change), self.assertRaises(ValueError):
                LAUNCH.sampling_plan({"sampling_phases": invalid})

    def test_expanded_case_requires_separate_complete_expectations(self):
        case = {
            "schema_version": 1, "id": "authored-timing",
            "application": {"sha256": REPLAY.APPLICATION_SHA}, "fixture_sha256": "a" * 64,
            "channels": ["OL1", "OL2"],
            "sampling_phases": [{"name": "settled", "at_seconds": 70, "latest_seconds": 90}],
            "expected_outputs": {"settled": {"OL1": 1, "OL2": 0}},
        }
        with patch.object(REPLAY, "digest", return_value="a" * 64):
            self.assertEqual(REPLAY.validate_case(case, Path("authored")), case)
            for change in (
                {"expected_outputs": {"settled": {"OL1": 1}}},
                {"expected_outputs": {"another_phase": {"OL1": 1, "OL2": 0}}},
                {"channels": ["OL1", "OL1"]}, {"channels": ["OL4"]},
                {"simulator_start_time": "25:00:00"}, {"simulator_time_edit": "unsupported"},
                {"purpose": "exploratory"},
            ):
                with self.subTest(change=change), self.assertRaises(ValueError):
                    REPLAY.validate_case({**case, **change}, Path("authored"))
            exploratory = {**case, "purpose": "exploratory", "expected_outputs": {}}
            REPLAY.validate_case(exploratory, Path("authored"))

    def test_phase_plan_rejects_unbounded_ambiguous_and_overlapping_windows(self):
        phase = {"name": "settled", "at_seconds": 70, "latest_seconds": 90}
        self.assertEqual(LAUNCH.sampling_plan({"wait_seconds": 70}),
                         [{"name": "after_first_calculation", "at_seconds": 70}])
        for phases in (
            [], [phase] * 2, [phase] * 7,
            [{**phase, "name": "..\\escape"}], [{**phase, "name": "initial_running"}],
            [{**phase, "at_seconds": 10}], [{**phase, "at_seconds": True}],
            [{**phase, "latest_seconds": 500}], [{**phase, "latest_seconds": 70}],
            [phase, {"name": "later", "at_seconds": 80, "latest_seconds": 100}],
            [{**phase, "simulator_time_window": ["11:01:00", "11:00:00"]}],
        ):
            with self.subTest(phases=phases), self.assertRaises(ValueError):
                LAUNCH.sampling_plan({"sampling_phases": phases})

    def test_extra_channels_require_distinct_calibrated_reference_pixels(self):
        with self.assertRaises(ValueError):
            REPLAY.channel_offsets({"channels": ["OL1", "OL2"]})
        boxes = {
            "OL1": {"pin": [10, 10, 35, 27], "wired_reference": [40, 10, 65, 27]},
            "OL2": {"pin": [10, 40, 35, 57], "wired_reference": [40, 40, 65, 57]},
        }
        case = {"channels": ["OL1", "OL2"], "readback": {"channel_boxes": boxes}}
        self.assertEqual(REPLAY.channel_offsets(case), boxes)
        with self.assertRaises(ValueError):
            REPLAY.validate_readback_calibration(case, Path.cwd())
        boxes["OL2"]["pin"] = boxes["OL1"]["pin"]
        with self.assertRaises(ValueError):
            REPLAY.channel_offsets(case)
        boxes["OL2"]["pin"] = [10, 25, 35, 42]
        self.assertEqual(REPLAY.channel_offsets(case), boxes)
        boxes["OL2"]["wired_reference"] = [20, 30, 45, 47]
        with self.assertRaises(ValueError):
            REPLAY.channel_offsets(case)

    def test_decoder_observes_each_channel_without_receiving_expectations(self):
        import official_sim_ocr
        root = Path(__file__).resolve().parents[1]
        run = root / ".reverse-engineering" / "native-simulator" / ("decode-test-" + uuid.uuid4().hex)
        run.mkdir(parents=True)
        try:
            image = run / "authored.png"
            image.write_bytes(b"\x89PNG\r\n\x1a\n" + b"\0\0\0\rIHDR" + struct.pack(">II", 700, 500))
            anchor = run / "anchor.json"
            REPLAY.write_json(anchor, {"header": {"text": "Energy Manager", "words": [
                {"rect": [10, 10, 44, 14]}, {"rect": [59, 10, 58, 14]},
            ]}})
            record = run / "authored-read.json"
            REPLAY.write_json(record, {"synthetic": True})
            case = {
                "channels": ["OL1", "OL2"],
                "readback": {"channel_boxes": {
                    "OL1": {"pin": [10, 30, 35, 47], "wired_reference": [40, 30, 65, 47]},
                    "OL2": {"pin": [10, 60, 35, 77], "wired_reference": [40, 60, 65, 77]},
                }},
                "expected_outputs": {"unused": {"OL1": 0, "OL2": 1}},
            }
            reads = [
                {"text": word, "value": value, "confidence": confidence, "record_path": str(record)}
                for word, value, confidence in (("Ein", 1, 96), ("Ein", 1, 95), ("Aus", 0, 94), ("Aus", 0, 91))
            ]
            profile = {"executable_sha256": "a" * 64, "model_sha256": "b" * 64}
            with patch.object(official_sim_ocr, "recognize_literal", side_effect=reads) as recognize:
                actual = REPLAY.decode_native_image(image, anchor, run / "outputs", profile, root, case)
            self.assertEqual(actual["outputs"], {"OL1": 1, "OL2": 0})
            self.assertEqual(recognize.call_count, 4)
            self.assertTrue(all(call.args[2] is profile for call in recognize.call_args_list))
            reads[-1]["confidence"] = 89.99
            with patch.object(official_sim_ocr, "recognize_literal", side_effect=reads):
                actual = REPLAY.decode_native_image(image, anchor, run / "low-confidence", profile, root, case)
            self.assertEqual(actual["outputs"], {"OL1": 1})
        finally:
            shutil.rmtree(run)

    def test_explicit_calibration_binds_actual_image_anchor_and_in_bounds_rows(self):
        root = Path(__file__).resolve().parents[1]
        run = root / ".reverse-engineering" / "native-simulator" / ("calibration-test-" + uuid.uuid4().hex)
        run.mkdir(parents=True)
        try:
            image = run / "authored.png"
            image.write_bytes(b"\x89PNG\r\n\x1a\n" + b"\0\0\0\rIHDR" + struct.pack(">II", 700, 500))
            anchor = run / "anchor.json"
            REPLAY.write_json(anchor, {"header": {"text": "Energy Manager", "words": [
                {"rect": [10, 10, 44, 14]}, {"rect": [59, 10, 58, 14]},
            ]}})
            case = {"channels": ["OL1"], "readback": {
                "channel_boxes": {"OL1": {"pin": [10, 30, 35, 47], "wired_reference": [40, 30, 65, 47]}},
                "calibration": {"image": REPLAY.artifact(image, root), "header_anchor": REPLAY.artifact(anchor, root),
                                "review": "Synthetic protocol test only; not native evidence."},
            }}
            self.assertEqual(REPLAY.validate_readback_calibration(case, root), [image, anchor])
            review = run / "review.json"
            REPLAY.write_json(review, {"synthetic": True})
            case["readback"]["calibration"]["evidence"] = [REPLAY.artifact(review, root)]
            self.assertEqual(REPLAY.validate_readback_calibration(case, root), [image, anchor, review])
            review.write_text("changed", encoding="utf-8")
            with self.assertRaises(ValueError):
                REPLAY.validate_readback_calibration(case, root)
            case["readback"]["calibration"]["evidence"] = []
            case["readback"]["channel_boxes"]["OL1"]["pin"] = [800, 30, 825, 47]
            with self.assertRaises(ValueError):
                REPLAY.validate_readback_calibration(case, root)
            image.write_bytes(image.read_bytes() + b"changed")
            with self.assertRaises(ValueError):
                REPLAY.validate_readback_calibration(case, root)
        finally:
            shutil.rmtree(run)


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

    def test_recovery_skip_requires_exact_opt_in_owned_question_and_no_button(self):
        request = self.request()
        request.update(recovery_policy="owned-skip", fixture_filename="input-independent-run.Loxone")
        record = {
            "window": {"Handle": 123, "ProcessId": 4321, "ClassName": "#32770"},
            "root_handle": 123, "already_handled": False,
            "identity": {"owned_pid": 4321, "session": 1, "actor_session": 1,
                         "application_sha256": request["application"]["sha256"],
                         "fixture_sha256": request["fixture_sha256"]},
            "native_text": [
                {"class": "Button", "id": 6, "text": "&Yes"},
                {"class": "Button", "id": 7, "text": "&No"},
                {"class": "Static", "id": 65535, "text":
                    "Es wurde eine automatische Sicherung dieses Dokuments gefunden:\n"
                    "input-independent-run.Loxone 2026-11-05 22:28:57\n\n"
                    "Soll diese wiederhergestellt werden?"},
            ],
        }
        accepted = LAUNCH.validate_recovery_dialog(request, record)
        self.assertEqual((accepted["action"], accepted["button_id"]), ("decline_restore", 7))
        for policy in (None, "block", "skip", True):
            with self.subTest(policy=policy), self.assertRaises(ValueError):
                LAUNCH.validate_recovery_dialog({**request, "recovery_policy": policy}, record)
        invalids = []
        for key, value in (("root_handle", 999), ("already_handled", True)):
            invalids.append({**copy.deepcopy(record), key: value})
        for key, value in (("owned_pid", 999), ("session", 0), ("actor_session", 2),
                           ("application_sha256", "d" * 64), ("fixture_sha256", "e" * 64)):
            altered = copy.deepcopy(record)
            altered["identity"][key] = value
            invalids.append(altered)
        for key, value in (("ClassName", "QMessageBox"), ("ProcessId", 999)):
            altered = copy.deepcopy(record)
            altered["window"][key] = value
            invalids.append(altered)
        for text in (
            record["native_text"][-1]["text"].replace("independent-run", "another-run"),
            record["native_text"][-1]["text"] + "\nAdditional question",
            "Restore this project?", "",
        ):
            altered = copy.deepcopy(record)
            altered["native_text"][-1]["text"] = text
            invalids.append(altered)
        for extra in (
            {"class": "Button", "id": 2, "text": "Cancel"},
            {"class": "Edit", "id": 4, "text": None},
            {"class": "Static", "id": 5, "text": "another project"},
        ):
            altered = copy.deepcopy(record)
            altered["native_text"].append(extra)
            invalids.append(altered)
        altered = copy.deepcopy(record)
        altered["native_text"][1]["id"] = 8
        invalids.append(altered)
        for altered in invalids:
            with self.subTest(altered=altered), self.assertRaises(ValueError):
                LAUNCH.validate_recovery_dialog(request, altered)

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
    def test_recognizer_snapshot_is_fresh_and_uses_its_retained_source_identity(self):
        root = Path(__file__).resolve().parents[1]
        run = root / ".reverse-engineering" / "native-simulator" / ("snapshot-test-" + uuid.uuid4().hex)
        run.mkdir(parents=True)
        try:
            snapshot = run / "ocr-helper-source.py"
            snapshot.write_text(
                "import hashlib\nfrom pathlib import Path\n"
                "def identity():\n"
                "    return hashlib.sha256(Path(__file__).read_bytes()).hexdigest()\n"
                "recognize_literal = identity\nrecognize_header = identity\nrecognize_clock = identity\n",
                encoding="utf-8",
            )
            with patch.dict(sys.modules, {"official_sim_ocr": object()}):
                first = REPLAY.load_ocr_snapshot(snapshot)
                second = REPLAY.load_ocr_snapshot(snapshot)
            self.assertIsNot(first, second)
            self.assertEqual(Path(first.__file__), snapshot)
            for name in ("recognize_literal", "recognize_header", "recognize_clock"):
                self.assertEqual(getattr(first, name)(), REPLAY.digest(snapshot))
        finally:
            shutil.rmtree(run)

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
