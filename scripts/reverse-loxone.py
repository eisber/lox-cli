#!/usr/bin/env python3
"""Retain local, versioned Loxone Config analysis using a caller-installed REA."""

import argparse
from collections import Counter
from contextlib import contextmanager, redirect_stderr, redirect_stdout
from datetime import datetime, timezone
import errno
import hashlib
import json
import os
from pathlib import Path
import shutil
import sqlite3
import struct
import subprocess
import sys
import time
import xml.etree.ElementTree as ET
import zlib


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_INSTALL = Path(r"C:\Program Files (x86)\Loxone\LoxoneConfig")
TOPICS = {
    "config-format": ["LoxCC", "LoxApp3", "IoData", "LightsceneC"],
    "block-semantics": ["OnPulseDelay", "DayTimer", "LightController2", "Jalousie"],
    "protocol": ["jdev/", "websocket", "getkey2", "binstatisticdata"],
}


def now():
    return datetime.now(timezone.utc).isoformat()


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def json_digest(value):
    return hashlib.sha256(
        json.dumps(value, sort_keys=True, separators=(",", ":")).encode("utf-8")
    ).hexdigest()


def write_json(path, value):
    temporary = path.with_suffix(path.suffix + ".tmp")
    with temporary.open("w", encoding="utf-8", newline="\n") as stream:
        json.dump(value, stream, indent=2, ensure_ascii=True)
        stream.write("\n")
    temporary.replace(path)


def read_json(path):
    with path.open(encoding="utf-8") as stream:
        return json.load(stream)


def inventory(directory):
    records = []
    for path in sorted(directory.rglob("*")):
        if path.is_symlink() or path.is_junction():
            raise RuntimeError(f"Installation contains a linked path: {path}")
        if path.is_file():
            records.append({
                "path": str(path.relative_to(directory)),
                "size": path.stat().st_size,
                "sha256": digest(path),
            })
    if not any(row["path"] == "LoxoneConfig.exe" for row in records):
        raise RuntimeError(f"No LoxoneConfig.exe in {directory}")
    return records


def installation_delta(before, after):
    old = {row["path"]: row for row in before}
    new = {row["path"]: row for row in after}
    return {
        "added": sorted(new.keys() - old.keys()),
        "removed": sorted(old.keys() - new.keys()),
        "changed": sorted(
            name for name in old.keys() & new.keys()
            if old[name]["sha256"] != new[name]["sha256"]
        ),
    }


def snapshot(installation, output):
    print(f"[{now()}] Hashing installation: {installation}", flush=True)
    files = inventory(installation)
    identity = json_digest(files)
    directory = output / "versions" / identity
    directory.mkdir(parents=True, exist_ok=True)
    manifest_path = directory / "installation.json"
    if manifest_path.exists():
        if read_json(manifest_path)["files"] != files:
            raise RuntimeError(f"Snapshot manifest conflicts with {identity}")
        for row in files:
            target = directory / "source" / row["path"]
            if not target.is_file() or digest(target) != row["sha256"]:
                raise RuntimeError(f"Frozen source was modified: {target}")
        return directory, read_json(manifest_path)

    print(f"[{now()}] Freezing {len(files)} installation files: {identity}", flush=True)
    for row in files:
        source = installation / row["path"]
        target = directory / "source" / row["path"]
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, target)
        if digest(target) != row["sha256"]:
            raise RuntimeError(f"Installation changed during snapshot: {source}; retry")
    if inventory(installation) != files:
        raise RuntimeError("Installation changed during snapshot; retry")
    manifest = {
        "schema_version": 1,
        "captured_at": now(),
        "installation": str(installation),
        "installation_sha256": identity,
        "files": files,
    }
    write_json(manifest_path, manifest)
    return directory, manifest


