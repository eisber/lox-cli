"""IMPLEMENTATION_BLOCKED private case01 caller, not a native entrypoint.

The backend deliberately refuses at-effect generation/owner and complete WFP
proofs it cannot obtain. Neither gate may be replaced by a file/caller assertion.
Synthetic backends prove control flow, never native evidence.
"""

import base64
import hashlib
import importlib.util
import json
import math
import os
from pathlib import Path
import time


APPLICATION_SHA256 = "7b55e3ef18401574ab967c5c6a0140cd5a66dc728521ebba04ad53093577d8b3"
FIXTURE_SHA256 = "674905cda0c10ffff853e7090507f696747c8b0fb8f3149479b96cff924823ce"
CASE_SHA256 = "de405802ed2eefaff63462421b6e283d0db60f556c108e32040007ad7cb215d7"
TRANSPORT_SHA256 = "f88b9d733f9f4cbe4ea6ceb9efcfac2a864b20bcb5f4dd713229414d36c15da8"
TRANSPORT_LF_SHA256 = "527e4743b0bae46fc25a8318d4504dc0b25a7872d256e22ab843248ac4cb3334"
ORDER = ("initial", "On_high", "On_low", "Reset_high", "Reset_low")
ROUTES = {
    "On": {
        "source_uuid": "1ea4f688-03b3-512d-ffff741afa8d2a84",
        "inputref_uuid": "5b0293c3-a270-12b9-ffff741afa8d2a84",
        "controller_input_uuid": "80af602c-88db-edd0-ffff741afa8d2a84",
        "visible_title": "SimL3Power", "capsule": "VI4",
    },
    "Reset": {
        "source_uuid": "1ea4f689-0213-564b-ffff741afa8d2a84",
        "inputref_uuid": "713062c8-d54e-cb41-ffff741afa8d2a84",
        "controller_input_uuid": "312974df-dc0a-cd58-ffff741afa8d2a84",
        "visible_title": "SimL2Power", "capsule": "VI5",
    },
}
SIGNAL_IDENTITIES = {
    "source_On": ROUTES["On"]["inputref_uuid"],
    "controller_On": ROUTES["On"]["controller_input_uuid"],
    "source_Reset": ROUTES["Reset"]["inputref_uuid"],
    "controller_Reset_displayed_Off": ROUTES["Reset"]["controller_input_uuid"],
    "channel_AQ1": "197dd04d-946d-7373-ffff741afa8d2a84",
    "channel_AQ2": "6dc8ef9f-e73f-2bd0-ffff741afa8d2a84",
    "channel_AQ10": "5252fe27-7979-e7fc-ffff741afa8d2a84",
    "observer_Q1": "1495b0eb-10ff-720e-ffff741afa8d2a84",
    "observer_Q2": "6564f985-cdd0-48b8-ffff741afa8d2a84",
    "observer_Q10": "27e14ad0-e6d4-3ff4-ffff741afa8d2a84",
}

# Deliberately load this exact sibling, not whatever a legacy sys.path selects.
_transport_path = Path(__file__).with_name("loxone_ux_transport.py")
if hashlib.sha256(_transport_path.read_bytes()).hexdigest() not in (
    TRANSPORT_SHA256, TRANSPORT_LF_SHA256,
):
    raise ImportError("Case01 requires the exact published bounded transport bytes")
_spec = importlib.util.spec_from_file_location("_case01_exact_transport", _transport_path)
TRANSPORT = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(TRANSPORT)
_control_path = Path(__file__).with_name("notebook_case01_control.py")
_control_spec = importlib.util.spec_from_file_location("_case01_exact_control", _control_path)
CONTROL = importlib.util.module_from_spec(_control_spec)
_control_spec.loader.exec_module(CONTROL)


class Refusal(Exception):
    def __init__(self, code):
        super().__init__(code)
        self.code = code


def require(condition, code):
    if not condition:
        raise Refusal(code)


def finite(value):
    return type(value) in (int, float) and math.isfinite(value)


def digest(value):
    return (isinstance(value, str) and len(value) == 64
            and all(c in "0123456789abcdef" for c in value))


def source_paths():
    return {
        "notebook_case01.py": Path(__file__),
        "notebook_case01.ps1": Path(__file__).with_suffix(".ps1"),
        "notebook_case01_control.py": Path(CONTROL.__file__),
        "loxone_ux_transport.py": _transport_path,
    }


def verify_sources(reviewed_pins):
    paths = source_paths()
    require(set(reviewed_pins) == set(paths), "SOURCE_COMPOSITION_INCOMPLETE")
    for name, path in paths.items():
        require(hashlib.sha256(path.read_bytes()).hexdigest() == reviewed_pins[name],
                "UNREVIEWED_SOURCE:" + name)


