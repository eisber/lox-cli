"""Synthetic control-flow tests only; never opens a socket or desktop."""

import hashlib
import json
from pathlib import Path
import subprocess
import sys
import time
import unittest
from unittest.mock import patch

import notebook_case01 as CASE


PIN = "a" * 64


def synthetic_route(budget):
    pins = {name: hashlib.sha256(path.read_bytes()).hexdigest()
            for name, path in CASE.source_paths().items()}
    return CASE.BoundedRoute("synthetic-no-socket", "synthetic-token", budget, pins)


def observation(now=100):
    return {
        "observed_unix": now, "actor_sid": "synthetic-actor",
        "actor_session": 7, "active_console_session": 7, "desktop": "Default",
        "dpi_awareness": 2,
        "application_sha256": CASE.APPLICATION_SHA256,
        "fixture_sha256": CASE.FIXTURE_SHA256,
        "fence": {
            "store": "ActiveStore", "profiles_enabled": ["Domain", "Private", "Public"],
            "program": "synthetic.exe", "direction": "Outbound", "action": "Block",
            "enabled": True, "profiles": "Any", "protocol": "Any",
            "local_address": "Any", "remote_address": "Any",
            "local_port": "Any", "remote_port": "Any",
            "competing_allow_or_bypass": [], "policy_status": "OK",
        },
        "owned": {"pid": 42, "start_utc": "synthetic-start", "session": 7,
                  "path": "synthetic.exe", "sha256": CASE.APPLICATION_SHA256},
        "foreground": 123, "root": 123, "foreground_pid": 42,
        "hit": 124, "hit_pid": 42, "hit_root": 123,
        "cloaked": 0, "cloak_hresult": 0, "frame_hresult": 0,
        "dwm_frame": [100, 100, 900, 700], "point": [130, 180],
        "screenshot_sha256": PIN, "visible": True,
    }


def selector():
    return {
        "frame_sha256": PIN, "source_uuid": CASE.ROUTES["On"]["source_uuid"],
        "inputref_uuid": CASE.ROUTES["On"]["inputref_uuid"],
        "controller_input_uuid": CASE.ROUTES["On"]["controller_input_uuid"],
        "visible_source_title": "SimL3Power", "visible_capsule": "VI4",
        "kind": "visible_InputRef_arrow_switch_glyph",
        "point": [130, 180], "glyph_polygon": [[126, 176], [134, 176], [134, 184], [126, 184]],
        "capsule_left": 150, "selection_receipt_sha256": "b" * 64,
    }


def sample(step):
    on, reset, active = {
        "initial": ("Aus", "Aus", False),
        "On_high": ("Ein", "Aus", True),
        "On_low": ("Aus", "Aus", True),
        "Reset_high": ("Aus", "Ein", False),
        "Reset_low": ("Aus", "Aus", False),
    }[step]
    values = {
        "source_On": on, "controller_On": on, "source_Reset": reset,
        "controller_Reset_displayed_Off": reset,
        **{f"channel_AQ{n}": "1" if active else "0" for n in (1, 2, 10)},
        **{f"observer_Q{n}": "Ein" if active else "Aus" for n in (1, 2, 10)},
    }
    return {
        "frame_sha256": PIN, "observed_unix": 100,
        "selection_receipt_sha256": "b" * 64,
        "signals": {key: {"visible_text": value, "binding": key,
                          "identity_uuid": CASE.SIGNAL_IDENTITIES[key], "frame_sha256": PIN}
                    for key, value in values.items()},
    }


