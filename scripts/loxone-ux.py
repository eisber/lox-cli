#!/usr/bin/env python3
"""Run bounded, version-gated Loxone Config UX steps through an owned MCP peer."""

import argparse
import base64
from datetime import datetime, timezone
import ipaddress
import json
import math
import os
from pathlib import Path, PureWindowsPath
import re
import sys
import time
from urllib.parse import urlsplit
import uuid

from loxone_ux_transport import MCPClient, UxError, parse_json


MAX_STEPS = 64
ENERGY_COLUMNS = {
    "priority": 0, "output": 1, "mode": 2, "icon": 3, "name": 4,
    "preparation": 5, "minimum-runtime": 6, "daily-runtime": 7,
    "deadline": 8, "switch-on-power": 9, "rated-power": 10,
}
READ_OPS = {"status", "windows", "inspect", "energy.read"}
TARGET_FIELDS = {"window_class", "selector"}
OP_FIELDS = {
    "status": set(), "windows": set(),
    "inspect": TARGET_FIELDS | {"limit", "depth"},
    "focus": TARGET_FIELDS, "invoke": TARGET_FIELDS,
    "set_value": TARGET_FIELDS | {"value"}, "keys": TARGET_FIELDS | {"value"},
    "wait": TARGET_FIELDS | {"timeout_ms", "value"},
    "energy.read": {"row", "column"},
    "energy.set": {"row", "column", "value"},
    "energy.select": {"row", "column", "index"},
    "energy.open": set(), "energy.accept": set(), "energy.cancel": set(),
    "save_as": {"path"},
}
SELECTOR_FIELDS = {"automation_id", "automation_id_suffix", "name", "control_type"}


def fail(message, code="validation"):
    raise UxError(code, message)


def closed_object(value, allowed, label):
    if not isinstance(value, dict) or set(value) - allowed:
        fail(f"{label} must be an object containing only documented fields.")


def text(value, label, maximum=8192):
    if not isinstance(value, str) or not value or len(value) > maximum or "\0" in value:
        fail(f"{label} must be a nonempty string of at most {maximum} characters.")
    return value


def integer(value, low, high, label):
    if type(value) is not int or not low <= value <= high:
        fail(f"{label} must be an integer from {low} to {high}.")
    return value


def sha256(value):
    if not isinstance(value, str) or not re.fullmatch(r"[a-fA-F0-9]{64}", value):
        fail("Executable pins must be exactly 64 hexadecimal SHA-256 characters.")
    return value.lower()


def windows_path(value, label):
    text(value, label, 2048)
    path = PureWindowsPath(value)
    if (not path.is_absolute() or not re.fullmatch(r"[A-Za-z]:", path.drive)
            or ".." in path.parts or any(char in value for char in '\r\n\0*?"<>|')
            or ":" in str(path)[2:]):
        fail(f"{label} must be an absolute Windows drive path without traversal or wildcards.")
    return str(path)


def load_json(path, label, limit):
    try:
        raw = Path(path).read_bytes()
        if len(raw) > limit:
            fail(f"{label} exceeds its size limit.")
        return parse_json(raw.decode("utf-8-sig"))
    except (OSError, ValueError, UnicodeError, RecursionError):
        fail(f"Cannot read {label}; supply a readable UTF-8 JSON file.", "local_file")


def validate_profile(profile):
    required = {"schema_version", "mcp_url", "token_file", "backend_path", "remote_artifact_root"}
    closed_object(profile, required | {"expected_exe_sha256"}, "Profile")
    if not required <= profile.keys() or type(profile["schema_version"]) is not int or profile["schema_version"] != 1:
        fail("Profile requires schema_version 1 and all documented connection/path fields.")
    url = text(profile["mcp_url"], "mcp_url", 2048)
    try:
        parsed = urlsplit(url)
        port = parsed.port
    except ValueError:
        fail("mcp_url must be a valid HTTP(S) endpoint without credentials.", "profile")
    if (parsed.scheme not in ("http", "https") or not parsed.hostname
            or parsed.username is not None or parsed.password is not None
            or parsed.query or parsed.fragment or port == 0
            or not url.isascii() or any(ord(char) < 33 or ord(char) == 127 for char in url)):
        fail("mcp_url must be an HTTP(S) endpoint without credentials, query, or fragment.", "profile")
    try:
        loopback = ipaddress.ip_address(parsed.hostname).is_loopback
    except ValueError:
        loopback = parsed.hostname.lower() == "localhost"
    if parsed.scheme == "http" and not loopback:
        fail("Plain HTTP is allowed only on loopback; use an existing SSH tunnel or HTTPS.", "profile")
    token_file = Path(os.path.expandvars(text(profile["token_file"], "token_file", 2048))).expanduser()
    if not token_file.is_absolute():
        fail("token_file must expand to an explicit absolute local path.", "profile")
    result = dict(profile)
    result["token_file"] = token_file
    for key in ("backend_path", "remote_artifact_root"):
        result[key] = windows_path(profile[key], key)
    if "expected_exe_sha256" in profile:
        result["expected_exe_sha256"] = sha256(profile["expected_exe_sha256"])
    return result


