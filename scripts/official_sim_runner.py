#!/usr/bin/env python3
"""Interactive-context child launcher for official-sim.py (never an oracle)."""

import argparse
import ctypes
from ctypes import wintypes
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time


ENVIRONMENT = {
    "SYSTEMROOT", "WINDIR", "SYSTEMDRIVE", "COMSPEC", "PATH", "PATHEXT",
    "TEMP", "TMP", "USERPROFILE", "LOCALAPPDATA", "APPDATA", "PROGRAMDATA",
    "PROGRAMFILES", "PROGRAMFILES(X86)", "PROGRAMW6432", "PUBLIC", "HOMEDRIVE",
    "HOMEPATH", "USERNAME", "USERDOMAIN", "PROCESSOR_ARCHITECTURE",
    "NUMBER_OF_PROCESSORS", "SESSIONNAME",
}
ENTRY_EFFECT = "PASSIVE_observer_exit_flush_not_live_snapshot"
ENTRY_RVAS = {0x371E10, 0x371EF0, 0x15EF30, 0x614600, 0x4B6DE0}


def sampling_plan(case):
    """Normalize legacy sampling without inventing simulator steps."""
    phases = case.get("sampling_phases")
    if phases is None:
        wait = case.get("wait_seconds")
        if type(wait) is not int or not 70 <= wait <= 120:
            raise ValueError("Legacy cases require a 70..120 second native interval.")
        return [{"name": "after_first_calculation", "at_seconds": wait}]
    if not isinstance(phases, list) or not 1 <= len(phases) <= 6:
        raise ValueError("Declare one to six bounded native sampling phases.")
    names = set()
    previous = 0
    for phase in phases:
        if (not isinstance(phase, dict) or not {"name", "at_seconds", "latest_seconds"} <= set(phase)
                or set(phase) - {"name", "at_seconds", "latest_seconds", "simulator_time_window"}):
            raise ValueError("Each phase requires name, at_seconds and latest_seconds.")
        name, start, end = (phase[key] for key in ("name", "at_seconds", "latest_seconds"))
        if (not isinstance(name, str) or not re.fullmatch(r"[a-z][a-z0-9_]{0,63}", name)
                or name in names or name in {"before_start", "initial_running", "progress_running",
                                             "post_attach_clock_1", "post_attach_clock_2"}
                or type(start) is not int or type(end) is not int
                or not 20 <= start <= 200 or not start < end <= min(start + 30, 210)
                or start < previous):
            raise ValueError("Sampling phases need unique names and ordered, nonoverlapping 20..210 second windows.")
        names.add(name)
        previous = end
        if "simulator_time_window" in phase:
            window = phase["simulator_time_window"]
            if (not isinstance(window, list) or len(window) != 2
                    or any(not isinstance(value, str) or not re.fullmatch(r"(?:[01][0-9]|2[0-3]):[0-5][0-9]:[0-5][0-9]", value)
                           for value in window) or window[0] > window[1]):
                raise ValueError("Native clock windows require ordered exact HH:MM:SS observations within one day.")
    return [dict(phase) for phase in phases]


def native_channels(case):
    channels = case.get("channels", ["OL1"])
    if (not isinstance(channels, list) or not 1 <= len(channels) <= 3
            or any(not isinstance(channel, str) or channel not in {"OL1", "OL2", "OL3"} for channel in channels)
            or len(set(channels)) != len(channels)):
        raise ValueError("Declare distinct native digital channels OL1..OL3.")
    return channels


def validate_recovery_dialog(request, record):
    if request.get("recovery_policy", "block") != "owned-skip":
        raise ValueError("Unexpected startup recovery remains blocked by default.")
    filename = fixture_basename(request)
    window = record.get("window", {})
    identity = record.get("identity", {})
    handle, owner = window.get("Handle"), window.get("ProcessId")
    if (window.get("ClassName") != "#32770" or type(handle) is not int or handle <= 0
            or type(owner) is not int or owner <= 0 or identity.get("owned_pid") != owner
            or record.get("root_handle") != handle or identity.get("fixture_sha256") != request["fixture_sha256"]
            or identity.get("application_sha256") != request["application"]["sha256"]
            or type(identity.get("session")) is not int or identity["session"] <= 0
            or identity.get("actor_session") != identity["session"] or record.get("already_handled") is not False):
        raise ValueError("Recovery dialog lacks exact owned root/session/application/fixture proof, or is repeated.")
    children = record.get("native_text")
    if not isinstance(children, list) or any(
        not isinstance(child, dict) or child.get("class") not in {"Static", "Button"} for child in children
    ):
        raise ValueError("Recovery dialog controls are malformed.")
    buttons = [child for child in children if child.get("class") == "Button"]
    if (len(buttons) != 2 or {child.get("id") for child in buttons} != {6, 7}
            or any(child.get("text") not in ({6: {"&Yes", "&Ja"}, 7: {"&No", "&Nein"}}[child["id"]])
                   for child in buttons)):
        raise ValueError("Recovery dialog does not have the exact known Yes/No controls.")
    texts = [child.get("text") for child in children if child.get("class") == "Static" and child.get("text")]
    expression = (
        r"Es wurde eine automatische Sicherung dieses Dokuments gefunden:\n"
        + re.escape(filename) + r" (?P<timestamp>\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})\n\n"
        + r"Soll diese wiederhergestellt werden\?"
    )
    if len(texts) != 1 or not isinstance(texts[0], str) or not re.fullmatch(expression, texts[0].replace("\r\n", "\n")):
        raise ValueError("Recovery question does not name exactly this unique owned fixture.")
    return {"accepted": True, "action": "decline_restore", "button_id": 7, "root_handle": handle,
            "fixture_filename": filename, "scope": "Native No may clear this lab recovery entry; no cache-preservation claim."}