def read_pinned_json(path, expected_sha256):
    require(digest(expected_sha256), "EXTERNAL_RECEIPT_PIN_REQUIRED")
    try:
        raw = Path(path).read_bytes()
        require(hashlib.sha256(raw).hexdigest() == expected_sha256, "EXTERNAL_RECEIPT_CHANGED")
        value = TRANSPORT.parse_json(raw)
    except (OSError, ValueError, UnicodeError):
        raise Refusal("EXTERNAL_RECEIPT_UNREADABLE") from None
    require(isinstance(value, dict), "EXTERNAL_RECEIPT_SCHEMA")
    return value


def load_selection(path, expected_sha256, *, owned, operation, now):
    require(path is not None, "VISUAL_SELECTION_REQUIRED")
    receipt = read_pinned_json(path, expected_sha256)
    require(receipt.get("case_sha256") == CASE_SHA256
            and receipt.get("fixture_sha256") == FIXTURE_SHA256, "VISUAL_RECEIPT_WRONG_CASE")
    identity = hashlib.sha256(json.dumps(owned, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
    require(receipt.get("owned_identity_sha256") == identity, "VISUAL_RECEIPT_WRONG_PROCESS")
    stamp = receipt.get("selected_unix")
    require(finite(stamp) and 0 <= now - stamp <= 2, "VISUAL_SELECTION_EXPIRED")
    target = receipt.get("selection")
    require(isinstance(target, dict) and digest(target.get("frame_sha256")),
            "VISUAL_SELECTION_REQUIRED")
    expected_kind = {
        "navigate": "visible_NAME_BODY_navigation_only",
        "move": "visible_explicit_point", "hover": "visible_explicit_point",
        "sim_start": "visible_simulator_start_control",
        "sim_stop": "visible_simulator_stop_control", "close": "visible_owned_close_control",
        "glyph": "visible_InputRef_arrow_switch_glyph",
    }.get(operation)
    require(expected_kind is not None and target.get("kind") == expected_kind,
            "VISUAL_TARGET_KIND_MISMATCH")
    target = dict(target)
    target["selection_receipt_sha256"] = expected_sha256
    return target


class Budget:
    def __init__(self, admitted, deadline, *, reserve, clock=time.time):
        require(all(finite(v) for v in (admitted, deadline, reserve)),
                "DEADLINE_INVALID")
        require(0 < deadline - admitted <= 1200 and 25 <= reserve < deadline - admitted,
                "CASE_DEADLINE_OR_RESERVE_INVALID")
        self.admitted, self.deadline, self.reserve, self.clock = admitted, deadline, reserve, clock
        self._monotonic_start = time.monotonic()
        self._wall_start = clock()
        require(admitted <= self._wall_start < deadline, "ADMISSION_EXPIRED_OR_FUTURE")

    def remaining(self, *, cleanup=False):
        # A backwards wall-clock change cannot extend this case's original budget.
        now = max(self.clock(), self._wall_start + time.monotonic() - self._monotonic_start)
        return self.deadline - now - (0 if cleanup else self.reserve)

    def exchange_deadline(self, requested, *, cleanup=False):
        require(finite(requested) and 0 < requested <= 25, "EXCHANGE_DEADLINE_REQUIRED_LE_25")
        remaining = self.remaining(cleanup=cleanup)
        require(remaining > 1, "CASE_BUDGET_EXHAUSTED")
        return min(requested, remaining)


class BoundedRoute:
    """Only the pinned supervisor owns initialize/notify/SSE/call/local cleanup."""

    READS = frozenset(("prelaunch", "capture", "read_fixture", "cleanup_probe"))

    def __init__(self, url, token, budget, reviewed_pins):
        self._url, self._token, self.budget = url, token, budget
        self.reviewed_pins = dict(reviewed_pins)
        self.parked = False
        self.remote_completion = "NOT_STARTED"

    def exchange(self, operation, request, *, deadline, cleanup=False):
        require(not self.parked, "PARKED_NO_RECONNECT_RETRY_OR_REMOTE_CLEANUP")
        require(operation in self.READS or operation in (
            "launch", "navigate", "move", "hover", "glyph", "sim_start", "sim_stop", "close"),
            "OPERATION_NOT_CASE01")
        total = self.budget.exchange_deadline(deadline, cleanup=cleanup)
        began = time.monotonic()
        verify_sources(self.reviewed_pins)
        require(isinstance(request, dict) and request.get("operation") == operation
                and set(request) <= {"operation", "actor_sid", "application_path",
                                     "fixture_path", "owned", "target", "deadline_unix",
                                     "bridge", "control_observed_unix", "control_generation"},
                "FIXED_CASE01_REQUEST_REQUIRED")
        data = base64.b64encode(json.dumps(request, allow_nan=False).encode()).decode("ascii")
        raw_script = Path(__file__).with_suffix(".ps1").read_bytes()
        require(hashlib.sha256(raw_script).hexdigest() == self.reviewed_pins["notebook_case01.ps1"],
                "UNREVIEWED_PAYLOAD_SNAPSHOT")
        command = ("$Case01Request=[Text.Encoding]::UTF8.GetString([Convert]::FromBase64String('"
                   + data + "'))|ConvertFrom-Json;\n" + raw_script.decode("utf-8"))
        total -= time.monotonic() - began
        require(total > 0, "LOCAL_PREPARATION_DEADLINE_EXHAUSTED")
        try:
            reply, metrics = TRANSPORT.powershell_bounded(
                self._url, self._token, command, timeout=min(15, total),
                deadline=total,
            )
        except Exception:
            # Even a local parse/HTTP failure can follow a completed remote effect.
            self.parked = True
            self.remote_completion = "UNKNOWN"
            raise Refusal("EXCHANGE_FAILED_REMOTE_COMPLETION_UNKNOWN") from None
        self.remote_completion = "REPLY_RECEIVED_NOT_NATIVE_PROOF"
        require(isinstance(reply, dict), "BACKEND_RECEIPT_INVALID")
        return reply, metrics


def require_supported_effect_binding(actual=None, *, generation=None, observed=None, now=None):
    require(isinstance(actual, dict) and set(actual) == {"state", "generation", "wait_seconds"}
            and actual.get("state") == "ai" and actual.get("wait_seconds") == 0
            and type(generation) is int and actual.get("generation") == generation
            and finite(observed) and finite(now) and 0 <= now - observed <= 2,
            "GuardSourceExact:CONTROL_BINDING_UNSUPPORTED_OR_STALE")


def validate_fence(data, application_path):
    require(isinstance(data, dict), "GuardSourceExact:FENCE_MISSING")
    expected = {
        "store": "ActiveStore", "program": application_path, "direction": "Outbound",
        "action": "Block", "enabled": True, "profiles": "Any", "protocol": "Any",
        "local_address": "Any", "remote_address": "Any",
        "local_port": "Any", "remote_port": "Any", "policy_status": "OK",
    }
    require(all(data.get(key) == value for key, value in expected.items()),
            "GuardSourceExact:EFFECTIVE_FENCE_MISMATCH")
    require(data.get("profiles_enabled") == ["Domain", "Private", "Public"]
            and data.get("competing_allow_or_bypass") == [],
            "GuardSourceExact:FENCE_PROFILE_ALLOW_OR_BYPASS")


def validate_os_observation(data, *, actor_sid, application_path, now, owned=None, target=None):
    require(isinstance(data, dict), "GuardSourceExact:OBSERVATION_MISSING")
    stamp = data.get("observed_unix")
    require(finite(stamp) and 0 <= now - stamp <= 2, "GuardSourceExact:EXPIRED_FRAME")
    require(data.get("actor_sid") == actor_sid, "GuardSourceExact:ACTOR_MISMATCH")
    session = data.get("actor_session")
    require(type(session) is int and session > 0
            and session == data.get("active_console_session") and data.get("desktop") == "Default",
            "GuardSourceExact:NOT_UNLOCKED_ACTIVE_DEFAULT_CONSOLE")
    require(data.get("dpi_awareness") == 2, "GuardSourceExact:PHYSICAL_COORDINATE_CONTEXT_UNSUPPORTED")
    require(data.get("application_sha256") == APPLICATION_SHA256
            and data.get("fixture_sha256") == FIXTURE_SHA256,
            "GuardSourceExact:BINARY_OR_FIXTURE_CHANGED")
    validate_fence(data.get("fence"), application_path)
    if owned is None:
        return
    actual = data.get("owned")
    require(isinstance(actual, dict) and actual == owned
            and actual.get("session") == session
            and actual.get("path") == application_path
            and actual.get("sha256") == APPLICATION_SHA256,
            "GuardSourceExact:OWNED_IDENTITY_MISMATCH")
    require(type(actual.get("pid")) is int and actual["pid"] > 0
            and isinstance(actual.get("start_utc"), str) and actual["start_utc"],
            "GuardSourceExact:OWNED_IDENTITY_INCOMPLETE")
    require(data.get("foreground") == data.get("root") and data.get("root")
            and data.get("foreground_pid") == actual["pid"],
            "GuardSourceExact:PHYSICAL_FOREGROUND_ROOT_MISMATCH")
    require(data.get("cloak_hresult") == 0 and data.get("cloaked") == 0
            and data.get("frame_hresult") == 0 and data.get("visible") is True,
            "GuardSourceExact:DWM_UNAVAILABLE_OR_CLOAKED")
    frame = data.get("dwm_frame")
    require(isinstance(frame, list) and len(frame) == 4 and all(finite(v) for v in frame)
            and frame[0] < frame[2] and frame[1] < frame[3], "GuardSourceExact:DWM_FRAME_INVALID")
    require(digest(data.get("screenshot_sha256")), "GuardSourceExact:SCREENSHOT_MISSING")
    if target is not None:
        point = data.get("point")
        require(data.get("hit_pid") == actual["pid"] and data.get("hit_root") == data["root"]
                and data.get("hit"), "GuardSourceExact:PHYSICAL_FOREGROUND_ROOT_MISMATCH")
        require(isinstance(point, list) and len(point) == 2 and all(finite(v) for v in point)
                and frame[0] <= point[0] < frame[2] and frame[1] <= point[1] < frame[3],
                "GuardSourceExact:PHYSICAL_POINT_OUTSIDE_DWM_FRAME")
        require(target.get("frame_sha256") == data["screenshot_sha256"]
                and target.get("point") == point, "GuardSourceExact:TARGET_FRAME_MISMATCH")


def _inside(point, polygon):
    if not (isinstance(polygon, list) and 3 <= len(polygon) <= 32):
        return False
    if not all(isinstance(p, list) and len(p) == 2 and all(finite(v) for v in p) for p in polygon):
        return False
    x, y = point
    inside = False
    for i, (x1, y1) in enumerate(polygon):
        x2, y2 = polygon[(i + 1) % len(polygon)]
        if (y1 > y) != (y2 > y) and x < (x2 - x1) * (y - y1) / (y2 - y1) + x1:
            inside = not inside
    return inside


def validate_glyph(selection, frame, route):
    require(isinstance(selection, dict), "VISUAL_SELECTION_REQUIRED")
    require(route in ROUTES and selection.get("source_uuid") == ROUTES[route]["source_uuid"]
            and selection.get("inputref_uuid") == ROUTES[route]["inputref_uuid"]
            and selection.get("controller_input_uuid") == ROUTES[route]["controller_input_uuid"]
            and selection.get("visible_source_title") == ROUTES[route]["visible_title"]
            and selection.get("visible_capsule") == ROUTES[route]["capsule"],
            "GLYPH_SOURCE_IDENTITY_MISMATCH")
    require(selection.get("kind") == "visible_InputRef_arrow_switch_glyph"
            and selection.get("frame_sha256") == frame.get("screenshot_sha256")
            and digest(selection.get("selection_receipt_sha256")), "GLYPH_FRAME_OR_SEMANTICS_MISSING")
    point = selection.get("point")
    require(isinstance(point, list) and len(point) == 2
            and all(type(v) is int for v in point)
            and point == frame.get("point")
            and _inside(point, selection.get("glyph_polygon")), "GLYPH_NOT_EXPLICIT_VISIBLE_POINT")
    require(finite(selection.get("capsule_left")) and point[0] < selection["capsule_left"],
            "GLYPH_NOT_LEFT_OF_VI_CAPSULE")


def validate_sample(data, step, *, now, frame_sha256):
    require(step in ORDER and isinstance(data, dict), "SAMPLE_INVALID")
    require(data.get("frame_sha256") == frame_sha256
            and digest(data.get("selection_receipt_sha256")), "SAMPLE_FRAME_UNBOUND")
    stamp = data.get("observed_unix")
    require(finite(stamp) and 0 <= now - stamp <= 2, "SAMPLE_STALE")
    signals = data.get("signals")
    on, reset, active = {
        "initial": ("Aus", "Aus", False), "On_high": ("Ein", "Aus", True),
        "On_low": ("Aus", "Aus", True), "Reset_high": ("Aus", "Ein", False),
        "Reset_low": ("Aus", "Aus", False),
    }[step]
    expected = {"source_On": on, "controller_On": on, "source_Reset": reset,
                "controller_Reset_displayed_Off": reset,
                **{f"observer_Q{n}": "Ein" if active else "Aus" for n in (1, 2, 10)}}
    require(isinstance(signals, dict)
            and set(signals) == set(expected) | {"channel_AQ1", "channel_AQ2", "channel_AQ10"},
            "TEN_INDEPENDENT_SIGNALS_REQUIRED")
    for key, item in signals.items():
        require(isinstance(item, dict) and item.get("binding") == key
                and item.get("identity_uuid") == SIGNAL_IDENTITIES[key]
                and item.get("frame_sha256") == frame_sha256, "SIGNAL_UNBOUND:" + key)
        text = item.get("visible_text")
        require(isinstance(text, str) and text not in ("", "...", "…", "UNKNOWN"),
                "SIGNAL_UNKNOWN:" + key)
        if key in expected:
            require(text == expected[key], "SIGNAL_PREDICATE:" + key)
        else:
            try:
                number = float(text)
            except ValueError:
                raise Refusal("SIGNAL_UNKNOWN:" + key) from None
            require(math.isfinite(number) and (number > 0 if active else number == 0),
                    "CHANNEL_PREDICATE:" + key)


class CaseState:
    """Finite initial + four pre/click/post transitions; no resume."""

    def __init__(self, persist):
        self.persist = persist
        self.first_failure = None
        self.parked = True
        self.cleanup = "UNKNOWN"
        self.index = 0
        self.awaiting_post = False
        self.pre_sample = False
        self.last_sample_time = -math.inf
        self.pending_glyph = False
        self.counts = {"launch": 0, "sim_start": 0, "navigate": 0, "glyph": 0}
        self.persist({"event": "CREATED", "utc_unix": time.time(),
                      "native_proof": "UNKNOWN", "cleanup": "UNKNOWN"})

    def fail(self, code, *, inflight=False):
        if self.first_failure is None:
            self.first_failure = {"code": code, "inflight": inflight, "utc_unix": time.time()}
            self.parked = True
            self.persist({"event": "TERMINAL", **self.first_failure,
                          "remote_completion": "UNKNOWN" if inflight else "NOT_INFERRED",
                          "remaining_observations": "UNKNOWN"})
        raise Refusal(self.first_failure["code"])

    def behavior(self):
        if self.first_failure:
            raise Refusal(self.first_failure["code"])

    def begin_action(self, operation):
        self.behavior()
        try:
            if operation in self.counts:
                limit = {"launch": 1, "sim_start": 1, "navigate": 2, "glyph": 4}[operation]
                require(self.counts[operation] < limit, "ACTION_BUDGET_EXHAUSTED:" + operation)
                if operation == "glyph":
                    require(1 <= self.index <= 4 and self.pre_sample and not self.awaiting_post,
                            "GLYPH_WITHOUT_DISTINCT_PRE_SAMPLE")
                    require(not self.pending_glyph, "GLYPH_ALREADY_INFLIGHT")
                    self.pending_glyph = True
                self.counts[operation] += 1
            else:
                require(operation in ("move", "hover"), "OPERATION_NOT_BEHAVIOR")
        except Refusal as error:
            self.fail(error.code)
        self.persist({"event": "ACTION_RESERVED", "operation": operation,
                      "counts": dict(self.counts), "utc_unix": time.time()})

    def _sample(self, data, step, position, binding):
        validate_sample(data, step, **binding)
        require(data["observed_unix"] > self.last_sample_time, "SAMPLE_REUSED_ACROSS_STEPS")
        self.last_sample_time = data["observed_unix"]
        self.persist({"event": "SAMPLE", "step": step, "position": position, "sample": data})

    def accept_initial(self, data, **binding):
        self.behavior()
        try:
            require(self.index == 0, "INITIAL_ALREADY_SAMPLED")
            self._sample(data, "initial", "initial", binding)
        except Refusal as error:
            self.fail(error.code)
        self.index = 1

    def accept_pre(self, data, **binding):
        self.behavior()
        try:
            require(1 <= self.index <= 4 and not self.awaiting_post and not self.pre_sample,
                    "PRE_SAMPLE_OUT_OF_ORDER")
            self._sample(data, ORDER[self.index - 1], "before", binding)
        except Refusal as error:
            self.fail(error.code)
        self.pre_sample = True

    def accept_click(self, step, receipt):
        self.behavior()
        try:
            require(isinstance(receipt, dict) and receipt.get("down") is True
                    and receipt.get("up") is True and receipt.get("inserted") == 2,
                    "INCOMPLETE_CLICK")
            require(1 <= self.index <= 4 and step == ORDER[self.index]
                    and self.pre_sample and not self.awaiting_post and self.pending_glyph,
                    "CLICK_OUT_OF_ORDER")
        except Refusal as error:
            self.fail(error.code, inflight=True)
        self.awaiting_post = True
        self.pending_glyph = False
        self.persist({"event": "CLICK", "step": step, "receipt": receipt})

    def accept_post(self, data, **binding):
        self.behavior()
        try:
            require(self.awaiting_post, "POST_WITHOUT_COMPLETE_CLICK")
            self._sample(data, ORDER[self.index], "after", binding)
        except Refusal as error:
            self.fail(error.code)
        self.index += 1
        self.awaiting_post = False
        self.pre_sample = False

    def finish_cleanup(self, receipt, *, expected_owned=None, now=None):
        # Only an actual guarded backend absence observation, never local kill.
        stamp = receipt.get("absence_observed_unix")
        proven = (isinstance(expected_owned, dict) and finite(now) and finite(stamp)
                  and 0 <= now - stamp <= 2 and receipt.get("absence_observed") is True
                  and receipt.get("absence_pid") == expected_owned.get("pid")
                  and receipt.get("absence_start_utc") == expected_owned.get("start_utc")
                  and receipt.get("fixture_sha256") == FIXTURE_SHA256)
        self.cleanup = "VERIFIED" if proven else "UNKNOWN"
        self.parked = self.cleanup != "VERIFIED"
        self.persist({"event": "CLEANUP", "utc_unix": time.time(), "cleanup": self.cleanup,
                      "behavior": "UNKNOWN" if self.index != 5 else "OBSERVATIONS_COMPLETE"})


class NotebookCase01:
    """Private parent integration API. It never loads an old profile or helper."""

    def __init__(self, runtime_profile, admission, *, evidence_directory, clock=time.time):
        verify_sources(admission["reviewed_source_sha256"])
        require(admission.get("case_sha256") == CASE_SHA256, "WRONG_IMMUTABLE_CASE")
        for key in ("parent_receipt_sha256", "independent_review_sha256",
                    "exclusive_owner_receipt_sha256", "runtime_profile_identity_sha256"):
            require(digest(admission.get(key)), "MISSING_EXTERNAL_ADMISSION:" + key)
        self.profile, self.admission, self.clock = runtime_profile, admission, clock
        require(type(admission.get("control_generation")) is int, "SUPPORTED_CONTROL_GENERATION_REQUIRED")
        require(digest(admission.get("bridge_guard_sha256"))
                and digest(admission.get("bridge_exe_sha256")),
                "SUPPORTED_BRIDGE_EXACT_SOURCE_REVIEW_REQUIRED")
        self.owned = None
        self.cleanup_attempts = set()
        self.budget = Budget(admission["admitted_unix"], admission["deadline_unix"],
                             reserve=admission["cleanup_reserve_seconds"], clock=clock)
        self._verify_admission_files()
        self.evidence = Path(evidence_directory)
        require(self.evidence.is_absolute() and not self.evidence.exists(),
                "FRESH_PRIVATE_EVIDENCE_DIRECTORY_REQUIRED")
        private_root = Path(runtime_profile["private_evidence_root"]).resolve(strict=True)
        require(private_root.is_absolute() and self.evidence.resolve().is_relative_to(private_root),
                "APPROVED_PRIVATE_EVIDENCE_ROOT_REQUIRED")
        require(not any((parent / ".git").exists() for parent in (private_root, *private_root.parents)),
                "RAW_NATIVE_EVIDENCE_MUST_NOT_BE_IN_A_REPOSITORY")
        self.evidence.mkdir(parents=False)
        self.sequence = 0
        self.state = CaseState(self._persist)
        self.route = BoundedRoute(runtime_profile["mcp_url"], runtime_profile["bearer"], self.budget,
                                  admission["reviewed_source_sha256"])

    def _persist(self, event):
        self.sequence += 1
        path = self.evidence / f"{self.sequence:04d}.json"
        with path.open("x", encoding="utf-8", newline="\n") as stream:
            json.dump(event, stream, allow_nan=False, sort_keys=True)
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())

    def _verify_admission_files(self):
        for field in ("parent_receipt_sha256", "independent_review_sha256",
                      "exclusive_owner_receipt_sha256"):
            receipt = read_pinned_json(self.admission["receipt_paths"][field], self.admission[field])
            require(receipt.get("execution_id") == self.admission.get("execution_id")
                    and receipt.get("case_sha256") == CASE_SHA256,
                    "EXTERNAL_ADMISSION_EXECUTION_MISMATCH")
        require(self.admission.get("execution_id") not in (None, "", "lox-8o2", "lox-cwd"),
                "NEW_EXECUTION_IDENTITY_REQUIRED")
        identity = {key: value for key, value in self.profile.items() if key != "bearer"}
        raw = json.dumps(identity, sort_keys=True, separators=(",", ":")).encode()
        require(hashlib.sha256(raw).hexdigest() == self.admission["runtime_profile_identity_sha256"],
                "PRIVATE_RUNTIME_PROFILE_IDENTITY_MISMATCH")
        require(hashlib.sha256(Path(self.admission["case_path"]).read_bytes()).hexdigest() == CASE_SHA256,
                "ACTUAL_CASE_BYTES_MISMATCH")
        require(hashlib.sha256(Path(self.admission["master_fixture_path"]).read_bytes()).hexdigest()
                == FIXTURE_SHA256, "ACTUAL_MASTER_FIXTURE_CHANGED")
        for key in ("common_contract", "external_contract", "bundle_manifest"):
            item = self.admission[key]
            require(digest(item.get("sha256"))
                    and hashlib.sha256(Path(item["path"]).read_bytes()).hexdigest() == item["sha256"],
                    "ACTUAL_SEALED_CONTRACT_CHANGED:" + key)

    def _request(self, operation, *, target=None, owned=None, control_observed=None,
                 control_generation=None):
        return {
            "operation": operation, "actor_sid": self.profile["actor_sid"],
            "application_path": self.profile["application_path"],
            "fixture_path": self.profile["fixture_path"], "owned": owned, "target": target,
            "deadline_unix": self.budget.deadline,
            "control_observed_unix": control_observed,
            "control_generation": (self.admission["control_generation"] if control_generation is None
                                   else control_generation),
            "bridge": {
                "pid": self.profile["bridge_pid"],
                "start_utc": self.profile["bridge_start_utc"],
                "guard_path": self.profile["bridge_guard_path"],
                "guard_sha256": self.admission["bridge_guard_sha256"],
                "exe_sha256": self.admission["bridge_exe_sha256"],
            },
        }

    def _control(self, deadline, *, cleanup=False, generation=None):
        self._verify_admission_files()
        verify_sources(self.admission["reviewed_source_sha256"])
        total = self.budget.exchange_deadline(deadline, cleanup=cleanup)
        observed = self.clock()
        try:
            status = CONTROL.control_status_bounded(self.profile["mcp_url"], self.profile["bearer"],
                                                    deadline=total)
        except Exception as error:
            self.route.parked = True
            self.route.remote_completion = "UNKNOWN"
            code = getattr(error, "code", "CONTROL_EXCHANGE_UNKNOWN_PARKED")
            raise Refusal(code) from None
        require_supported_effect_binding(
            status, generation=(self.admission["control_generation"] if generation is None else generation),
            observed=observed, now=self.clock())
        self._persist({"event": "SUPPORTED_CONTROL", "observed_unix": observed, "status": status,
                       "exclusive_owner_receipt_sha256": self.admission["exclusive_owner_receipt_sha256"]})
        return observed

    def observe(self, operation, *, deadline, owned=None, target=None):
        self.state.behavior()
        try:
            require(owned is None or owned == self.owned, "OWNED_READ_IDENTITY_MISMATCH")
            require(operation in BoundedRoute.READS, "READ_ONLY_OPERATION_REQUIRED")
            control_time = self._control(deadline)
            reply, metrics = self.route.exchange(
                operation, self._request(operation, target=target, owned=owned,
                                         control_observed=control_time), deadline=deadline)
            require(reply.get("ok") is True, str(reply.get("error", "BACKEND_REFUSAL")))
            validate_os_observation(
                reply["observation"], actor_sid=self.profile["actor_sid"],
                application_path=self.profile["application_path"], now=self.clock(),
                owned=owned, target=target)
            if owned is not None:
                try:
                    image = base64.b64decode(reply["observation"]["screenshot_base64"], validate=True)
                except (KeyError, ValueError):
                    raise Refusal("SCREENSHOT_RECEIPT_INVALID") from None
                require(len(image) <= 4000000 and hashlib.sha256(image).hexdigest()
                        == reply["observation"]["screenshot_sha256"], "SCREENSHOT_BYTES_MISMATCH")
            if operation == "read_fixture":
                try:
                    raw = base64.b64decode(reply["fixture_base64"], validate=True)
                except (KeyError, ValueError):
                    raise Refusal("BOUNDED_TRANSFER_INVALID") from None
                require(len(raw) <= 4000000 and hashlib.sha256(raw).hexdigest() == FIXTURE_SHA256,
                        "BOUNDED_TRANSFER_FIXTURE_MISMATCH")
            self._persist({"event": "OBSERVATION", "operation": operation,
                           "reply": reply, "transport": metrics})
            return reply
        except (Refusal, KeyError, TypeError, ValueError, OSError) as error:
            self.state.fail(error.code if isinstance(error, Refusal) else "RECEIPT_INCOMPLETE",
                            inflight=self.route.parked)

    def sample(self, position, *, deadline, sample_file, sample_sha256):
        self.state.behavior()
        try:
            require(self.owned is not None and self.state.counts["sim_start"] == 1,
                    "NEW_OWNED_SIMULATOR_REQUIRED")
            capture = self.observe("capture", deadline=deadline, owned=self.owned)
            value = read_pinned_json(sample_file, sample_sha256)
            identity = hashlib.sha256(json.dumps(
                self.owned, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
            require(value.get("owned_identity_sha256") == identity
                    and value.get("case_sha256") == CASE_SHA256
                    and value.get("fixture_sha256") == FIXTURE_SHA256,
                    "SAMPLE_EXTERNAL_IDENTITY_UNBOUND")
            binding = {"now": self.clock(),
                       "frame_sha256": capture["observation"]["screenshot_sha256"]}
            require(position in ("initial", "before", "after"), "SAMPLE_POSITION_INVALID")
            value["selection_receipt_sha256"] = sample_sha256
            {"initial": self.state.accept_initial, "before": self.state.accept_pre,
             "after": self.state.accept_post}[position](value, **binding)
        except (Refusal, KeyError, TypeError, ValueError, OSError) as error:
            code = error.code if isinstance(error, Refusal) else "ACTION_RECEIPT_OR_SOURCE_INVALID"
            self.state.fail(code, inflight=self.route.parked)

    def effect(self, operation, *, deadline, selection_file=None, selection_sha256=None, owned=None):
        self.state.behavior()
        attempted = False
        try:
            target = None
            require(operation in ("launch", "navigate", "move", "hover", "glyph", "sim_start"),
                    "SEPARATE_CLEANUP_AUTHORITY_REQUIRED")
            require((operation == "launch" and owned is None and self.owned is None)
                    or (operation != "launch" and owned == self.owned and owned is not None),
                    "NEW_OWNED_PROCESS_BINDING_REQUIRED")
            if operation != "launch":
                target = load_selection(selection_file, selection_sha256, owned=owned,
                                        operation=operation, now=self.clock())
                if operation == "glyph":
                    require(1 <= self.state.index <= 4, "GLYPH_ORDER_EXHAUSTED")
                    route = ORDER[self.state.index].split("_")[0]
                    validate_glyph(target, {
                        "screenshot_sha256": target.get("frame_sha256"),
                        "point": target.get("point"),
                    }, route)
            self.state.begin_action(operation)
            control_time = self._control(deadline)
            require(self.clock() - control_time <= 2, "GuardSourceExact:CONTROL_BINDING_STALE")
            attempted = True
            reply, metrics = self.route.exchange(
                operation, self._request(operation, target=target, owned=owned,
                                         control_observed=control_time), deadline=deadline)
            self._persist({"event": "ACTION_REPLY", "operation": operation,
                           "reply": reply, "transport": metrics})
            require(reply.get("ok") is True, str(reply.get("error", "BACKEND_REFUSAL")))
            validate_os_observation(
                reply["observation"], actor_sid=self.profile["actor_sid"],
                application_path=self.profile["application_path"], now=self.clock(),
                owned=owned, target=target)
            if operation == "launch":
                new = reply.get("launched_identity")
                require(isinstance(new, dict) and new.get("sha256") == APPLICATION_SHA256
                        and new.get("path") == self.profile["application_path"]
                        and type(new.get("pid")) is int and new["pid"] > 0
                        and new.get("session") == reply["observation"]["actor_session"],
                        "NEW_PROCESS_IDENTITY_UNKNOWN")
                self.owned = new
            elif operation == "glyph":
                self.state.accept_click(ORDER[self.state.index], reply.get("completion"))
            else:
                complete = reply.get("completion", {})
                require(complete.get("completed") is True or complete.get("inserted") == 2,
                        "ACTION_COMPLETION_UNKNOWN")
            self._control(deadline)
            return reply
        except (Refusal, KeyError, TypeError, ValueError, OSError) as error:
            code = error.code if isinstance(error, Refusal) else "ACTION_RECEIPT_OR_SOURCE_INVALID"
            self.state.fail(code, inflight=attempted or self.route.parked)

    def cleanup(self, operation, *, deadline, authority_file, authority_sha256,
                selection_file, selection_sha256):
        """Separate cleanup-only authority never clears first_failure."""
        try:
            require(operation in ("sim_stop", "close") and operation not in self.cleanup_attempts,
                    "CLEANUP_BUDGET_EXHAUSTED")
            require(not self.route.parked, "INFLIGHT_UNKNOWN_NO_REMOTE_CLEANUP")
            authority = read_pinned_json(authority_file, authority_sha256)
            require(authority.get("execution_id") == self.admission["execution_id"]
                    and authority.get("scope") == "GRACEFUL_OWNED_CLEANUP_ONLY"
                    and authority.get("owned") == self.owned, "FRESH_CLEANUP_AUTHORITY_REQUIRED")
            require(finite(authority.get("deadline_unix"))
                    and self.clock() < authority["deadline_unix"] <= self.budget.deadline,
                    "CLEANUP_AUTHORITY_EXPIRED")
            target = load_selection(selection_file, selection_sha256, owned=self.owned,
                                    operation=operation, now=self.clock())
            self.cleanup_attempts.add(operation)
            require(type(authority.get("control_generation")) is int,
                    "CLEANUP_SUPPORTED_CONTROL_GENERATION_REQUIRED")
            generation = authority["control_generation"]
            control_time = self._control(deadline, cleanup=True, generation=generation)
            reply, metrics = self.route.exchange(
                operation, self._request(operation, target=target, owned=self.owned,
                                         control_observed=control_time, control_generation=generation),
                deadline=deadline, cleanup=True)
            self._persist({"event": "CLEANUP_REPLY", "operation": operation,
                           "reply": reply, "transport": metrics})
            require(reply.get("ok") is True, str(reply.get("error", "CLEANUP_REFUSAL")))
            validate_os_observation(
                reply["observation"], actor_sid=self.profile["actor_sid"],
                application_path=self.profile["application_path"], now=self.clock(),
                owned=self.owned, target=target)
            if operation == "close":
                self._verify_admission_files()
                self.state.finish_cleanup(reply, expected_owned=self.owned, now=self.clock())
            else:
                complete = reply.get("completion", {})
                require(complete.get("down") is True and complete.get("up") is True
                        and complete.get("inserted") == 2, "CLEANUP_CLICK_INCOMPLETE")
            return reply
        except (Refusal, KeyError, TypeError, ValueError, OSError) as error:
            self.state.finish_cleanup({})
            code = error.code if isinstance(error, Refusal) else "CLEANUP_RECEIPT_OR_SOURCE_INVALID"
            self.state.fail(code, inflight=self.route.parked)