@contextmanager
def exclusive_run(output):
    # OS locks disappear on process death, unlike a PID-only lock file.
    with (output / "run.lock").open("a+b") as lock:
        lock.seek(0, os.SEEK_END)
        if lock.tell() == 0:
            lock.write(b"\0")
            lock.flush()
        lock.seek(0)
        try:
            if os.name == "nt":
                import msvcrt
                msvcrt.locking(lock.fileno(), msvcrt.LK_NBLCK, 1)
            else:
                import fcntl
                fcntl.flock(lock.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        except OSError as error:
            if error.errno in (errno.EACCES, errno.EAGAIN):
                raise RuntimeError(f"Another analysis already owns {output}; retry later") from error
            raise
        try:
            yield
        finally:
            lock.seek(0)
            if os.name == "nt":
                msvcrt.locking(lock.fileno(), msvcrt.LK_UNLCK, 1)
            else:
                fcntl.flock(lock.fileno(), fcntl.LOCK_UN)


def tool_value(result):
    """Unwrap MCP structured output without turning error content into success."""
    if result.get("isError"):
        raise RuntimeError(f"REA tool failed: {json.dumps(result)}")
    value = result.get("structuredContent")
    if value is None:
        texts = [
            item["text"] for item in result.get("content", [])
            if item.get("type") == "text"
        ]
        if len(texts) != 1:
            raise RuntimeError("REA did not return one structured tool result")
        value = json.loads(texts[0])
    if isinstance(value, dict) and value.get("ok") is False:
        raise RuntimeError(f"REA operation failed: {json.dumps(value)}")
    return value


def normalized(value):
    """Project the observed REA Evidence envelope, retaining the original on disk."""
    if isinstance(value, dict) and "normalized_result" in value:
        return value["normalized_result"]
    return value


class Rea:
    def __init__(self, args, directory, environment):
        self.directory = directory
        self.sequence = 0
        self.last_progress = 0
        self.stderr = (directory / "rea.stderr.log").open("a", encoding="utf-8")
        self.process = subprocess.Popen(
            [args.node, str(args.rea_root / "scripts" / "rea.mjs"), "mcp"],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=self.stderr,
            text=True, encoding="utf-8", env=environment,
        )

    def request(self, method, parameters):
        self.sequence += 1
        request = {
            "jsonrpc": "2.0", "id": self.sequence,
            "method": method, "params": parameters,
        }
        self.process.stdin.write(json.dumps(request) + "\n")
        self.process.stdin.flush()
        while True:
            line = self.process.stdout.readline()
            if not line:
                raise RuntimeError(
                    f"REA closed its transport; see {self.directory / 'rea.stderr.log'}"
                )
            response = json.loads(line)
            if response.get("id") == self.sequence:
                if "error" in response:
                    raise RuntimeError(f"REA RPC failed: {json.dumps(response['error'])}")
                return response["result"]
            if response.get("method") == "notifications/progress":
                if time.monotonic() - self.last_progress >= 30:
                    print(f"[{now()}] REA progress: {response['params']}", flush=True)
                    self.last_progress = time.monotonic()

    def initialize(self):
        response = self.request("initialize", {
            "protocolVersion": "2024-11-05", "capabilities": {},
            "clientInfo": {"name": "lox-cli-local-analysis", "version": "1"},
        })
        write_json(self.directory / "server.json", response)
        self.process.stdin.write(
            '{"jsonrpc":"2.0","method":"notifications/initialized"}\n'
        )
        self.process.stdin.flush()
        tools = self.request("tools/list", {})
        write_json(self.directory / "tools.json", tools)
        self.tools = {tool["name"] for tool in tools["tools"]}

    def call(self, name, arguments, filename):
        if name not in self.tools:
            raise RuntimeError(f"Installed REA does not provide required tool {name}")
        print(f"[{now()}] {name}", flush=True)
        result = self.request("tools/call", {
            "name": name, "arguments": arguments,
            "_meta": {"progressToken": str(self.sequence + 1)},
        })
        write_json(self.directory / filename, result)
        return tool_value(result)

    def shutdown(self):
        self.process.stdin.close()
        try:
            code = self.process.wait(timeout=120)
        except subprocess.TimeoutExpired:
            self.process.terminate()
            self.process.wait(timeout=30)
            raise RuntimeError("REA did not shut down cleanly; inspect rea.stderr.log")
        finally:
            self.process.stdout.close()
            self.stderr.close()
        if code:
            raise RuntimeError(f"REA exited with status {code}")


def toolchain(args):
    rea = args.rea_root
    required = [
        rea / "package.json", rea / "package-lock.json",
        rea / "scripts" / "rea.mjs",
        rea / "native" / "windows" / "build" / "manifest.json",
        args.ghidra_dir / "Ghidra" / "application.properties",
        args.java_home / "release",
        args.lox,
        Path(__file__),
    ]
    runtime = sorted((rea / "dist").rglob("*"))
    bridges = sorted((rea / "bridge" / "ghidra").rglob("*"))
    if not runtime:
        raise RuntimeError("Build REA first: npm ci && npm run build:cached")
    facts = {
        "rea_version": read_json(rea / "package.json")["version"],
        "files": {
            str(path): digest(path)
            for path in required + runtime + bridges if not path.is_dir()
        },
        "settings": {
            "provider": "ghidra",
            "startup_timeout_ms": args.startup_timeout * 1000,
            "decompile_timeout_seconds": args.decompile_timeout,
            "heap": args.heap,
            "binaries": args.binary,
            "topics": TOPICS,
        },
    }
    return facts


def analysis_environment(args):
    environment = os.environ.copy()
    # These settings belong to a prior investigation, not this pipeline's profile.
    for name in list(environment):
        if name.startswith(("REA_GHIDRA_", "REA_ANALYSIS_PROVIDER")):
            del environment[name]
    environment.update({
        "GHIDRA_INSTALL_DIR": str(args.ghidra_dir),
        "JAVA_HOME": str(args.java_home),
        "REA_ANALYSIS_PROVIDER": "ghidra",
        "REA_GHIDRA_STARTUP_TIMEOUT_MS": str(args.startup_timeout * 1000),
        "REA_GHIDRA_DECOMPILE_TIMEOUT_SECONDS": str(args.decompile_timeout),
        "GHIDRA_HEADLESS_MAXMEM": args.heap,
    })
    return environment


def analyze_binary(args, source, directory):
    directory.mkdir(parents=True, exist_ok=True)
    rea = Rea(args, directory, analysis_environment(args))
    opened = False
    try:
        rea.initialize()
        rea.call("open_binary", {
            "path": str(source), "provider_id": "ghidra",
        }, "open.json")
        opened = True
        rea.call("inspect_pe_imports", {"path": str(source)}, "imports.json")
        rea.call("binary_overview", {}, "overview.json")
        for tool in ("list_procedures", "list_names", "list_strings"):
            rea.call(tool, {"detail": "summary"}, tool + ".json")
        rea.call("list_segments", {}, "segments.json")
        for topic, terms in TOPICS.items():
            rea.call("search_strings", {
                "pattern": "|".join(terms), "mode": "regex", "case_sensitive": False,
                "detail": "summary",
            }, topic + ".json")
        rea.call("export_evidence_bundle", {
            "path": str(directory / "inventory.evidence.json"), "overwrite": True,
        }, "inventory-export.json")
        code = directory / "code"
        result = rea.call("decompile_all", {
            "output_directory": str(code), "resume": code.exists(),
        }, "decompile.json")
        rea.call("export_evidence_bundle", {
            "path": str(directory / "complete.evidence.json"), "overwrite": True,
        }, "complete-export.json")
        return normalized(result)
    finally:
        try:
            if opened:
                rea.call("close_binary", {}, "close.json")
        finally:
            rea.shutdown()


def index_code(directory):
    """Build a local search index; no recovered text is written to tracked files."""
    index = read_json(directory / "code" / "decompile-index.json")
    database = sqlite3.connect(directory / "research.sqlite3")
    try:
        database.execute("DROP TABLE IF EXISTS functions")
        database.execute(
            "CREATE VIRTUAL TABLE functions USING fts5(address UNINDEXED, name, "
            "path UNINDEXED, body)"
        )
        topics = {topic: [] for topic in TOPICS}
        for row in index["functions"]:
            if row["status"] != "written":
                continue
            path = directory / "code" / Path(row["relative_path"].replace("/", os.sep))
            if digest(path) != row["sha256"]:
                raise RuntimeError(f"Decompile output digest mismatch: {path}")
            body = path.read_text(encoding="utf-8")
            database.execute(
                "INSERT INTO functions VALUES (?, ?, ?, ?)",
                (row["address"], row["name"], str(path), body),
            )
            for topic, terms in TOPICS.items():
                found = [term for term in terms if term.lower() in body.lower()]
                if found:
                    topics[topic].append({
                        "address": row["address"], "name": row["name"],
                        "path": str(path), "terms": found,
                    })
        database.commit()
        write_json(directory / "topics.json", topics)
        return dict(Counter(row["status"] for row in index["functions"]))
    finally:
        database.close()


def catalog_blocks(root):
    return [
        {
            "attributes": dict(block.attrib),
            "groups": [
                {
                    "attributes": dict(group.attrib),
                    "io": [dict(io.attrib) for io in group.findall("IO")],
                }
                for group in block.findall("IOGroup")
            ],
        }
        for block in root.iter("FunctionBlock")
    ]


def extract_resources(args, version, analysis):
    resources = analysis / "resources"
    resources.mkdir(exist_ok=True)
    environment = os.environ.copy()
    environment["LOX_TELEMETRY"] = "0"
    records = []
    with (resources / "extraction.log").open("a", encoding="utf-8") as log:
        for source in sorted((version / "source").glob("*.LxRes")):
            target = resources / (source.name + ".xml")
            print(f"[{now()}] Extracting {source.name}", flush=True)
            subprocess.run(
                [str(args.lox), "--quiet", "config", "extract", str(source),
                 "--save-as", str(target)],
                env=environment, stdout=log, stderr=log, timeout=120, check=True,
            )
            header = source.read_bytes()[:16]
            if len(header) != 16:
                raise RuntimeError(f"Truncated LoxCC resource header: {source}")
            magic, _, size, crc = struct.unpack("<IIII", header)
            content = target.read_bytes()
            if magic != 0xAABBCCEE or len(content) != size or zlib.crc32(content) != crc:
                raise RuntimeError(f"Resource header/length/CRC mismatch: {source}")
            root = ET.fromstring(content)
            blocks = catalog_blocks(root)
            record = {
                "source": source.name, "source_sha256": digest(source),
                "xml": str(target), "xml_sha256": digest(target),
                "root": root.tag, "function_blocks": len(blocks),
            }
            records.append(record)
            if source.name == "tdc_DEV.LxRes":
                catalog = {
                    "source": record, "blocks": blocks,
                    "limitations": [
                        "IO Id/Name/ShortName and LxType are resource observations; "
                        "they do not independently establish serialized connector keys."
                    ],
                }
                write_json(resources / "block-catalog.json", catalog)
    write_json(resources / "manifest.json", records)
    return {"extracted": len(records), "manifest": str(resources / "manifest.json")}


def run(args):
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    with exclusive_run(output):
        version, manifest = snapshot(args.install_dir, output)
        latest_path = output / "latest.json"
        previous = read_json(latest_path) if latest_path.exists() else None
        if previous and previous["installation_sha256"] != manifest["installation_sha256"]:
            old = read_json(Path(previous["version_directory"]) / "installation.json")
            write_json(version / "installation-diff.json", {
                "from": old["installation_sha256"],
                "to": manifest["installation_sha256"],
                **installation_delta(old["files"], manifest["files"]),
            })
        facts = toolchain(args)
        analysis_id = json_digest(facts)
        analysis = version / "analysis" / analysis_id
        analysis.mkdir(parents=True, exist_ok=True)
        write_json(analysis / "toolchain.json", facts)
        status_path = analysis / "status.json"
        if status_path.exists() and read_json(status_path)["status"] == "complete":
            write_json(latest_path, read_json(status_path))
            print(f"Unchanged installation and toolchain; analysis retained at {analysis}")
            return 0
        status = {
            "status": "running", "started_at": now(), "analysis_id": analysis_id,
            "installation_sha256": manifest["installation_sha256"],
            "version_directory": str(version), "analysis_directory": str(analysis),
            "binaries": {},
        }
        write_json(status_path, status)
        write_json(latest_path, status)
        try:
            status["resources"] = extract_resources(args, version, analysis)
            write_json(status_path, status)
            for binary in args.binary:
                source = version / "source" / binary
                if not source.is_file():
                    raise RuntimeError(f"Requested binary is not installed: {source}")
                directory = analysis / binary
                result = analyze_binary(args, source, directory)
                counts = index_code(directory)
                status["binaries"][binary] = {
                    "counts": counts, "export": result,
                    "database": str(directory / "research.sqlite3"),
                }
                write_json(status_path, status)
            incomplete = any(
                value["counts"].get("failed", 0)
                or value["export"]["counts"]["remaining"]
                for value in status["binaries"].values()
            )
            status["status"] = "partial" if incomplete else "complete"
            status["finished_at"] = now()
        except (
            OSError, RuntimeError, ValueError, KeyError, sqlite3.Error,
            subprocess.SubprocessError, ET.ParseError,
        ) as error:
            status.update(status="failed", failed_at=now(), error=str(error))
            raise
        finally:
            write_json(status_path, status)
            write_json(latest_path, status)
        print(json.dumps(status, indent=2), flush=True)
        return 2 if incomplete else 0


def arguments():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--install-dir", type=Path, default=DEFAULT_INSTALL)
    parser.add_argument("--rea-root", type=Path, required=True)
    parser.add_argument("--output", type=Path, default=ROOT / ".reverse-engineering")
    parser.add_argument("--ghidra-dir", type=Path, required=True)
    parser.add_argument("--java-home", type=Path, required=True)
    parser.add_argument("--node", default="node")
    parser.add_argument("--lox", type=Path, default=ROOT / "target" / "release" / "lox.exe",
                        help="Built CLI used to decode installed LxRes resources")
    parser.add_argument("--log-file", type=Path, help="Append unattended run output here")
    parser.add_argument("--binary", action="append",
                        help="Installed PE filename; repeat for additional vendor binaries")
    parser.add_argument("--heap", default="6G", help="Ghidra JVM heap")
    parser.add_argument("--startup-timeout", type=int, default=7200, help="Seconds")
    parser.add_argument("--decompile-timeout", type=int, default=60, help="Seconds/function")
    args = parser.parse_args()
    args.binary = args.binary or ["LoxoneConfig.exe"]
    for binary in args.binary:
        if Path(binary).name != binary or binary in (".", ".."):
            parser.error("--binary must be a filename inside the installation root")
    for field in ("rea_root", "ghidra_dir", "java_home", "install_dir", "lox"):
        setattr(args, field, getattr(args, field).resolve(strict=True))
    if args.startup_timeout < 1 or args.decompile_timeout < 1:
        parser.error("timeouts must be positive")
    return args


def execute(args):
    try:
        return run(args)
    except (
        OSError, RuntimeError, ValueError, KeyError, sqlite3.Error,
        subprocess.SubprocessError, ET.ParseError,
    ) as error:
        print(f"Analysis failed: {error}", file=sys.stderr, flush=True)
        return 1


if __name__ == "__main__":
    options = arguments()
    if options.log_file:
        options.log_file.parent.mkdir(parents=True, exist_ok=True)
        with options.log_file.open("a", encoding="utf-8", buffering=1) as log:
            with redirect_stdout(log), redirect_stderr(log):
                sys.exit(execute(options))
    else:
        sys.exit(execute(options))