def validate_step(step):
    if not isinstance(step, dict) or not isinstance(step.get("op"), str) or step["op"] not in OP_FIELDS:
        fail("Every step must use a documented operation.")
    op = step["op"]
    closed_object(step, OP_FIELDS[op] | {"op"}, "Step")
    step = dict(step)
    if "window_class" in step:
        text(step["window_class"], "window_class", 512)
    if "selector" in step:
        selector = step["selector"]
        closed_object(selector, SELECTOR_FIELDS, "Selector")
        if not selector:
            fail("Selector must contain at least one documented field.")
        for value in selector.values():
            text(value, "Selector value", 512)
    if op == "inspect":
        step["limit"] = integer(step.get("limit", 12), 1, 50, "Inspection limit")
        step["depth"] = integer(step.get("depth", 2), 0, 5, "Inspection depth")
    if op in ("set_value", "keys", "energy.set"):
        value = step.get("value")
        if not isinstance(value, str) or len(value) > 8192 or "\0" in value:
            fail("Value must be a string of at most 8192 characters.")
    if op == "wait":
        if "selector" not in step:
            fail("wait requires a selector.")
        step["timeout_ms"] = integer(step.get("timeout_ms", 5000), 1, 30000, "Wait timeout")
        if "value" in step and (not isinstance(step["value"], str) or len(step["value"]) > 8192 or "\0" in step["value"]):
            fail("Wait value must be a bounded string without NUL.")
    if op in ("energy.read", "energy.set", "energy.select"):
        integer(step.get("row"), 1, 12, "Energy row")
        integer(step.get("column"), 0, 10, "Energy column")
    if op == "energy.select":
        integer(step.get("index"), 0, 27, "Energy selection index")
    if op == "save_as":
        step["path"] = windows_path(step.get("path"), "Save path")
    return step


def validate_recipe(recipe):
    closed_object(recipe, {"schema_version", "name", "requires", "steps"}, "Recipe")
    if type(recipe.get("schema_version")) is not int or recipe["schema_version"] != 1:
        fail("Recipe requires schema_version 1.")
    text(recipe.get("name"), "Recipe name", 128)
    steps = recipe.get("steps")
    if not isinstance(steps, list) or not 1 <= len(steps) <= MAX_STEPS:
        fail(f"Recipe must contain 1 to {MAX_STEPS} steps.")
    pin = None
    if "requires" in recipe:
        closed_object(recipe["requires"], {"exe_sha256"}, "Recipe requires")
        pin = sha256(recipe["requires"].get("exe_sha256"))
    return [validate_step(step) for step in steps], pin


def build_request(profile, pid, expected_sha, steps, recipe_pin=None):
    integer(pid, 1, 2147483647, "Owned PID")
    if expected_sha is not None:
        expected_sha = sha256(expected_sha)
    else:
        expected_sha = profile.get("expected_exe_sha256")
    if recipe_pin is not None:
        recipe_pin = sha256(recipe_pin)
        if expected_sha is not None and expected_sha != recipe_pin:
            fail("Recipe executable pin conflicts with the selected profile/CLI pin.", "version_pin")
        expected_sha = recipe_pin
    if any(step["op"] not in READ_OPS for step in steps) and expected_sha is None:
        fail("Input operations require an executable SHA-256 pin; inspect status first.", "version_pin")
    run_name = "run-" + datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ-") + uuid.uuid4().hex
    return {
        "schema_version": 1,
        "target": {"pid": pid, "expected_exe_sha256": expected_sha},
        "artifact_dir": str(PureWindowsPath(profile["remote_artifact_root"]) / run_name),
        "steps": steps,
    }


def powershell_command(backend_path, request):
    def encoded(value):
        return base64.b64encode(value.encode("utf-8")).decode("ascii")
    path_data = encoded(backend_path)
    request_data = encoded(json.dumps(request, ensure_ascii=False, separators=(",", ":")))
    # Both variable inputs are data, never PowerShell source, including the script path.
    return (
        "$ProgressPreference='SilentlyContinue';$ErrorActionPreference='Stop';"
        "$uxBackend=[Text.Encoding]::UTF8.GetString([Convert]::FromBase64String('"
        + path_data + "'));"
        "$uxRequest=[Text.Encoding]::UTF8.GetString([Convert]::FromBase64String('"
        + request_data + "'));"
        ". $uxBackend;Invoke-LoxoneUx -RequestJson $uxRequest"
    )


