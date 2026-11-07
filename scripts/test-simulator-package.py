#!/usr/bin/env python3
"""Run simulator tests and benchmark smoke tests from its actual Cargo archive."""

import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tarfile


def unpack_archive(archive_path, work_dir):
    digest = hashlib.sha256(archive_path.read_bytes()).hexdigest()
    destination = work_dir / digest
    spec = importlib.util.spec_from_file_location(
        "verify_packages", Path(__file__).resolve().with_name("verify-packages.py")
    )
    verifier = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(verifier)
    with tarfile.open(archive_path, "r:gz") as archive:
        inventory = verifier.inspect_archive(archive, archive_path)
        if inventory["errors"]:
            raise ValueError("; ".join(inventory["errors"]))
        members = [member for member in archive.getmembers() if member.isfile()]
        roots = {member.name.split("/")[0] for member in members}
        if len(roots) != 1:
            raise ValueError("expected one package directory in the archive")
        root = roots.pop()
        if not root.startswith("lox-sim-"):
            raise ValueError(f"expected a simulator package, got {root}")
        destination.mkdir(parents=True, exist_ok=True)
        for member in members:
            path = destination.joinpath(*member.name.split("/"))
            data = archive.extractfile(member).read()
            if path.exists():
                if path.read_bytes() != data:
                    raise ValueError(f"previously extracted source was modified: {path}")
            else:
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(data)
    # Isolate the crate from the enclosing checkout without changing its files.
    (destination / "Cargo.toml").write_text(
        '[workspace]\nresolver = "2"\nmembers = [' + json.dumps(root) + "]\n",
        encoding="utf-8",
    )
    (destination / "Cargo.lock").write_bytes(
        (destination / root / "Cargo.lock").read_bytes()
    )
    return destination / root, digest, inventory


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    selection = parser.add_mutually_exclusive_group(required=True)
    selection.add_argument("--archive", type=Path)
    selection.add_argument("--package-dir", type=Path)
    parser.add_argument(
        "--manifest-path", type=Path,
        default=Path(__file__).resolve().parent.parent / "lox-sim" / "Cargo.toml",
    )
    parser.add_argument("--work-dir", type=Path, required=True)
    parser.add_argument("--target-dir", type=Path, required=True)
    parser.add_argument("--offline", action="store_true")
    parser.add_argument("--jobs", type=int)
    args = parser.parse_args()
    archive_path = args.archive
    if archive_path is None:
        command = [
            "cargo", "metadata", "--manifest-path", str(args.manifest_path.resolve()),
            "--format-version", "1", "--no-deps", "--locked",
        ]
        if args.offline:
            command.append("--offline")
        metadata = json.loads(subprocess.check_output(command, text=True))
        package = next(p for p in metadata["packages"] if p["name"] == "lox-sim")
        archive_path = args.package_dir / f"lox-sim-{package['version']}.crate"
    source, digest, inventory = unpack_archive(
        archive_path.resolve(), args.work_dir.resolve()
    )
    target = args.target_dir.resolve() / digest
    report = {
        "archive": str(archive_path.resolve()),
        "archive_sha256": digest,
        "source": str(source),
        "target": str(target),
        "inventory": inventory,
        "workspace": "single extracted package, own lockfile, no parent fixtures",
    }
    print(json.dumps(report, indent=2), flush=True)
    for label, selectors in [("tests", []), ("benchmark-smoke", ["--bench", "sim_bench"])]:
        command = [
            sys.executable, str(Path(__file__).resolve().with_name("measure-build.py")),
            "--label", label, "--cwd", str(source), "--target-dir", str(target),
            "--output", str(source.parent / "test-results.jsonl"),
        ]
        if args.jobs is not None:
            command.extend(["--jobs", str(args.jobs)])
        command.extend(["--", "cargo", "test", "--locked", *selectors])
        if args.offline:
            command.append("--offline")
        result = subprocess.run(command)
        if result.returncode:
            return result.returncode
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