class DeadlineTests(unittest.TestCase):
    def test_no_arbitrary_command_or_unreviewed_payload_escapes_to_legacy_client(self):
        budget = CASE.Budget(90, 1290, reserve=60, clock=lambda: 100)
        route = synthetic_route(budget)
        with patch.object(CASE.TRANSPORT, "powershell_bounded") as send:
            for value in ("arbitrary-code", {"operation": "capture", "command": "arbitrary-code"}):
                with self.subTest(value=value), self.assertRaises(CASE.Refusal):
                    route.exchange("capture", value, deadline=25)
            route.reviewed_pins["notebook_case01.ps1"] = "0" * 64
            with self.assertRaises(CASE.Refusal):
                route.exchange("read_fixture", {"operation": "read_fixture"}, deadline=25)
            send.assert_not_called()

    def test_explicit_deadline_and_cleanup_reserve_cover_every_route(self):
        budget = CASE.Budget(90, 1290, reserve=60, clock=lambda: 100)
        calls = []

        def bounded(*args, **kwargs):
            calls.append((args, kwargs))
            return {"operation": "capture"}, {}

        with patch.object(CASE.TRANSPORT, "powershell_bounded", bounded):
            route = synthetic_route(budget)
            for operation in ("prelaunch", "capture", "read_fixture", "cleanup_probe"):
                route.exchange(operation, {"operation": operation}, deadline=25)
        self.assertEqual(len(calls), 4)
        self.assertTrue(all(24 < call[1]["deadline"] <= 25 for call in calls))
        self.assertTrue(all(call[1]["timeout"] <= 15 for call in calls))
        for invalid in (None, True, 0, 25.1, float("nan")):
            with self.subTest(invalid=invalid), self.assertRaises(CASE.Refusal):
                route.exchange("capture", {"operation": "capture"}, deadline=invalid)
        with self.assertRaises(CASE.Refusal):
            CASE.Budget(0, 1201, reserve=60, clock=lambda: 100)
        budget.clock = lambda: 1240
        with self.assertRaises(CASE.Refusal):
            route.exchange("capture", {"operation": "capture"}, deadline=25)
        with patch.object(CASE.TRANSPORT, "powershell_bounded", bounded):
            route.exchange("cleanup_probe", {"operation": "cleanup_probe"}, deadline=25, cleanup=True)

    def test_pinned_supervisor_stalls_are_bounded_without_any_network(self):
        real_popen = subprocess.Popen
        for stage in ("initialize", "notifications/initialized", "SSE", "tools/call", "cleanup"):
            children = []

            def synthetic_worker(*_args, **kwargs):
                child = real_popen(
                    [sys.executable, "-B", "-c",
                     "import json,sys,time;sys.stdin.buffer.read();"
                     f"print(json.dumps({{'stage': {stage!r}, 'metrics': {{}}}}),flush=True);"
                     "time.sleep(10)"],
                    **kwargs,
                )
                children.append(child)
                return child

            budget = CASE.Budget(90, 1290, reserve=60, clock=lambda: 100)
            route = synthetic_route(budget)
            started = time.monotonic()
            with self.subTest(stage=stage):
                with patch.object(CASE.TRANSPORT.subprocess, "Popen", synthetic_worker):
                    with self.assertRaises(CASE.Refusal):
                        route.exchange("capture", {"operation": "capture"}, deadline=0.5)
                self.assertLess(time.monotonic() - started, 2)
                self.assertEqual(len(children), 1)
                self.assertIsNotNone(children[0].poll())
                self.assertTrue(route.parked)
                self.assertEqual(route.remote_completion, "UNKNOWN")

    def test_timeout_parks_and_never_reconnects_or_cleans_remote(self):
        budget = CASE.Budget(90, 1290, reserve=60, clock=lambda: 100)
        error = CASE.TRANSPORT.UxError("deadline", "synthetic stall")
        for stage in ("initialize", "notification", "SSE", "call", "cleanup"):
            error.details = {"stage": stage, "remoteCompletion": "UNKNOWN"}
            with self.subTest(stage=stage):
                route = synthetic_route(budget)
                with patch.object(CASE.TRANSPORT, "powershell_bounded", side_effect=error) as send:
                    with self.assertRaises(CASE.Refusal):
                        route.exchange("capture", {"operation": "capture"}, deadline=20)
                    with self.assertRaises(CASE.Refusal):
                        route.exchange("cleanup_probe", {"operation": "cleanup_probe"},
                                       deadline=20, cleanup=True)
                    self.assertEqual(send.call_count, 1)
                self.assertEqual(route.remote_completion, "UNKNOWN")
                self.assertTrue(route.parked)

    def test_control_status_supervisor_covers_init_notification_and_tool_stalls(self):
        real_popen = subprocess.Popen
        for stage in ("initialize", "notification", "ControlStatus"):
            children = []

            def synthetic_worker(*_args, **kwargs):
                child = real_popen([sys.executable, "-B", "-c",
                                    "import sys,time;sys.stdin.buffer.read();time.sleep(10)"], **kwargs)
                children.append(child)
                return child

            with self.subTest(stage=stage):
                began = time.monotonic()
                with patch.object(CASE.CONTROL.subprocess, "Popen", synthetic_worker):
                    with self.assertRaises(CASE.CONTROL.TRANSPORT.UxError):
                        CASE.CONTROL.control_status_bounded("synthetic-no-socket", "synthetic-token",
                                                            deadline=0.5)
                self.assertLess(time.monotonic() - began, 2)
                self.assertEqual(len(children), 1)
                self.assertIsNotNone(children[0].poll())


