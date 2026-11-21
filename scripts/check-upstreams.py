#!/usr/bin/env python3
"""Observe GitHub branch metadata without changing reviewed upstream pins.

Usage: python3 scripts/check-upstreams.py [--registry FILE] [--timeout SECONDS]
JSON goes to stdout. Exit 1 means incomplete/error, not an unchanged result.
Only GitHub's exact git/ref endpoint is requested; catalog links are never fetched.
"""

import argparse
import http.client
import json
import os
import re
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path
from urllib.parse import quote

DEFAULT_REGISTRY = Path(__file__).resolve().parents[1] / "docs/knowledge/upstreams.json"
COMMIT = re.compile(r"[0-9a-fA-F]{40}", re.ASCII)
REPOSITORY = re.compile(
    r"[A-Za-z0-9][A-Za-z0-9-]{0,38}/[A-Za-z0-9][A-Za-z0-9._-]{0,99}", re.ASCII
)
REF_PART = re.compile(r"[A-Za-z0-9][A-Za-z0-9._-]*", re.ASCII)
MAX_RESPONSE_BYTES = 65536


class CheckError(Exception):
    """A safe, credential-free diagnostic suitable for JSON output."""


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


def api_url(repository, ref):
    """Construct an allowlisted API URL from restricted repository/branch names."""
    if (
        not isinstance(repository, str)
        or not REPOSITORY.fullmatch(repository)
        or ".." in repository
        or repository.endswith(".")
    ):
        raise CheckError("Invalid repository; expected owner/name")
    if not isinstance(ref, str) or not ref.startswith("refs/heads/") or len(ref) > 255:
        raise CheckError("Invalid ref; expected refs/heads/branch")
    parts = ref[len("refs/heads/") :].split("/")
    if any(
        not REF_PART.fullmatch(part) or ".." in part or part.endswith((".", ".lock"))
        for part in parts
    ):
        raise CheckError("Invalid branch name")
    branch = quote("/".join(parts), safe="/")
    return f"https://api.github.com/repos/{repository}/git/ref/heads/{branch}"


def validate_registry(registry):
    if not isinstance(registry, dict) or registry.get("schema_version") != 1:
        raise CheckError("Unsupported registry schema")
    entries = registry.get("repositories")
    if not isinstance(entries, list) or not 1 <= len(entries) <= 32:
        raise CheckError("Registry must contain 1..32 repositories")
    seen = set()
    for entry in entries:
        if not isinstance(entry, dict):
            raise CheckError("Invalid repository record")
        api_url(entry.get("id"), entry.get("ref"))
        for field in ("reviewed_sha", "latest_seen_sha"):
            value = entry.get(field)
            if not isinstance(value, str) or not COMMIT.fullmatch(value):
                raise CheckError(f"Invalid {field}; expected full 40-hex commit")
        identity = entry["id"].lower()
        if identity in seen:
            raise CheckError("Duplicate repository")
        seen.add(identity)
    return entries


def observe(entry, opener, timeout, token):
    url = api_url(entry["id"], entry["ref"])
    headers = {
        "Accept": "application/vnd.github+json",
        "User-Agent": "lox-upstream-metadata-check",
        "X-GitHub-Api-Version": "2022-11-28",
    }
    if token:
        headers["Authorization"] = f"Bearer {token}"
    try:
        request = urllib.request.Request(url, headers=headers, method="GET")
        with opener.open(request, timeout=timeout) as response:
            if response.status != 200 or response.geturl() != url:
                raise CheckError("Unexpected GitHub response status or URL")
            raw = response.read(MAX_RESPONSE_BYTES + 1)
        if len(raw) > MAX_RESPONSE_BYTES:
            raise CheckError("GitHub ref response exceeds size limit")
        data = json.loads(raw)
    except urllib.error.HTTPError as error:
        raise CheckError(f"GitHub HTTP {error.code}") from None
    except (OSError, urllib.error.URLError, http.client.HTTPException, ValueError):
        # Exception messages, headers and response bodies may contain credentials.
        raise CheckError("GitHub network failure or invalid JSON response") from None
    if not isinstance(data, dict) or data.get("ref") != entry["ref"]:
        raise CheckError("Unexpected GitHub ref response")
    obj = data.get("object")
    if (
        not isinstance(obj, dict)
        or obj.get("type") != "commit"
        or not isinstance(obj.get("sha"), str)
        or not COMMIT.fullmatch(obj["sha"])
    ):
        raise CheckError("Expected GitHub ref to identify a full commit SHA")
    return obj["sha"].lower()


def check(registry, timeout=10, token=None):
    if isinstance(timeout, bool) or not isinstance(timeout, (int, float)):
        raise CheckError("Timeout must be between 1 and 30 seconds")
    if not 1 <= timeout <= 30:
        raise CheckError("Timeout must be between 1 and 30 seconds")
    entries = validate_registry(registry)
    opener = urllib.request.build_opener(NoRedirect())
    report = {
        "schema_version": 1,
        "checked_utc": datetime.now(timezone.utc).isoformat(),
        "status": "unchanged",
        "reviewed_pins_modified": False,
        "repositories": [],
    }
    for entry in entries:
        result = {
            "id": entry["id"],
            "ref": entry["ref"],
            "reviewed_sha": entry["reviewed_sha"],
            "previous_latest_seen_sha": entry["latest_seen_sha"],
            "latest_seen_sha": None,
            "needs_review": None,
            "compare_url": None,
            "status": "error",
        }
        try:
            latest = observe(entry, opener, timeout, token)
            needs_review = latest != entry["reviewed_sha"].lower()
            result.update(
                latest_seen_sha=latest,
                needs_review=needs_review,
                compare_url=(
                    f"https://github.com/{entry['id']}/compare/"
                    f"{entry['reviewed_sha']}...{latest}"
                ),
                status="needs_review" if needs_review else "unchanged",
            )
        except CheckError as error:
            result["error"] = str(error)
        report["repositories"].append(result)
    statuses = {result["status"] for result in report["repositories"]}
    if "error" in statuses:
        report["status"] = "error"
    elif "needs_review" in statuses:
        report["status"] = "needs_review"
    return report


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--registry", type=Path, default=DEFAULT_REGISTRY)
    parser.add_argument(
        "--timeout", type=int, default=10, help="1..30 seconds per request"
    )
    args = parser.parse_args(argv)
    try:
        registry = json.loads(args.registry.read_text(encoding="utf-8"))
        token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
        report = check(registry, timeout=args.timeout, token=token)
    except (OSError, ValueError):
        report = {
            "schema_version": 1,
            "status": "error",
            "error": "Cannot read registry JSON",
        }
    except CheckError as error:
        report = {"schema_version": 1, "status": "error", "error": str(error)}
    print(json.dumps(report, indent=2))
    return 1 if report["status"] == "error" else 0


if __name__ == "__main__":
    raise SystemExit(main())
