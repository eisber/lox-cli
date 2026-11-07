"""Synthetic tests: no GUI, credentials, or non-loopback endpoint access."""

import base64
import contextlib
import http.client
import importlib.util
import io
import json
import os
from pathlib import Path, PureWindowsPath
import re
import subprocess
import threading
import time
from types import SimpleNamespace
import unittest
from unittest.mock import patch
import urllib.error
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

import loxone_ux_transport as TRANSPORT


SPEC = importlib.util.spec_from_file_location("loxone_ux", Path(__file__).with_name("loxone-ux.py"))
UX = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(UX)
PIN = "a" * 64
OTHER_PIN = "b" * 64
TOKEN = "offline-test-token-not-a-credential"


class Response(io.BytesIO):
    def __init__(self, value=b"", status=200, content_type="application/json"):
        if not isinstance(value, bytes):
            value = json.dumps(value).encode("utf-8")
        super().__init__(value)
        self.status = status
        self.headers = {"Content-Type": content_type}


def rpc(result, request_id=1):
    return {"jsonrpc": "2.0", "id": request_id, "result": result}


def tool_result(reply, status=0, is_error=False):
    return {"isError": is_error, "structuredContent": {
        "result": "Response: " + json.dumps(reply) + f"\nStatus Code: {status}",
    }}


def profile():
    return {
        "schema_version": 1, "mcp_url": "http://127.0.0.1:18765/mcp",
        "token_file": str(Path.home() / "offline-token"),
        "backend_path": r"C:\Owned Tools\loxone-ux.ps1",
        "remote_artifact_root": r"D:\Private Runs", "expected_exe_sha256": PIN,
    }


def backend_reply(request, *, ok=True):
    return {
        "schema_version": 1, "ok": ok, "pid": request["target"]["pid"],
        "target": {"exe_sha256": PIN, "version": "synthetic"},
        "completed_steps": len(request["steps"]) if ok else 1,
        "failed_step": None if ok else 1,
        "error": None if ok else {"code": "selector_not_unique", "message": "Stopped before input."},
        "results": [
            {"index": i, "op": step["op"], "value": {"text": "Größe"}}
            for i, step in enumerate(request["steps"] if ok else request["steps"][:1])
        ],
        "elapsed_ms": 12.5,
        "artifacts": {
            "actions": str(PureWindowsPath(request["artifact_dir"]) / "actions.json"),
            "report": str(PureWindowsPath(request["artifact_dir"]) / "report.json"),
        },
        "metrics": {"owned_steps": len(request["steps"])},
    }


class Wire:
    def __init__(self, responses):
        self.responses = iter(responses)
        self.requests = []
        self.response_bytes = 0

    def __call__(self, request, **kwargs):
        self.requests.append((request, kwargs))
        item = next(self.responses)
        if callable(item):
            item = item(json.loads(request.data))
        if isinstance(item, Exception):
            raise item
        self.response_bytes += len(item.getvalue())
        return item


