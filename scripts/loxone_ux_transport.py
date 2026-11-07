"""Small authenticated JSON-RPC HTTP/SSE transport for the UX CLI."""

import http.client
import json
import re
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

    def __init__(self, url, token, *, opener=None):
        self.url = url
        self._token = token
        self._open = opener or urllib.request.build_opener(
            urllib.request.ProxyHandler({}), _NoRedirect()
        ).open
        self._next_id = 0
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
