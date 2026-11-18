#!/usr/bin/env python3
"""Run toolkit checks with conservative, content-bound successful-result reuse."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import time
import tomllib
import uuid

ROOT = Path(__file__).resolve().parent.parent


def digest(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def identity(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True).encode()).hexdigest()


def files_identity(paths):
    return {str(path): digest(path) for path in sorted(set(map(Path, paths)))}


def source_identity(paths):
    result = {}
    for path in paths:
        if path.is_symlink():
            raise ValueError(f"source symlink requires an explicit input contract: {path}")
        result[str(path)] = digest(path) if path.is_file() else None
    return result


def source_files():
    names = subprocess.check_output(
        ["git", "-C", str(ROOT), "ls-files", "-z", "--cached", "--others", "--exclude-standard"]
    ).split(b"\0")
    return sorted({ROOT / os.fsdecode(name) for name in names if name})


def executable(name, env):
    path = shutil.which(str(name), path=env.get("PATH"))
    if path is None:
        raise ValueError(f"executable not found: {name}")
    return Path(path).resolve()


def configuration_files(env):
    paths = []
    for folder in (ROOT, *ROOT.parents, Path(env["CARGO_HOME"])):
        base = folder if folder == Path(env["CARGO_HOME"]) else folder / ".cargo"
        paths.extend(path for path in (base / "config", base / "config.toml") if path.is_file())
    return paths


def reuse_blockers(env):
    blockers = []
    for path in configuration_files(env):
        config = tomllib.loads(path.read_text(encoding="utf-8"))
        for key in ("include", "env", "alias", "paths", "patch", "replace"):
            if key in config:
                blockers.append(f"{path}: {key} has inputs outside the supported reuse contract")
        for key in ("rustc", "rustc-wrapper", "rustc-workspace-wrapper"):
            if config.get("build", {}).get(key):
                blockers.append(f"{path}: build.{key} execution override")
        for target in config.get("target", {}).values():
            if isinstance(target, dict) and any(key in target for key in ("linker", "runner")):
                blockers.append(f"{path}: target linker/runner execution override")
        for source in config.get("source", {}).values():
            if isinstance(source, dict) and any(key in source for key in ("directory", "local-registry")):
                blockers.append(f"{path}: local dependency source override")
    for key in ("RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER"):
        if env.get(key):
            blockers.append(f"{key} may load untracked implementation inputs")
    for key, value in env.items():
        if (value and key.upper().startswith("CARGO_TARGET_")
                and key.upper().endswith(("_RUNNER", "_LINKER"))):
            blockers.append(f"{key} execution override")
    return blockers


def native_tool_payloads(java_home, kotlin_compiler):
    kotlin_lib = Path(kotlin_compiler).resolve().parent.parent / "lib"
    if not (kotlin_lib / "kotlin-compiler.jar").is_file():
        raise ValueError("Native reuse requires the Kotlin distribution lib/kotlin-compiler.jar")
    paths = [path for path in kotlin_lib.rglob("*") if path.is_file()]
    paths.extend(Path(java_home) / name for name in ("lib/modules", "bin/server/jvm.dll"))
    if not all(path.is_file() for path in paths):
        raise ValueError("Native reuse requires the actual Java modules and JVM implementation")
    return files_identity(paths)


def inputs(env, commands, paths=None, tools=()):
    rust_programs = ["cargo", "rustc", "rustdoc", "rustfmt", "cargo-clippy", "clippy-driver"]
    programs = [*rust_programs, *tools]
    programs.extend(env[key] for key in ("RUSTC", "RUSTDOC", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER")
                    if env.get(key))
    resolved = [executable(name, env) for name in programs]
    if shutil.which("rustup", path=env.get("PATH")):
        # PATH executables can be identical rustup proxies rather than compilers.
        resolved.extend(executable(subprocess.check_output(
            ["rustup", "which", name], cwd=ROOT, env=env, text=True).strip(), env)
            for name in rust_programs)
    return {
        "files": source_identity(source_files() if paths is None else paths),
        "cargo_configuration": files_identity(configuration_files(env)),
        "commands": commands,
        "executables": files_identity(resolved),
        "rustc": subprocess.check_output(["rustc", "-Vv"], cwd=ROOT, env=env, text=True).strip(),
        "cargo": subprocess.check_output(["cargo", "-V"], cwd=ROOT, env=env, text=True).strip(),
        "platform": [platform.system(), platform.machine(), platform.python_version()],
        # Hash rather than disclose credentials or other inherited private values.
        "environment_sha256": identity(dict(env)),
        "working_directory": str(ROOT),
        "reuse_blockers": reuse_blockers(env),
    }


def valid_result(record, key, command_count):
    if not record or record.get("key") != key or record.get("status") != "passed":
        return False
    commands = record.get("commands", [])
    artifacts = record.get("artifacts", {})
    if len(commands) != command_count or not artifacts or any(row["exit_code"] for row in commands):
        return False
    return all(Path(path).is_file() and digest(path) == expected
               for path, expected in artifacts.items())


def write(path, data):
    temporary = path.with_name(path.name + ".tmp")
    temporary.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    temporary.replace(path)


def save_result(path, manifest, record):
    write(Path(record["run_directory"]) / "result.json", record)
    write(path, manifest)


def execute(command, env, log):
    started = time.monotonic()
    with log.open("wb") as output:
        result = subprocess.run(command, cwd=ROOT, env=env, stdout=output, stderr=subprocess.STDOUT)
    return {"command": list(command), "exit_code": result.returncode,
            "seconds": time.monotonic() - started, "log": str(log)}


def cargo_artifacts(log):
    paths = []
    for line in log.read_text(encoding="utf-8", errors="replace").splitlines():
        if not line.startswith("{"):
            continue
        message = json.loads(line)
        if message.get("reason") == "compiler-artifact":
            paths.extend(Path(name) for name in message["filenames"] if Path(name).is_file())
    return paths


def environment(results, target):
    env = os.environ.copy()
    cargo_home = env.get("CARGO_HOME", str(Path.home() / ".cargo"))
    rustup_home = env.get("RUSTUP_HOME", str(Path.home() / ".rustup"))
    for key in list(env):
        upper = key.upper()
        if upper.startswith(("LOX", "MINISERVER")) or any(word in upper for word in
                ("PASSWORD", "TOKEN", "SECRET", "CREDENTIAL", "CONNECTIONSTRING")):
            del env[key]
    home = results.parent / "validation-home"
    temp = results.parent / "validation-temp"
    home.mkdir(exist_ok=True)
    temp.mkdir(exist_ok=True)
    config = home / "config.yaml"
    if config.exists() and config.read_bytes() != b"telemetry: false\n":
        raise ValueError(f"private validation configuration was modified: {config}")
    config.write_bytes(b"telemetry: false\n")
    env.update(CARGO_HOME=cargo_home, RUSTUP_HOME=rustup_home, CARGO_TARGET_DIR=str(target),
               CARGO_INCREMENTAL="0", LOX_TELEMETRY="0", DO_NOT_TRACK="1",
               HOME=str(home), USERPROFILE=str(home), APPDATA=str(home), LOCALAPPDATA=str(home),
               LOX_HOME=str(home / ".lox"), LOX_CONFIG=str(config), TEMP=str(temp), TMP=str(temp))
    env.pop("RUST_MIN_STACK", None)
    for key in ("HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "http_proxy", "https_proxy", "all_proxy"):
        env[key] = "http://127.0.0.1:9"
    env["NO_PROXY"] = env["no_proxy"] = ""
    return env


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("phase", choices=("dev", "gates", "packages", "native"))
    parser.add_argument("--target-dir", type=Path, required=True)
    parser.add_argument("--results", type=Path, required=True)
    parser.add_argument("--offline", action="store_true")
    parser.add_argument("--force", action="store_true", help="Run even if identical success is retained")
    parser.add_argument("--test-threads", type=int, default=1)
    parser.add_argument("--features", default="")
    parser.add_argument("--maturin", type=Path)
    parser.add_argument("--java-generator", type=Path)
    parser.add_argument("--java-home", type=Path)
    parser.add_argument("--kotlin-compiler", type=Path)
    args = parser.parse_args(argv)
    if args.test_threads < 1:
        parser.error("--test-threads must be positive")
    if args.features and args.phase not in ("dev", "gates"):
        parser.error("--features applies only to dev and gates")
    if args.phase == "native" and (os.name != "nt" or platform.machine() != "AMD64"):
        parser.error("the existing native artifact builder supports Windows AMD64 only")
    target, results = args.target_dir.resolve(), args.results.resolve()
    if results.is_relative_to(ROOT) or target.is_relative_to(ROOT):
        parser.error("Keep result manifests and compiler targets outside the source checkout")
    results.parent.mkdir(parents=True, exist_ok=True)
    env = environment(results, target)
    cargo = ["--locked"] + (["--offline"] if args.offline else [])
    features = ["--features", args.features] if args.features else []
    commands = {
        "dev": [["cargo", "check", "--workspace", *cargo, *features, "--message-format=json"]],
        "gates": [
            ["cargo", "fmt", "--all", "--check"],
            ["cargo", "clippy", "--workspace", *cargo, *features, "--message-format=json", "--", "-D", "warnings"],
            ["cargo", "build", "--release", "--workspace", *cargo, *features, "--message-format=json"],
            ["cargo", "test", "--workspace", *cargo, *features, "--message-format=json",
             "--", "--test-threads=" + str(args.test_threads)],
        ],
        "packages": [["cargo", "package", "--workspace", *cargo]],
    }
    manifest = json.loads(results.read_text(encoding="utf-8")) if results.exists() else {}
    run_dir = results.parent / "validation-runs" / (args.phase + "-" + uuid.uuid4().hex)
    run_dir.mkdir(parents=True)
    tools = []
    native_key = None
    native_source_guard = None
    native_payloads = None
    library_build = None
    if args.phase == "native":
        if not all((args.maturin, args.java_generator, args.java_home, args.kotlin_compiler)):
            parser.error("native requires --maturin, --java-generator, --java-home and --kotlin-compiler")
        # Always ask Cargo for the CURRENT library before considering native-proof reuse.
        native_source_guard = source_identity(source_files())
        build = ["cargo", "rustc", "--manifest-path", str(ROOT / "crates" / "lox-bindings" / "Cargo.toml"),
                 "--lib", "--release", *cargo, "--message-format=json", "--crate-type", "cdylib"]
        row = execute(build, env, run_dir / "current-library.log")
        library_build = row
        if row["exit_code"]:
            record = {"key": None, "status": "failed", "commands": [row],
                      "artifacts": files_identity([run_dir / "current-library.log"]),
                      "run_directory": str(run_dir), "error": "current native-library build failed"}
            manifest["native"] = record
            save_result(results, manifest, record)
            print(json.dumps(row))
            return row["exit_code"]
        library = target / "release" / ("lox_archive.dll" if os.name == "nt" else "liblox_archive.so")
        if library not in cargo_artifacts(run_dir / "current-library.log"):
            raise RuntimeError("Cargo did not identify the expected current native library")
        if source_identity(source_files()) != native_source_guard:
            raise RuntimeError("Source changed during the current native-library build")
        native_key = digest(library)
        shutil.copy2(library, run_dir / "current-library.dll")
        if digest(run_dir / "current-library.dll") != native_key:
            raise RuntimeError("Current native-library retention differs from the producer")
        tools = [sys.executable, args.maturin, args.java_generator, args.kotlin_compiler,
                 args.java_home / "bin" / "java.exe", args.java_home / "bin" / "javac.exe",
                 args.java_home / "bin" / "jar.exe", "pwsh"]
        native_payloads = native_tool_payloads(args.java_home, args.kotlin_compiler)
        # Output paths are allocated per run, not part of the stable logical command.
        commands["native"] = [["pwsh", "-NoProfile", "-File", str(ROOT / "scripts" / "build-archive-bindings.ps1"),
            "-Maturin", str(args.maturin), "-JavaGenerator", str(args.java_generator),
            "-JavaHome", str(args.java_home), "-KotlinCompiler", str(args.kotlin_compiler),
            "-Python", sys.executable, "-OutputDirectory", "{fresh-native-output}",
            "-CargoTargetDirectory", str(target)]]
    # Conservatively include the full source inventory: the native builder also
    # executes Core tests whose assertions do not affect release-library bytes.
    fingerprint = inputs(env, commands[args.phase], tools=tools)
    if native_key:
        fingerprint["current_native_library_sha256"] = native_key
        fingerprint["native_tool_payloads"] = native_payloads
    key = identity(fingerprint)
    previous = manifest.get(args.phase)
    expected_count = len(commands[args.phase]) + (2 if args.phase == "packages" else 0)
    if fingerprint.get("reuse_blockers"):
        print("Result reuse disabled; checks will run: " + "; ".join(fingerprint["reuse_blockers"]),
              file=sys.stderr)
    if (args.phase != "packages" and not args.force and not fingerprint.get("reuse_blockers")
            and valid_result(previous, key, expected_count)):
        print(json.dumps({"phase": args.phase, "status": "reused", "key": key,
                          "current_library_build_seconds": library_build["seconds"] if library_build else 0,
                          "saved_command_seconds": sum(row["seconds"] for row in previous["commands"])}))
        return 0
    record = {"key": key, "status": "running", "inputs": fingerprint,
              "commands": [], "artifacts": {}, "target": str(target), "run_directory": str(run_dir)}
    if library_build:
        record["current_library_build"] = library_build
    manifest[args.phase] = record
    save_result(results, manifest, record)
    if args.phase == "packages":
        # Fresh per-release verification remains mandatory even with identical versions.
        env["CARGO_TARGET_DIR"] = str(target / "clean-packages" / uuid.uuid4().hex)
        record["target"] = env["CARGO_TARGET_DIR"]
    native_dir = run_dir / "native"
    artifacts = [run_dir / "current-library.log", run_dir / "current-library.dll"] if library_build else []

    def refuse(message):
        record.update(status="failed", error=message, artifacts=files_identity(artifacts))
        save_result(results, manifest, record)
        raise RuntimeError(message)

    def run(command, name, child_env):
        log = run_dir / (name + ".log")
        row = execute(command, child_env, log)
        record["commands"].append(row)
        artifacts.append(log)
        if "--message-format=json" in command:
            artifacts.extend(cargo_artifacts(log))
        save_result(results, manifest, record)
        if row["exit_code"]:
            record["status"] = "failed"
            record["artifacts"] = files_identity(artifacts)
            save_result(results, manifest, record)
            print(json.dumps({"phase": args.phase, "status": "failed", **row}))
            return False
        return True

    for number, logical in enumerate(commands[args.phase]):
        command = [str(native_dir) if item == "{fresh-native-output}" else item for item in logical]
        child_env = env.copy()
        if args.phase == "gates" and number == 3:
            child_env.update(RUST_MIN_STACK="16777216", NO_PROXY="127.0.0.1,localhost,::1",
                             no_proxy="127.0.0.1,localhost,::1")
        if not run(command, str(number), child_env):
            return record["commands"][-1]["exit_code"]
    if args.phase == "packages":
        archives = sorted((Path(env["CARGO_TARGET_DIR"]) / "package").glob("*.crate"))
        if not archives:
            raise RuntimeError("Cargo produced no package archives")
        artifacts.extend(archives)
        command = [sys.executable, str(ROOT / "scripts" / "verify-packages.py"), *map(str, archives)]
        if not run(command, "inventory", env):
            return record["commands"][-1]["exit_code"]
        command = [sys.executable, str(ROOT / "scripts" / "test-simulator-package.py"),
                   "--package-dir", str(Path(env["CARGO_TARGET_DIR"]) / "package"),
                   "--work-dir", str(run_dir / "simulator"),
                   "--target-dir", str(Path(env["CARGO_TARGET_DIR"]) / "standalone")]
        if args.offline:
            command.append("--offline")
        if not run(command, "standalone", env):
            return record["commands"][-1]["exit_code"]
    if args.phase == "native":
        proof = native_dir / "ARTIFACTS.json"
        report = json.loads(proof.read_text(encoding="utf-8-sig"))
        artifacts.extend(native_dir / row["path"] for row in report["artifact_manifest"])
        ledger = native_dir / "commands.json"
        steps = json.loads(ledger.read_text(encoding="utf-8-sig"))
        if not steps or any(row["exit_code"] for row in steps):
            refuse("Native command evidence is incomplete or failed")
        artifacts.extend([proof, ledger, *native_dir.glob("*.log")])
        actual = native_dir / "venv" / "Lib" / "site-packages" / "lox_archive" / "lox_archive.dll"
        if digest(actual) != native_key:
            refuse("Qualified native library differs from the current Cargo build; no reusable success")
        if source_identity(source_files()) != native_source_guard:
            refuse("Source changed during native qualification")
    # Check the same logical inputs, never the fresh-output command expansion.
    final = inputs(environment(results, target), commands[args.phase], tools=tools)
    if native_key:
        final["current_native_library_sha256"] = digest(library)
        final["native_tool_payloads"] = native_tool_payloads(args.java_home, args.kotlin_compiler)
    if final != fingerprint:
        refuse("Inputs changed during validation; success was not recorded")
    record["artifacts"] = files_identity(artifacts)
    record["status"] = "passed"
    save_result(results, manifest, record)
    print(json.dumps({"phase": args.phase, "status": "passed", "key": key,
                      "command_seconds": sum(row["seconds"] for row in record["commands"])}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