class TransportTests(unittest.TestCase):
    def test_handshake_batch_auth_and_body_metrics_without_discovery(self):
        wire = Wire([
            Response(rpc({"protocolVersion": TRANSPORT.PROTOCOL})),
            Response(b"", status=202),
            Response(rpc(tool_result({"ok": True}), request_id=2)),
        ])
        client = TRANSPORT.MCPClient("http://127.0.0.1:1/mcp", TOKEN, opener=wire)
        client.initialize()
        self.assertEqual(client.powershell("data-only-command", 120), {"ok": True})
        payloads = [json.loads(query.data) for query, _kwargs in wire.requests]
        self.assertEqual([item["method"] for item in payloads], [
            "initialize", "notifications/initialized", "tools/call",
        ])
        self.assertNotIn("id", payloads[1])
        self.assertEqual([kwargs["timeout"] for _query, kwargs in wire.requests], [90, 90, 135])
        self.assertEqual(payloads[2]["params"], {
            "name": "PowerShell", "arguments": {"command": "data-only-command", "timeout": 120},
        })
        self.assertTrue(all(query.get_header("Authorization") == "Bearer " + TOKEN
                            for query, _kwargs in wire.requests))
        self.assertTrue(all(TOKEN.encode() not in query.data for query, _kwargs in wire.requests))
        self.assertEqual(client.metrics, {
            "http_requests": 3, "rpc_calls": 2, "notifications": 1, "mcp_tool_calls": 1,
            "request_bytes": sum(len(query.data) for query, _kwargs in wire.requests),
            "response_bytes": wire.response_bytes,
        })

    def test_sse_matches_id_across_events_comments_multiline_and_eof(self):
        data = (
            ': ping\r\nevent: message\r\ndata: {"jsonrpc":"2.0","method":"notice"}\r\n\r\n'
            'data: {"jsonrpc":"2.0","id":9,"error":{"message":"unrelated"}}\n\n'
            'event: message\ndata: {"jsonrpc":"2.0",\ndata: "id":1,"result":{"text":"Größe"}}'
        ).encode("utf-8")
        wire = Wire([Response(data, content_type="text/event-stream; charset=utf-8")])
        client = TRANSPORT.MCPClient("http://127.0.0.1:1/mcp", TOKEN, opener=wire)
        self.assertEqual(client.request("test", {}), {"text": "Größe"})
        self.assertEqual(client.metrics["response_bytes"], len(data))

    def test_missing_mismatched_malformed_and_rpc_error_are_not_success(self):
        cases = [
            (b"", "application/json", "response_json"),
            (rpc({}, request_id=2), "application/json", "rpc_id"),
            (rpc({}, request_id=True), "application/json", "rpc_id"),
            ({"jsonrpc": "2.0", "id": 1, "error": {"message": TOKEN}}, "application/json", "rpc_error"),
            ({"jsonrpc": "2.0", "id": 1}, "application/json", "rpc_response"),
            (b"data: {\"id\":9,\"result\":{}}\n\n", "text/event-stream", "rpc_missing"),
            (b"data: nope\n\n", "text/event-stream", "response_json"),
            (b"data: \xff\n\n", "text/event-stream", "response_json"),
        ]
        for value, content_type, code in cases:
            with self.subTest(code=code, value=value):
                client = TRANSPORT.MCPClient("http://127.0.0.1:1/mcp", TOKEN, opener=Wire([
                    Response(value, content_type=content_type),
                ]))
                with self.assertRaises(TRANSPORT.UxError) as raised:
                    client.request("test", {})
                self.assertEqual(raised.exception.code, code)
                self.assertNotIn(TOKEN, str(raised.exception))

    def test_http_errors_and_protocol_failures_are_redacted_without_retry(self):
        http_error = urllib.error.HTTPError(
            "http://private.invalid/" + TOKEN, 401, TOKEN, {"Authorization": TOKEN},
            io.BytesIO(TOKEN.encode()),
        )
        for response, code in (
            (http_error, "http_status"),
            (urllib.error.URLError(TOKEN), "transport"),
            (http.client.BadStatusLine(TOKEN), "transport"),
            (Response(b"private", status=503), "http_status"),
            (Response(rpc({"protocolVersion": "wrong"})), "protocol_version"),
        ):
            with self.subTest(code=code):
                wire = Wire([response])
                client = TRANSPORT.MCPClient("http://127.0.0.1:1/mcp", TOKEN, opener=wire)
                with self.assertRaises(TRANSPORT.UxError) as raised:
                    client.initialize()
                self.assertEqual(raised.exception.code, code)
                self.assertNotIn(TOKEN, str(raised.exception))
                self.assertEqual(len(wire.requests), 1)

    def test_powershell_actual_exit_and_json_ok_are_separate_checks(self):
        cases = [
            (tool_result({"ok": True}, status=1), "powershell_exit"),
            (tool_result({"ok": True}, is_error=True), "tool_error"),
            ({"isError": False, "structuredContent": {"result": "Response: {}"}}, "powershell_status"),
            ({"content": [{"type": "text", "text": "success"}]}, "powershell_result"),
            ({"structuredContent": {"result": "Response: noise\n{}\nStatus Code: 0"}}, "backend_json"),
        ]
        for result, code in cases:
            with self.subTest(code=code):
                client = TRANSPORT.MCPClient("http://127.0.0.1:1/mcp", TOKEN, opener=Wire([
                    Response(rpc(result)),
                ]))
                with self.assertRaises(TRANSPORT.UxError) as raised:
                    client.powershell("data", 120)
                self.assertEqual(raised.exception.code, code)

    def test_notification_rejects_bad_status_and_response_size_is_bounded(self):
        for value, kwargs, code in (
            (Response(b"", status=201), {"notification": True}, "http_status"),
            (Response(b"x" * 20), {}, "response_size"),
            (Response(b"data: " + b"x" * 20, content_type="text/event-stream"), {}, "response_size"),
        ):
            client = TRANSPORT.MCPClient("http://127.0.0.1:1/mcp", TOKEN, opener=Wire([value]))
            with patch.object(TRANSPORT, "MAX_RESPONSE_BYTES", 16):
                with self.assertRaises(TRANSPORT.UxError) as raised:
                    client.request("test", {}, **kwargs)
            self.assertEqual(raised.exception.code, code)

    def test_default_opener_disables_redirects_and_environment_proxy(self):
        with patch.object(TRANSPORT.urllib.request, "build_opener") as build:
            TRANSPORT.MCPClient("http://127.0.0.1:1/mcp", TOKEN)
        proxy, redirects = build.call_args.args
        self.assertEqual(proxy.proxies, {})
        self.assertIsNone(redirects.redirect_request(None, None, 302, "", {}, "https://other.invalid"))