def validate_reply(reply, request):
    fields = {
        "schema_version", "ok", "pid", "target", "completed_steps", "failed_step",
        "error", "results", "elapsed_ms", "artifacts", "metrics",
    }
    if not isinstance(reply, dict) or set(reply) != fields:
        fail("Backend response does not match schema 1.", "backend_response")
    count = len(request["steps"])
    if (type(reply["schema_version"]) is not int or reply["schema_version"] != 1
            or type(reply["ok"]) is not bool or type(reply["pid"]) is not int
            or reply["pid"] != request["target"]["pid"]
            or type(reply["completed_steps"]) is not int
            or not 0 <= reply["completed_steps"] <= count
            or not isinstance(reply["target"], dict)
            or not isinstance(reply["artifacts"], dict)
            or not isinstance(reply["metrics"], dict)
            or not isinstance(reply["results"], list)
            or type(reply["elapsed_ms"]) not in (int, float)
            or not math.isfinite(reply["elapsed_ms"]) or reply["elapsed_ms"] < 0):
        fail("Backend returned invalid identity, progress, or timing data.", "backend_response")
    failed = reply["failed_step"]
    if failed is not None and (type(failed) is not int or not 0 <= failed < count):
        fail("Backend returned an invalid failed_step index.", "backend_response")
    error = reply["error"]
    if reply["ok"]:
        if reply["completed_steps"] != count or failed is not None or error is not None:
            fail("Backend success contradicted its step/error data.", "backend_response")
        actual = reply["target"].get("exe_sha256")
        if not isinstance(actual, str) or not re.fullmatch(r"[a-fA-F0-9]{64}", actual):
            fail("Backend success omitted executable identity.", "backend_response")
        pin = request["target"]["expected_exe_sha256"]
        if pin is not None and actual.lower() != pin:
            fail("Backend executable identity did not match the requested pin.", "version_pin")
    elif (not isinstance(error, dict) or not isinstance(error.get("code"), str)
          or not isinstance(error.get("message"), str)):
        fail("Backend failure omitted its structured error.", "backend_response")
    seen = set()
    for result in reply["results"]:
        if not isinstance(result, dict) or set(result) != {"index", "op", "value"}:
            fail("Backend returned an invalid step result.", "backend_response")
        index = result["index"]
        if (type(index) is not int or not 0 <= index < count or index in seen
                or result["op"] != request["steps"][index]["op"]):
            fail("Backend result did not match the requested step.", "backend_response")
        seen.add(index)
    return reply


def compact_reply(reply):
    summary = {key: reply[key] for key in (
        "schema_version", "ok", "completed_steps", "failed_step", "error", "elapsed_ms", "artifacts",
    )}
    reads = []
    budget = 8192
    for result in reply["results"]:
        if result["op"] not in READ_OPS:
            continue
        size = len(json.dumps(result, ensure_ascii=True, separators=(",", ":")).encode("ascii"))
        if size > min(2048, budget):
            report = reply["artifacts"].get("report")
            if not isinstance(report, str) or not report:
                fail("Backend omitted the report needed to reference read results.", "backend_artifact")
            reads.append({
                "index": result["index"], "op": result["op"], "omitted": True,
                "reason": "read_size_limit",
                "artifact": report,
            })
        else:
            reads.append(result)
            budget -= size
    summary["results"] = reads
    return summary


def encode_output(output):
    metrics = output.setdefault("transport_metrics", {})
    metrics["output_bytes"] = 0
    while True:
        rendered = json.dumps(output, ensure_ascii=True, separators=(",", ":"), allow_nan=False) + "\n"
        size = len(rendered.encode("ascii"))
        if metrics["output_bytes"] == size:
            return rendered
        metrics["output_bytes"] = size


class Parser(argparse.ArgumentParser):
    def error(self, message):
        raise UxError("arguments", message)


def energy_column(value):
    if value in ENERGY_COLUMNS:
        return ENERGY_COLUMNS[value]
    try:
        column = int(value)
    except ValueError:
        raise argparse.ArgumentTypeError("Use an Energy field name or column 0..10.") from None
    if not 0 <= column <= 10:
        raise argparse.ArgumentTypeError("Energy column must be 0..10.")
    return column


