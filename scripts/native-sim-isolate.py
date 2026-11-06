"""Console-only, pinned layer/TLS isolation for the native entry observer."""

import argparse
import hashlib
import importlib.util
import json
import ntpath
import os
from pathlib import Path
import re
import subprocess
import time


SPEC = importlib.util.spec_from_file_location(
    "isolation_observer", Path(__file__).with_name("native-sim-observe.py"))
OBS = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(OBS)
KINDS = ("native", "no-client", "drcov", "noop", "drmgr", "observer")
TLS_FIELDS = {
    "thread", "teb", "array", "index", "slot_read", "slot", "offset_known",
    "offset", "marker_read", "marker", "dr_query", "under_dr",
}


def tls_rows(raw):
    if raw and not raw.endswith(b"\n"):
        raise ValueError("partial fixture output")
    rows = {}
    for line in raw.decode("ascii").splitlines():
        if not line.startswith("TLS1 "):
            continue
        pairs = [part.split("=", 1) for part in line.split()[1:]]
        if any(len(pair) != 2 for pair in pairs):
            raise ValueError("malformed fixture TLS line")
        fields = dict(pairs)
        if len(fields) != len(pairs) or set(fields) != TLS_FIELDS | {"stage", "role"}:
            raise ValueError("unknown or duplicate fixture TLS field")
        if fields["stage"] not in ("ready", "attached", "triggered") or fields["role"] not in ("main", "timer"):
            raise ValueError("unknown fixture TLS stage/thread")
        key = (fields.pop("stage"), fields.pop("role"))
        if key in rows or any(not re.fullmatch(r"[0-9a-f]{1,16}", value) for value in fields.values()):
            raise ValueError("duplicate or invalid fixture TLS observation")
        rows[key] = {name: int(value, 16) for name, value in fields.items()}
    return rows


def evaluate_stdout(raw, kind, expected_return=None):
    rows = tls_rows(raw)
    wanted = {(stage, role) for stage in ("ready", "attached", "triggered") for role in ("main", "timer")}
    if set(rows) != wanted:
        raise ValueError("fixture did not capture both threads at all TLS stages")
    for row in rows.values():
        if (row["slot_read"], row["offset_known"], row["marker_read"], row["marker"]) != (
                1, 1, 1, 0x1122334455667788):
            raise ValueError("fixture TLS slot/marker was unavailable or changed before its calls")
    matches = re.findall(
        rb"^fixture-v1 value=([0-9a-f]{16}) clock=([0-9a-f]{16}) calls=80 "
        rb"return_rva=([0-9a-f]+) fp_sentinels=1 tls=1 fp_calls=64\r?$", raw, re.M)
    if len(matches) != 1:
        raise ValueError("fixture FP/TLS/output completion is missing or ambiguous")
    value, clock, returned = (int(part, 16) for part in matches[0])
    if value != 0x0A7FAC99710CDF63 or clock != 0x0123456789ABCDEF:
        raise ValueError("fixture output changed under instrumentation")
    if expected_return is not None and returned != expected_return:
        raise ValueError("fixture caller return RVA differs from the native control")
    return rows, returned


def thread_control_witnessed(rows):
    return all((rows[stage, role]["dr_query"], rows[stage, role]["under_dr"]) == (1, 1)
               for stage in ("attached", "triggered") for role in ("main", "timer"))


def layer_argv(kind, profile, clients, directory, fixture, *, pid=None):
    if kind not in KINDS:
        raise ValueError("unknown isolation layer")
    if kind == "native":
        return [str(fixture), "--matrix-trigger", str(directory / "fixture-trigger")]
    argv = [profile["drrun_path"], "-s", "60"]
    if pid is None:
        argv += ["-pidfile", str(directory / "target.pid")]
    else:
        OBS.integer(pid, 1, 0xFFFFFFFF, "owned PID")
        argv += ["-attach", str(pid), "-takeover_sleep", "-takeovers", "32"]
    if kind == "drcov":
        argv += ["-t", "drcov", "-dump_binary", "-logdir", str(directory / "coverage")]
    elif kind in ("noop", "drmgr"):
        argv += ["-c", clients[kind + "_path"]]
    elif kind == "observer":
        argv += ["-c", profile["client_path"], str(directory / "observer.cfg"),
                 str(directory / "events.jsonl"), OBS.sha256(directory / "observer.cfg")]
    if pid is None:
        argv += ["--", str(fixture), "--matrix-trigger", str(directory / "fixture-trigger")]
    return argv


def wait_for(condition, target, seconds, reason):
    deadline = time.monotonic() + seconds
    while not condition():
        target.assert_same()
        if time.monotonic() >= deadline:
            raise TimeoutError(reason)
        time.sleep(0.025)