@contextlib.contextmanager
def loopback_peer(mode):
    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *_args):
            pass

        def do_POST(self):
            body = self.rfile.read(int(self.headers["Content-Length"]))
            payload = json.loads(body)
            self.server.requests.append((payload, self.headers.get("Authorization")))
            method = payload["method"]
            if mode == "forbidden":
                self.send_response(403)
                self.send_header("Content-Length", "0")
                self.end_headers()
                return
            if mode == "cumulative":
                self.server.stop.wait(0.5 if method == "tools/call" else 0.1)
            stalled = (
                (mode == "initialize" and method == "initialize")
                or (mode == "notification" and method == "notifications/initialized")
                or (mode == "tool" and method == "tools/call")
            )
            if stalled:
                self.server.stop.wait(3)
                return
            if method == "notifications/initialized":
                data = b""
            elif method == "initialize":
                data = json.dumps(rpc({"protocolVersion": TRANSPORT.PROTOCOL})).encode()
            else:
                data = json.dumps(rpc(tool_result({"ok": True}), request_id=2)).encode()
            try:
                self.send_response(202 if method == "notifications/initialized" else 200)
                sse = mode in ("sse", "trickle") and method == "initialize"
                self.send_header("Content-Type", "text/event-stream" if sse else "application/json")
                if not sse:
                    self.send_header("Content-Length", str(len(data)))
                self.end_headers()
                if sse:
                    # No matching response or completed line; socket timeouts alone cannot bound this.
                    while not self.server.stop.wait(0.03):
                        self.wfile.write(b":" if mode == "trickle" else b": keepalive\n\n")
                        self.wfile.flush()
                else:
                    self.wfile.write(data)
            except (BrokenPipeError, ConnectionResetError, ConnectionAbortedError):
                pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    server.daemon_threads = True
    server.stop = threading.Event()
    server.requests = []
    thread = threading.Thread(target=server.serve_forever, kwargs={"poll_interval": 0.02})
    thread.start()
    try:
        yield server, f"http://127.0.0.1:{server.server_port}/mcp"
    finally:
        server.stop.set()
        server.shutdown()
        server.server_close()
        thread.join(timeout=2)
        if thread.is_alive():
            raise AssertionError("Synthetic loopback server did not stop.")


