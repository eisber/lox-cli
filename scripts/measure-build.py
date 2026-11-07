#!/usr/bin/env python3
"""Run one isolated Cargo workload and append reproducible measurements."""

import argparse
import ctypes
import datetime
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import time


def windows_family_sampler():
    from ctypes import wintypes

    class ProcessEntry(ctypes.Structure):
        _fields_ = [
            ("dwSize", wintypes.DWORD),
            ("cntUsage", wintypes.DWORD),
            ("th32ProcessID", wintypes.DWORD),
            ("th32DefaultHeapID", ctypes.c_size_t),
            ("th32ModuleID", wintypes.DWORD),
            ("cntThreads", wintypes.DWORD),
            ("th32ParentProcessID", wintypes.DWORD),
            ("pcPriClassBase", wintypes.LONG),
            ("dwFlags", wintypes.DWORD),
            ("szExeFile", wintypes.WCHAR * 260),
        ]

    class MemoryCounters(ctypes.Structure):
        _fields_ = [
            ("cb", wintypes.DWORD),
            ("PageFaultCount", wintypes.DWORD),
            ("PeakWorkingSetSize", ctypes.c_size_t),
            ("WorkingSetSize", ctypes.c_size_t),
            ("QuotaPeakPagedPoolUsage", ctypes.c_size_t),
            ("QuotaPagedPoolUsage", ctypes.c_size_t),
            ("QuotaPeakNonPagedPoolUsage", ctypes.c_size_t),
            ("QuotaNonPagedPoolUsage", ctypes.c_size_t),
            ("PagefileUsage", ctypes.c_size_t),
            ("PeakPagefileUsage", ctypes.c_size_t),
        ]

    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    psapi = ctypes.WinDLL("psapi", use_last_error=True)
    kernel.CreateToolhelp32Snapshot.argtypes = [wintypes.DWORD, wintypes.DWORD]
    kernel.CreateToolhelp32Snapshot.restype = wintypes.HANDLE
    kernel.Process32FirstW.argtypes = [wintypes.HANDLE, ctypes.POINTER(ProcessEntry)]
    kernel.Process32NextW.argtypes = [wintypes.HANDLE, ctypes.POINTER(ProcessEntry)]
    kernel.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
    kernel.OpenProcess.restype = wintypes.HANDLE
    kernel.CloseHandle.argtypes = [wintypes.HANDLE]
    kernel.GetProcessTimes.argtypes = [wintypes.HANDLE] + [
        ctypes.POINTER(wintypes.FILETIME)
    ] * 4
    psapi.GetProcessMemoryInfo.argtypes = [
        wintypes.HANDLE,
        ctypes.POINTER(MemoryCounters),
        wintypes.DWORD,
    ]
    cpu_by_process = {}
    family = set()
    peak_bytes = 0

    def sample(root_pid):
        nonlocal peak_bytes, family
        family.add(root_pid)
        snapshot = kernel.CreateToolhelp32Snapshot(2, 0)
        if snapshot == ctypes.c_void_p(-1).value:
            return
        entry = ProcessEntry()
        entry.dwSize = ctypes.sizeof(entry)
        processes = {}
        try:
            valid = kernel.Process32FirstW(snapshot, ctypes.byref(entry))
            while valid:
                processes[entry.th32ProcessID] = entry.th32ParentProcessID
                valid = kernel.Process32NextW(snapshot, ctypes.byref(entry))
        finally:
            kernel.CloseHandle(snapshot)
        live_family = {pid for pid in family if pid in processes}
        while True:
            descendants = {
                pid for pid, parent in processes.items() if parent in live_family
            }
            if descendants <= live_family:
                break
            live_family.update(descendants)
        family = live_family
        memory_bytes = 0
        for pid in live_family:
            handle = kernel.OpenProcess(0x410, False, pid)
            if not handle:
                continue
            try:
                times = [wintypes.FILETIME() for _ in range(4)]
                if kernel.GetProcessTimes(
                    handle, *(ctypes.byref(value) for value in times)
                ):
                    ticks = [
                        (value.dwHighDateTime << 32) | value.dwLowDateTime
                        for value in times
                    ]
                    # Creation time disambiguates reused process IDs.
                    cpu_by_process[(pid, ticks[0])] = (ticks[2] + ticks[3]) / 1e7
                counters = MemoryCounters()
                counters.cb = ctypes.sizeof(counters)
                if psapi.GetProcessMemoryInfo(
                    handle, ctypes.byref(counters), counters.cb
                ):
                    memory_bytes += counters.WorkingSetSize
            finally:
                kernel.CloseHandle(handle)
        peak_bytes = max(peak_bytes, memory_bytes)

    def result():
        return {
            "sampled_family_cpu_seconds": round(sum(cpu_by_process.values()), 3),
            "sampled_peak_family_working_set_bytes": peak_bytes,
            "resource_metric_note": (
                "Windows process-family samples; short-lived/exited children and "
                "between-sample peaks can be missed; CPU is a lower bound."
            ),
        }

    return sample, result


