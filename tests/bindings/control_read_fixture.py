"""Loopback-only control GET fixture with explicit request/EOF barriers."""

import base64
import json
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import socket
import threading
import time

USER = "control-user"
PASSWORD = "control-secret"
IDENTIFIER = "1fbc668c-005c-7471-ffffed57184a04d2"
VALUE = "control-owned-value-" + "x" * 65536
AUTH = "Basic " + base64.b64encode(f"{USER}:{PASSWORD}".encode()).decode()


class ControlFixture:
    def __init__(self, signals):
        self.signals = Path(signals)
        self.signals.mkdir(parents=True)
        self.requests = []
        self.failures = []
        self.lock = threading.Lock()
        fixture = self

        class Handler(BaseHTTPRequestHandler):
            protocol_version = "HTTP/1.1"

            def log_message(self, *_):
                pass

            def do_GET(self):
                parts = self.path.strip("/").split("/")
                case = "/".join(parts[:-4])
                mode = parts[-5]
                with fixture.lock:
                    fixture.requests.append((case, parts[-1]))
                if parts[-4:] != ["jdev", "sps", "io", IDENTIFIER]:
                    fixture.failures.append("Unexpected control request path")
                if self.headers.get("Authorization") != AUTH:
                    fixture.failures.append("Basic authorization mismatch")
                    self.reply(401, b"")
                    return
                if mode in ("blocked", "close", "timeout"):
                    self.send_response(200)
                    self.send_header("Content-Length", "65536")
                    self.send_header("Connection", "close")
                    self.end_headers()
                    self.wfile.write(b'{"LL":')
                    self.wfile.flush()
                    fixture.mark(case, "entered")
                    self.connection.settimeout(6)
                    try:
                        if self.connection.recv(1) == b"":
                            fixture.mark(case, "eof")
                        else:
                            fixture.failures.append("Unexpected blocked-socket data")
                    except (ConnectionResetError, ConnectionAbortedError):
                        fixture.mark(case, "eof")
                    except TimeoutError:
                        fixture.failures.append("Blocked request did not reach EOF")
                    self.close_connection = True
                    return
                if mode == "callback":
                    fixture.mark(case, "entered")
                    release = fixture.signal(case, "release")
                    deadline = time.monotonic() + 6
                    while not release.exists() and time.monotonic() < deadline:
                        time.sleep(0.01)
                    if not release.exists():
                        fixture.failures.append("Response gate not released")
                        return
                if mode == "retry":
                    self.reply(503, b"private-backend-body")
                    fixture.mark(case, "retry-response")
                    return
                if mode == "http":
                    self.reply(401, b"control-secret private-backend-body")
                    return
                if mode in ("json", "utf8", "body"):
                    payload = (b'{"LL":{"Code":200,"value":"\xff"}}' if mode == "utf8"
                               else b'{"LL":')
                    self.reply(200, payload, 65536 if mode == "body" else None)
                    return
                ll = {"Code": 200, "value": VALUE}
                if mode == "number":
                    ll["value"] = -0.0
                elif mode == "empty":
                    ll["value"] = ""
                elif mode == "lower":
                    ll = {"code": 200, "value": "lowercase"}
                elif mode == "missing":
                    del ll["value"]
                elif mode == "missing-code":
                    del ll["Code"]
                elif mode == "unsupported-code":
                    ll["Code"] = True
                elif mode == "status":
                    ll["Code"] = 403
                elif mode == "conflict":
                    ll["Code"] = 403
                    ll["code"] = "200"
                elif mode == "nontext":
                    ll["value"] = {"private": "control-secret"}
                payload = [] if mode == "envelope" else {"LL": ll}
                self.reply(200, json.dumps(payload).encode())

            def reply(self, status, body, declared_length=None):
                self.send_response(status)
                self.send_header("Content-Length", str(len(body) if declared_length is None else declared_length))
                self.send_header("Connection", "close")
                self.end_headers()
                self.wfile.write(body)
                self.wfile.flush()
                self.close_connection = True

        self.server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        self.server.daemon_threads = True
        self.thread = threading.Thread(target=self.server.serve_forever, name="control-read-fixture")
        # Bound, deliberately not listening: deterministic local connection refusal,
        # with no port handoff to another process.
        self.refused = socket.socket()
        self.refused.bind(("127.0.0.1", 0))

    @property
    def host(self):
        return f"http://127.0.0.1:{self.server.server_port}"

    @property
    def refused_host(self):
        return f"http://127.0.0.1:{self.refused.getsockname()[1]}"

    def signal(self, case, event):
        return self.signals / (case.replace("/", "__") + "__" + event)

    def mark(self, case, event):
        self.signal(case, event).write_text("observed\n")

    def __enter__(self):
        self.thread.start()
        return self

    def __exit__(self, *_):
        self.server.shutdown()
        self.server.server_close()
        self.thread.join()
        self.refused.close()

    def verify(self):
        if self.failures:
            raise AssertionError(self.failures)
        groups = {}
        for case, identifier in self.requests:
            assert identifier == IDENTIFIER
            assert not case.endswith("unused"), ("zero-I/O contract", case)
            groups[case] = groups.get(case, 0) + 1
        for case, count in groups.items():
            if case.split("/")[-1] in ("blocked", "close", "timeout", "retry"):
                assert count == 1, (case, count)
        return groups