def digest(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def pinned(path, expected):
    if digest(path) != expected:
        raise ValueError("Child executable/tool identity does not match its pin.")
    return str(path)


def fixture_basename(request, *, allow_legacy=False):
    value = request.get("fixture_filename")
    if value is None and allow_legacy:
        return "input.Loxone"
    if (not isinstance(value, str) or not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.-]{0,100}\.Loxone", value)
            or value.lower() == "input.loxone"
            or value.split(".", 1)[0].upper() in {
                "CON", "PRN", "AUX", "NUL", *(f"COM{index}" for index in range(1, 10)),
                *(f"LPT{index}" for index in range(1, 10)),
            }):
        raise ValueError("New fixture_filename must be a unique simple .Loxone basename without path/stream/device syntax.")
    return value


def load_entry_module(path, expected):
    pinned(path, expected)
    spec = importlib.util.spec_from_file_location("entry_snapshot_" + expected, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    pinned(path, expected)
    return module


def entry_config(payload, observer, target_sha256, client_sha256, pe=None):
    """Bind the ASCII recipe; only the notebook supplies actual PE section metadata."""
    expected = observer.read_config(payload)
    if expected["target_sha256"] != target_sha256 or expected["client_sha256"] != client_sha256:
        raise ValueError("Entry config target/client pins differ from the request.")
    if any(s["rva"] not in ENTRY_RVAS or (s["post"] != 0 and s["rva"] != 0x4B6DE0)
           for s in expected["sites"]):
        raise ValueError("Use reviewed entry-only RVAs; only the selected clock may opt into RETURN.")
    if pe is not None:
        if (pe["sha256"] != target_sha256 or pe["name"] != expected["module"]
                or pe["size"] != expected["module_size"]
                or pe["timestamp"] != expected["module_timestamp"]):
            raise ValueError("Entry config differs from actual pinned PE metadata.")
        if observer.config_bytes(
            pe, client_sha256, expected["sites"], expected["max_events"],
            expected["max_objects"], expected["prefix_bytes"],
        ) != payload:
            raise ValueError("Entry config differs from actual-PE regeneration.")
    return expected


def validate_entry_mode(request):
    if request.get("mode") != "entry-observer":
        return
    if (request.get("instrumentation_start") != "post-start"
            or request.get("observer_mode") != "native-win32"
            or request.get("capture_scope") != "canvas"):
        raise ValueError("Entry observation requires post-start/native-win32/canvas.")
    normalized_legacy = [{"name": "after_first_calculation", "at_seconds": request.get("wait_seconds")}]
    phases = (
        sampling_plan({"wait_seconds": request["wait_seconds"]})
        if request.get("sampling_phases") == normalized_legacy else sampling_plan(request)
    )
    if max(p.get("latest_seconds", p["at_seconds"]) for p in phases) > 90:
        raise ValueError("The first entry-observer case must finish within 90 simulator seconds.")


def validate_native_entry_readiness(profile):
    if profile.get("late_attach_verified") is not True:
        raise ValueError("Native entry observation requires explicit late_attach_verified=true for every backend; diagnostic-only profiles cannot launch an app.")


def entry_inputs(request):
    """Read-only notebook preflight; no staging, process launch, or attachment."""
    validate_entry_mode(request)
    root = Path(request["run_dir"])
    binding = request["entry_observer"]
    module = load_entry_module(root / "native-sim-observe.py", binding["module_sha256"])
    source = root / "instrumentation-profile.json"
    pinned(source, binding["profile_sha256"])
    profile = json.loads(source.read_text(encoding="utf-8-sig"))
    if profile != request["instrumentation"]:
        raise ValueError("Entry profile snapshot differs from the original request.")
    validate_native_entry_readiness(profile)
    module.validate_profile(profile)
    for key, value in {
        "nudge_client_id": "00000000", "nudge_argument": "1",
        "nudge_timeout_ms": 5000, "nudge_outer_timeout_seconds": 8,
        "nudge_semantics": "process exit and exit-callback flush; not live snapshot",
    }.items():
        if type(profile.get(key)) is not type(value) or profile[key] != value:
            raise ValueError("Unsupported passive-observer exit/flush interface.")
    exit_profile = root / "exit-dump-profile.json"
    pinned(exit_profile, binding["profile_sha256"])
    if request.get("exit_dump_method") != profile:
        raise ValueError("Entry cleanup requires the same observer profile, not drcov evidence.")
    config = root / "entry" / "observer.cfg"
    pinned(config, binding["config_sha256"])
    expected = entry_config(config.read_bytes(), module, request["application"]["sha256"],
                            profile["client_sha256"], module.read_pe(request["application"]["path"]))
    if expected != binding["expected"]:
        raise ValueError("Entry recipe differs from its immutable requested identities/sites.")
    if profile.get("supports_return") is False and any(s["post"] for s in expected["sites"]):
        raise ValueError("Entry-only backend rejects RETURN sites before launch.")
    return module, profile, expected


def entry_context(request, *, stage=False):
    """Notebook-only actual tool/PE verification; never skip remote profile pins."""
    module, profile, expected = entry_inputs(request)
    root = Path(request["run_dir"])
    directory = root / "entry"
    effective_path = directory / "profile.json"
    effective = dict(profile)
    libraries = ("client", "drmgr") if profile.get("backend") == "drmgr-entry" else ("client", "drmgr", "drwrap")
    for name in libraries:
        destination = directory / "tools" / Path(profile[name + "_path"]).name
        if stage:
            destination.parent.mkdir(exist_ok=True)
            with Path(profile[name + "_path"]).open("rb") as src, destination.open("xb") as dst:
                shutil.copyfileobj(src, dst)
        pinned(destination, profile[name + "_sha256"])
        effective[name + "_path"] = str(destination)
    if stage:
        with effective_path.open("x", encoding="utf-8") as stream:
            json.dump(effective, stream)
        (directory / "dr-config").mkdir()
        with (directory / "prepared.json").open("x", encoding="utf-8") as stream:
            json.dump({"expected": expected, "profile_sha256": digest(effective_path),
                       "actual_pe_sections_verified": True}, stream)
    if json.loads(effective_path.read_text(encoding="utf-8-sig")) != effective:
        raise ValueError("Staged observer profile changed.")
    module.validate_profile(effective)
    prepared = json.loads((directory / "prepared.json").read_text(encoding="utf-8"))
    if (prepared.get("expected") != expected or prepared.get("profile_sha256") != digest(effective_path)
            or prepared.get("actual_pe_sections_verified") is not True):
        raise ValueError("Entry preparation/PE validation changed.")
    return module, effective, expected


def validate_entry_log(observer, path, expected, profile=None):
    result = {"status": "blocked", "expected": expected,
              "binding_complete": False, "coverage_claim": False}
    try:
        parsed = observer.parse_log(path, expected)
        backend = parsed["header"].get("backend", "drwrap")
        result.update(backend=backend, schema=parsed["header"]["schema"])
        if profile is not None and backend != profile.get("backend", "drwrap"):
            raise ValueError("Observation backend differs from the pinned profile.")
        entries = [event for event in parsed["events"] if event["type"] == "ENTRY"]
        seen = sorted({event["site"] for event in entries})
        result.update(
            summary=parsed["summary"], site_ids_seen=seen,
            missing_site_ids=sorted({site["id"] for site in expected["sites"]} - set(seen)),
            entry_count=len(entries), bytes=Path(path).stat().st_size, sha256=parsed["sha256"],
        )
        if not entries:
            raise ValueError("No actual selected ENTRY was observed.")
        if (parsed["summary"]["context_failures"] != 0 or any(
                event["context_ok"] is not True or
                (event["type"] == "ENTRY" and event.get("return_ok") is not True)
                for event in parsed["events"])):
            raise ValueError("Entry/return context or caller address is unavailable.")
        result["status"] = "validated"
    except (OSError, ValueError, KeyError, TypeError) as error:
        result["reason"] = str(error)
    return result


def inspect_entry(request):
    module, profile, expected = entry_context(request)
    return validate_entry_log(module, Path(request["run_dir"]) / "entry" / "observations.jsonl", expected, profile)


def child_environment(request):
    result = {key: value for key, value in os.environ.items() if key.upper() in ENVIRONMENT}
    if request["mode"] == "entry-observer":
        result["DYNAMORIO_CONFIGDIR"] = str(Path(request["run_dir"]) / "entry" / "dr-config")
    return result


def tool_profile(request):
    if request["mode"] == "entry-observer":
        return entry_context(request)[1]
    profile = request["instrumentation"]
    for prefix in ("drrun", "client", "runtime", "tool_config"):
        pinned(profile[prefix + "_path"], profile[prefix + "_sha256"])
    if (profile["args"] != ["-t", "drcov", "-dump_binary"]
            or profile["logdir_flag"] != "-logdir"
            or profile["pidfile_flag"] != "-pidfile"
            or profile["time_limit_flag"] != "-s"):
        raise ValueError("Unsupported instrumentation launch interface.")
    return profile


def launcher_argv(request, root):
    if request["mode"] not in ("plain", "drcov", "entry-observer"):
        raise ValueError("Unsupported native execution mode.")
    exe = pinned(request["application"]["path"], request["application"]["sha256"])
    project = root / fixture_basename(request, allow_legacy="run_id" not in request)
    pinned(project, request["fixture_sha256"])
    if request["mode"] == "plain":
        return [exe, str(project)]
    profile = tool_profile(request)
    if request.get("instrumentation_start", "launch") in ("post-init", "post-start"):
        return [exe, str(project)]
    return [
        profile["drrun_path"], "-s", str(request["timeout_seconds"]),
        "-pidfile", str(root / "target.pid"), *profile["args"],
        "-logdir", str(root / "coverage"), "--", exe, str(project),
    ]


def image_path(pid):
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
    kernel.OpenProcess.restype = wintypes.HANDLE
    kernel.QueryFullProcessImageNameW.argtypes = [
        wintypes.HANDLE, wintypes.DWORD, wintypes.LPWSTR, ctypes.POINTER(wintypes.DWORD),
    ]
    kernel.CloseHandle.argtypes = [wintypes.HANDLE]
    handle = kernel.OpenProcess(0x1000, False, pid)
    if not handle:
        raise OSError("Cannot inspect the launched target process.")
    try:
        size = wintypes.DWORD(32768)
        name = ctypes.create_unicode_buffer(size.value)
        if not kernel.QueryFullProcessImageNameW(handle, 0, name, ctypes.byref(size)):
            raise OSError("Cannot read the launched target image.")
        return name.value
    finally:
        kernel.CloseHandle(handle)


def creation_ticks(pid):
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
    kernel.OpenProcess.restype = wintypes.HANDLE
    kernel.GetProcessTimes.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4
    kernel.CloseHandle.argtypes = [wintypes.HANDLE]
    handle = kernel.OpenProcess(0x1000, False, pid)
    if not handle:
        raise OSError("Cannot inspect owned process creation time.")
    try:
        values = [wintypes.FILETIME() for _ in range(4)]
        if not kernel.GetProcessTimes(handle, *(ctypes.byref(value) for value in values)):
            raise OSError("Cannot read owned process creation time.")
        return (values[0].dwHighDateTime << 32) | values[0].dwLowDateTime
    finally:
        kernel.CloseHandle(handle)


def interactive_session(pid):
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.ProcessIdToSessionId.argtypes = [wintypes.DWORD, ctypes.POINTER(wintypes.DWORD)]
    value = wintypes.DWORD()
    if not kernel.ProcessIdToSessionId(pid, ctypes.byref(value)):
        raise OSError("Cannot inspect owned process session.")
    return value.value


def held_exit_code(kernel, handle):
    """Inspect only an already-held, signaled process instance, never reopen its PID."""
    result = {"exit_code_available": False}
    if kernel.WaitForSingleObject(handle, 0) != 0:
        return result
    code = wintypes.DWORD()
    if kernel.GetExitCodeProcess(handle, ctypes.byref(code)):
        result.update(
            exit_code_available=True, exit_code=code.value,
            exit_code_hex=f"0x{code.value:08x}",
            exit_code_signed=ctypes.c_int32(code.value).value,
        )
    else:
        result["exit_code_error"] = ctypes.get_last_error()
    return result


def watch_owned(record_path):
    """Keep a handle to this exact instance; PID reuse cannot redirect cleanup."""
    record = json.loads(record_path.read_text(encoding="utf-8-sig"))
    request = json.loads((record_path.parent / "request.json").read_text(encoding="utf-8-sig"))
    pid = record["target_pid"]
    actual_image = image_path(pid)
    expected = ({"path": request["instrumentation"]["drrun_path"],
                 "sha256": request["instrumentation"]["drrun_sha256"]}
                if record.get("role") == "attacher" else request["application"])
    if Path(actual_image) != Path(expected["path"]):
        raise ValueError("Watchdog target image changed.")
    pinned(actual_image, expected["sha256"])
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
    kernel.OpenProcess.restype = wintypes.HANDLE
    kernel.WaitForSingleObject.argtypes = [wintypes.HANDLE, wintypes.DWORD]
    kernel.WaitForSingleObject.restype = wintypes.DWORD
    kernel.GetProcessTimes.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4
    kernel.GetExitCodeProcess.argtypes = [wintypes.HANDLE, ctypes.POINTER(wintypes.DWORD)]
    kernel.GetExitCodeProcess.restype = wintypes.BOOL
    kernel.TerminateProcess.argtypes = [wintypes.HANDLE, wintypes.UINT]
    kernel.CloseHandle.argtypes = [wintypes.HANDLE]
    handle = kernel.OpenProcess(0x100000 | 0x1000 | 1, False, pid)
    if not handle:
        return
    try:
        created, exited, kernel_time, user_time = (wintypes.FILETIME() for _ in range(4))
        if not kernel.GetProcessTimes(handle, ctypes.byref(created), ctypes.byref(exited),
                                      ctypes.byref(kernel_time), ctypes.byref(user_time)):
            raise OSError("Cannot verify watchdog target creation time.")
        epoch = ((created.dwHighDateTime << 32) | created.dwLowDateTime) / 10_000_000 - 11644473600
        ticks = (created.dwHighDateTime << 32) | created.dwLowDateTime
        if record.get("target_creation_ticks") != ticks:
            raise ValueError("Watchdog target creation identity changed.")
        if epoch < record["launch_epoch"] - 1 or epoch > record["launch_epoch"] + 40:
            raise ValueError("Watchdog refuses a reused or pre-existing process.")
        remaining = max(0, record.get("deadline_epoch", record["launch_epoch"] + request["timeout_seconds"]) - time.time())
        wait = kernel.WaitForSingleObject(handle, int(remaining * 1000))
        if wait == 258:
            if not kernel.TerminateProcess(handle, 124):
                raise OSError("Owned bounded target termination failed.")
            status = "owned_timeout_terminated"
        elif wait == 0:
            status = "target_exited"
        else:
            raise OSError("Owned target wait failed.")
        diagnostics = {
            "status": status, "pid": pid, "role": record.get("role", "target"),
            "target_creation_ticks": ticks, "image_sha256": expected["sha256"],
            "observed_exit_epoch": time.time(), **held_exit_code(kernel, handle),
        }
        if kernel.GetProcessTimes(handle, ctypes.byref(created), ctypes.byref(exited),
                                  ctypes.byref(kernel_time), ctypes.byref(user_time)):
            exit_ticks = (exited.dwHighDateTime << 32) | exited.dwLowDateTime
            if exit_ticks:
                diagnostics["process_exit_ticks"] = exit_ticks
        filename = "attach-watchdog-result.json" if record.get("role") == "attacher" else "watchdog-result.json"
        with (record_path.parent / filename).open("x", encoding="utf-8") as stream:
            json.dump(diagnostics, stream)
    finally:
        kernel.CloseHandle(handle)


def start_watchdog(root, record_path, prefix):
    with (root / (prefix + ".log")).open("xb") as log:
        watcher = subprocess.Popen(
            [sys.executable, str(Path(__file__).resolve()), "--watch-owned", str(record_path)],
            cwd=root, env={key: value for key, value in os.environ.items() if key.upper() in ENVIRONMENT},
            stdin=subprocess.DEVNULL, stdout=log, stderr=log, close_fds=True,
            creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0,
        )
    with (root / (prefix + ".json")).open("x", encoding="utf-8") as stream:
        json.dump({"pid": watcher.pid}, stream)


def attach_argv(request, root, target):
    if request["mode"] not in ("drcov", "entry-observer") or request.get("instrumentation_start") not in ("post-init", "post-start"):
        raise ValueError("Post-init attachment must be explicitly selected.")
    profile = tool_profile(request)
    if request["mode"] == "entry-observer":
        module, _, _ = entry_context(request)
        return module.launch_argv(
            profile, root / "entry" / "observer.cfg", root / "entry" / "observations.jsonl",
            request["entry_observer"]["config_sha256"], root / "target.pid",
            seconds=120, attach_pid=target,
        )
    return [
        profile["drrun_path"], "-s", "180", "-attach", str(target),
        "-takeover_sleep", "-takeovers", "32", *profile["args"],
        "-logdir", str(root / "coverage"),
    ]


def attach_owned(request):
    root = Path(request["run_dir"])
    owner = json.loads((root / "launched.json").read_text(encoding="utf-8-sig"))
    target = owner["target_pid"]
    if (Path(image_path(target)) != Path(request["application"]["path"])
            or creation_ticks(target) != owner["target_creation_ticks"]
            or interactive_session(target) != interactive_session(os.getpid())
            or interactive_session(target) == 0):
        raise ValueError("Post-init attachment target identity/session changed.")
    pinned(request["application"]["path"], request["application"]["sha256"])
    argv = attach_argv(request, root, target)
    started = time.time()
    with (root / "attach-stdout.log").open("xb") as out, (root / "attach-stderr.log").open("xb") as err:
        process = subprocess.Popen(
            argv, cwd=root,
            env=child_environment(request),
            stdin=subprocess.DEVNULL, stdout=out, stderr=err, close_fds=True,
            creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0,
        )
    watch_record = {
        "role": "attacher", "target_pid": process.pid, "launch_epoch": started,
        "target_creation_ticks": creation_ticks(process.pid),
        "deadline_epoch": owner["launch_epoch"] + request["timeout_seconds"],
    }
    watch_path = root / "attach-watchdog-owned.json"
    with watch_path.open("x", encoding="utf-8") as stream:
        json.dump(watch_record, stream)
    start_watchdog(root, watch_path, "attach-watchdog")
    record = {
        "target_pid": target, "target_creation_ticks": owner["target_creation_ticks"],
        "attacher_pid": process.pid, "attacher_creation_ticks": watch_record["target_creation_ticks"],
        "attach_start_epoch": started, "argv": argv,         "scope": ("Post-SimulationStart attachment; application startup and SimulationStart are excluded."
                  if request.get("instrumentation_start") == "post-start"
                  else "Post-project-load attachment before Simulation; startup is excluded."),
        "independent_attach_deadline_seconds": 30, "status": "blocked",
    }
    until = time.monotonic() + 30
    try:
        while time.monotonic() < until:
            if creation_ticks(target) != owner["target_creation_ticks"]:
                raise ValueError("Native target identity changed during attachment.")
            ready = ((root / "entry" / "observations.jsonl").is_file()
                     if request["mode"] == "entry-observer"
                     else bool(list((root / "coverage").glob("*.log"))))
            if ready:
                record["status"] = "client_log_opened"
                record["readiness_only_not_observation_proof"] = True
                break
            if process.poll() is not None:
                raise RuntimeError("Instrumentation attacher exited before opening a native log.")
            time.sleep(0.1)
        else:
            raise TimeoutError("Independent post-init attachment deadline expired.")
    except (OSError, ValueError, RuntimeError) as error:
        record["reason"] = str(error)
        if process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=3)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=3)
    finally:
        record["attach_end_epoch"] = time.time()
        with (root / "attached.json").open("x", encoding="utf-8") as stream:
            json.dump(record, stream)
    return record


