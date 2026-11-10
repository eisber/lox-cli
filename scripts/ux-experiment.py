#!/usr/bin/env python3
"""Generic opt-in property execution entry; the campaign CLI remains file-only."""

import argparse
import json
from pathlib import Path
import re
import sys
import uuid

from ux_property_executor import Executor, canonical, durable, error_details, load_authority, sha
from loxone_ux_transport import MCPClient


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("queue", "catalog", "handoff", "profile", "authority", "artifact-root"):
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--run-id", required=True)
    parser.add_argument("--authority-sha256", required=True)
    args = parser.parse_args(argv)
    campaign = Executor.queue_tool()
    queue = None
    diagnostic_root = None
    stage = "artifact-root"
    try:
        diagnostic_root = campaign.private_destination(args.artifact_root)
        diagnostic_root.mkdir(parents=True, exist_ok=True)
        stage = "catalog"
        catalog = campaign.read_json(args.catalog)
        queue = campaign.Queue(args.queue, catalog)
        stage = "handoff"
        handoff = campaign.read_json(args.handoff)
        events = [event for event in queue.events()
                  if event["type"] == "handoff" and event["run_id"] == args.run_id]
        if (len(events) != 1 or campaign.definition_hash(handoff) != events[0]["handoff_sha256"]
                or handoff["authority_source_sha256"] != args.authority_sha256):
            raise ValueError("Immutable handoff/source pin mismatch.")
        gui = campaign.load_tool("loxone-ux")
        stage = "profile"
        profile = gui.validate_profile(campaign.read_json(args.profile))
        if profile.get("expected_exe_sha256", catalog["application"]["sha256"]) != catalog["application"]["sha256"]:
            raise ValueError("Existing profile executable pin conflicts with the pilot.")
        stage = "token"
        token = profile["token_file"].read_text(encoding="utf-8-sig").strip()
        if len(token) > 16384 or not re.fullmatch(r"[A-Za-z0-9._~+/\-]+=*", token):
            raise ValueError("Invalid existing token file.")
        stage = "authority-source"
        authority = load_authority(args.authority, args.authority_sha256)
        stage = "transport-initialize"
        client = MCPClient(profile["mcp_url"], token)
        client.initialize()
        stage = "execute"
        result, capture = Executor(
            queue, args.run_id, handoff, client, authority, args.artifact_root).execute()
        stage = "collect"
        collected = queue.collect(args.run_id, capture, args.artifact_root)
        print(json.dumps({"outcome": result["outcome"], "collection": collected["status"],
                          "actual_native_start": result["actual_native_start"],
                          "code": result["code"], "stage": result.get("stage"),
                          "diagnostic": result.get("diagnostic")}))
        return 0 if result["outcome"] == "complete" and collected["status"] == "observed" else 1
    except Exception as error:
        details = error_details(error, stage)
        if queue is not None:
            try:
                queue.unknown(args.run_id, "Executor entry failed; inspect retained intent, never replay.")
            except Exception as closeout_error:
                details["journal_closeout_error"] = error_details(closeout_error, "journal-closeout")
        response = {"outcome": "UNKNOWN", "error": {
            "type": details["type"], "code": details["code"], "stage": stage},
            "automatic_retry": False}
        if diagnostic_root is not None:
            try:
                path = diagnostic_root / ("entry-error-" + uuid.uuid4().hex + ".json")
                raw = canonical(details)
                durable(path, raw)
                response["diagnostic"] = {"path": path.name, "sha256": sha(raw)}
            except Exception as diagnostic_error:
                response["diagnostic_write_error"] = {
                    "type": type(diagnostic_error).__name__, "stage": "diagnostic-write"}
        print(json.dumps(response))
        return 1


if __name__ == "__main__":
    sys.stdout.reconfigure(encoding="utf-8")
    sys.exit(main())
