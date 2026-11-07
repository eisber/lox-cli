#!/usr/bin/env python3
"""Inspect actual Cargo archives for private assets and missing embedded files."""

import argparse
import json
from pathlib import Path, PurePosixPath
import posixpath
import re
import tarfile


PRIVATE_COMPONENTS = {".reverse-engineering", ".beads", ".dolt", ".lox", "target"}
EMBEDDED_FILE = re.compile(r'include_(?:str|bytes)!\s*\(\s*"([^"]+)"', re.MULTILINE)


def inspect_archive(archive, label):
    members = [member for member in archive.getmembers() if member.isfile()]
    names = {member.name for member in members}
    errors = []
    for member in archive.getmembers():
        parts = PurePosixPath(member.name).parts
        if (
            not parts or PurePosixPath(member.name).is_absolute()
            or ".." in parts or "\\" in member.name
            or any(":" in part for part in parts)
            or member.issym() or member.islnk()
        ):
            errors.append(f"unsafe archive entry: {member.name}")
        if PRIVATE_COMPONENTS.intersection(parts) or ".env" in parts:
            errors.append(f"private asset: {member.name}")
    for member in members:
        if member.name.endswith(".rs"):
            source = archive.extractfile(member).read().decode("utf-8")
            for embedded in EMBEDDED_FILE.findall(source):
                required = posixpath.normpath(
                    posixpath.join(posixpath.dirname(member.name), embedded)
                )
                if required not in names:
                    errors.append(f"{member.name}: missing embedded file {embedded}")
    return {
        "archive": str(label),
        "file_count": len(members),
        "uncompressed_file_bytes": sum(member.size for member in members),
        "errors": errors,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archives", type=Path, nargs="+")
    args = parser.parse_args()
    reports = []
    for path in args.archives:
        with tarfile.open(path, "r:gz") as archive:
            report = inspect_archive(archive, path)
            report["compressed_bytes"] = path.stat().st_size
            reports.append(report)
    print(json.dumps({"packages": reports}, indent=2))
    return int(any(report["errors"] for report in reports))


if __name__ == "__main__":
    raise SystemExit(main())