def exit_dump_argv(request, target):
    if request["mode"] == "entry-observer":
        profile = tool_profile(request)
        return [profile["drconfig_path"], "-nudge_timeout", "5000",
                "-nudge_pid", str(target), profile["nudge_client_id"], profile["nudge_argument"]]
    profile = request.get("exit_dump_method", {})
    if (profile.get("schema_version") != 1 or profile.get("kind") != "dynamorio-drcov-exit-dump-profile"
            or profile.get("console_runtime_verified") is not True
            or profile.get("effect") != "process_exit_and_drcov_dump_not_live_snapshot"
            or profile.get("independent_timeout_seconds") != 8
            or profile.get("args_template") != [
                "-nudge_timeout", "5000", "-nudge_pid", "{verified_owned_target_pid}", "00000000", "1",
            ]):
        raise ValueError("Use the independently console-verified bounded exit-and-dump profile.")
    executable = pinned(profile["drconfig_path"], profile["drconfig_sha256"])
    return [executable, "-nudge_timeout", "5000", "-nudge_pid", str(target), "00000000", "1"]


def exit_dump_owned(request):
    root = Path(request["run_dir"])
    owner = json.loads((root / "launched.json").read_text(encoding="utf-8-sig"))
    target = owner["target_pid"]
    if (Path(image_path(target)) != Path(request["application"]["path"])
            or creation_ticks(target) != owner["target_creation_ticks"]
            or interactive_session(target) != interactive_session(os.getpid())
            or interactive_session(target) == 0):
        raise ValueError("Exit-and-dump target identity/session changed.")
    pinned(request["application"]["path"], request["application"]["sha256"])
    argv = exit_dump_argv(request, target)
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
    kernel.OpenProcess.restype = wintypes.HANDLE
    kernel.GetProcessTimes.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4
    kernel.WaitForSingleObject.argtypes = [wintypes.HANDLE, wintypes.DWORD]
    kernel.WaitForSingleObject.restype = wintypes.DWORD
    kernel.CloseHandle.argtypes = [wintypes.HANDLE]
    handle = kernel.OpenProcess(0x100000 | 0x1000, False, target)
    if not handle:
        raise OSError("Cannot retain exact owned target handle for bounded exit-and-dump.")
    record = {
        "target_pid": target, "target_creation_ticks": owner["target_creation_ticks"],
        "argv": argv, "independent_timeout_seconds": 8, "start_epoch": time.time(),
        "effect": ENTRY_EFFECT if request["mode"] == "entry-observer" else "process_exit_and_drcov_dump_not_live_snapshot",
        "status": "blocked",
    }
    try:
        times = [wintypes.FILETIME() for _ in range(4)]
        if not kernel.GetProcessTimes(handle, *(ctypes.byref(value) for value in times)):
            raise OSError("Cannot verify retained target handle creation time.")
        if ((times[0].dwHighDateTime << 32) | times[0].dwLowDateTime) != owner["target_creation_ticks"]:
            raise ValueError("Retained exit-and-dump target handle identity changed.")
        result = subprocess.run(
            argv, cwd=root,
            env=child_environment(request),
            stdin=subprocess.DEVNULL, capture_output=True, timeout=8, check=False,
            creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0,
        )
        (root / "exit-dump.stdout.log").write_bytes(result.stdout)
        (root / "exit-dump.stderr.log").write_bytes(result.stderr)
        record["exit_code"] = result.returncode
        record["target_exited"] = kernel.WaitForSingleObject(handle, 5000) == 0
        if request["mode"] == "entry-observer":
            record["entry_observation"] = inspect_entry(request)
            if (result.returncode == 0 and record["target_exited"]
                    and record["entry_observation"]["status"] == "validated"):
                record["status"] = "exited_and_observer_flushed"
        else:
            record["nonempty_logs"] = [str(file) for file in (root / "coverage").glob("*.log") if file.stat().st_size > 0]
            if result.returncode == 0 and record["target_exited"] and record["nonempty_logs"]:
                record["status"] = "exited_and_dumped"
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        record["reason"] = str(error)
    finally:
        kernel.CloseHandle(handle)
        record["end_epoch"] = time.time()
        with (root / "exit-dump.json").open("x", encoding="utf-8") as stream:
            json.dump(record, stream)
    return record