def run_case(kind, mode, profile, clients, fixture, pe, directory, expected_return):
    directory.mkdir()
    (directory / "coverage").mkdir()
    drdir = directory / "dr-config"
    drdir.mkdir()
    env = OBS.sanitized_env(drdir)
    trigger = directory / "fixture-trigger"
    if kind == "observer":
        sites = [{"id": index, "rva": pe["exports"][name], "post": 0, "mask": mask}
                 for index, name, mask in ((1, "observe_entry", 13), (2, "observe_clock", 0),
                                          (3, "observe_fp", 0))]
        (directory / "observer.cfg").write_bytes(
            OBS.config_bytes(pe, profile["client_sha256"], sites, 256, 16, 32))
    record = {"kind": kind, "mode": mode, "status": "blocked", "timing": [],
              "scope": "Independent console fixture only; not vendor GUI admission."}
    processes, handles = [], []
    target = None
    target_owned = False
    began = time.monotonic()
    try:
        with (directory / "fixture.stdout").open("xb") as stdout, \
                (directory / "fixture.stderr").open("xb") as stderr, \
                (directory / "attach.stdout").open("xb") as aout, \
                (directory / "attach.stderr").open("xb") as aerr:
            normal = mode == "normal" or kind == "native"
            argv = (layer_argv(kind, profile, clients, directory, fixture) if normal else
                    [str(fixture), "--matrix-trigger", str(trigger)])
            record["start_argv"] = argv
            process = subprocess.Popen(argv, stdout=stdout, stderr=stderr, env=env,
                                       shell=False, close_fds=True, creationflags=OBS.CREATE_NO_WINDOW)
            processes.append(process)
            first = OBS.OwnedProcess(process.pid)
            handles.append(first)
            expected_image = str(fixture) if kind == "native" or not normal else profile["drrun_path"]
            if (ntpath.normcase(first.identity["image_path"]) != ntpath.normcase(expected_image)
                    or first.identity["session_id"] != 0 or OBS.parent_pid(process.pid) != os.getpid()):
                raise ValueError("owned launch image/session/ancestry mismatch")
            if normal and kind != "native":
                pidfile = directory / "target.pid"
                wait_for(pidfile.exists, first, 10, "owned target PID file missing")
                value = pidfile.read_text(encoding="utf-8-sig").strip()
                if not value.isdecimal():
                    raise ValueError("invalid target PID")
                target = OBS.OwnedProcess(int(value))
                handles.append(target)
                if OBS.parent_pid(target.pid) != first.pid:
                    raise ValueError("instrumented target is not the owned launcher's child")
            else:
                target = first
            if (ntpath.normcase(target.identity["image_path"]) != ntpath.normcase(str(fixture))
                    or OBS.sha256(target.identity["image_path"]) != pe["sha256"]
                    or target.identity["session_id"] != 0):
                raise ValueError("fixture image/hash/session mismatch")
            target_owned = True
            record["target"] = target.identity
            wait_for(lambda: b"fixture-v1 ready" in (directory / "fixture.stdout").read_bytes(),
                     target, 5, "fixture initialization not witnessed")
            if not normal:
                argv = layer_argv(kind, profile, clients, directory, fixture, pid=target.pid)
                record["attach_argv"] = argv
                attacher = subprocess.Popen(argv, stdout=aout, stderr=aerr, env=env,
                                            shell=False, close_fds=True, creationflags=OBS.CREATE_NO_WINDOW)
                processes.append(attacher)
                handle = OBS.OwnedProcess(attacher.pid)
                handles.append(handle)
                if (ntpath.normcase(handle.identity["image_path"]) != ntpath.normcase(profile["drrun_path"])
                        or handle.identity["session_id"] != 0 or OBS.parent_pid(handle.pid) != os.getpid()):
                    raise ValueError("owned attacher image/session/ancestry mismatch")
                record["attacher"] = handle.identity
                # A delay is not attachment proof; the fixture's public SDK query below is.
                time.sleep(1)
                target.assert_same()
            with Path(str(trigger) + ".probe").open("xb") as stream:
                stream.write(b"owned fixture TLS probe\n")
            def probed():
                raw = (directory / "fixture.stdout").read_bytes()
                return raw.endswith(b"\n") and len([
                    key for key in tls_rows(raw) if key[0] == "attached"]) == 2

            wait_for(probed,
                     target, 5, "both fixture-thread TLS probes were not witnessed")
            with trigger.open("xb") as stream:
                stream.write(b"owned fixture calls\n")
            if target.k.WaitForSingleObject(target.handle, 12000) != 0:
                raise TimeoutError("fixture did not complete within its deadline")
            record["target_exit_code"] = target.exit_code()
            rows = tls_rows((directory / "fixture.stdout").read_bytes())
            record["tls"] = [{"stage": stage, "role": role, **value}
                             for (stage, role), value in rows.items()]
            record["target_exit_hex"] = f"0x{record['target_exit_code']:08x}"
            if record["target_exit_code"] != 0:
                raise RuntimeError("fixture exited " + record["target_exit_hex"])
            rows, returned = evaluate_stdout((directory / "fixture.stdout").read_bytes(), kind, expected_return)
            record["return_rva"] = returned
            record["fixture_integrity_verified"] = True
            record["instrumentation_verified"] = kind == "native" or thread_control_witnessed(rows)
            if kind == "observer":
                config = OBS.read_config(directory / "observer.cfg")
                parsed = OBS.parse_log(directory / "events.jsonl", config)
                OBS.verify_fixture(parsed, returned)
                record["instrumentation_verified"] = True
                record["instrumentation_evidence"] = {
                    "kind": "exact entry/caller/prefix and two-thread FP entry evidence",
                    "log_sha256": parsed["sha256"],
                }
            for child in processes:
                code = child.wait(timeout=5)
                if code != 0:
                    raise RuntimeError(f"owned launcher/attacher exited {code}")
            if any((directory / name).stat().st_size for name in ("fixture.stderr", "attach.stderr")):
                raise ValueError("fixture or injector stderr is not empty")
            record["status"] = "pass" if record["instrumentation_verified"] else "observed"
            if record["status"] == "observed":
                record["reason"] = "Fixture integrity verified; public SDK query is unavailable and thread control is unproven."
    except (OSError, ValueError, RuntimeError, TimeoutError, subprocess.SubprocessError) as error:
        record["reason"] = f"{type(error).__name__}: {error}"
    finally:
        if target is not None:
            record["target_exit_code"] = target.exit_code()
        cleanup = []
        try:
            record["tls"] = [{"stage": stage, "role": role, **value}
                             for (stage, role), value in tls_rows(
                                 (directory / "fixture.stdout").read_bytes()).items()]
        except (UnicodeDecodeError, ValueError) as error:
            record["tls_diagnostic_error"] = str(error)
        finally:
            for handle in reversed(handles):
                if handle.running() and (handle is not target or target_owned):
                    handle.terminate_owned()
                    cleanup.append(handle.identity["pid"])
                record.setdefault("held_exit_codes", []).append(
                    {"pid": handle.identity["pid"], "exit_code": handle.exit_code()})
                handle.close()
            for child in processes:
                if child.poll() is None:
                    child.terminate()
                child.wait(timeout=5)
        record["forced_cleanup_pids"] = cleanup
        record["elapsed_seconds"] = time.monotonic() - began
        record["artifacts"] = [{"path": str(file.relative_to(directory)), "sha256": OBS.sha256(file)}
                               for file in directory.rglob("*") if file.is_file()]
        OBS.write_json(directory / "receipt.json", record)
    return record