class GuardTests(unittest.TestCase):
    def test_owned_identity_and_every_effective_scope_are_independently_rechecked(self):
        owned = observation()["owned"]
        for key in ("pid", "start_utc", "session", "path", "sha256"):
            data = observation()
            data["owned"][key] = 999 if key in ("pid", "session") else "different"
            with self.subTest(identity=key), self.assertRaises(CASE.Refusal):
                CASE.validate_os_observation(
                    data, actor_sid="synthetic-actor", application_path="synthetic.exe",
                    now=100, owned=owned, target=selector())
        for key in ("profiles", "program", "protocol", "local_address", "remote_address",
                    "local_port", "remote_port", "policy_status"):
            data = observation()
            data["fence"][key] = "restricted-or-unknown"
            with self.subTest(fence=key), self.assertRaises(CASE.Refusal):
                CASE.validate_os_observation(
                    data, actor_sid="synthetic-actor", application_path="synthetic.exe", now=100)

    def test_supported_control_reply_requires_actual_schema_generation_and_no_refusal(self):
        for value in (True, {"state": "ai", "generation": True, "wait_seconds": 0},
                      {"state": "ready", "generation": 7, "wait_seconds": 0},
                      {"code": "USER_CONTROL", "status": {"generation": 7}},
                      {"code": "CONTROL_UNAVAILABLE", "status": {"generation": 7}}):
            reply = {"content": [{"type": "text", "text": json.dumps(value)}]}
            with self.subTest(value=value), self.assertRaises(CASE.CONTROL.TRANSPORT.UxError):
                CASE.CONTROL.decode_status(reply)
        actual = CASE.CONTROL.decode_status({"content": [{
            "type": "text", "text": '{"state":"ai","generation":7,"wait_seconds":0}',
        }]})
        CASE.require_supported_effect_binding(actual, generation=7, observed=100, now=101)
        for generation, observed in ((8, 100), (7, 90)):
            with self.assertRaises(CASE.Refusal):
                CASE.require_supported_effect_binding(actual, generation=generation,
                                                       observed=observed, now=101)

    def test_visual_receipt_is_byte_pinned_not_a_caller_visibility_flag(self):
        owned = observation()["owned"]
        identity = hashlib.sha256(json.dumps(owned, sort_keys=True,
                                             separators=(",", ":")).encode()).hexdigest()
        receipt = {"case_sha256": CASE.CASE_SHA256, "fixture_sha256": CASE.FIXTURE_SHA256,
                   "owned_identity_sha256": identity, "selected_unix": 100,
                   "selection": selector()}
        raw = json.dumps(receipt).encode()
        pin = hashlib.sha256(raw).hexdigest()
        with patch.object(Path, "read_bytes", return_value=raw):
            selected = CASE.load_selection("synthetic-receipt", pin, owned=owned,
                                           operation="glyph", now=101)
            self.assertEqual(selected["point"], selector()["point"])
            with self.assertRaises(CASE.Refusal):
                CASE.load_selection("synthetic-receipt", "0" * 64, owned=owned,
                                    operation="glyph", now=101)
            with self.assertRaises(CASE.Refusal):
                CASE.load_selection("synthetic-receipt", pin, owned=owned,
                                    operation="glyph", now=103)

    def test_real_payload_mocked_os_refuses_incomplete_effective_proofs(self):
        # This executes extracted guard definitions with all OS reads mocked.
        completed = subprocess.run(
            ["pwsh", "-NoProfile", "-NonInteractive", "-File",
             str(Path(__file__).with_name("test_notebook_case01_backend.ps1"))],
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=20, check=False,
        )
        self.assertEqual(completed.returncode, 0,
                         completed.stderr.decode("utf-8", errors="replace"))
        self.assertIn(b"native API calls=0", completed.stdout)

    def test_prelaunch_needs_no_nonexistent_app_window(self):
        data = observation()
        for key in ("owned", "foreground", "root", "hit", "dwm_frame", "point",
                    "foreground_pid", "hit_pid", "hit_root", "screenshot_sha256",
                    "cloaked", "cloak_hresult", "frame_hresult", "visible"):
            data.pop(key)
        CASE.validate_os_observation(data, actor_sid="synthetic-actor",
                                     application_path="synthetic.exe", now=100)

    def test_each_physical_guard_failure_is_terminal_before_effect(self):
        for key, value in (
            ("observed_unix", 90), ("actor_sid", "other"), ("desktop", "Winlogon"),
            ("dpi_awareness", 0),
            ("active_console_session", 8), ("foreground_pid", 9),
            ("foreground", 999), ("hit_root", 999), ("hit_pid", 9),
            ("cloaked", 1), ("cloak_hresult", -1), ("frame_hresult", -1),
            ("point", [99, 180]), ("visible", False),
        ):
            data = observation()
            data[key] = value
            with self.subTest(key=key), self.assertRaises(CASE.Refusal):
                CASE.validate_os_observation(
                    data, actor_sid="synthetic-actor", application_path="synthetic.exe",
                    now=100, owned=observation()["owned"], target=selector())
        data = observation()
        data["fence"]["remote_port"] = "443"
        with self.assertRaises(CASE.Refusal):
            CASE.validate_os_observation(data, actor_sid="synthetic-actor",
                                         application_path="synthetic.exe", now=100)

    def test_glyph_never_uses_body_center_or_inferred_rectangle(self):
        for changes in (
            {"kind": "body_center"}, {"frame_sha256": "c" * 64},
            {"point": [500, 400]}, {"source_uuid": "title-not-identity"},
            {"glyph_polygon": []}, {"capsule_left": 100},
        ):
            target = selector()
            target.update(changes)
            with self.subTest(changes=changes), self.assertRaises(CASE.Refusal):
                CASE.validate_glyph(target, observation(), "On")
        CASE.validate_glyph(selector(), observation(), "On")

    def test_missing_selector_requires_visual_selection_not_coordinate_guess(self):
        with self.assertRaises(CASE.Refusal) as failure:
            CASE.validate_glyph(None, observation(), "On")
        self.assertEqual(failure.exception.code, "VISUAL_SELECTION_REQUIRED")

    def test_actual_control_api_not_a_caller_boolean(self):
        for caller_claim in (True, {"state": "ai", "generation": 12}, {"guard": True}):
            with self.subTest(claim=caller_claim), self.assertRaises(CASE.Refusal) as failure:
                CASE.require_supported_effect_binding(caller_claim)
            self.assertEqual(failure.exception.code,
                             "GuardSourceExact:CONTROL_BINDING_UNSUPPORTED_OR_STALE")


