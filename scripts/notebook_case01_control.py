"""Case01-only bounded read of the supported ControlStatus tool.

No acquire, takeover, retry, discovery, arbitrary tool, or PowerShell route.
"""

import hashlib
import importlib.util
import json
import math
import os
from pathlib import Path
import subprocess
import sys
import time


_path = Path(__file__).with_name("loxone_ux_transport.py")
if hashlib.sha256(_path.read_bytes()).hexdigest() not in (
    "f88b9d733f9f4cbe4ea6ceb9efcfac2a864b20bcb5f4dd713229414d36c15da8",
    "527e4743b0bae46fc25a8318d4504dc0b25a7872d256e22ab843248ac4cb3334",
):
    raise ImportError("ControlStatus requires exact published transport bytes")
_spec = importlib.util.spec_from_file_location("_case01_control_transport", _path)
TRANSPORT = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(TRANSPORT)


def decode_status(reply):
    if not isinstance(reply, dict):
        raise TRANSPORT.UxError("control_refused", "ControlStatus was refused.")
    content = reply.get("content")
    if not (isinstance(content, list) and len(content) == 1
            and isinstance(content[0], dict) and content[0].get("type") == "text"):
        raise TRANSPORT.UxError("control_schema", "Unsupported ControlStatus response schema.")
    try:
        status = TRANSPORT.parse_json(content[0]["text"])
    except (ValueError, KeyError, TypeError):
        raise TRANSPORT.UxError("control_schema", "Unsupported ControlStatus JSON.") from None
    if isinstance(status, dict) and status.get("code") in ("USER_CONTROL", "CONTROL_UNAVAILABLE"):
        raise TRANSPORT.UxError(status["code"], "Supported control guard refused.")
    if reply.get("isError", False) is not False:
        raise TRANSPORT.UxError("control_refused", "ControlStatus was refused.")
    if not (isinstance(status, dict) and set(status) == {"state", "generation", "wait_seconds"}
            and type(status["generation"]) is int and status["generation"] >= 0
            and type(status["wait_seconds"]) in (int, float)
            and math.isfinite(status["wait_seconds"]) and status["wait_seconds"] == 0
            and status["state"] == "ai"):
        raise TRANSPORT.UxError("control_unavailable", "No current supported AI control.")
    return status


def control_status_bounded(url, token, *, deadline):
    if type(deadline) not in (int, float) or not math.isfinite(deadline) or not 0 < deadline <= 25:
        raise TRANSPORT.UxError("deadline_value", "ControlStatus requires explicit deadline <=25.")
    began = time.monotonic()
    reserve = min(1, deadline / 10)
    process = subprocess.Popen(
        [sys.executable, "-B", str(Path(__file__).resolve())],
        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
        shell=False, close_fds=True,
        creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0,
    )
    try:
        payload = json.dumps({"url": url, "token": token}, allow_nan=False).encode("utf-8")
        try:
            output, _ = process.communicate(payload, timeout=max(0, deadline - reserve
                                                               - (time.monotonic() - began)))
        except subprocess.TimeoutExpired:
            raise TRANSPORT.UxError("deadline", "ControlStatus completion UNKNOWN.") from None
        if process.returncode != 0 or len(output) > 65536:
            raise TRANSPORT.UxError("control_worker", "ControlStatus worker failed.")
        message = TRANSPORT.parse_json(output)
        if isinstance(message, dict) and isinstance(message.get("error"), str):
            raise TRANSPORT.UxError(message["error"], "ControlStatus observation failed.")
        if not isinstance(message, dict) or "status" not in message:
            raise TRANSPORT.UxError("control_worker", "ControlStatus worker omitted observation.")
        return message["status"]
    finally:
        if process.poll() is None:
            process.kill()  # This is solely the owned local read-only RPC client.
            try:
                process.wait(timeout=max(0, deadline - (time.monotonic() - began)))
            except subprocess.TimeoutExpired:
                raise TRANSPORT.UxError("client_cleanup_unknown",
                                        "Local ControlStatus client cleanup UNKNOWN.") from None
        process.stdin.close()
        process.stdout.close()


def _worker():
    try:
        request = TRANSPORT.parse_json(sys.stdin.buffer.read(65537))
        client = TRANSPORT.MCPClient(request["url"], request["token"])
        client.initialize()
        status = decode_status(client.request("tools/call", {"name": "ControlStatus", "arguments": {}}))
        message = {"status": status}
    except TRANSPORT.UxError as error:
        message = {"error": error.code}
    sys.stdout.buffer.write(json.dumps(message, allow_nan=False).encode("ascii"))


if __name__ == "__main__":
    _worker()
