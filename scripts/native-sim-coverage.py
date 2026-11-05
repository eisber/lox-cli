#!/usr/bin/env python3
"""Strict, offline import of the validated Windows DynamoRIO drcov format."""

from __future__ import annotations

import argparse
import bisect
import hashlib
import io
import json
import ntpath
import os
from pathlib import Path
import re
import struct
import sys
from dataclasses import dataclass


BB_ENTRY = struct.Struct("<IHH")
MAX_LOG_BYTES = 128 * 1024 * 1024
MAX_INPUT_BYTES = 512 * 1024 * 1024
MAX_BBS = 4_000_000
MAX_MODULES = 65_535
UNKNOWN_MODULE = 65_535
COLUMNS = (
    "Columns: id, containing_id, start, end, entry, offset, preferred_base, "
    "checksum, timestamp, path"
)


class CoverageError(ValueError):
    pass


def require(condition, message):
    if not condition:
        raise CoverageError(message)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def file_digest(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def sha256(value, label):
    require(isinstance(value, str) and re.fullmatch(r"[0-9a-fA-F]{64}", value),
            f"{label} must be a SHA-256 digest")
    return value.lower()


def number(value, label, maximum=(1 << 64) - 1):
    if isinstance(value, str):
        require(re.fullmatch(r"0x[0-9a-fA-F]+|[0-9]+", value),
                f"invalid {label}")
        value = int(value, 16 if value.startswith("0x") else 10)
    require(type(value) is int and 0 <= value <= maximum, f"invalid {label}")
    return value


def hex_number(value, label, maximum=(1 << 64) - 1, prefix=True):
    pattern = r"0x[0-9a-fA-F]{1,16}" if prefix else r"(?:0x)?[0-9a-fA-F]{1,16}"
    require(re.fullmatch(pattern, value), f"invalid {label}")
    result = int(value, 16)
    require(result <= maximum, f"invalid {label}")
    return result


def unique_keys(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, f"duplicate JSON key: {key}")
        result[key] = value
    return result


def read_json(path):
    with Path(path).open("rb") as stream:
        data = stream.read(MAX_INPUT_BYTES + 1)
    require(len(data) <= MAX_INPUT_BYTES, "JSON input exceeds bounded size limit")

    def reject_constant(value):
        raise CoverageError(f"non-JSON numeric constant: {value}")

    value = json.loads(data.decode("utf-8-sig"), object_pairs_hook=unique_keys,
                       parse_constant=reject_constant)
    require(isinstance(value, dict), "JSON input must be an object")
    return value, digest(data)


def windows_path(path):
    require(isinstance(path, str) and bool(path) and
            not any(ord(char) < 32 for char in path), "invalid module path")
    require(ntpath.isabs(path) and bool(ntpath.splitdrive(path)[0]),
            "target module path must be an absolute Windows path")
    return ntpath.normcase(ntpath.normpath(path))


@dataclass(frozen=True)
class Module:
    id: int
    path: str
    start: int
    end: int
    entry: int
    preferred_base: int
    checksum: int
    timestamp: int

    @property
    def size(self):
        return self.end - self.start


@dataclass(frozen=True)
class Drcov:
    modules: tuple[Module, ...]
    blocks: bytes
    sha256: str
    byte_count: int

    @property
    def bb_count(self):
        return len(self.blocks) // BB_ENTRY.size

    def entries(self):
        return BB_ENTRY.iter_unpack(self.blocks)


def parse_drcov(data):
    """Only v3 / Windows module-table v5 / binary <IHH>, observed on the tool."""
    require(len(data) <= MAX_LOG_BYTES, "drcov log exceeds bounded size limit")
    stream = io.BytesIO(data)

    def line():
        raw = stream.readline(65_537)
        require(raw.endswith(b"\n") and len(raw) <= 65_536,
                "truncated or oversized drcov header line")
        try:
            return raw.removesuffix(b"\n").removesuffix(b"\r").decode("utf-8")
        except UnicodeDecodeError as error:
            raise CoverageError("drcov header is not UTF-8") from error

    require(line() == "DRCOV VERSION: 3", "unsupported drcov version (expected 3)")
    require(line() == "DRCOV FLAVOR: drcov", "unsupported drcov flavor")
    match = re.fullmatch(r"Module Table: version 5, count ([0-9]+)", line())
    require(match is not None, "unsupported module table (expected Windows v5)")
    module_count = number(match[1], "module count", MAX_MODULES)
    require(line() == COLUMNS, "unsupported module columns or custom fields")
    modules = []
    for expected in range(module_count):
        fields = line().split(",", 9)
        require(len(fields) == 10, "invalid module row")
        fields = [field.strip() for field in fields]
        module_id = number(fields[0], "module id", UNKNOWN_MODULE - 1)
        containing_id = number(fields[1], "containing module id", UNKNOWN_MODULE - 1)
        require(module_id == expected, "module ids must be unique and sequential")
        require(containing_id == module_id,
                "segmented module tables are not supported by this Windows profile")
        start, end, entry = (hex_number(fields[i], "module address") for i in (2, 3, 4))
        offset = hex_number(fields[5], "module offset", prefix=False)
        preferred = hex_number(fields[6], "preferred image base")
        checksum = hex_number(fields[7], "PE checksum", (1 << 32) - 1)
        timestamp = hex_number(fields[8], "PE timestamp", (1 << 32) - 1)
        require(offset == 0, "nonzero module offset is unsupported on Windows")
        require(start < end and end - start <= 1 << 32, "invalid module bounds")
        require(entry == 0 or start <= entry < end, "module entry outside module bounds")
        path = fields[9]
        require(bool(path) and not any(ord(char) < 32 for char in path),
                "invalid module path")
        modules.append(Module(module_id, path, start, end, entry, preferred,
                              checksum, timestamp))
    match = re.fullmatch(r"BB Table: ([0-9]+) bbs", line())
    require(match is not None, "missing binary BB table")
    bb_count = number(match[1], "BB count", MAX_BBS)
    blocks = stream.read()
    require(len(blocks) == bb_count * BB_ENTRY.size,
            "BB count/length mismatch (truncated, trailing, or text-format data)")
    for rva, size, module_id in BB_ENTRY.iter_unpack(blocks):
        require(size > 0, "zero-sized BB")
        if module_id == UNKNOWN_MODULE:
            # Upstream explicitly truncates these PCs to 32 bits. Never map them.
            continue
        require(module_id < module_count, "BB references an undeclared module id")
        require(rva + size <= modules[module_id].size, "BB outside module bounds")
    return Drcov(tuple(modules), blocks, digest(data), len(data))


def read_drcov(path):
    with Path(path).open("rb") as stream:
        data = stream.read(MAX_LOG_BYTES + 1)
    return parse_drcov(data)


@dataclass(frozen=True)
class PE:
    sha256: str
    image_base: int
    image_size: int
    entry_rva: int
    checksum: int
    timestamp: int


def read_pe(path):
    with Path(path).open("rb") as stream:
        data = stream.read(MAX_INPUT_BYTES + 1)
    require(len(data) <= MAX_INPUT_BYTES, "PE input exceeds bounded size limit")
    require(len(data) >= 64 and data[:2] == b"MZ", "target is not a PE file")
    offset = struct.unpack_from("<I", data, 60)[0]
    require(64 <= offset <= len(data) - 24 and data[offset:offset + 4] == b"PE\0\0",
            "invalid PE signature or header bounds")
    machine, sections, timestamp = struct.unpack_from("<HHI", data, offset + 4)
    optional_size = struct.unpack_from("<H", data, offset + 20)[0]
    optional = offset + 24
    require(machine == 0x8664 and 0 < sections <= 96 and optional_size >= 112,
            "only native x64 PE32+ targets are supported")
    require(optional + optional_size + 40 * sections <= len(data),
            "truncated PE optional header or section table")
    require(struct.unpack_from("<H", data, optional)[0] == 0x20B,
            "target must be PE32+")
    entry = struct.unpack_from("<I", data, optional + 16)[0]
    base = struct.unpack_from("<Q", data, optional + 24)[0]
    size = struct.unpack_from("<I", data, optional + 56)[0]
    checksum = struct.unpack_from("<I", data, optional + 64)[0]
    require(base > 0 and size > 0 and entry < size and base + size < 1 << 64,
            "invalid PE image geometry")
    return PE(digest(data), base, size, entry, checksum, timestamp)


def select_target(log, target_sha256, manifest=None, pe=None, target_name=None):
    if manifest is not None:
        require(type(manifest.get("schema_version")) is int and
                manifest["schema_version"] == 1, "unsupported module manifest schema")
        require(sha256(manifest.get("raw_sha256"), "manifest raw_sha256") == log.sha256,
                "stale module manifest: raw log SHA-256 mismatch")
        pins = manifest.get("modules")
        require(isinstance(pins, list) and bool(pins), "manifest requires target modules")
        selected = set()
        for pin in pins:
            require(isinstance(pin, dict), "invalid module pin")
            module_id = number(pin.get("module_id"), "pinned module id", UNKNOWN_MODULE - 1)
            require(module_id not in selected and module_id < len(log.modules),
                    "duplicate or missing pinned module id")
            require(sha256(pin.get("sha256"), "pinned binary SHA-256") == target_sha256,
                    "module manifest and decompile index target SHA-256 differ")
            require(windows_path(pin.get("path")) == windows_path(log.modules[module_id].path),
                    "pinned module path differs from drcov module path")
            selected.add(module_id)
        status = "manifest_pinned"
    else:
        require(pe is not None and target_name, "provide --module-manifest or --target-pe")
        selected = {module.id for module in log.modules
                    if ntpath.basename(module.path).casefold() == target_name.casefold()}
        require(bool(selected), "target PE filename is absent from drcov module table")
        require(len(selected) == 1,
                "multiple matching module instances require an explicit module manifest")
        windows_path(log.modules[next(iter(selected))].path)
        status = "local_pe_metadata_matched"
    if pe is not None:
        require(pe.sha256 == target_sha256, "target PE and decompile index SHA-256 differ")
        for module_id in selected:
            module = log.modules[module_id]
            require(module.size >= pe.image_size and
                    module.entry == module.start + pe.entry_rva and
                    module.checksum == pe.checksum and module.timestamp == pe.timestamp,
                    "target module PE metadata differs from pinned PE")
    return selected, status


class RangeMapper:
    def __init__(self, index, image_base, image_size=None):
        functions = index.get("functions")
        require(isinstance(functions, list), "decompile index requires functions[]")
        self.functions = {}
        self.external_count = 0
        self.unavailable_count = 0
        intervals = []
        seen_addresses = set()
        for function in functions:
            require(isinstance(function, dict), "invalid function index entry")
            if function.get("status") == "external":
                self.external_count += 1
                continue
            address = function.get("address")
            require(isinstance(address, str) and re.fullmatch(r"0x[0-9a-fA-F]+", address),
                    "unsupported function address space")
            address = number(address, "function address")
            require(address not in seen_addresses, "duplicate function address")
            seen_addresses.add(address)
            body = function.get("body")
            require(isinstance(body, dict) and type(body.get("available")) is bool,
                    "function requires explicit body availability")
            ranges = body.get("ranges")
            require(isinstance(ranges, list), "function body requires exact ranges[]")
            if not body["available"] or not ranges:
                self.unavailable_count += 1
                continue
            require(isinstance(function.get("name"), str), "function name is missing")
            self.functions[address] = function["name"]
            for span in ranges:
                require(isinstance(span, dict), "invalid function range")
                start = number(span.get("start"), "range start")
                end = number(span.get("end"), "inclusive range end")
                require(image_base <= start <= end < 1 << 64, "invalid function body range")
                if image_size is not None:
                    require(end < image_base + image_size, "function range outside PE image")
                intervals.append((start - image_base, end - image_base + 1, address))
        intervals.sort()
        self.intervals = intervals
        self.starts = [interval[0] for interval in intervals]
        self.max_ends = []
        maximum = 0
        for _, end, _ in intervals:
            maximum = max(maximum, end)
            self.max_ends.append(maximum)

    def map(self, rva, size):
        end = rva + size
        position = bisect.bisect_left(self.starts, end) - 1
        candidates = set()
        whole = set()
        while position >= 0 and self.max_ends[position] > rva:
            start, stop, address = self.intervals[position]
            if stop > rva:
                candidates.add(address)
                if start <= rva and end <= stop:
                    whole.add(address)
            position -= 1
        if len(candidates) > 1:
            status = "ambiguous"
        elif len(whole) == 1:
            status = "confirmed"
        elif candidates:
            status = "crosses_range"
        else:
            status = "unmapped"
        return {"status": status, "functions": [f"0x{value:x}" for value in sorted(candidates)]}


def relative_path(path, root):
    try:
        return str(Path(path).resolve().relative_to(root.resolve()))
    except ValueError as error:
        raise CoverageError("raw log and output artifacts must be beneath --artifact-root") from error


def artifact(path, root, kind, known_hash=None):
    return {"kind": kind, "path": relative_path(path, root),
            "sha256": known_hash or file_digest(path), "bytes": Path(path).stat().st_size}


def build_report(log, mapper, selected, target_sha256, image_base):
    rows = []
    cached = {}
    counts = dict.fromkeys(("confirmed", "ambiguous", "crosses_range", "unmapped"), 0)
    executed = set()
    unknown_count = 0
    for ordinal, (rva, size, module_id) in enumerate(log.entries()):
        if module_id == UNKNOWN_MODULE:
            unknown_count += 1
        if module_id not in selected:
            continue
        key = (rva, size)
        if key not in cached:
            cached[key] = mapper.map(rva, size)
        mapping = cached[key]
        counts[mapping["status"]] += 1
        if mapping["status"] == "confirmed":
            executed.update(mapping["functions"])
        rows.append({"record": ordinal, "module_id": module_id,
                     "rva": f"0x{rva:x}", "size": size, **mapping})
    report = {
        "schema_version": 1,
        "kind": "native-basic-block-coverage",
        "application": {"sha256": target_sha256, "version": None},
        "measurement": {
            "tool": "DynamoRIO/drcov", "tool_version": None,
            "raw_log_sha256": log.sha256,
            "drcov_version": 3, "module_table_version": 5, "bb_entry": "<IHH",
            "unit": "observed_basic_block_entries_not_edges_or_hit_counts",
        },
        "scope": {
            "status": "unknown", "unknown_count": len(mapper.functions),
            "reason": "Recovered function roles have not been reviewed as simulator scope.",
        },
        "observed": {
            "module_count": len(log.modules), "bb_count": log.bb_count,
            "target_module_count": len(selected), "target_bb_count": len(rows),
            "target_unique_bb_count": len(cached),
            "unknown_module_bb_count": unknown_count,
            "executed_function_count": len(executed),
            "function_count_scope": "whole_executable_not_simulator_specific",
        },
        "mapping": {
            **counts, "recognized_function_count": len(mapper.functions),
            "external_function_count": mapper.external_count,
            "functions_without_ranges": mapper.unavailable_count,
            "image_base": f"0x{image_base:x}",
            "rule": "whole_BB_inside_one_exact_range_with_no_other_function_overlap",
        },
        "coverage": {
            "basic_block_percentage": None, "branch_percentage": None,
            "source_line_percentage": None, "full_engine_coverage": False,
            "completion_status": "blocked",
            "blockers": ["simulator_scope_unknown", "static_CFG_BB_denominator_unavailable",
                         "BB_entries_do_not_measure_branches_or_pseudocode_lines"],
        },
    }
    bb_map = {
        "schema_version": 1, "kind": "native-basic-block-function-map",
        "application_sha256": target_sha256, "raw_log_sha256": log.sha256,
        "blocks": rows,
        "executed_functions": [
            {"address": address, "name": mapper.functions[int(address, 16)]}
            for address in sorted(executed, key=lambda value: int(value, 16))
        ],
    }
    return report, bb_map


def write_exclusive(path, value):
    created = False
    try:
        with Path(path).open("x", encoding="utf-8", newline="\n") as stream:
            created = True
            json.dump(value, stream, separators=(",", ":"), ensure_ascii=True)
            stream.write("\n")
    except BaseException:
        if created:
            Path(path).unlink(missing_ok=True)
        raise


def run(args):
    root = args.artifact_root.resolve()
    require(root.is_dir(), "--artifact-root must be an existing directory")
    relative_path(args.raw_log, root)
    relative_path(args.output, root)
    require(not args.output.exists(), "refusing to overwrite output")
    log = read_drcov(args.raw_log)
    raw_artifact = artifact(args.raw_log, root, "drcov-log", log.sha256)
    if args.command == "inspect":
        report = {
            "schema_version": 1, "kind": "native-drcov-structure-inspection",
            "raw_log_sha256": log.sha256,
            "format": {"drcov_version": 3, "module_table_version": 5, "bb_entry": "<IHH"},
            "observed": {"module_count": len(log.modules), "bb_count": log.bb_count,
                         "unknown_module_bb_count": sum(
                             module_id == UNKNOWN_MODULE for _, _, module_id in log.entries())},
            "identity_status": "not_checked", "coverage_claim": False,
            "artifact_root": os.path.relpath(root, args.output.parent.resolve()),
            "artifacts": [raw_artifact],
        }
        write_exclusive(args.output, report)
        return report

    require(args.module_manifest is not None or args.target_pe is not None,
            "provide --module-manifest or --target-pe")
    index, index_hash = read_json(args.decompile_index)
    target_sha256 = sha256(index.get("target_sha256"), "decompile index target_sha256")
    manifest, manifest_hash = read_json(args.module_manifest) if args.module_manifest else (None, None)
    pe = read_pe(args.target_pe) if args.target_pe else None
    if pe is not None:
        require(args.image_base is None or args.image_base == pe.image_base,
                "--image-base differs from pinned PE ImageBase")
        image_base = pe.image_base
    else:
        require(args.image_base is not None, "manifest-only import requires explicit --image-base")
        image_base = args.image_base
    selected, identity = select_target(
        log, target_sha256, manifest, pe, args.target_pe.name if pe else None)
    image_sizes = {log.modules[module_id].size for module_id in selected}
    require(len(image_sizes) == 1, "target module instances disagree on image size")
    mapper = RangeMapper(index, image_base, pe.image_size if pe else next(iter(image_sizes)))
    report, bb_map = build_report(log, mapper, selected, target_sha256, image_base)
    report["identity"] = {
        "status": identity, "target_module_ids": sorted(selected),
        "local_pe_checked": pe is not None,
        "limitation": "drcov does not embed binary hashes; a manifest is a collector assertion, "
                      "not a cryptographic measurement of loaded memory",
    }
    if pe is not None:
        # The real Windows console smoke has an expanded runtime span and a
        # logged preferred_base equal to its ASLR base, not its on-disk ImageBase.
        report["identity"]["pe_image_size"] = pe.image_size
        report["identity"]["runtime_geometry"] = [
            {"module_id": module_id,
             "module_span_matches_pe": log.modules[module_id].size == pe.image_size,
             "logged_preferred_base_matches_pe":
                 log.modules[module_id].preferred_base == pe.image_base}
            for module_id in sorted(selected)
        ]
    report["inputs"] = {"decompile_index_sha256": index_hash,
                        "module_manifest_sha256": manifest_hash}
    report["artifact_root"] = os.path.relpath(root, args.output.parent.resolve())
    if args.tool_receipt is not None:
        receipt, receipt_hash = read_json(args.tool_receipt)
        require(type(receipt.get("schema_version")) is int and receipt["schema_version"] == 1 and
                receipt.get("repository") == "DynamoRIO/dynamorio",
                "unsupported tool receipt")
        archive_hash = sha256(receipt.get("sha256"), "tool archive SHA-256")
        require(receipt.get("github_digest") == "sha256:" + archive_hash,
                "tool receipt archive and GitHub digest differ")
        release = receipt.get("release")
        require(isinstance(release, str) and bool(release), "tool receipt release is missing")
        report["measurement"].update(tool_version=release, tool_archive_sha256=archive_hash,
                                     tool_receipt_sha256=receipt_hash)
    map_path = args.bb_map or args.output.with_name(args.output.stem + ".bb-map.json")
    relative_path(map_path, root)
    require(map_path.resolve() != args.output.resolve() and not map_path.exists(),
            "refusing to overwrite BB map")
    created = []
    try:
        # Exclusive opens, including the second output, prevent overwrite races.
        with map_path.open("x", encoding="utf-8", newline="\n") as stream:
            created.append(map_path)
            json.dump(bb_map, stream, separators=(",", ":"), ensure_ascii=True)
            stream.write("\n")
        report["artifacts"] = [raw_artifact, artifact(map_path, root, "basic-block-function-map")]
        with args.output.open("x", encoding="utf-8", newline="\n") as stream:
            created.append(args.output)
            json.dump(report, stream, separators=(",", ":"), ensure_ascii=True)
            stream.write("\n")
    except BaseException:
        for path in created:
            path.unlink(missing_ok=True)
        raise
    return report


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    subcommands = parser.add_subparsers(dest="command", required=True)
    for command in ("inspect", "import"):
        sub = subcommands.add_parser(command)
        sub.add_argument("raw_log", type=Path)
        sub.add_argument("--artifact-root", type=Path, required=True)
        sub.add_argument("--output", type=Path, required=True)
        if command == "import":
            sub.add_argument("--module-manifest", type=Path)
            sub.add_argument("--target-pe", type=Path)
            sub.add_argument("--image-base", type=lambda value: number(value, "image base"))
            sub.add_argument("--decompile-index", type=Path, required=True)
            sub.add_argument("--tool-receipt", type=Path)
            sub.add_argument("--bb-map", type=Path)
    args = parser.parse_args(argv)
    try:
        report = run(args)
    except (OSError, ValueError, UnicodeError, struct.error) as error:
        print(json.dumps({"error": str(error)}), file=sys.stderr)
        return 2
    print(json.dumps({"artifact": str(args.output), "kind": report["kind"],
                      "observed": report["observed"]}, separators=(",", ":")))
    return 0


if __name__ == "__main__":
    sys.exit(main())
