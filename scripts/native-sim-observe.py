"""Bounded passive entry evidence; no GUI, vendor calls, or replay admission."""

import argparse
import ctypes
import datetime
import hashlib
import json
import ntpath
import os
from pathlib import Path
import re
import struct
import subprocess
import time
import traceback


ARCHIVE_SHA256 = "eb26ba44658cdcc3e165028da1a3fff7519805182edaead71668779ea0fe0d66"
TOOL_PINS = {
    r"bin64\drrun.exe": "8400c89d99ab03e783489a4001d044e3c3ef16dc8825a6cfad218e3953ee6d55",
    r"bin64\drconfig.exe": "a6daed6ea20e4a00f6c76460c77e648a0ae1de6c9ac9839c7b9e5658a8ce08d3",
    r"lib64\release\dynamorio.dll": "15b0963c7dc8161d3c1fb57309d28f5d18394422ef82de1a90a95567c790a696",
    r"ext\lib64\release\drmgr.dll": "48babcf5ec785588498b3ff439bea5c025a4ee8a52525d204f61744f01865302",
    r"ext\lib64\release\drwrap.dll": "f29ab78bbcc07bed991de7f3a170e8f0f702f821fd7a74b148b11e9aff425273",
}
CREATE_NO_WINDOW = 0x08000000
MAX_LOG_BYTES = 16 * 1024 * 1024
REGISTERS = ("rcx", "rdx", "r8", "r9")
HASH = re.compile(r"[a-f0-9]{64}\Z")
POINTER = re.compile(r"0x[0-9a-fA-F]{1,16}\Z")


