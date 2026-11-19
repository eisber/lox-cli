"""Exercise the installed wheel and the same generated JVM API over scripted loopback."""

import argparse
import asyncio
from contextlib import ExitStack, contextmanager
import hashlib
from http.server import BaseHTTPRequestHandler, HTTPServer
import json
import os
from pathlib import Path
import select
import socket
import subprocess
import sys
import threading

import lox_archive as native


PATH = "/jdev/cfg/api"
MARKER = "private-bootstrap-marker"
CASES = ("available", "unavailable", "missing-slots", "missing-tls", "http", "command", "invalid")
JAVA_LIBRARY_OVERRIDE = "uniffi.component.lox_archive.libraryOverride"
JAVA_OPTION_ENV = ("JDK_JAVA_OPTIONS", "JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS")


def reply_body(slots):
    value = json.dumps({**slots, "notProjected": MARKER})
    return json.dumps({"LL": {"control": "dev/cfg/api", "Code": 200, "value": value}}).encode()


REPLIES = {
    "available": (200, reply_body({"hasEventSlots": True, "httpsStatus": 1})),
    "unavailable": (200, reply_body({"hasEventSlots": False, "httpsStatus": 2})),
    "missing-slots": (200, reply_body({"httpsStatus": 9})),
    "missing-tls": (200, reply_body({"hasEventSlots": None})),
    "http": (503, MARKER.encode()),
    "command": (200, json.dumps({"LL": {"control": "dev/cfg/api", "Code": 403, "value": MARKER}}).encode()),
    "invalid": (200, b'{"bad":"' + MARKER.encode()),
    "body-timeout": (200, b'{"LL":'),
}


def origin(sock, scheme):
    return f"{scheme}://127.0.0.1:{sock.getsockname()[1]}"