class DeadlineTests(unittest.TestCase):
    def test_total_deadline_stops_owned_client_across_all_blocking_stages(self):
        real_popen = subprocess.Popen
        for mode, stage, count in (
            ("initialize", "initialize", 1),
            ("notification", "notifications/initialized", 2),
            ("sse", "initialize", 1),
            ("trickle", "initialize", 1),
            ("tool", "tools/call", 3),
            ("cumulative", "tools/call", 3),
        ):
            children = []

            def start_child(*args, **kwargs):
                self.assertNotIn(TOKEN, repr(args))
                child = real_popen(*args, **kwargs)
                children.append(child)
                return child

            with self.subTest(mode=mode), loopback_peer(mode) as (server, url):
                began = time.monotonic()
                with patch.object(TRANSPORT.subprocess, "Popen", side_effect=start_child):
                    with self.assertRaises(TRANSPORT.UxError) as raised:
                        TRANSPORT.powershell_bounded(url, TOKEN, "synthetic", 20, deadline=0.65)
                elapsed = time.monotonic() - began
                self.assertEqual(raised.exception.code, "deadline")
                self.assertEqual(raised.exception.details["stage"], stage)
                self.assertEqual(raised.exception.details["deadline_seconds"], 0.65)
                self.assertEqual(raised.exception.details["remoteCompletion"], "UNKNOWN")
                self.assertNotIn(TOKEN, str(raised.exception))
                self.assertGreaterEqual(elapsed, 0.5)
                self.assertLess(elapsed, 1.65)
                self.assertEqual(len(children), 1)
                self.assertIsNotNone(children[0].poll())
                self.assertEqual(len(server.requests), count)
                self.assertTrue(all(auth == "Bearer " + TOKEN for _payload, auth in server.requests))

    def test_success_and_metrics_unchanged_and_child_reaped(self):
        with loopback_peer("success") as (server, url):
            reply, metrics = TRANSPORT.powershell_bounded(
                url, TOKEN, "synthetic", 20, deadline=3,
            )
        self.assertEqual(reply, {"ok": True})
        self.assertEqual(metrics["http_requests"], 3)
        self.assertEqual(metrics["rpc_calls"], 2)
        self.assertEqual(metrics["notifications"], 1)
        self.assertEqual(metrics["mcp_tool_calls"], 1)
        self.assertEqual([payload["method"] for payload, _auth in server.requests], [
            "initialize", "notifications/initialized", "tools/call",
        ])
        self.assertEqual(server.requests[-1][0]["params"]["arguments"]["timeout"], 20)
        self.assertNotIn(TOKEN, json.dumps((reply, metrics)))

    def test_worker_failure_is_preserved_without_retry_or_success_fallback(self):
        with loopback_peer("forbidden") as (server, url):
            with self.assertRaises(TRANSPORT.UxError) as raised:
                TRANSPORT.powershell_bounded(url, TOKEN, "synthetic", 20, deadline=3)
        self.assertEqual(raised.exception.code, "http_status")
        self.assertEqual(raised.exception.metrics["http_requests"], 1)
        self.assertEqual(len(server.requests), 1)
        self.assertNotIn(TOKEN, str(raised.exception))

    def test_invalid_deadlines_fail_before_starting_a_child(self):
        with patch.object(TRANSPORT.subprocess, "Popen") as child:
            for value in (0, -1, float("nan"), float("inf"), True, "bad"):
                with self.subTest(value=value), self.assertRaises(TRANSPORT.UxError) as raised:
                    TRANSPORT.powershell_bounded(
                        "http://127.0.0.1:1/mcp", TOKEN, "synthetic", 20, deadline=value,
                    )
                self.assertEqual(raised.exception.code, "deadline_value")
            child.assert_not_called()


