#!/usr/bin/env python3
"""Export portable issue state without local owner routing or database artifacts."""

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile


ROOT = Path(__file__).resolve().parents[1]


def portable_issue(issue):
    if not isinstance(issue, dict) or not isinstance(issue.get("id"), str):
        raise ValueError("Expected an issue object with a string ID")
    return {key: value for key, value in issue.items() if key != "owner"}


def export_issues(bd, output):
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="beads-export-", dir=output.parent) as temporary:
        raw = Path(temporary) / "issues.jsonl"
        subprocess.run(
            [str(bd), "export", "--output", str(raw), "--actor", "coordinator",
             "--sandbox", "--json"],
            cwd=ROOT, check=True,
        )
        issues = []
        with raw.open(encoding="utf-8") as stream:
            for line in stream:
                if line.strip():
                    issues.append(portable_issue(json.loads(line)))
        staged = Path(temporary) / "portable.jsonl"
        with staged.open("w", encoding="utf-8", newline="\n") as stream:
            for issue in issues:
                stream.write(json.dumps(issue, ensure_ascii=True, separators=(",", ":")) + "\n")
        staged.replace(output)
    return len(issues)


def default_bd():
    installed = shutil.which("bd")
    if installed:
        return Path(installed)
    local = ROOT / ".reverse-engineering" / "tools" / "beads" / (
        "bd.exe" if os.name == "nt" else "bd"
    )
    if local.is_file():
        return local
    raise ValueError("Beads is not installed; install bd or supply --bd PATH")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bd", type=Path)
    parser.add_argument("--output", type=Path, default=ROOT / ".beads" / "backup" / "issues.jsonl")
    args = parser.parse_args()
    try:
        count = export_issues(args.bd or default_bd(), args.output.resolve())
        print(json.dumps({"ok": True, "issues": count, "output": str(args.output.resolve())}))
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        print(f"Issue export failed: {error}", file=sys.stderr)
        sys.exit(1)