class CaseTests(unittest.TestCase):
    def test_action_budget_is_consumed_before_attempt_not_after_receipt(self):
        state = CASE.CaseState(lambda _event: None)
        state.begin_action("launch")
        with self.assertRaises(CASE.Refusal):
            state.begin_action("launch")
        with self.assertRaises(CASE.Refusal):
            state.begin_action("sim_start")
        self.assertEqual(state.counts["launch"], 1)

    def test_complete_order_requires_nine_fresh_independent_samples(self):
        journal = []
        state = CASE.CaseState(journal.append)
        initial = sample("initial")
        state.accept_initial(initial, now=100, frame_sha256=PIN)
        for index, step in enumerate(CASE.ORDER[1:]):
            pre = sample(CASE.ORDER[index])
            pre["observed_unix"] = 101 + index * 2
            state.accept_pre(pre, now=pre["observed_unix"], frame_sha256=PIN)
            state.begin_action("glyph")
            state.accept_click(step, {"down": True, "up": True, "inserted": 2})
            post = sample(step)
            post["observed_unix"] = 102 + index * 2
            state.accept_post(post, now=post["observed_unix"], frame_sha256=PIN)
        self.assertEqual(state.index, 5)
        self.assertEqual(state.counts["glyph"], 4)
        self.assertEqual(len([event for event in journal if event["event"] == "SAMPLE"]), 9)
        with self.assertRaises(CASE.Refusal):
            state.begin_action("glyph")

    def test_stale_batch_guard_cannot_be_reused_as_next_sample(self):
        state = CASE.CaseState(lambda _event: None)
        state.accept_initial(sample("initial"), now=100, frame_sha256=PIN)
        with self.assertRaises(CASE.Refusal):
            state.accept_pre(sample("initial"), now=100, frame_sha256=PIN)
        self.assertTrue(state.parked)

    def test_ordered_independent_ten_signal_predicates_not_off_on_falling_on(self):
        for step in CASE.ORDER:
            CASE.validate_sample(sample(step), step, now=100, frame_sha256=PIN)
        bad = sample("On_low")
        bad["signals"]["channel_AQ10"]["visible_text"] = "0"
        with self.assertRaises(CASE.Refusal):
            CASE.validate_sample(bad, "On_low", now=100, frame_sha256=PIN)

    def test_unknown_missing_or_unbound_never_normalizes_to_off(self):
        for key in sample("initial")["signals"]:
            for change in ("missing", "...", "…", "UNKNOWN", "unbound"):
                data = sample("initial")
                if change == "missing":
                    del data["signals"][key]
                elif change == "unbound":
                    data["signals"][key]["binding"] = "different"
                else:
                    data["signals"][key]["visible_text"] = change
                with self.subTest(key=key, change=change), self.assertRaises(CASE.Refusal):
                    CASE.validate_sample(data, "initial", now=100, frame_sha256=PIN)

    def test_partial_click_and_first_failure_latch_no_behavior_resume(self):
        # A fake receipt proves control flow only, not SendInput or native propagation.
        journal = []
        state = CASE.CaseState(journal.append)
        state.accept_initial(sample("initial"), now=100, frame_sha256=PIN)
        with self.assertRaises(CASE.Refusal):
            state.accept_click("On_high", {"down": True, "up": False, "inserted": 1})
        with self.assertRaises(CASE.Refusal):
            state.accept_click("On_high", {"down": True, "up": True, "inserted": 2})
        self.assertEqual(len([event for event in journal if event["event"] == "TERMINAL"]), 1)
        self.assertEqual(journal[-1]["event"], "TERMINAL")
        self.assertTrue(state.parked)
        self.assertEqual(state.first_failure["code"], "INCOMPLETE_CLICK")

    def test_cleanup_absence_unknown_and_no_force_payload(self):
        script = Path(CASE.__file__).with_suffix(".ps1").read_text(encoding="utf-8")
        for forbidden in ("Stop-Process", "taskkill", ".Kill(", "mouse_event", "SetValue",
                          "SendMessage(", "PostMessage("):
            self.assertNotIn(forbidden, script)
        state = CASE.CaseState(lambda _event: None)
        state.finish_cleanup({"absence_observed": False,
                              "fixture_sha256": CASE.FIXTURE_SHA256})
        self.assertTrue(state.parked)
        self.assertEqual(state.cleanup, "UNKNOWN")

    def test_raw_source_pins_include_transport_and_new_composition(self):
        paths = CASE.source_paths()
        pins = {name: hashlib.sha256(path.read_bytes()).hexdigest()
                for name, path in paths.items()}
        CASE.verify_sources(pins)
        pins["notebook_case01.ps1"] = "0" * 64
        with self.assertRaises(CASE.Refusal):
            CASE.verify_sources(pins)


if __name__ == "__main__":
    unittest.main()