class RecipeTests(unittest.TestCase):
    def test_closed_recipe_validation_rejects_before_network(self):
        base = {"schema_version": 1, "name": "Synthetic", "steps": [{"op": "status"}]}
        invalid = [
            {**base, "steps": []},
            {**base, "steps": [{"op": "status"}] * 65},
            {**base, "schema_version": True},
            {**base, "requires": {"exe_sha256": "bad"}},
            {**base, "template": "not-supported"},
        ]
        invalid_steps = [
            {"op": "launch"}, {"op": "status", "command": "not-supported"},
            {"op": "energy.read", "row": 0, "column": 1},
            {"op": "energy.read", "row": True, "column": 1},
            {"op": "energy.set", "row": 1, "column": 11, "value": ""},
            {"op": "energy.select", "row": 1, "column": 0, "index": 28},
            {"op": "wait"}, {"op": "wait", "selector": {"name": "x"}, "timeout_ms": 30001},
            {"op": "wait", "selector": {"name": "x"}, "value": 1},
            {"op": "invoke", "selector": {"xpath": "*"}},
            {"op": "invoke", "selector": {}},
            {"op": "set_value", "value": 1},
            {"op": "inspect", "depth": 6},
            {"op": "save_as", "path": r"C:relative.Loxone"},
            {"op": "save_as", "path": r"C:\private\..\escape.Loxone"},
        ]
        invalid.extend({**base, "steps": [step]} for step in invalid_steps)
        with patch.object(UX, "powershell_bounded") as remote:
            for recipe in invalid:
                with self.subTest(recipe=recipe):
                    with patch.object(Path, "read_bytes", return_value=json.dumps(recipe).encode()):
                        with self.assertRaises(UX.UxError):
                            UX.execute(UX.make_parser().parse_args(["run", "offline.json"]))
            remote.assert_not_called()

    def test_pin_resolution_and_windows_remote_paths_do_not_use_local_resolution(self):
        checked = UX.validate_profile(profile())
        steps = [{"op": "energy.set", "row": 1, "column": 0, "value": "Synthetic"}]
        with patch.object(Path, "resolve", side_effect=AssertionError("Not a local Windows path")):
            one = UX.build_request(checked, 123, None, steps, PIN.upper())
            two = UX.build_request(checked, 123, None, steps, PIN)
        self.assertEqual(one["target"], {"pid": 123, "expected_exe_sha256": PIN})
        self.assertEqual(PureWindowsPath(one["artifact_dir"]).parent, PureWindowsPath(r"D:\Private Runs"))
        self.assertNotEqual(one["artifact_dir"], two["artifact_dir"])
        self.assertNotIn("Synthetic", one["artifact_dir"])
        self.assertEqual(UX.build_request(checked, 123, OTHER_PIN, steps)["target"]["expected_exe_sha256"], OTHER_PIN)
        for override in (None, OTHER_PIN):
            with self.assertRaises(UX.UxError) as raised:
                UX.build_request(checked, 123, override, steps, OTHER_PIN if override is None else PIN)
            self.assertEqual(raised.exception.code, "version_pin")
        del checked["expected_exe_sha256"]
        with self.assertRaises(UX.UxError):
            UX.build_request(checked, 123, None, steps)
        self.assertIsNone(UX.build_request(checked, 123, None, [{"op": "status"}])["target"]["expected_exe_sha256"])
        self.assertEqual(UX.build_request(checked, 123, None, steps, PIN)["target"]["expected_exe_sha256"], PIN)

    def test_base64_encodes_all_user_data_including_backend_path(self):
        malicious = "'; throw 'not-executed'; # Grüße $(not-executed)"
        request = UX.build_request(profile(), 123, PIN, [
            UX.validate_step({"op": "set_value", "selector": {"name": malicious}, "value": malicious}),
        ])
        path = r"C:\Owned Tools\we'ird.ps1"
        command = UX.powershell_command(path, request)
        encoded = re.findall(r"FromBase64String\('([A-Za-z0-9+/=]+)'\)", command)
        self.assertEqual(len(encoded), 2)
        self.assertEqual(base64.b64decode(encoded[0]).decode(), path)
        self.assertEqual(json.loads(base64.b64decode(encoded[1])), request)
        self.assertNotIn(malicious, command)
        self.assertNotIn(path, command)
        self.assertNotIn("Invoke-Expression", command)

    def test_profile_token_expansion_and_endpoint_constraints(self):
        with patch.dict(os.environ, {"LOX_UX_TOKEN_FILE": str(Path.home() / "offline-token")}):
            checked = UX.validate_profile({**profile(), "token_file": "${LOX_UX_TOKEN_FILE}"})
        self.assertEqual(checked["token_file"], Path.home() / "offline-token")
        for changes in (
            {"mcp_url": "http://example.invalid/mcp"},
            {"mcp_url": "https://name:secret@example.invalid/mcp"},
            {"mcp_url": "http://127.0.0.1/mcp?token=not-allowed"},
            {"mcp_url": "http://127.0.0.1/mcp\x01"},
            {"backend_path": r"relative\backend.ps1"},
            {"remote_artifact_root": r"Z:relative"},
            {"token_file": "relative-token"}, {"expected_exe_sha256": "bad"},
        ):
            with self.subTest(changes=changes):
                with self.assertRaises(UX.UxError):
                    UX.validate_profile({**profile(), **changes})

    def test_json_rejects_duplicate_fields_and_nonfinite_numbers(self):
        for data in ('{"op":"status","op":"invoke"}', '{"value":NaN}'):
            with self.subTest(data=data):
                with self.assertRaises(ValueError):
                    TRANSPORT.parse_json(data)

    def test_conflicting_recipe_pin_fails_before_token_read_or_remote_creation(self):
        args = UX.make_parser().parse_args([
            "--profile", "private.json", "--pid", "123", "run", "offline.json",
        ])
        with patch.object(UX, "command_steps", return_value=([{"op": "status"}], OTHER_PIN)):
            with patch.object(Path, "read_bytes", return_value=json.dumps(profile()).encode()):
                with patch.object(Path, "read_text") as token_read, patch.object(UX, "powershell_bounded") as remote:
                    with self.assertRaises(UX.UxError) as raised:
                        UX.execute(args)
        self.assertEqual(raised.exception.code, "version_pin")
        token_read.assert_not_called()
        remote.assert_not_called()