def launch(request):
    root = Path(request["run_dir"])
    if request["mode"] == "entry-observer":
        entry_context(request, stage=True)
    argv = launcher_argv(request, root)
    started = time.time()
    with (root / "child-stdout.log").open("xb") as out, (root / "child-stderr.log").open("xb") as err:
        console_launcher = request["mode"] == "drcov" and request.get("instrumentation_start", "launch") == "launch"
        process = subprocess.Popen(
            argv, cwd=root,
            env=child_environment(request),
            stdin=subprocess.DEVNULL, stdout=out, stderr=err, close_fds=True,
            creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" and console_launcher else 0,
        )
    ownership = {
        "launcher_pid": process.pid, "target_pid": process.pid if request["mode"] == "plain" or request.get("instrumentation_start") in ("post-init", "post-start") else None,
        "launch_epoch": started, "argv": argv, "mode": request["mode"],
    }
    owner_path = root / "launcher.json"
    with owner_path.open("x", encoding="utf-8") as stream:
        json.dump(ownership, stream)
    if request["mode"] == "drcov" and request.get("instrumentation_start", "launch") == "launch":
        until = time.monotonic() + 40
        while time.monotonic() < until:
            pidfile = root / "target.pid"
            if pidfile.exists():
                value = pidfile.read_text(encoding="ascii").strip()
                if value.isdecimal() and int(value) > 0:
                    ownership["target_pid"] = int(value)
                    break
            if process.poll() is not None:
                raise RuntimeError("Instrumentation launcher exited before its target PID was available.")
            time.sleep(0.2)
        else:
            raise TimeoutError("Instrumentation target PID file was not produced.")
    actual_image = image_path(ownership["target_pid"])
    if Path(actual_image) != Path(request["application"]["path"]):
        raise ValueError("Target PID is not the requested executable.")
    pinned(actual_image, request["application"]["sha256"])
    ownership["target_creation_ticks"] = creation_ticks(ownership["target_pid"])
    with (root / "launched.json").open("x", encoding="utf-8") as stream:
        json.dump(ownership, stream)
    if request["mode"] == "plain" or request.get("instrumentation_start") in ("post-init", "post-start"):
        start_watchdog(root, root / "launched.json", "watchdog")
    return ownership