def make_parser():
    common = argparse.ArgumentParser(add_help=False)
    common.add_argument("--profile", default=argparse.SUPPRESS, help="Private JSON profile, or LOX_UX_PROFILE")
    common.add_argument("--pid", type=int, default=argparse.SUPPRESS, help="Explicit owned Loxone Config PID")
    common.add_argument("--expected-sha", default=argparse.SUPPRESS, help="Executable SHA-256 pin")
    common.add_argument("--verbose", action="store_true", default=argparse.SUPPRESS, help="Return the full backend reply")
    parser = Parser(description=__doc__, parents=[common])
    commands = parser.add_subparsers(dest="command", required=True, parser_class=Parser)
    for command in ("status", "windows", "energy-open", "energy-accept", "energy-cancel"):
        commands.add_parser(command, parents=[common])
    for command in ("inspect", "focus", "invoke", "set-value", "keys", "wait"):
        child = commands.add_parser(command, parents=[common])
        child.add_argument("--window-class")
        selectors = child.add_mutually_exclusive_group()
        selectors.add_argument("--selector", help="JSON object of exact selector fields")
        selectors.add_argument("--name", help="Exact accessible name")
        if command in ("set-value", "keys"):
            child.add_argument("value")
        if command == "wait":
            child.add_argument("--timeout-ms", type=int, default=5000)
            child.add_argument("--value", help="Wait for this exact accessible value")
        if command == "inspect":
            child.add_argument("--limit", type=int, default=12)
            child.add_argument("--depth", type=int, default=2)
    for command in ("energy-read", "energy-set", "energy-select"):
        child = commands.add_parser(command, parents=[common])
        child.add_argument("row", type=int)
        child.add_argument("column", type=energy_column)
        if command == "energy-set":
            child.add_argument("value")
        if command == "energy-select":
            child.add_argument("index", type=int)
    child = commands.add_parser("save-as", parents=[common])
    child.add_argument("path")
    child = commands.add_parser("run", parents=[common])
    child.add_argument("recipe")
    return parser


def command_steps(args):
    if args.command == "run":
        return validate_recipe(load_json(args.recipe, "recipe", 1024 * 1024))
    op = {
        "set-value": "set_value", "save-as": "save_as",
        "energy-read": "energy.read", "energy-set": "energy.set", "energy-select": "energy.select",
        "energy-open": "energy.open", "energy-accept": "energy.accept", "energy-cancel": "energy.cancel",
    }.get(args.command, args.command)
    step = {"op": op}
    for field in OP_FIELDS[op]:
        value = getattr(args, field, None)
        if value is not None:
            step[field] = value
    if getattr(args, "selector", None) is not None:
        try:
            step["selector"] = parse_json(args.selector)
        except (ValueError, RecursionError):
            fail("--selector must be a JSON object of documented fields.")
    if getattr(args, "name", None) is not None:
        step["selector"] = {"name": args.name}
    return [validate_step(step)], None


def execute(args):
    steps, recipe_pin = command_steps(args)
    profile_path = getattr(args, "profile", None) or os.environ.get("LOX_UX_PROFILE")
    if not profile_path:
        fail("Supply --profile or LOX_UX_PROFILE; no private endpoint is inferred.", "profile")
    profile = validate_profile(load_json(Path(profile_path).expanduser(), "profile", 65536))
    request = build_request(
        profile, getattr(args, "pid", None), getattr(args, "expected_sha", None), steps, recipe_pin,
    )
    try:
        token = profile["token_file"].read_text(encoding="utf-8-sig").strip()
    except (OSError, UnicodeError):
        fail("Cannot read the bearer token file; check its private local permissions/path.", "auth")
    if len(token) > 16384 or not re.fullmatch(r"[A-Za-z0-9._~+/\-]+=*", token):
        fail("Bearer token file must contain one nonempty token, not headers or JSON.", "auth")
    client = MCPClient(profile["mcp_url"], token)
    start = time.monotonic()
    try:
        client.initialize()
        timeout = max(120, 60 + math.ceil(sum(step.get("timeout_ms", 0) for step in steps) / 1000))
        reply = validate_reply(client.powershell(
            powershell_command(profile["backend_path"], request), timeout,
        ), request)
        output = reply.copy() if getattr(args, "verbose", False) else compact_reply(reply)
        output["transport_metrics"] = dict(client.metrics)
        output["transport_metrics"]["elapsed_ms"] = round((time.monotonic() - start) * 1000, 3)
        return output, 0 if reply["ok"] else 1
    except UxError as error:
        error.metrics = dict(client.metrics)
        error.metrics["elapsed_ms"] = round((time.monotonic() - start) * 1000, 3)
        error.artifact_dir = request["artifact_dir"]
        raise


def main(argv=None):
    try:
        output, code = execute(make_parser().parse_args(argv))
    except UxError as error:
        output = {"ok": False, "error": {"code": error.code, "message": str(error)}}
        if error.metrics is not None:
            output["transport_metrics"] = error.metrics
        if error.artifact_dir is not None:
            output["requested_artifact_dir"] = error.artifact_dir
        code = 1
    rendered = encode_output(output)
    if hasattr(sys.stdout, "buffer"):
        sys.stdout.buffer.write(rendered.encode("ascii"))
    else:
        sys.stdout.write(rendered)
    return code


if __name__ == "__main__":
    sys.exit(main())