@contextmanager
def fixture(expected, evidence):
    observed = evidence["requests"]
    closed = evidence["released_sockets"]
    failures = []

    class Handler(BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.1"
        timeout = 5

        def log_message(self, *_):
            pass

        def do_GET(self):
            self.close_connection = True
            i = len(observed)
            if i >= len(expected) or self.path != PATH:
                failures.append("unexpected request count or path")
                self.send_error(404)
                return
            label = expected[i]
            observed.append(label)
            if any(name in self.headers for name in ("Authorization", "Proxy-Authorization", "Cookie", "Upgrade")):
                failures.append("authentication or WebSocket headers on HTTP preflight")
            case = label.split("/", 1)[1]
            status, body = REPLIES[case]
            stalled = case == "body-timeout"
            self.send_response(status)
            self.send_header("Content-Length", str(1024 if stalled else len(body)))
            self.end_headers()
            self.wfile.write(body)
            self.wfile.flush()
            # Keep the connection open on the server side until the client releases it.
            if stalled:
                self.connection.settimeout(20)
            try:
                if self.rfile.read(1) != b"":
                    failures.append(f"{label}: unexpected data after response")
                    return
            except (ConnectionResetError, ConnectionAbortedError):
                pass
            except TimeoutError:
                failures.append(f"{label}: socket not released")
                return
            closed.append(label)

    with HTTPServer(("127.0.0.1", 0), Handler) as server:
        thread = threading.Thread(target=server.serve_forever)
        thread.start()
        try:
            yield origin(server.socket, "http")
        finally:
            server.shutdown()
            thread.join()
        assert not failures, failures
        assert observed == closed == expected, evidence


async def python_checks(http, sink, ws):
    direct = native.BootstrapProxyPolicy.DIRECT

    async def preflight(url=http + PATH, websocket=ws, proxy=direct):
        request = native.BootstrapRequest(http_url=url, websocket_url=websocket, proxy_policy=proxy)
        return await asyncio.to_thread(native.bootstrap_preflight, request)

    async def fails(error_type, *, fields=None, **request):
        try:
            await preflight(**request)
        except error_type as error:
            for name, value in (fields or {}).items():
                assert getattr(error, name) == value
            assert all(value not in str(error) + repr(error) for value in (MARKER, "127.0.0.1", PATH))
            assert error.__cause__ is None
        else:
            raise AssertionError(f"expected {error_type.__name__}")

    slots, tls = native.BootstrapEventSlots, native.BootstrapTlsStatus
    for event_slots, tls_status in (
        (slots.REPORTED_AVAILABLE, tls.REPORTED_AVAILABLE),
        (slots.REPORTED_UNAVAILABLE, tls.REPORTED_CERTIFICATE_EXPIRED),
        (slots.MISSING, tls.UNKNOWN),
        (slots.UNKNOWN, tls.MISSING),
    ):
        result = await preflight()
        assert (result.event_slots, result.tls_status) == (event_slots, tls_status)
        assert MARKER not in repr(result)

    error = native.BootstrapError
    await fails(error.HttpStatus, fields={"status": 503})
    await fails(error.CommandStatus, fields={"status": 403})
    await fails(error.Payload, fields={"issue": native.BootstrapPayloadIssue.INVALID_ENVELOPE})
    await fails(error.Transport, fields={"kind": native.BootstrapTransportFailure.TIMEOUT})
    await fails(error.WrongRoute, url=sink + "/" + MARKER)
    await fails(error.UnsupportedProxyPolicy, url=sink + PATH, proxy=native.BootstrapProxyPolicy.SYSTEM)
    await fails(error.InvalidEndpoint, url=sink + PATH, websocket=MARKER, fields={
        "endpoint": native.BootstrapEndpointKind.WEB_SOCKET,
        "issue": native.BootstrapEndpointIssue.SCHEME,
    })


def verify_wheel_dll(path, sha256):
    resolved = path.resolve(strict=True)
    if not resolved.is_file():
        raise RuntimeError("Installed wheel DLL is not a regular file")
    with resolved.open("rb") as dll:
        if hashlib.file_digest(dll, "sha256").hexdigest() != sha256:
            raise RuntimeError("Installed wheel DLL does not match the pinned SHA-256")
    return resolved


def main():
    parser = argparse.ArgumentParser()
    for name in ("java", "api-jar", "classes", "kotlin-jar", "dll-sha256"):
        parser.add_argument("--" + name, required=True)
    parser.add_argument("--wheel-dll", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    installed_module = Path(native.__file__).resolve(strict=True)
    wheel_dll = verify_wheel_dll(args.wheel_dll, args.dll_sha256)
    if (not installed_module.is_relative_to(Path(sys.prefix).resolve(strict=True))
            or wheel_dll != installed_module.with_name("lox_archive.dll")):
        raise RuntimeError("Pinned DLL is not next to the installed venv Python binding")

    def recheck_pin():
        if verify_wheel_dll(args.wheel_dll, args.dll_sha256) != wheel_dll:
            raise RuntimeError("Installed wheel DLL resolved path changed")

    java_env = {key: value for key, value in os.environ.items() if key.upper() not in JAVA_OPTION_ENV}
    evidence = {"requests": [], "released_sockets": [], "commands": []}
    expected = [f"{head}/{case}" for head in ("python", "java", "kotlin")
                for case in (CASES + ("body-timeout",) if head == "python" else CASES)]
    with ExitStack() as stack:
        http_sink, ws_sink = [stack.enter_context(socket.socket()) for _ in range(2)]
        for sock in (http_sink, ws_sink):
            sock.bind(("127.0.0.1", 0))
            sock.listen()
        http = stack.enter_context(fixture(expected, evidence))
        sink, ws = origin(http_sink, "http"), origin(ws_sink, "ws") + "/unused"
        asyncio.run(python_checks(http, sink, ws))
        for head, entry, classpath in (
            ("java", "BootstrapPreflightSmoke", [args.api_jar, args.classes]),
            ("kotlin", "BootstrapPreflightSmokeKt", [args.api_jar, args.kotlin_jar]),
        ):
            command = [args.java, "--enable-native-access=ALL-UNNAMED", f"-D{JAVA_LIBRARY_OVERRIDE}={wheel_dll}",
                       "-cp", os.pathsep.join(classpath), entry, http, sink, ws]
            recheck_pin()
            result = subprocess.run(command, env=java_env, capture_output=True, timeout=60)
            (args.output / f"bootstrap-preflight-{head}.log").write_bytes(result.stdout + result.stderr)
            evidence["commands"].append({"command": command, "exit_code": result.returncode})
            result.check_returncode()
        assert not select.select([http_sink, ws_sink], [], [], 0)[0], "refused HTTP or unused WS route contacted"
    recheck_pin()
    evidence["jvm_dll_pin"] = {
        "loader_override_property": JAVA_LIBRARY_OVERRIDE,
        "resolved_installed_wheel_dll": str(wheel_dll),
        "sha256": args.dll_sha256,
        "verified_before_each_jvm_launch_and_before_proof": True,
        "excluded_launcher_option_environment_variables": JAVA_OPTION_ENV,
        "limit": "Explicit loader override and on-disk hashes; no added library search path, "
                 "independent mapped-module measurement, or protection against concurrent file replacement.",
    }
    evidence["refused_http_and_ws_sink_connections"] = 0
    (args.output / "BOOTSTRAP-PREFLIGHT-PROOF.json").write_text(json.dumps(evidence, indent=2) + "\n", encoding="utf-8")
    print("Installed Python and shared Java/Kotlin preflight: typed data/errors, body deadline, socket release PASS")


if __name__ == "__main__":
    main()
