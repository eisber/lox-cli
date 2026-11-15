"""Loopback-only clock fixture with socket-EOF and request-order evidence."""

import base64
import json
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import threading
import time

USER = "clock-user"
PASSWORD = "clock-secret"
AUTH = "Basic " + base64.b64encode(f"{USER}:{PASSWORD}".encode()).decode()


class ClockFixture:
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
                case = "/".join(parts[:-3])
                mode = parts[-4] if len(parts) >= 4 else ""
                endpoint = parts[-1]
                with fixture.lock:
                    fixture.requests.append((case, endpoint))
                if self.headers.get("Authorization") != AUTH:
                    self.reply(401, b"")
                    fixture.failures.append("Basic authorization mismatch")
                    return
                blocked = ((mode in ("blocked-date", "close-date") and endpoint == "date")
                           or (mode in ("blocked-time", "close-time") and endpoint == "time"))
                if blocked:
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
                        fixture.failures.append("Blocked socket not cancelled before fixture deadline")
                    self.close_connection = True
                    return
                if mode == "retry":
                    self.reply(503, b"retry")
                    fixture.mark(case, "retry-response")
                    return
                if mode == "callback" and endpoint == "date":
                    fixture.mark(case, "entered")
                    release = fixture.signals / (case.replace("/", "__") + "__release")
                    deadline = time.monotonic() + 6
                    while not release.exists() and time.monotonic() < deadline:
                        time.sleep(0.01)
                    if not release.exists():
                        fixture.failures.append("Callback test response gate not released")
                        return
                if mode == "http" and endpoint == "date":
                    self.reply(401, b"private-backend-body")
                    return
                if mode == "json" and endpoint == "date":
                    self.reply(200, b'{"LL":')
                    return
                if mode == "utf8" and endpoint == "date":
                    self.reply(200, b'{"LL":{"Code":200,"value":"\xff"}}')
                    return
                value = "2026-11-15" if endpoint == "date" else "09:10:11"
                ll = {"Code": 200, "value": value}
                if mode == "number":
                    ll["value"] = 42 if endpoint == "date" else 12.5
                if mode == "missing" and endpoint == "date":
                    del ll["value"]
                if mode == "status" and endpoint == "date":
                    ll["Code"] = 403
                self.reply(200, json.dumps({"LL": ll}).encode())

            def reply(self, status, body):
                self.send_response(status)
                self.send_header("Content-Length", str(len(body)))
                self.send_header("Connection", "close")
                self.end_headers()
                self.wfile.write(body)
                self.wfile.flush()
                self.close_connection = True

        self.server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        self.server.daemon_threads = True
        self.thread = threading.Thread(target=self.server.serve_forever, name="clock-fixture")

    @property
    def host(self):
        return f"http://127.0.0.1:{self.server.server_port}"

    def mark(self, case, event):
        (self.signals / (case.replace("/", "__") + "__" + event)).write_text("observed\n")

    def __enter__(self):
        self.thread.start()
        return self

    def __exit__(self, *_):
        self.server.shutdown()
        self.server.server_close()
        self.thread.join()

    def verify(self):
        if self.failures:
            raise AssertionError(self.failures)
        groups = {}
        for case, endpoint in self.requests:
            groups.setdefault(case, []).append(endpoint)
        for case, endpoints in groups.items():
            mode = case.split("/")[-1]
            expected = (["date"] if mode in ("blocked-date", "close-date", "json", "utf8", "http", "retry")
                        else ["date", "time"])
            if mode == "normal":
                expected *= 2
            if endpoints != expected:
                raise AssertionError((case, endpoints, expected))
        return groups
