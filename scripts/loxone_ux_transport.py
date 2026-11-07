"""Small authenticated JSON-RPC HTTP/SSE transport for the UX CLI."""

import http.client
import json
import math
import os
from pathlib import Path
import re
import subprocess
import sys
import threading
import time
import urllib.error
import urllib.request


PROTOCOL = "2025-06-18"
MAX_RESPONSE_BYTES = 8 * 1024 * 1024


class UxError(Exception):
    def __init__(self, code, message):
        super().__init__(message)
        self.code = code
        self.metrics = None
        self.artifact_dir = None
        self.details = None


def parse_json(data):
    def constant(_value):
        raise ValueError("Non-finite JSON number")

    def object_fields(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError("Duplicate JSON field")
            result[key] = value
        return result

    return json.loads(data, parse_constant=constant, object_pairs_hook=object_fields)


class _NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


class MCPClient:
    """No discovery, retries, redirects, proxy inheritance, or raw response logging."""

    def __init__(self, url, token, *, opener=None, progress=None):
        self.url = url
        self._token = token
        self._open = opener or urllib.request.build_opener(
            urllib.request.ProxyHandler({}), _NoRedirect()
        ).open
        self._next_id = 0
        self._progress = progress
        self.metrics = {
            "http_requests": 0, "rpc_calls": 0, "notifications": 0,
            "mcp_tool_calls": 0, "request_bytes": 0, "response_bytes": 0,
        }

    def initialize(self):
        result = self.request("initialize", {
            "protocolVersion": PROTOCOL,
            "capabilities": {},
            "clientInfo": {"name": "loxone-ux", "version": "1"},
        })
        if not isinstance(result, dict) or result.get("protocolVersion") != PROTOCOL:
            raise UxError("protocol_version", "MCP protocol mismatch; use the documented peer version.")
        self.request("notifications/initialized", {}, notification=True)

    def request(self, method, params, *, notification=False, timeout=90):
        payload = {"jsonrpc": "2.0", "method": method, "params": params}
        request_id = None
        if notification:
            self.metrics["notifications"] += 1
        else:
            self._next_id += 1
            request_id = self._next_id
            payload["id"] = request_id
            self.metrics["rpc_calls"] += 1
        if method == "tools/call":
            self.metrics["mcp_tool_calls"] += 1
        body = json.dumps(payload, ensure_ascii=True, separators=(",", ":")).encode("utf-8")
        query = urllib.request.Request(self.url, data=body, method="POST", headers={
            "Authorization": "Bearer " + self._token,
            "Accept": "application/json, text/event-stream",
            "Content-Type": "application/json",
            "MCP-Protocol-Version": PROTOCOL,
        })
        self.metrics["http_requests"] += 1
        self.metrics["request_bytes"] += len(body)
        if self._progress is not None:
            self._progress(method, self.metrics)
        try:
            with self._open(query, timeout=timeout) as response:
                status = response.status
                if notification:
                    self._read(response)
                    if status not in (200, 202, 204):
                        raise UxError("http_status", f"MCP notification returned HTTP {status}.")
                    return None
                if status != 200:
                    self._read(response)
                    raise UxError("http_status", f"MCP request returned HTTP {status}.")
                if "text/event-stream" in response.headers.get("Content-Type", "").lower():
                    message = self._sse(response, request_id)
                else:
                    message = self._json(self._read(response))
        except urllib.error.HTTPError as error:
            # Do not echo URLs, response bodies, headers, or the exception's text.
            try:
                self._read(error)
            except (OSError, http.client.HTTPException, ValueError):
                pass
            finally:
                error.close()
            raise UxError("http_status", f"MCP request failed with HTTP {error.code}.") from None
        except (urllib.error.URLError, OSError, http.client.HTTPException, ValueError):
            raise UxError(
                "transport", "MCP connection failed; inspect the owned run before retrying any input."
            ) from None
        if not isinstance(message, dict) or message.get("jsonrpc") != "2.0":
            raise UxError("rpc_response", "MCP returned an invalid JSON-RPC response.")
        if type(message.get("id")) is not int or message["id"] != request_id:
            raise UxError("rpc_id", "MCP response did not match the request ID.")
        if "error" in message:
            raise UxError("rpc_error", "MCP returned a JSON-RPC error; no success was inferred.")
        if "result" not in message:
            raise UxError("rpc_response", "MCP response omitted its result.")
        return message["result"]

    def _read(self, response):
        data = response.read(MAX_RESPONSE_BYTES + 1)
        self.metrics["response_bytes"] += len(data)
        if len(data) > MAX_RESPONSE_BYTES:
            raise UxError("response_size", "MCP response exceeded the transport size limit.")
        return data

    @staticmethod
    def _json(data):
        try:
            return parse_json(data)
        except (ValueError, UnicodeError, RecursionError):
            raise UxError("response_json", "MCP response was not valid JSON.") from None

    def _sse(self, response, request_id):
        data = []
        size = 0
        first = True
        while True:
            raw = response.readline(MAX_RESPONSE_BYTES - size + 1)
            size += len(raw)
            self.metrics["response_bytes"] += len(raw)
            if size > MAX_RESPONSE_BYTES:
                raise UxError("response_size", "MCP stream exceeded the transport size limit.")
            try:
                line = raw.decode("utf-8-sig" if first else "utf-8").rstrip("\r\n")
            except UnicodeError:
                raise UxError("response_json", "MCP stream was not valid UTF-8.") from None
            first = False
            if line.startswith("data:"):
                field = line[5:]
                data.append(field[1:] if field.startswith(" ") else field)
            elif not line and data:
                candidate = self._json("\n".join(data))
                data = []
                if (isinstance(candidate, dict) and type(candidate.get("id")) is int
                        and candidate["id"] == request_id):
                    return candidate
            if not raw:
                raise UxError("rpc_missing", "MCP stream ended without a matching response.")

    def powershell(self, command, timeout):
        result = self.request("tools/call", {
            "name": "PowerShell", "arguments": {"command": command, "timeout": timeout},
        }, timeout=timeout + 15)
        if not isinstance(result, dict) or result.get("isError", False) is not False:
            raise UxError("tool_error", "The MCP PowerShell tool failed; no success was inferred.")
        structured = result.get("structuredContent")
        text = structured.get("result") if isinstance(structured, dict) else None
        if not isinstance(text, str):
            raise UxError("powershell_result", "PowerShell omitted its structured result.")
        match = re.fullmatch(r"Response: (.*)\r?\nStatus Code:\s*(-?\d+)\s*", text, re.DOTALL)
        if match is None:
            raise UxError("powershell_status", "PowerShell omitted its actual command exit status.")
        if int(match.group(2)) != 0:
            raise UxError(
                "powershell_exit", "PowerShell returned a nonzero exit status; inspect the owned run."
            )
        try:
            reply = parse_json(match.group(1))
        except (ValueError, RecursionError):
            raise UxError("backend_json", "PowerShell stdout was not one backend JSON response.") from None
        return reply


def validate_deadline(value):
    if type(value) not in (int, float) or not math.isfinite(value) or value <= 0:
        raise UxError("deadline_value", "Local deadline must be a finite positive number of seconds.")
    return value


def powershell_bounded(url, token, command, timeout, *, deadline=300):
    """Bound the entire local MCP exchange; never infer remote cancellation."""
    validate_deadline(deadline)
    began = time.monotonic()
    # Leave cleanup time inside the caller's budget, rather than adding a grace
    # period to a network wait that has already consumed the entire deadline.
    until = began + deadline - min(1, deadline / 10)
    data = json.dumps({
        "url": url, "token": token, "command": command, "timeout": timeout,
    }, ensure_ascii=True, allow_nan=False).encode("ascii")
    try:
        process = subprocess.Popen(
            [sys.executable, "-B", str(Path(__file__).resolve())],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
            close_fds=True, shell=False,
            creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0,
        )
    except OSError:
        raise UxError("client_process", "Cannot start the owned local MCP client; check Python access.") from None
    state = {"stage": "starting", "metrics": {}}
    finished = {}

    def exchange():
        try:
            with process.stdin:
                process.stdin.write(data)
            # Only this owned worker writes the pipe. Allow JSON escaping overhead;
            # the HTTP/SSE response itself still has its original 8 MiB cap.
            limit = MAX_RESPONSE_BYTES * 6 + 65536
            with process.stdout:
                while True:
                    line = process.stdout.readline(limit + 1)
                    if not line:
                        break
                    if len(line) > limit:
                        finished["invalid"] = True
                        return
                    message = parse_json(line)
                    if not isinstance(message, dict):
                        finished["invalid"] = True
                        return
                    state.update(message)
            finished["done"] = True
        except (OSError, ValueError, UnicodeError, RecursionError):
            finished["invalid"] = True

    reader = threading.Thread(target=exchange, name="loxone-ux-owned-client")
    try:
        reader.start()
        reader.join(max(0, until - time.monotonic()))
        remaining = until - time.monotonic()
        if reader.is_alive() or remaining <= 0:
            raise _deadline_error(state, deadline)
        try:
            process.wait(timeout=remaining)
        except subprocess.TimeoutExpired:
            raise _deadline_error(state, deadline) from None
        if process.returncode != 0 or not finished.get("done") or finished.get("invalid"):
            raise UxError(
                "client_process", "Owned local MCP client failed; no success or remote cancellation was inferred."
            )
        if "error" in state:
            error = UxError(state["error"]["code"], state["error"]["message"])
            error.metrics = state["metrics"]
            raise error
        if "reply" not in state:
            raise UxError("client_process", "Owned local MCP client omitted its result; inspect the owned run.")
        return state["reply"], state["metrics"]
    finally:
        if process.poll() is None:
            process.kill()
        process.wait()
        if reader.ident is not None:
            reader.join()
        process.stdin.close()
        process.stdout.close()


def _deadline_error(state, deadline):
    error = UxError(
        "deadline",
        "Local MCP caller deadline reached its cleanup reserve; only the owned local client is stopped. "
        "Remote completion is UNKNOWN; inspect the owned run before any separately admitted retry.",
    )
    error.details = {
        "stage": state["stage"], "deadline_seconds": deadline,
        "remoteCompletion": "UNKNOWN", "metrics_complete": False,
    }
    error.metrics = dict(state["metrics"])
    return error


def _worker():
    def emit(message):
        sys.stdout.buffer.write(
            (json.dumps(message, ensure_ascii=True, allow_nan=False) + "\n").encode("ascii")
        )
        sys.stdout.buffer.flush()

    def progress(stage, metrics):
        emit({"stage": stage, "metrics": metrics})

    request = parse_json(sys.stdin.buffer.read())
    client = MCPClient(request["url"], request["token"], progress=progress)
    try:
        client.initialize()
        reply = client.powershell(request["command"], request["timeout"])
        emit({"reply": reply, "metrics": client.metrics})
    except UxError as error:
        emit({"error": {"code": error.code, "message": str(error)}, "metrics": client.metrics})


if __name__ == "__main__":
    _worker()