def sha256(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def write_json(path, value):
    with Path(path).open("x", encoding="utf-8", newline="\n") as stream:
        json.dump(value, stream, indent=2, ensure_ascii=True)
        stream.write("\n")


def load_json(path):
    return json.loads(Path(path).read_text(encoding="utf-8-sig"))


def integer(value, low, high, label):
    if type(value) is not int or not low <= value <= high:
        raise ValueError(f"invalid {label}")
    return value


def pointer(value):
    if not isinstance(value, str) or not POINTER.fullmatch(value):
        raise ValueError("invalid pointer")
    return int(value, 16)


def read_pe(path):
    """Read identity, executable sections and exports, never execute a PE."""
    data = Path(path).read_bytes()
    if len(data) < 256 or data[:2] != b"MZ":
        raise ValueError("not a PE")
    pe = struct.unpack_from("<I", data, 60)[0]
    if pe + 264 > len(data) or data[pe:pe + 4] != b"PE\0\0":
        raise ValueError("invalid PE header")
    machine, count, timestamp = struct.unpack_from("<HHI", data, pe + 4)
    optional_size = struct.unpack_from("<H", data, pe + 20)[0]
    optional = pe + 24
    if machine != 0x8664 or struct.unpack_from("<H", data, optional)[0] != 0x20B:
        raise ValueError("AMD64 PE32+ required")
    image_size = struct.unpack_from("<I", data, optional + 56)[0]
    sections = []
    for i in range(count):
        at = optional + optional_size + 40 * i
        virtual_size, rva, raw_size, raw_at = struct.unpack_from("<IIII", data, at + 8)
        flags = struct.unpack_from("<I", data, at + 36)[0]
        sections.append((rva, max(virtual_size, raw_size), raw_at, raw_size, flags))

    def offset(rva, size=1):
        for start, _, raw, raw_size, _ in sections:
            if start <= rva and rva + size <= start + raw_size:
                at = raw + rva - start
                if at + size <= len(data):
                    return at
        raise ValueError("PE RVA outside file-backed section")

    exports = {}
    export_rva, export_size = struct.unpack_from("<II", data, optional + 112)
    if export_rva:
        at = offset(export_rva, 40)
        functions, names, function_rvas, name_rvas, ordinals = struct.unpack_from(
            "<IIIII", data, at + 20)
        if names > 65536 or functions > 65536:
            raise ValueError("oversized export table")
        for i in range(names):
            name_rva = struct.unpack_from("<I", data, offset(name_rvas + 4 * i, 4))[0]
            name_at = offset(name_rva)
            end = data.find(b"\0", name_at, name_at + 256)
            if end < 0:
                raise ValueError("unterminated export name")
            name = data[name_at:end].decode("ascii")
            ordinal = struct.unpack_from("<H", data, offset(ordinals + 2 * i, 2))[0]
            if ordinal >= functions:
                raise ValueError("invalid export ordinal")
            rva = struct.unpack_from("<I", data, offset(function_rvas + 4 * ordinal, 4))[0]
            if not export_rva <= rva < export_rva + export_size:
                exports[name] = rva
    return {
        "name": Path(path).name, "sha256": sha256(path), "size": image_size,
        "timestamp": timestamp, "sections": sections, "exports": exports,
    }


def validate_sites(pe, sites):
    if not 1 <= len(sites) <= 16:
        raise ValueError("one to sixteen selected sites required")
    ids, rvas = set(), set()
    for site in sites:
        sid = integer(site["id"], 0, 0xFFFFFFFF, "site id")
        rva = integer(site["rva"], 0, pe["size"] - 1, "site RVA")
        integer(site["post"], 0, 1, "post")
        integer(site["mask"], 0, 15, "snapshot mask")
        if sid in ids or rva in rvas:
            raise ValueError("duplicate site")
        if not any(start <= rva < start + size and flags & 0x20000000
                   for start, size, _, _, flags in pe["sections"]):
            raise ValueError("site not in executable section")
        ids.add(sid)
        rvas.add(rva)


def config_bytes(pe, client_sha256, sites, max_events=256, max_objects=64, prefix=64):
    validate_sites(pe, sites)
    integer(max_events, 1, 4096, "event budget")
    integer(max_objects, 1, 512, "object budget")
    integer(prefix, 8, 256, "prefix")
    if not re.fullmatch(r"[A-Za-z0-9_.-]{1,127}", pe["name"]):
        raise ValueError("module basename must be a simple ASCII filename")
    if not HASH.fullmatch(client_sha256) or not HASH.fullmatch(pe["sha256"]):
        raise ValueError("invalid SHA-256")
    lines = [
        f"OBS1 {pe['name']} {pe['size']} {pe['timestamp']} {max_events} "
        f"{max_objects} {prefix} {pe['sha256']} {client_sha256}"
    ]
    lines.extend(f"{s['id']} {s['rva']:x} {s['post']} {s['mask']}" for s in sites)
    return ("\n".join(lines) + "\n").encode("ascii")


def read_config(path_or_bytes):
    """Decode canonical OBS1 bytes; executable sections still need actual read_pe."""
    payload = path_or_bytes if isinstance(path_or_bytes, bytes) else Path(path_or_bytes).read_bytes()
    if not 0 < len(payload) < 8191:
        raise ValueError("empty or oversized entry config")
    try:
        lines = payload.decode("ascii").splitlines()
        magic, name, size, timestamp, events, objects, prefix, target, client = lines[0].split(" ")
        if magic != "OBS1" or not re.fullmatch(r"[A-Za-z0-9_.-]{1,127}", name):
            raise ValueError("invalid OBS1 module")
        if not HASH.fullmatch(target) or not HASH.fullmatch(client):
            raise ValueError("invalid OBS1 hashes")
        size = integer(int(size), 1, 0xFFFFFFFF, "module size")
        timestamp = integer(int(timestamp), 0, 0xFFFFFFFF, "module timestamp")
        events = integer(int(events), 1, 4096, "event budget")
        objects = integer(int(objects), 1, 512, "object budget")
        prefix = integer(int(prefix), 8, 256, "prefix")
        sites, ids, rvas = [], set(), set()
        for line in lines[1:]:
            sid, rva, post, mask = line.split(" ")
            site = {
                "id": integer(int(sid), 0, 0xFFFFFFFF, "site id"),
                "rva": integer(int(rva, 16), 0, size - 1, "site RVA"),
                "post": integer(int(post), 0, 1, "post"),
                "mask": integer(int(mask), 0, 15, "snapshot mask"),
            }
            if site["id"] in ids or site["rva"] in rvas:
                raise ValueError("duplicate OBS1 site")
            ids.add(site["id"])
            rvas.add(site["rva"])
            sites.append(site)
        if not 1 <= len(sites) <= 16:
            raise ValueError("one to sixteen OBS1 sites required")
        canonical = (
            f"OBS1 {name} {size} {timestamp} {events} {objects} {prefix} {target} {client}\n"
            + "".join(f"{s['id']} {s['rva']:x} {s['post']} {s['mask']}\n" for s in sites)
        ).encode("ascii")
        if canonical != payload:
            raise ValueError("config is not canonical newline-terminated OBS1 ASCII")
        return {
            "target_sha256": target, "client_sha256": client,
            "config_sha256": hashlib.sha256(payload).hexdigest(),
            "module": name, "module_size": size, "module_timestamp": timestamp, "sites": sites,
            "max_events": events, "max_objects": objects, "prefix_bytes": prefix,
        }
    except (IndexError, TypeError, UnicodeError) as error:
        raise ValueError("malformed OBS1 configuration") from error


def launch_argv(profile, config_path, log_path, config_sha256, pidfile,
                application=None, app_args=(), seconds=40, attach_pid=None):
    """Pure argv builder. Caller owns live PID/session/hash/creation validation."""
    integer(seconds, 1, 120, "deadline")
    if not HASH.fullmatch(config_sha256):
        raise ValueError("invalid config hash")
    if (application is None) == (attach_pid is None):
        raise ValueError("choose launch OR attach")
    argv = [profile["drrun_path"], "-s", str(seconds)]
    if attach_pid is not None:
        integer(attach_pid, 1, 0xFFFFFFFF, "owned attach PID")
        argv += ["-attach", str(attach_pid), "-takeover_sleep", "-takeovers", "32"]
    else:
        argv += ["-pidfile", str(pidfile)]
    argv += ["-c", profile["client_path"], str(config_path), str(log_path), config_sha256]
    if application is not None:
        argv += ["--", str(application), *map(str, app_args)]
    return argv


def validate_profile(profile):
    if profile.get("kind") != "passive-entry-observer-profile" or profile.get("schema") != 1:
        raise ValueError("unsupported observer profile")
    if profile.get("archive_sha256") != ARCHIVE_SHA256:
        raise ValueError("unrecognized upstream archive")
    backend = profile.get("backend", "drwrap")
    if backend not in ("drwrap", "drmgr-entry"):
        raise ValueError("unsupported observer backend")
    if backend == "drmgr-entry" and (
        profile.get("client_version") != 5 or profile.get("supports_return") is not False
        or profile.get("format_schema") != "passive-entry-observer-jsonl-v2"
    ):
        raise ValueError("invalid entry-only backend capabilities")
    components = ("drrun", "drconfig", "runtime", "client", "drmgr")
    if backend == "drwrap":
        components += ("drwrap",)
    for name in components:
        if sha256(profile[name + "_path"]) != profile[name + "_sha256"]:
            raise ValueError(f"pinned {name} mismatch")
    upstream = [("drrun", r"bin64\drrun.exe"),
                           ("drconfig", r"bin64\drconfig.exe"),
                           ("runtime", r"lib64\release\dynamorio.dll"),
                           ("drmgr", r"ext\lib64\release\drmgr.dll")]
    if backend == "drwrap":
        upstream.append(("drwrap", r"ext\lib64\release\drwrap.dll"))
    for name, relative in upstream:
        if profile[name + "_sha256"] != TOOL_PINS[relative]:
            raise ValueError(f"unrecognized {name}")


def create_profile(tool_root, client, output, backend="drmgr-entry"):
    tool_root, client = Path(tool_root).resolve(), Path(client).resolve()
    value = {
        "schema": 1, "kind": "passive-entry-observer-profile",
        "tool_version": "11.91.20756 -- build 0",
        "archive_sha256": ARCHIVE_SHA256,
        "mode": "single-client-only", "gui_parity_verified": False,
        "late_attach_verified": False,
        "format_schema": ("passive-entry-observer-jsonl-v2" if backend == "drmgr-entry"
                          else "passive-entry-observer-jsonl-v1"),
        "backend": backend,
        "units": {"registers": "raw unsigned 64-bit bit-pattern",
                  "addresses": "application virtual address",
                  "rva": "bytes relative to recorded main-module base",
                  "clock_rax": ("unsupported by entry-only backend" if backend == "drmgr-entry"
                                else "raw value; no vendor time unit or epoch established")},
        "nudge_client_id": "00000000", "nudge_argument": "1",
        "nudge_semantics": "process exit and exit-callback flush; not live snapshot",
        "nudge_timeout_ms": 5000, "nudge_outer_timeout_seconds": 8,
        "limitations": [
            "ENTRY/RETURN observations are not BB, branch, line or behavioral coverage",
            "registers are raw app context, not recovered typed ABI",
            "object/frame/UUID layout and frame-clock identity remain unknown",
            "bounded safe-read byte samples may race application writes",
            ("ENTRY only; RETURN/RAX observation unsupported" if backend == "drmgr-entry"
             else "post hooks are opt-in, exceptional/unmatched returns may be absent"),
            "tool/config/target SHA declarations require independent launcher verification",
            "no application state or context writes; no vendor function invocation",
        ],
    }
    if backend == "drmgr-entry":
        value.update(client_version=5, supports_return=False, supported_observations=["ENTRY"],
                     clean_call_save_fpstate=True)
    paths = {
        "drrun": tool_root / "bin64" / "drrun.exe",
        "drconfig": tool_root / "bin64" / "drconfig.exe",
        "runtime": tool_root / "lib64" / "release" / "dynamorio.dll",
        "client": client, "drmgr": client.parent / "drmgr.dll",
    }
    if backend == "drwrap":
        paths["drwrap"] = client.parent / "drwrap.dll"
    for name, path in paths.items():
        value[name + "_path"] = str(path)
        value[name + "_sha256"] = sha256(path)
    validate_profile(value)
    write_json(output, value)
    return value


def parse_log(path, expected=None):
    """Strict bounded JSONL validator; requires complete exit SUMMARY."""
    path = Path(path)
    if not 0 < path.stat().st_size <= MAX_LOG_BYTES:
        raise ValueError("empty or oversized observer log")

    def no_duplicates(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError("duplicate JSON key")
            result[key] = value
        return result

    raw = path.read_bytes()
    if not raw.endswith(b"\n"):
        raise ValueError("truncated log")
    records = [json.loads(line, object_pairs_hook=no_duplicates) for line in raw.splitlines()]
    if not 3 <= len(records) <= 4114:
        raise ValueError("invalid record count")
    header, summary = records[0], records[-1]
    if header.get("type") != "HEADER" or header.get("schema") not in (1, 2) or \
            header.get("kind") != "passive-entry-observer" or summary.get("type") != "SUMMARY":
        raise ValueError("invalid header/summary")
    entry_only = header["schema"] == 2
    if entry_only and (header.get("backend") != "drmgr-entry" or header.get("client_version") != 5
                       or header.get("supports_return") is not False):
        raise ValueError("invalid schema-2 backend")
    if not entry_only and header.get("backend", "drwrap") != "drwrap":
        raise ValueError("invalid schema-1 backend")
    for key in ("target_sha256", "client_sha256", "config_sha256"):
        if not HASH.fullmatch(header[key]):
            raise ValueError("invalid identity")
        if expected is not None and header[key] != expected[key]:
            raise ValueError("identity mismatch: " + key)
    if header["module_seen"] is not True or header["module_mismatch"] is not False:
        raise ValueError("target module not matched")
    base = pointer(header["module_base"])
    size = integer(header["module_size"], 1, 0xFFFFFFFF, "module size")
    integer(header["module_timestamp"], 0, 0xFFFFFFFF, "module timestamp")
    if expected is not None:
        for key in ("module", "module_size", "module_timestamp"):
            if key in expected and header[key] != expected[key]:
                raise ValueError("module identity mismatch: " + key)
    max_events = integer(header["max_events"], 1, 4096, "event budget")
    max_objects = integer(header["max_objects"], 1, 512, "object budget")
    prefix = integer(header["prefix_bytes"], 8, 256, "prefix")
    if expected is not None:
        for key in ("max_events", "max_objects", "prefix_bytes"):
            if key in expected and header[key] != expected[key]:
                raise ValueError("configured budget mismatch: " + key)
    sites, events = {}, []
    seen_event = False
    for record in records[1:-1]:
        kind = record.get("type")
        if kind == "SITE" and not seen_event:
            sid = integer(record["id"], 0, 0xFFFFFFFF, "site id")
            if sid in sites or any(s["rva"] == record["rva"] for s in sites.values()):
                raise ValueError("duplicate site")
            integer(record["rva"], 0, size - 1, "site RVA")
            integer(record["post"], 0, 1, "post")
            integer(record["mask"], 0, 15, "mask")
            if entry_only:
                if (record.get("selected") is not True or type(record.get("instrumented")) is not bool
                        or "wrapped" in record or record["post"] != 0):
                    raise ValueError("invalid entry-only site selection/instrumentation")
            elif record["wrapped"] is not True:
                raise ValueError("site not wrapped")
            if len(sites) == 16:
                raise ValueError("excess sites")
            sites[sid] = record
            continue
        seen_event = True
        if kind not in ("ENTRY", "RETURN") or \
                integer(record["seq"], 0, 4095, "sequence") != len(events):
            raise ValueError("invalid event sequence")
        if record["site"] not in sites or type(record["context_ok"]) is not bool:
            raise ValueError("invalid event site/context")
        integer(record["thread"], 1, 0xFFFFFFFF, "thread")
        site = sites[record["site"]]
        if entry_only and (kind != "ENTRY" or site["instrumented"] is not True):
            raise ValueError("entry-only backend requires an instrumented ENTRY")
        if kind == "ENTRY" and record["context_ok"]:
            if pointer(record["pc"]) != base + site["rva"]:
                raise ValueError("entry PC mismatch")
            for name in (*REGISTERS, "rsp"):
                pointer(record[name])
            if type(record["return_ok"]) is not bool:
                raise ValueError("invalid return-read status")
            if record["return_ok"]:
                pointer(record["return_address"])
            elif "return_address" in record:
                raise ValueError("failed return read cannot have an address")
            if len(record["samples"]) != 4:
                raise ValueError("invalid samples")
            for i, sample in enumerate(record["samples"]):
                if sample["reg"] != i:
                    raise ValueError("sample register order")
                status = integer(sample["status"], 0, 4, "read status")
                address = pointer(sample["address"])
                enabled = bool(site["mask"] & (1 << i))
                if enabled != (status != 0):
                    raise ValueError("snapshot mask mismatch")
                if enabled and address != pointer(record[REGISTERS[i]]):
                    raise ValueError("snapshot pointer mismatch")
                _validate_read(sample, "size", "bytes", status, prefix)
                vstatus = integer(sample["vtable_status"], 0, 3, "vtable status")
                _validate_read(sample, "vtable_size", "vtable_bytes", vstatus, 8)
                if sample["size"] >= 8:
                    if pointer(sample["vtable"]) != int.from_bytes(
                            bytes.fromhex(sample["bytes"])[:8], "little") or vstatus == 0:
                        raise ValueError("vtable/prefix mismatch")
                elif vstatus != 0 or "vtable" in sample:
                    raise ValueError("vtable from unavailable prefix")
        elif kind == "RETURN":
            index = integer(record["entry"], 0, len(events) - 1, "entry reference")
            entry = events[index]
            if not site["post"] or entry["type"] != "ENTRY" or \
                    entry["site"] != record["site"] or entry["thread"] != record["thread"]:
                raise ValueError("invalid return association")
            if any(e.get("entry") == index and e["type"] == "RETURN" for e in events):
                raise ValueError("duplicate return")
            if record["context_ok"]:
                pointer(record["rax"])
            elif "rax" in record:
                raise ValueError("unavailable return context cannot have RAX")
        events.append(record)
    if not sites or len(events) > max_events or summary["events"] != len(events):
        raise ValueError("event count/budget mismatch")
    integer(summary["unique_objects"], 0, max_objects, "unique objects")
    for name in ("dropped_events", "unreadable_reads", "partial_reads",
                 "truncated_reads", "object_limit_reads", "context_failures"):
        integer(summary[name], 0, (1 << 64) - 1, name)
    if summary["truncated_reads"] != summary["partial_reads"]:
        raise ValueError("truncation counter mismatch")
    if type(summary["nudge_exit"]) is not bool:
        raise ValueError("invalid nudge status")
    if expected is not None and "sites" in expected:
        actual = [{k: s[k] for k in ("id", "rva", "post", "mask")} for s in sites.values()]
        if actual != expected["sites"]:
            raise ValueError("selected site configuration mismatch")
    return {"header": header, "sites": list(sites.values()), "events": events,
            "summary": summary, "sha256": sha256(path)}


def _validate_read(sample, size_key, bytes_key, status, wanted):
    size = integer(sample[size_key], 0, wanted, size_key)
    text = sample[bytes_key]
    if not isinstance(text, str) or not re.fullmatch(r"[0-9a-f]*", text) or len(text) != size * 2:
        raise ValueError("invalid read bytes")
    if (status == 1 and size != wanted) or (status == 2 and not 0 < size < wanted) or \
            (status in (0, 3, 4) and size != 0):
        raise ValueError("read status/length mismatch")


class OwnedProcess:
    """Held Windows handle, immutable creation/image identity; no PID adoption."""

    def __init__(self, pid):
        from ctypes import wintypes as w
        self.k = ctypes.WinDLL("kernel32", use_last_error=True)
        self.k.OpenProcess.argtypes = [w.DWORD, w.BOOL, w.DWORD]
        self.k.OpenProcess.restype = w.HANDLE
        self.k.CloseHandle.argtypes = [w.HANDLE]
        self.k.QueryFullProcessImageNameW.argtypes = [w.HANDLE, w.DWORD, w.LPWSTR,
                                                    ctypes.POINTER(w.DWORD)]
        self.k.GetProcessTimes.argtypes = [w.HANDLE] + [ctypes.POINTER(w.FILETIME)] * 4
        self.k.WaitForSingleObject.argtypes = [w.HANDLE, w.DWORD]
        self.k.TerminateProcess.argtypes = [w.HANDLE, w.UINT]
        self.k.GetExitCodeProcess.argtypes = [w.HANDLE, ctypes.POINTER(w.DWORD)]
        self.k.GetExitCodeProcess.restype = w.BOOL
        self.handle = self.k.OpenProcess(0x1000 | 0x100000 | 1, False, pid)
        if not self.handle:
            raise ctypes.WinError(ctypes.get_last_error())
        self.pid = pid
        self.identity = self.metadata()

    def metadata(self):
        from ctypes import wintypes as w
        size = w.DWORD(32768)
        path = ctypes.create_unicode_buffer(size.value)
        times = [w.FILETIME() for _ in range(4)]
        if not self.k.QueryFullProcessImageNameW(self.handle, 0, path, ctypes.byref(size)) or \
                not self.k.GetProcessTimes(self.handle, *(ctypes.byref(t) for t in times)):
            raise ctypes.WinError(ctypes.get_last_error())
        return {
            "pid": self.pid, "image_path": path.value,
            "creation_ticks": (times[0].dwHighDateTime << 32) | times[0].dwLowDateTime,
            "session_id": process_session(self.pid),
        }

    def running(self):
        return self.k.WaitForSingleObject(self.handle, 0) == 258

    def assert_same(self):
        if not self.running() or self.metadata() != self.identity:
            raise ValueError("owned process identity/liveness changed")

    def terminate_owned(self):
        self.assert_same()
        if not self.k.TerminateProcess(self.handle, 125):
            raise ctypes.WinError(ctypes.get_last_error())
        if self.k.WaitForSingleObject(self.handle, 5000) != 0:
            raise TimeoutError("owned process cleanup deadline")

    def exit_code(self):
        from ctypes import wintypes as w
        if self.running():
            return None
        value = w.DWORD()
        if not self.k.GetExitCodeProcess(self.handle, ctypes.byref(value)):
            raise ctypes.WinError(ctypes.get_last_error())
        return value.value

    def close(self):
        if self.handle:
            self.k.CloseHandle(self.handle)
            self.handle = None


def process_session(pid):
    from ctypes import wintypes as w
    k = ctypes.WinDLL("kernel32", use_last_error=True)
    k.ProcessIdToSessionId.argtypes = [w.DWORD, ctypes.POINTER(w.DWORD)]
    value = w.DWORD()
    if not k.ProcessIdToSessionId(pid, ctypes.byref(value)):
        raise ctypes.WinError(ctypes.get_last_error())
    return value.value


def parent_pid(pid):
    from ctypes import wintypes as w

    class Entry(ctypes.Structure):
        _fields_ = [
            ("size", w.DWORD), ("usage", w.DWORD), ("pid", w.DWORD),
            ("heap", ctypes.c_size_t), ("module", w.DWORD), ("threads", w.DWORD),
            ("parent", w.DWORD), ("priority", w.LONG), ("flags", w.DWORD),
            ("name", w.WCHAR * 260),
        ]

    k = ctypes.WinDLL("kernel32", use_last_error=True)
    k.CreateToolhelp32Snapshot.argtypes = [w.DWORD, w.DWORD]
    k.CreateToolhelp32Snapshot.restype = w.HANDLE
    k.Process32FirstW.argtypes = [w.HANDLE, ctypes.POINTER(Entry)]
    k.Process32NextW.argtypes = [w.HANDLE, ctypes.POINTER(Entry)]
    k.CloseHandle.argtypes = [w.HANDLE]
    snapshot = k.CreateToolhelp32Snapshot(2, 0)
    if snapshot == ctypes.c_void_p(-1).value:
        raise ctypes.WinError(ctypes.get_last_error())
    try:
        entry = Entry()
        entry.size = ctypes.sizeof(entry)
        ok = k.Process32FirstW(snapshot, ctypes.byref(entry))
        while ok:
            if entry.pid == pid:
                return entry.parent
            ok = k.Process32NextW(snapshot, ctypes.byref(entry))
    finally:
        k.CloseHandle(snapshot)
    raise ValueError("owned target absent from process snapshot")


def sanitized_env(directory):
    env = {key: os.environ[key] for key in (
        "SystemRoot", "WINDIR", "SystemDrive", "ComSpec", "USERPROFILE",
        "APPDATA", "LOCALAPPDATA") if key in os.environ}
    env["PATH"] = str(Path(env["SystemRoot"]) / "System32")
    env["DYNAMORIO_CONFIGDIR"] = str(directory)
    return env


def _command(argv, directory, name, env, timeout):
    with (directory / (name + ".stdout")).open("xb") as out, \
            (directory / (name + ".stderr")).open("xb") as err:
        code = subprocess.run(argv, stdout=out, stderr=err, env=env, shell=False,
                              close_fds=True, creationflags=CREATE_NO_WINDOW,
                              timeout=timeout, check=False).returncode
    if code != 0:
        raise RuntimeError(f"{name} exited {code}")
    return (directory / (name + ".stdout")).read_bytes()


def observed_console(profile, fixture, pe, sites, directory, *, max_events=256,
                     max_objects=16, nudge=False):
    directory.mkdir()
    config = directory / "observer.cfg"
    payload = config_bytes(pe, profile["client_sha256"], sites, max_events, max_objects, 32)
    config.write_bytes(payload)
    config_hash = sha256(config)
    pidfile, log = directory / "target.pid", directory / "events.jsonl"
    drdir = directory / "dr-config"
    drdir.mkdir()
    env = sanitized_env(drdir)
    argv = launch_argv(profile, config, log, config_hash, pidfile, fixture,
                       ["--hold"] if nudge else [], seconds=40)
    launcher = None
    launcher_handle = target = None
    target_owned = False
    result = {"launch_argv": argv, "config_sha256": config_hash}
    try:
        with (directory / "launcher.stdout").open("xb") as stdout, \
                (directory / "launcher.stderr").open("xb") as stderr:
            launcher = subprocess.Popen(
                argv, stdout=stdout, stderr=stderr, env=env, shell=False,
                close_fds=True, creationflags=CREATE_NO_WINDOW)
            launcher_handle = OwnedProcess(launcher.pid)
            if ntpath.normcase(launcher_handle.identity["image_path"]) != \
                    ntpath.normcase(profile["drrun_path"]):
                raise ValueError("launcher image mismatch")
            deadline = time.monotonic() + 10
            while not pidfile.exists() and time.monotonic() < deadline and launcher.poll() is None:
                time.sleep(0.025)
            if not pidfile.exists():
                raise RuntimeError("target PID file missing")
            pid = int(pidfile.read_text(encoding="utf-8-sig").strip())
            target = OwnedProcess(pid)
            identity = target.identity
            if (identity["session_id"] != 0 or parent_pid(pid) != launcher.pid or
                    identity["creation_ticks"] < launcher_handle.identity["creation_ticks"] or
                    ntpath.normcase(identity["image_path"]) != ntpath.normcase(str(fixture)) or
                    sha256(identity["image_path"]) != pe["sha256"]):
                raise ValueError("target image/creation/session/ancestry mismatch")
            target_owned = True
            result.update(target=identity, launcher=launcher_handle.identity,
                          target_parent_pid=launcher.pid)
            if nudge:
                ready = directory / "launcher.stdout"
                deadline = time.monotonic() + 10
                while ready.stat().st_size == 0 and time.monotonic() < deadline:
                    target.assert_same()
                    time.sleep(0.025)
                if ready.stat().st_size == 0:
                    raise RuntimeError("fixture did not reach known calls")
                target.assert_same()
                launcher_handle.assert_same()
                if sha256(fixture) != pe["sha256"]:
                    raise ValueError("target image changed")
                nudge_argv = [
                    profile["drconfig_path"], "-nudge_timeout", "5000",
                    "-nudge_pid", str(pid), "00000000", "1",
                ]
                began = time.monotonic()
                _command(nudge_argv, directory, "nudge", env, 8)
                result.update(nudge_argv=nudge_argv,
                              nudge_seconds=time.monotonic() - began,
                              independent_nudge_deadline_seconds=8)
            code = launcher.wait(timeout=12)
            if code != 0 or target.running():
                raise RuntimeError(f"observer launcher exit/liveness failure ({code})")
            result["launcher_exit_code"] = code
        expected = {"target_sha256": pe["sha256"], "client_sha256": profile["client_sha256"],
                    "config_sha256": config_hash, "module": pe["name"],
                    "module_size": pe["size"], "module_timestamp": pe["timestamp"],
                    "sites": sites}
        parsed = parse_log(log, expected)
        if parsed["summary"]["nudge_exit"] is not nudge:
            raise ValueError("exit method mismatch")
        result.update(log_sha256=sha256(log), summary=parsed["summary"],
                      stdout_sha256=sha256(directory / "launcher.stdout"),
                      stderr_sha256=sha256(directory / "launcher.stderr"))
        write_json(directory / "receipt.json", result)
        return result, parsed
    finally:
        if target is not None:
            if target_owned and target.running():
                target.terminate_owned()
            target.close()
        if launcher_handle is not None:
            if launcher_handle.running():
                launcher_handle.terminate_owned()
            launcher_handle.close()
        if launcher is not None:
            launcher.wait(timeout=5)


def verify_fixture(parsed, expected_return_rva):
    entries = [e for e in parsed["events"] if e["type"] == "ENTRY" and e["site"] == 1]
    returns = [e for e in parsed["events"] if e["type"] == "RETURN"]
    entry_only = parsed["header"]["schema"] == 2
    if len(entries) != 80 or len(returns) != (0 if entry_only else 1) or \
            len(parsed["events"]) != (145 if entry_only else 82):
        raise ValueError("synthetic invocation/return count mismatch")
    base, size = pointer(parsed["header"]["module_base"]), parsed["header"]["module_size"]
    for e in entries:
        if not e["context_ok"] or pointer(e["rdx"]) != 0x13579BDF2468ACE0 or \
                pointer(e["r8"]) != 1 or pointer(e["rsp"]) % 16 != 8 or \
                not e["return_ok"] or not base <= pointer(e["return_address"]) < base + size or \
                pointer(e["return_address"]) - base != expected_return_rva:
            raise ValueError("real entry register/stack/return address mismatch")
        self_sample, disabled, bad, frame = e["samples"]
        if disabled["status"] != 0 or bad["status"] != 3 or bad["bytes"] != "" or \
                pointer(bad["address"]) != 1:
            raise ValueError("explicit invalid read was not rejected")
        for sample, marker, fixture_id in (
            (self_sample, 0x1020304050607080, "1032547698badcfe0123456789abcdef"),
            (frame, 0x0908070605040302, "f0e1d2c3b4a5968778695a4b3c2d1e0f"),
        ):
            data = bytes.fromhex(sample["bytes"])
            if sample["status"] != 1 or len(data) != 32 or \
                    int.from_bytes(data[8:16], "little") != marker or \
                    data[16:].hex() != fixture_id or sample["vtable_status"] != 1 or \
                    sample["vtable_bytes"] != "8877665544332211":
                raise ValueError("known object/frame prefix or vtable mismatch")
    if entry_only:
        fp = [e for e in parsed["events"] if e["type"] == "ENTRY" and e["site"] == 3]
        if (len(fp) != 64 or len({e["thread"] for e in fp}) != 2 or
                any(e["context_ok"] is not True or e.get("return_ok") is not True for e in fp)):
            raise ValueError("FP/static-TLS main/timer-thread entry evidence mismatch")
    elif returns[0]["site"] != 2 or not returns[0]["context_ok"] or \
            pointer(returns[0]["rax"]) != 0x0123456789ABCDEF:
        raise ValueError("known clock RAX mismatch")
    summary = parsed["summary"]
    if summary["unreadable_reads"] != 80 or summary["partial_reads"] != 0 or \
            summary["context_failures"] != 0 or summary["dropped_events"] != 0:
        raise ValueError("synthetic counters mismatch")


def late_console(profile, fixture, pe, sites, directory, expected_return_rva):
    """Normal owned start, public attach, then synthetic-only file trigger."""
    directory.mkdir()
    drdir = directory / "dr-config"
    drdir.mkdir()
    env = sanitized_env(drdir)
    config, log, trigger = (directory / name for name in (
        "observer.cfg", "events.jsonl", "fixture-trigger"))
    config.write_bytes(config_bytes(pe, profile["client_sha256"], sites, 256, 16, 32))
    config_hash = sha256(config)
    target_process = attacher = None
    target = attacher_handle = None
    record = {
        "mode": "normal-start-then-late-attach",
        "independent_attach_deadline_seconds": 30,
        "fixture_start_argv": [str(fixture), "--late-trigger", str(trigger)],
    }
    try:
        with (directory / "fixture.stdout").open("xb") as out, \
                (directory / "fixture.stderr").open("xb") as err, \
                (directory / "attach.stdout").open("xb") as aout, \
                (directory / "attach.stderr").open("xb") as aerr:
            target_process = subprocess.Popen(
                record["fixture_start_argv"], stdout=out, stderr=err, env=env,
                shell=False, close_fds=True, creationflags=CREATE_NO_WINDOW)
            target = OwnedProcess(target_process.pid)
            if (target.identity["session_id"] != 0 or parent_pid(target.pid) != os.getpid() or
                    ntpath.normcase(target.identity["image_path"]) != ntpath.normcase(str(fixture)) or
                    sha256(target.identity["image_path"]) != pe["sha256"]):
                raise ValueError("normally started owned fixture identity mismatch")
            deadline = time.monotonic() + 5
            while (directory / "fixture.stdout").stat().st_size == 0 and time.monotonic() < deadline:
                target.assert_same()
                time.sleep(0.025)
            if (directory / "fixture.stdout").read_bytes().strip() != b"fixture-v1 ready":
                raise ValueError("normal fixture initialization not witnessed")
            target.assert_same()
            attach_argv = launch_argv(
                profile, config, log, config_hash, directory / "unused.pid",
                seconds=40, attach_pid=target.pid)
            record.update(attach_argv=attach_argv, target=target.identity,
                          target_parent_pid=os.getpid())
            began = time.monotonic()
            attacher = subprocess.Popen(
                attach_argv, stdout=aout, stderr=aerr, env=env, shell=False,
                close_fds=True, creationflags=CREATE_NO_WINDOW)
            attacher_handle = OwnedProcess(attacher.pid)
            record["attacher"] = attacher_handle.identity
            if ntpath.normcase(attacher_handle.identity["image_path"]) != \
                    ntpath.normcase(profile["drrun_path"]):
                raise ValueError("attacher image mismatch")
            deadline = began + 30
            while not log.exists() and time.monotonic() < deadline:
                target.assert_same()
                if attacher.poll() not in (None, 0):
                    raise RuntimeError(f"attach exited {attacher.returncode}")
                time.sleep(0.025)
            if not log.exists():
                raise TimeoutError("independent 30-second attachment deadline")
            record["attachment_ready_seconds"] = time.monotonic() - began
            record["stage"] = "waiting-known-fixture-calls"
            target.assert_same()
            # Only the independently authored fixture reads this owned control file.
            with trigger.open("xb") as stream:
                stream.write(b"synthetic console calls only\n")
            deadline = time.monotonic() + 5
            while b"calls=80" not in (directory / "fixture.stdout").read_bytes() and \
                    time.monotonic() < deadline:
                target.assert_same()
                time.sleep(0.025)
            if b"calls=80" not in (directory / "fixture.stdout").read_bytes():
                raise RuntimeError("attached fixture did not execute known calls")
            target.assert_same()
            validate_profile(profile)
            if sha256(fixture) != pe["sha256"]:
                raise ValueError("live fixture image pin changed")
            nudge_argv = [
                profile["drconfig_path"], "-nudge_timeout", "5000", "-nudge_pid",
                str(target.pid), "00000000", "1",
            ]
            began = time.monotonic()
            _command(nudge_argv, directory, "nudge", env, 8)
            record.update(nudge_argv=nudge_argv, nudge_seconds=time.monotonic() - began,
                          independent_nudge_deadline_seconds=8)
            target_code = target_process.wait(timeout=8)
            attach_code = attacher.wait(timeout=8)
            if target.running() or target_code != 0 or attach_code != 0:
                raise RuntimeError("late-attach exit did not complete normally")
            record.update(target_exit_code=target_code, attach_exit_code=attach_code,
                          attacher=attacher_handle.identity)
        parsed = parse_log(log, {
            "target_sha256": pe["sha256"], "client_sha256": profile["client_sha256"],
            "config_sha256": config_hash, "module": pe["name"],
            "module_size": pe["size"], "module_timestamp": pe["timestamp"], "sites": sites})
        verify_fixture(parsed, expected_return_rva)
        if not parsed["summary"]["nudge_exit"]:
            raise ValueError("late-attach nudge flush not observed")
        if (directory / "fixture.stderr").read_bytes() or \
                (directory / "attach.stderr").read_bytes():
            raise ValueError("late-attach stderr is not empty")
        record.update(config_sha256=config_hash, log_sha256=sha256(log),
                      summary=parsed["summary"],
                      stdout_sha256=sha256(directory / "fixture.stdout"))
        write_json(directory / "receipt.json", record)
        return record
    except Exception as error:
        record.update(status="blocked", error=f"{type(error).__name__}: {error}",
                      traceback=traceback.format_exc())
        for name, handle in (("target", target), ("attacher", attacher_handle)):
            if handle is not None:
                record[name] = handle.identity
                try:
                    record[name + "_exit_code"] = handle.exit_code()
                except OSError as diagnostic_error:
                    record[name + "_exit_code_error"] = str(diagnostic_error)
        write_json(directory / "blocked-receipt.json", record)
        raise
    finally:
        for handle in (target, attacher_handle):
            if handle is not None:
                if handle.running():
                    handle.terminate_owned()
                handle.close()
        for process in (target_process, attacher):
            if process is not None:
                process.wait(timeout=5)


def console_proof(profile_path, fixture, output):
    if os.name != "nt" or process_session(os.getpid()) != 0:
        raise ValueError("authorized notebook Session-0 console host required")
    profile, fixture, output = load_json(profile_path), Path(fixture).resolve(), Path(output).resolve()
    validate_profile(profile)
    pe = read_pe(fixture)
    entry_only = profile.get("backend") == "drmgr-entry"
    exports = {"observe_entry", "observe_clock", *({"observe_fp"} if entry_only else set())}
    if pe["name"] != "observer_fixture.exe" or set(pe["exports"]) != exports:
        raise ValueError("only independently authored observer_fixture.exe is admitted")
    sites = [
        {"id": 1, "rva": pe["exports"]["observe_entry"], "post": 0, "mask": 13},
        {"id": 2, "rva": pe["exports"]["observe_clock"], "post": 0 if entry_only else 1, "mask": 0},
    ]
    if entry_only:
        sites.append({"id": 3, "rva": pe["exports"]["observe_fp"], "post": 0, "mask": 0})
    output.mkdir()
    receipt = {
        "schema": 1, "kind": "passive-observer-console-proof", "status": "blocked",
        "gui_parity_verified": False, "simulator_coverage_claim": False,
        "profile_sha256": sha256(profile_path), "script_sha256": sha256(__file__),
        "backend": profile.get("backend", "drwrap"), "format_schema": profile["format_schema"],
        "fixture_sha256": pe["sha256"], "sites": sites,
        "created_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    }
    try:
        configdir = output / "plain-config"
        configdir.mkdir()
        env = sanitized_env(configdir)
        version = _command([profile["drrun_path"], "-version"], output, "version", env, 5)
        version += (output / "version.stderr").read_bytes()
        if b"11.91.20756 -- build 0" not in version:
            raise ValueError("actual launcher version mismatch")
        plain = _command([str(fixture)], output, "plain", env, 8)
        value = 0x1020304050607080 ^ 0x13579BDF2468ACE0 ^ 1 ^ 0x0908070605040302
        expected = f"fixture-v1 value={value:016x} clock=0123456789abcdef calls=80 return_rva="
        suffix = rb" fp_sentinels=1 tls=1 fp_calls=64" if entry_only else b""
        match = re.fullmatch(re.escape(expected.encode()) + rb"([0-9a-f]+)" + suffix + rb"\r?\n", plain)
        if match is None or (output / "plain.stderr").read_bytes():
            raise ValueError("plain synthetic known output mismatch")
        expected_return_rva = int(match[1], 16)
        if not any(start <= expected_return_rva < start + size and flags & 0x20000000
                   for start, size, _, _, flags in pe["sections"]):
            raise ValueError("fixture return address is outside code")
        receipt["fixture_return_rva"] = expected_return_rva
        normal, parsed = observed_console(profile, fixture, pe, sites, output / "normal")
        verify_fixture(parsed, expected_return_rva)
        if (output / "normal" / "launcher.stdout").read_bytes() != plain or \
                (output / "normal" / "launcher.stderr").read_bytes():
            raise ValueError("plain-versus-observed synthetic output mismatch")
        bounded, small = observed_console(
            profile, fixture, pe, sites, output / "event-bounds", max_events=5)
        if len(small["events"]) != 5 or small["summary"]["dropped_events"] != (140 if entry_only else 76):
            raise ValueError("event budget not enforced")
        objects, limited = observed_console(
            profile, fixture, pe, sites, output / "object-bounds", max_objects=1)
        if limited["summary"]["unique_objects"] != 1 or \
                limited["summary"]["object_limit_reads"] != 160:
            raise ValueError("unique object budget not enforced")
        nudged, flushed = observed_console(
            profile, fixture, pe, sites, output / "nudge", nudge=True)
        verify_fixture(flushed, expected_return_rva)
        if (output / "nudge" / "launcher.stdout").read_bytes() != plain:
            raise ValueError("nudge synthetic output mismatch")
        late = late_console(profile, fixture, pe, sites, output / "late-attach", expected_return_rva)
        if (output / "late-attach" / "fixture.stdout").read_bytes() != \
                b"fixture-v1 ready\r\n" + plain:
            raise ValueError("late-attach synthetic output mismatch")
        receipt.update(status="pass", normal=normal, event_bounds=bounded,
                       object_bounds=objects, exit_flush=nudged, late_attach=late,
                       verified=["real RCX/RDX/R8/R9/RSP",
                                 "exact return RVA independently read by fixture _ReturnAddress",
                                 "known object/frame IDs and markers", "readable vtable prefix",
                                 "explicit unreadable address rejection", "opt-in clock RAX",
                                 "event/object budgets", "plain/observed synthetic output parity",
                                 "exact owned handle/PID/session/creation/image/ancestry",
                                 "normal initialization then bounded late attach",
                                 "pre-existing module wrapping and registration deduplication",
                                 "bounded public nudge exit and complete exit flush"])
        if entry_only:
            receipt["verified"].remove("opt-in clock RAX")
            receipt["verified"] += [
                "ENTRY-only backend with no drwrap/RETURN support",
                "FP XMM0 arguments/result, XMM6..15 and nonvolatile GPR sentinels",
                "x87 live-stack sentinel and MXCSR transparency",
                "main and pre-existing native timer-thread static TLS transparency",
            ]
    except Exception as error:
        receipt["error"] = f"{type(error).__name__}: {error}"
        raise
    finally:
        write_json(output / "proof.json", receipt)
    return receipt


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    profile = commands.add_parser("profile")
    profile.add_argument("--tool-root", required=True, type=Path)
    profile.add_argument("--client", required=True, type=Path)
    profile.add_argument("--output", required=True, type=Path)
    profile.add_argument("--backend", choices=("drmgr-entry", "drwrap"), default="drmgr-entry")
    inspect = commands.add_parser("inspect")
    inspect.add_argument("log", type=Path)
    prepare = commands.add_parser("prepare")
    prepare.add_argument("--profile", required=True, type=Path)
    prepare.add_argument("--target", required=True, type=Path)
    prepare.add_argument("--site", action="append", required=True,
                         help="decimal-id:hex-rva:post-0-or-1:decimal-register-mask")
    prepare.add_argument("--output", required=True, type=Path)
    prepare.add_argument("--max-events", type=int, default=256)
    prepare.add_argument("--max-objects", type=int, default=64)
    prepare.add_argument("--prefix", type=int, default=64)
    proof = commands.add_parser("console-proof")
    proof.add_argument("--profile", required=True, type=Path)
    proof.add_argument("--fixture", required=True, type=Path)
    proof.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    if args.command == "profile":
        create_profile(args.tool_root, args.client, args.output, args.backend)
        result = {"profile": str(args.output), "sha256": sha256(args.output)}
    elif args.command == "inspect":
        parsed = parse_log(args.log)
        result = {"header": parsed["header"], "sites": parsed["sites"],
                  "summary": parsed["summary"], "sha256": parsed["sha256"]}
    elif args.command == "prepare":
        profile = load_json(args.profile)
        validate_profile(profile)
        pe = read_pe(args.target)
        sites = []
        for value in args.site:
            sid, rva, post, mask = value.split(":")
            sites.append(dict(id=int(sid), rva=int(rva, 16), post=int(post), mask=int(mask)))
        if profile.get("supports_return") is False and any(s["post"] for s in sites):
            raise ValueError("entry-only backend rejects RETURN sites")
        payload = config_bytes(pe, profile["client_sha256"], sites, args.max_events,
                               args.max_objects, args.prefix)
        args.output.mkdir()
        config = args.output.resolve() / "observer.cfg"
        config.write_bytes(payload)
        result = {
            "schema": 1, "target_sha256": pe["sha256"], "sites": sites,
            "config_path": str(config), "config_sha256": sha256(config),
            "profile_sha256": sha256(args.profile),
            "launch_argv": launch_argv(
                profile, config, args.output.resolve() / "events.jsonl", sha256(config),
                args.output.resolve() / "target.pid", args.target.resolve()),
            "execution_performed": False,
        }
        write_json(args.output / "plan.json", result)
    else:
        receipt = console_proof(args.profile, args.fixture, args.output)
        result = {"status": receipt["status"], "proof": str(args.output / "proof.json")}
    print(json.dumps(result, separators=(",", ":")))


if __name__ == "__main__":
    try:
        main()
    except (ValueError, KeyError, OSError, RuntimeError, subprocess.SubprocessError) as error:
        print(json.dumps({"status": "blocked", "error": str(error)}))
        raise SystemExit(2)