class CliTests(unittest.TestCase):
    def run_mocked_cli(self, command, *, ok=True, change_reply=None):
        calls = []

        def backend(payload):
            calls.append(payload)
            encoded = re.findall(r"FromBase64String\('([A-Za-z0-9+/=]+)'\)",
                                 payload["params"]["arguments"]["command"])
            request = json.loads(base64.b64decode(encoded[1]))
            reply = backend_reply(request, ok=ok)
            if change_reply:
                change_reply(reply)
            return Response(rpc(tool_result(reply), request_id=2))

        wire = Wire([
            Response(rpc({"protocolVersion": TRANSPORT.PROTOCOL})),
            Response(b"", status=202), backend,
        ])
        output = io.StringIO()

        def bounded(url, token, command, timeout, *, deadline):
            self.assertEqual(deadline, 300)
            client = TRANSPORT.MCPClient(url, token, opener=wire)
            client.initialize()
            return client.powershell(command, timeout), client.metrics

        with patch.object(Path, "read_bytes", return_value=json.dumps(profile()).encode()):
            with patch.object(Path, "read_text", return_value=TOKEN):
                with patch.object(UX, "powershell_bounded", side_effect=bounded):
                    with contextlib.redirect_stdout(output):
                        code = UX.main(["--profile", "private-profile.json", "--pid", "123", *command])
        return code, output.getvalue(), wire, calls

    def test_single_command_ascii_output_and_precise_metrics(self):
        code, raw, wire, calls = self.run_mocked_cli(["status"])
        output = json.loads(raw)
        self.assertEqual(code, 0)
        self.assertEqual(output["results"][0]["value"], {"text": "Größe"})
        self.assertIn(r"\u00f6", raw)
        self.assertNotIn("private-profile", raw)
        self.assertNotIn(TOKEN, raw)
        self.assertNotIn("backend_path", raw)
        self.assertEqual(len(calls), 1)
        self.assertEqual(len(wire.requests), 3)
        self.assertEqual(output["transport_metrics"]["output_bytes"], len(raw.encode("ascii")))
        self.assertEqual(output["transport_metrics"]["request_bytes"],
                         sum(len(query.data) for query, _kwargs in wire.requests))
        self.assertEqual(output["transport_metrics"]["response_bytes"], wire.response_bytes)

    def test_deadline_option_and_structured_error_reach_cli_without_secret_output(self):
        for argv in (
            ["--deadline-seconds", "0.5", "status"],
            ["status", "--deadline-seconds", "0.5"],
        ):
            error = TRANSPORT._deadline_error({"stage": "tools/call", "metrics": {}}, 0.5)
            output = io.StringIO()
            with patch.object(Path, "read_bytes", return_value=json.dumps(profile()).encode()):
                with patch.object(Path, "read_text", return_value=TOKEN):
                    with patch.object(UX, "powershell_bounded", side_effect=error) as bounded:
                        with contextlib.redirect_stdout(output):
                            code = UX.main(["--profile", "private.json", "--pid", "123", *argv])
            result = json.loads(output.getvalue())
            self.assertEqual(code, 1)
            self.assertEqual(bounded.call_args.kwargs["deadline"], 0.5)
            self.assertEqual(result["error"]["stage"], "tools/call")
            self.assertEqual(result["error"]["remoteCompletion"], "UNKNOWN")
            self.assertIn("requested_artifact_dir", result)
            self.assertNotIn(TOKEN, output.getvalue())
        with patch.object(UX, "powershell_bounded") as bounded:
            for value in ("nan", "inf", "-1", "0", "bad"):
                with contextlib.redirect_stdout(io.StringIO()):
                    self.assertEqual(UX.main(["status", "--deadline-seconds", value]), 1)
            bounded.assert_not_called()

    def test_backend_false_preserves_partial_progress_and_stops_without_retry(self):
        args = UX.make_parser().parse_args([
            "--profile", "private.json", "--pid", "123", "run", "offline.json",
        ])
        steps = [{"op": "status"}, {"op": "invoke", "selector": {"name": "Synthetic"}}]
        with patch.object(UX, "command_steps", return_value=(steps, PIN)):
            code, raw, wire, calls = self.run_mocked_cli(["run", args.recipe], ok=False)
        output = json.loads(raw)
        self.assertEqual(code, 1)
        self.assertFalse(output["ok"])
        self.assertEqual(output["completed_steps"], 1)
        self.assertEqual(output["failed_step"], 1)
        self.assertEqual(output["error"]["code"], "selector_not_unique")
        self.assertEqual(len(output["results"]), 1)
        self.assertEqual(len(wire.requests), 3)
        self.assertEqual(len(calls), 1)

    def test_compact_single_control_inspection_returns_value_without_another_call(self):
        value = {"controls": [{"name": "Synthetic", "automation_id": "synthetic"}]}
        code, raw, wire, calls = self.run_mocked_cli(
            ["inspect", "--limit", "1", "--name", "Synthetic"],
            change_reply=lambda reply: reply["results"][0].update(value=value),
        )
        self.assertEqual(code, 0)
        self.assertEqual(json.loads(raw)["results"], [{"index": 0, "op": "inspect", "value": value}])
        self.assertEqual(len(calls), 1)
        self.assertEqual(len(wire.requests), 3)

    def test_compact_inspection_batch_obeys_shared_read_budget(self):
        request = UX.build_request(profile(), 123, PIN, [{"op": "inspect"}] * 8 + [{"op": "status"}])
        reply = backend_reply(request)
        for result in reply["results"][:-1]:
            result["value"] = {"text": "x" * 1500}
        results = UX.compact_reply(reply)["results"]
        self.assertTrue(all(result.get("value") == {"text": "x" * 1500} for result in results[:5]))
        for result in results[5:8]:
            self.assertTrue(result["omitted"])
            self.assertEqual(result["reason"], "read_size_limit")
            self.assertEqual(result["artifact"], reply["artifacts"]["report"])
        self.assertEqual(results[-1], reply["results"][-1])

    def test_compact_oversized_inspections_and_reads_reference_full_private_report(self):
        for command in (["inspect"], ["status"]):
            code, raw, _wire, _calls = self.run_mocked_cli(
                command, change_reply=lambda reply: reply["results"][0].update(value={"tree": "x" * 10000}),
            )
            result = json.loads(raw)["results"][0]
            self.assertEqual(code, 0)
            self.assertTrue(result["omitted"])
            self.assertEqual(result["reason"], "read_size_limit")
            self.assertTrue(result["artifact"].endswith("report.json"))
            self.assertNotIn("tree", raw)
            self.assertLess(len(raw), 2048)
        code, raw, _wire, _calls = self.run_mocked_cli(["status", "--verbose"])
        self.assertEqual(code, 0)
        self.assertIn("target", json.loads(raw))
        self.assertIn("metrics", json.loads(raw))
        self.assertNotIn("transport_metrics", json.loads(raw)["metrics"])

    def test_backend_cannot_claim_success_for_wrong_identity_or_partial_steps(self):
        for change in (
            lambda reply: reply["target"].update(exe_sha256=OTHER_PIN),
            lambda reply: reply.update(completed_steps=0),
            lambda reply: reply.update(pid=999),
            lambda reply: reply["results"][0].update(index=5),
        ):
            code, raw, _wire, _calls = self.run_mocked_cli(["status"], change_reply=change)
            output = json.loads(raw)
            self.assertEqual(code, 1)
            self.assertFalse(output["ok"])
            self.assertIn(output["error"]["code"], ("version_pin", "backend_response"))
            self.assertIn("requested_artifact_dir", output)

    def test_invalid_token_is_a_redacted_local_error(self):
        with patch.object(Path, "read_bytes", return_value=json.dumps(profile()).encode()):
            with patch.object(Path, "read_text", return_value="Bearer " + TOKEN):
                with patch.object(UX, "powershell_bounded") as remote:
                    output = io.StringIO()
                    with contextlib.redirect_stdout(output):
                        code = UX.main(["--profile", "private.json", "--pid", "123", "status"])
        self.assertEqual(code, 1)
        self.assertEqual(json.loads(output.getvalue())["error"]["code"], "auth")
        self.assertNotIn(TOKEN, output.getvalue())
        remote.assert_not_called()

    def test_binary_stdout_has_exact_size_without_windows_newline_translation(self):
        output = io.BytesIO()
        with patch.object(UX.sys, "stdout", SimpleNamespace(buffer=output)):
            code = UX.main(["energy-read", "0", "0"])
        raw = output.getvalue()
        self.assertEqual(code, 1)
        self.assertEqual(json.loads(raw)["transport_metrics"]["output_bytes"], len(raw))
        self.assertTrue(raw.endswith(b"\n"))
        self.assertNotIn(b"\r", raw)

    def test_cli_maps_supported_commands_and_common_flags_after_subcommand(self):
        cases = [
            (["energy-set", "1", "name", "Synthetic"], {"op": "energy.set", "row": 1, "column": 4, "value": "Synthetic"}),
            (["energy-select", "2", "mode", "0"], {"op": "energy.select", "row": 2, "column": 2, "index": 0}),
            (["energy-read", "1", "0"], {"op": "energy.read", "row": 1, "column": 0}),
            (["energy-accept"], {"op": "energy.accept"}),
            (["energy-cancel"], {"op": "energy.cancel"}),
            (["energy-open"], {"op": "energy.open"}),
            (["save-as", r"C:\Owned\Synthetic.Loxone"], {"op": "save_as", "path": r"C:\Owned\Synthetic.Loxone"}),
            (["set-value", "", "--name", "Synthetic"], {"op": "set_value", "selector": {"name": "Synthetic"}, "value": ""}),
            (["keys", "^s"], {"op": "keys", "value": "^s"}),
            (["focus", "--window-class", "Synthetic"], {"op": "focus", "window_class": "Synthetic"}),
            (["invoke", "--selector", '{"automation_id_suffix":"synthetic"}'],
             {"op": "invoke", "selector": {"automation_id_suffix": "synthetic"}}),
            (["wait", "--name", "Synthetic", "--timeout-ms", "1"],
             {"op": "wait", "selector": {"name": "Synthetic"}, "timeout_ms": 1}),
            (["wait", "--name", "Synthetic", "--value", "ready"],
             {"op": "wait", "selector": {"name": "Synthetic"}, "timeout_ms": 5000, "value": "ready"}),
            (["windows"], {"op": "windows"}),
        ]
        for command, step in cases:
            with self.subTest(command=command):
                args = UX.make_parser().parse_args([*command, "--pid", "123", "--profile", "private.json"])
                self.assertEqual(args.pid, 123)
                self.assertEqual(UX.command_steps(args), ([step], None))


if __name__ == "__main__":
    unittest.main()
