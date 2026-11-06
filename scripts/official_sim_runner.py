#!/usr/bin/env python3
"""Interactive-context child launcher for official-sim.py (never an oracle)."""

import argparse
import ctypes
from ctypes import wintypes
import hashlib
import json
import os
from pathlib import Path
import re
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


def tool_profile(request):
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
    if request["mode"] not in ("plain", "drcov"):
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
        filename = "attach-watchdog-result.json" if record.get("role") == "attacher" else "watchdog-result.json"
        with (record_path.parent / filename).open("x", encoding="utf-8") as stream:
            json.dump({"status": status, "pid": pid}, stream)
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
    if request["mode"] != "drcov" or request.get("instrumentation_start") not in ("post-init", "post-start"):
        raise ValueError("Post-init attachment must be explicitly selected.")
    profile = tool_profile(request)
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
            env={key: value for key, value in os.environ.items() if key.upper() in ENVIRONMENT},
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
            if list((root / "coverage").glob("*.log")):
                record["status"] = "client_log_opened"
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
        "effect": "process_exit_and_drcov_dump_not_live_snapshot", "status": "blocked",
    }
    try:
        times = [wintypes.FILETIME() for _ in range(4)]
        if not kernel.GetProcessTimes(handle, *(ctypes.byref(value) for value in times)):
            raise OSError("Cannot verify retained target handle creation time.")
        if ((times[0].dwHighDateTime << 32) | times[0].dwLowDateTime) != owner["target_creation_ticks"]:
            raise ValueError("Retained exit-and-dump target handle identity changed.")
        result = subprocess.run(
            argv, cwd=root,
            env={key: value for key, value in os.environ.items() if key.upper() in ENVIRONMENT},
            stdin=subprocess.DEVNULL, capture_output=True, timeout=8, check=False,
            creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0,
        )
        (root / "exit-dump.stdout.log").write_bytes(result.stdout)
        (root / "exit-dump.stderr.log").write_bytes(result.stderr)
        record["exit_code"] = result.returncode
        record["target_exited"] = kernel.WaitForSingleObject(handle, 5000) == 0
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
    argv = launcher_argv(request, root)
    started = time.time()
    with (root / "child-stdout.log").open("xb") as out, (root / "child-stderr.log").open("xb") as err:
        console_launcher = request["mode"] == "drcov" and request.get("instrumentation_start", "launch") == "launch"
        process = subprocess.Popen(
            argv, cwd=root,
            env={key: value for key, value in os.environ.items() if key.upper() in ENVIRONMENT},
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


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("request", type=Path, nargs="?")
    parser.add_argument("--watch-owned", type=Path)
    parser.add_argument("--attach-owned", type=Path)
    parser.add_argument("--exit-dump-owned", type=Path)
    parser.add_argument("--recovery-dialog", type=Path)
    args = parser.parse_args()
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
    if args.request is None:
        parser.error("request is required unless --watch-owned is selected")
    request = json.loads(args.request.read_text(encoding="utf-8-sig"))
    if args.recovery_dialog:
        record = json.loads(args.recovery_dialog.read_text(encoding="utf-8-sig"))
        print(json.dumps(validate_recovery_dialog(request, record)))
        return
    print(json.dumps(launch(request)))


if __name__ == "__main__":
    main()