def toolchain_metadata(command, executable, *, cwd, env):
    direct_cargo = Path(executable).name.casefold() in {"cargo", "cargo.exe"}
    selector = (
        command[1]
        if direct_cargo and len(command) > 1
        and command[1].startswith("+") and len(command[1]) > 1
        else None
    )
    cargo_probe = [executable if direct_cargo else "cargo"]
    if selector:
        cargo_probe.append(selector)
    cargo_probe.append("-V")
    rustc_probe = [env.get("RUSTC", "rustc")]
    rustc_probe.append("-Vv")
    if selector:
        rustc_probe = ["rustup", "run", selector[1:], *rustc_probe]
    return {
        "rustc": subprocess.check_output(
            rustc_probe, cwd=cwd, env=env, text=True
        ).strip(),
        "cargo": subprocess.check_output(
            cargo_probe, cwd=cwd, env=env, text=True
        ).strip(),
        "toolchain_selector": selector,
        "toolchain_metadata_scope": (
            "direct-cargo" if direct_cargo else "workload-cwd/environment-context"
        ),
        "toolchain_metadata_note": (
            "Version probes use the workload cwd/environment. Explicit direct-Cargo "
            "selectors are forwarded to Cargo and queried with rustup run for the "
            "configured compiler. RUSTC overrides are probed as configured. "
            "Cargo-configured compiler overrides, compiler wrappers, and toolchains "
            "selected inside opaque commands are not introspected."
        ),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--label", required=True)
    parser.add_argument("--cwd", type=Path, default=Path.cwd())
    parser.add_argument("--target-dir", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--jobs", type=int)
    parser.add_argument("--sample-seconds", type=float, default=0.1)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command
    if command and command[0] == "--":
        command = command[1:]
    if not command or args.sample_seconds <= 0 or (
        args.jobs is not None and args.jobs <= 0
    ):
        parser.error("supply a command and positive sampling interval/job count")
    cwd = args.cwd.resolve()
    target = args.target_dir.resolve()
    common_git = Path(subprocess.check_output(
        ["git", "rev-parse", "--git-common-dir"], cwd=cwd, text=True
    ).strip())
    if not common_git.is_absolute():
        common_git = cwd / common_git
    if target in {
        (cwd / "target").resolve(), (common_git.resolve().parent / "target").resolve()
    }:
        parser.error("use an isolated target directory, not the workspace target")
    output = args.output.resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ, CARGO_TARGET_DIR=str(target), CARGO_TERM_COLOR="never")
    if args.jobs is not None:
        env["CARGO_BUILD_JOBS"] = str(args.jobs)
    executable = shutil.which(command[0], path=env.get("PATH"))
    if executable is None:
        parser.error(f"command not found: {command[0]}")
    target.mkdir(parents=True, exist_ok=True)
    binding = target / ".measurement-source.json"
    if binding.exists():
        previous = json.loads(binding.read_text(encoding="utf-8"))
        if Path(previous["cwd"]).resolve() != cwd:
            parser.error(
                f"target belongs to {previous['cwd']}; choose a new target for {cwd}"
            )
    else:
        if any(target.iterdir()):
            parser.error(
                "nonempty target has no source binding; choose a new empty target"
            )
        binding.write_text(json.dumps({"cwd": str(cwd)}), encoding="utf-8")
    sample, resource_result = (
        windows_family_sampler()
        if sys.platform == "win32"
        else (
            lambda _: None,
            lambda: {
                "sampled_family_cpu_seconds": None,
                "sampled_peak_family_working_set_bytes": None,
                "resource_metric_note": "Resource metrics unmeasured on this platform.",
            },
        )
    )
    log = output.with_name(f"{output.stem}-{args.label}.log")
    if log.exists():
        log = log.with_name(f"{log.stem}-{time.time_ns()}.log")
    source_diff = subprocess.check_output(["git", "diff", "--binary"], cwd=cwd)
    patch = log.with_suffix(".patch")
    patch.write_bytes(source_diff)
    record = {
        "label": args.label,
        "started_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "cwd": str(cwd),
        "target_dir": str(target),
        "command": command,
        "git_head": subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=cwd, text=True
        ).strip(),
        "git_status": subprocess.check_output(
            ["git", "status", "--short"], cwd=cwd, text=True
        ).strip(),
        "source_diff_sha256": hashlib.sha256(source_diff).hexdigest(),
        "source_diff_file": str(patch),
        **toolchain_metadata(command, executable, cwd=cwd, env=env),
        "cargo_environment": {
            name: value
            for name, value in env.items()
            if name in {
                "CARGO_TARGET_DIR", "CARGO_BUILD_JOBS", "CARGO_INCREMENTAL",
                "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTUP_TOOLCHAIN",
                "RUSTC", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER",
            }
        },
        "logical_processors": os.cpu_count(),
        "sampling_interval_seconds": args.sample_seconds,
        "log": str(log),
    }
    start = time.perf_counter()
    with log.open("w", encoding="utf-8") as stream:
        process = subprocess.Popen(
            [executable, *command[1:]], cwd=cwd, env=env,
            stdout=stream, stderr=subprocess.STDOUT,
            start_new_session=sys.platform != "win32",
        )
        try:
            while process.poll() is None:
                sample(process.pid)
                time.sleep(args.sample_seconds)
        finally:
            if process.poll() is None:
                if sys.platform == "win32":
                    subprocess.run(
                        ["taskkill", "/PID", str(process.pid), "/T", "/F"],
                        check=False, stdout=subprocess.DEVNULL,
                    )
                else:
                    os.killpg(process.pid, signal.SIGTERM)
                process.wait()
    record.update(resource_result())
    record.update(
        elapsed_seconds=round(time.perf_counter() - start, 3),
        exit_code=process.returncode,
    )
    with output.open("a", encoding="utf-8") as stream:
        stream.write(json.dumps(record) + "\n")
    print(json.dumps(record, indent=2))
    if process.returncode:
        print(log.read_text(encoding="utf-8", errors="replace")[-12000:])
    return process.returncode


if __name__ == "__main__":
    sys.exit(main())