def matrix(profile_path, clients_path, fixture, output):
    if os.name != "nt" or OBS.process_session(os.getpid()) != 0:
        raise ValueError("authorized Windows notebook Session-0 console required")
    profile = OBS.load_json(profile_path)
    OBS.validate_profile(profile)
    clients = OBS.load_json(clients_path)
    if clients.get("kind") != "native-tls-isolation-clients" or clients.get("schema") != 1:
        raise ValueError("unsupported isolation-client manifest")
    for kind in ("noop", "drmgr"):
        if OBS.sha256(clients[kind + "_path"]) != clients[kind + "_sha256"]:
            raise ValueError("isolation-client pin mismatch")
    for name in ("client", "tool_config"):
        if OBS.sha256(clients["drcov"][name + "_path"]) != clients["drcov"][name + "_sha256"]:
            raise ValueError("drcov tool pin mismatch")
    pe = OBS.read_pe(fixture)
    if pe["name"] != "observer_fixture.exe" or pe["sha256"] != clients["fixture_sha256"] or \
            set(pe["exports"]) != {"observe_entry", "observe_clock", "observe_fp"}:
        raise ValueError("only the exact independently authored diagnostic fixture is admitted")
    output = Path(output).resolve()
    output.mkdir()
    records, expected_return = [], None
    for kind in KINDS:
        for mode in ("normal", "late"):
            record = run_case(kind, mode, profile, clients, Path(fixture).resolve(), pe,
                              output / (kind + "-" + mode), expected_return)
            records.append(record)
            if kind == "native" and record["status"] == "pass":
                expected_return = record["return_rva"]
            if kind == "native" and record["status"] != "pass":
                raise RuntimeError("native fixture control failed; matrix stopped")
    result = {"schema": 1, "kind": "native-tls-layer-isolation", "fixture_sha256": pe["sha256"],
              "profile_sha256": OBS.sha256(profile_path), "clients_sha256": OBS.sha256(clients_path),
              "script_sha256": OBS.sha256(__file__), "cases": records,
              "vendor_gui_admitted": False, "late_attach_profile_admitted": False}
    OBS.write_json(output / "matrix.json", result)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--clients", type=Path, required=True)
    parser.add_argument("--fixture", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        result = matrix(args.profile, args.clients, args.fixture, args.output)
    except (OSError, ValueError, RuntimeError, TimeoutError, subprocess.SubprocessError) as error:
        print(json.dumps({"status": "blocked", "reason": str(error)}))
        return 2
    print(json.dumps({"status": "complete", "cases": [
        {"kind": row["kind"], "mode": row["mode"], "status": row["status"],
         "reason": row.get("reason"), "target_exit_code": row.get("target_exit_code")}
        for row in result["cases"]]}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