def record_launcher(request_path):
    """Capture the existing launcher worker's actual streams/status, including preflight errors."""
    root = request_path.resolve().parent
    paths = {name: root / ("sanitized-launcher." + name) for name in (
        "stdout.log", "stderr.log", "json")}
    if any(path.exists() for path in paths.values()):
        raise ValueError("Immutable sanitized-launcher diagnostics already exist.")
    record = {
        "schema_version": 1, "kind": "sanitized-native-launcher-diagnostics",
        "status": "blocked", "exit_code": None, "start_epoch": time.time(),
        "request_sha256": digest(request_path), "runner_sha256": digest(__file__),
        "argv": [sys.executable, "-B", str(Path(__file__).resolve()), str(request_path.resolve())],
    }
    ownership = None
    with paths["stdout.log"].open("xb") as out, paths["stderr.log"].open("xb") as err:
        try:
            request = json.loads(request_path.read_text(encoding="utf-8-sig"))
            if Path(request["run_dir"]).resolve() != root:
                raise ValueError("Launcher diagnostic root differs from its immutable request.")
            timeout = request["timeout_seconds"]
            if type(timeout) is not int or not 1 <= timeout <= 300:
                raise ValueError("Launcher deadline must remain within 300 seconds.")
            result = subprocess.run(
                record["argv"], cwd=root, env=child_environment(request),
                stdin=subprocess.DEVNULL, stdout=out, stderr=err, close_fds=True,
                timeout=timeout, check=False,
                creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0,
            )
            record["exit_code"] = result.returncode
        except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
            record["reason"] = f"{type(error).__name__}: {error}"
            record["worker_exception"] = True
    stdout = paths["stdout.log"].read_text(encoding="utf-8-sig", errors="replace")
    stderr = paths["stderr.log"].read_text(encoding="utf-8-sig", errors="replace")
    if record["exit_code"] == 0:
        try:
            ownership = json.loads(stdout)
            if (not isinstance(ownership, dict) or type(ownership.get("target_pid")) is not int
                    or ownership["target_pid"] <= 0):
                raise ValueError("Launcher stdout lacks its exact owned target PID.")
            record["status"] = "launched"
        except (ValueError, TypeError) as error:
            record["reason"] = f"Invalid launcher stdout: {error}"
    elif "reason" not in record:
        lines = stderr.strip().splitlines() or stdout.strip().splitlines()
        record["reason"] = lines[-1][-2000:] if lines else "Launcher exited without diagnostic text."
    record.update(
        end_epoch=time.time(),
        stdout={"path": paths["stdout.log"].name, "sha256": digest(paths["stdout.log"]),
                "bytes": paths["stdout.log"].stat().st_size},
        stderr={"path": paths["stderr.log"].name, "sha256": digest(paths["stderr.log"]),
                "bytes": paths["stderr.log"].stat().st_size},
    )
    with paths["json"].open("x", encoding="utf-8") as stream:
        json.dump(record, stream)
    return {**record, "ownership": ownership}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("request", type=Path, nargs="?")
    parser.add_argument("--watch-owned", type=Path)
    parser.add_argument("--attach-owned", type=Path)
    parser.add_argument("--exit-dump-owned", type=Path)
    parser.add_argument("--recovery-dialog", type=Path)
    parser.add_argument("--inspect-entry", type=Path)
    parser.add_argument("--record-launch", type=Path)
    args = parser.parse_args()
    if args.record_launch:
        record = record_launcher(args.record_launch)
        print(json.dumps(record))
        return 0 if record["status"] == "launched" else 2
    if args.watch_owned:
        watch_owned(args.watch_owned)
        return
    if args.attach_owned:
        request = json.loads(args.attach_owned.read_text(encoding="utf-8-sig"))
        print(json.dumps(attach_owned(request)))
        return
    if args.exit_dump_owned:
        request = json.loads(args.exit_dump_owned.read_text(encoding="utf-8-sig"))
        print(json.dumps(exit_dump_owned(request)))
        return
    if args.inspect_entry:
        request = json.loads(args.inspect_entry.read_text(encoding="utf-8-sig"))
        print(json.dumps(inspect_entry(request)))
        return
    if args.request is None:
        parser.error("request is required unless --watch-owned is selected")
    request = json.loads(args.request.read_text(encoding="utf-8-sig"))
    if args.recovery_dialog:
        record = json.loads(args.recovery_dialog.read_text(encoding="utf-8-sig"))
        print(json.dumps(validate_recovery_dialog(request, record)))
        return
    print(json.dumps(launch(request)))


if __name__ == "__main__":
    sys.exit(main())
