#!/usr/bin/env python3
"""Capture real lox subprocess output in a synthetic sandbox, then render a GIF."""

import argparse
import contextlib
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.request import build_opener, ProxyHandler


ROOT = Path(__file__).resolve().parents[1]
LIGHT = "00000000-0000-0000-0000-000000000001"
FIXTURE = f"""<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267">
  <C Type="Place" V="175" U="demo-room" Title="Studio" WF="16384"/>
  <C Type="Category" V="175" U="demo-category" Title="Lighting" WF="16384"/>
  <C Type="Page" V="175" U="demo-page" Title="Studio" WF="16384">
    <C Type="Switch" V="175" U="{LIGHT}" Title="Demo light" WF="16384">
      <Co K="Q" U="demo-output"/>
      <IoData Pr="demo-room" Cr="demo-category"/>
    </C>
  </C>
</ControlList>
"""

SCENES = [
    ("CONNECT / synthetic server clock", 4000, [
        (["live", "time"], 0),
    ]),
    ("INSPECT / local synthetic config", 4000, [
        (["config", "controls", "demo.Loxone", "-o", "json"], 0),
    ]),
    ("SWITCH ON / mock memory only", 4000, [
        (["live", "set", "Demo light", "on", "--write"], 0),
        (["live", "get", "Demo light"], 0),
    ]),
    ("SWITCH OFF / mock memory only", 4000, [
        (["live", "set", "Demo light", "off", "--write"], 0),
        (["live", "get", "Demo light"], 0),
    ]),
    ("BUILD / apply a room template locally", 6000, [
        (["config", "template", "demo.Loxone", "hallway", "--room", "Studio"], 0),
    ]),
    ("DISCOVER / bundled block library", 4000, [
        (["blocks", "ls", "--category", "lighting", "--limit", "3"], 0),
    ]),
    ("DEPLOYMENT GUARD / deliberately no --force", 4000, [
        (["config", "push", "demo.Loxone", "--non-interactive"], 1),
    ]),
]


class MockMiniserver(ThreadingHTTPServer):
    """Only these endpoints exist; mutations affect an in-memory integer."""

    def __init__(self):
        super().__init__(("127.0.0.1", 0), MockHandler)
        self.state = 0
        self.requests = []
        self.unexpected = []


class MockHandler(BaseHTTPRequestHandler):
    def log_message(self, *_args):
        pass

    def do_GET(self):
        self.server.requests.append(self.path)
        prefix = f"/jdev/sps/io/{LIGHT}"
        values = {"/health": "synthetic", "/jdev/sys/date": "2026-01-01",
                  "/jdev/sys/time": "12:00:00"}
        if self.path in values:
            value = values[self.path]
        elif self.path in (prefix, prefix + "/on", prefix + "/off"):
            if self.path.endswith("/on"):
                self.server.state = 1
            elif self.path.endswith("/off"):
                self.server.state = 0
            value = str(self.server.state)
        else:
            self.server.unexpected.append(self.path)
            self.send_error(404, "Not a synthetic demo endpoint")
            return
        body = json.dumps({"LL": {"value": value, "Code": "200"}}).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)


def isolated_environment(sandbox):
    env = {key: value for key, value in os.environ.items()
           if not key.upper().startswith("LOX_")
           and not key.upper().endswith("_PROXY")}
    home = sandbox / "home"
    (home / ".lox").mkdir(parents=True)
    (home / ".lox" / ".telemetry_noticed").write_text("shown\n", encoding="utf-8")
    env.update({
        "HOME": str(home), "USERPROFILE": str(home), "LOX_HOME": str(home),
        "LOX_CONFIG": str(sandbox / "connection.yaml"),
        "LOX_TELEMETRY": "0", "DO_NOT_TRACK": "1", "NO_COLOR": "1",
        "NO_PROXY": "*",
    })
    return env


@contextlib.contextmanager
def sandbox_server():
    sandbox = ROOT / "target" / "readme-demo"
    sandbox.mkdir(parents=True, exist_ok=False)
    server = None
    thread = None
    try:
        server = MockMiniserver()
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        host = f"http://127.0.0.1:{server.server_port}"
        (sandbox / "connection.yaml").write_text(
            f'host: "{host}"\nuser: ""\npass: ""\ntelemetry: false\n'
            f'aliases:\n  "Demo light": "{LIGHT}"\n', encoding="utf-8")
        (sandbox / "demo.Loxone").write_text(FIXTURE, encoding="utf-8")
        opener = build_opener(ProxyHandler({}))
        with opener.open(host + "/health", timeout=5) as response:
            if json.load(response)["LL"]["value"] != "synthetic":
                raise RuntimeError("Synthetic HTTP server failed readiness check")
        yield sandbox, server, isolated_environment(sandbox)
    finally:
        if server:
            server.shutdown()
            server.server_close()
        if thread:
            thread.join(timeout=5)
        shutil.rmtree(sandbox)


def capture(binary):
    scenes = []
    with sandbox_server() as (sandbox, server, env):
        for title, duration, commands in SCENES:
            records = []
            for args, expected_exit in commands:
                # Never add --force, upload, FTP, reload or reboot to this demo.
                result = subprocess.run(
                    [str(binary), *args], cwd=sandbox, env=env,
                    stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                    text=True, encoding="utf-8", errors="strict", timeout=30,
                    check=False,
                )
                if result.returncode != expected_exit:
                    raise RuntimeError(f"lox {shlex.join(args)}: {result.stdout}")
                records.append({"argv": ["lox", *args], "exit_code": result.returncode,
                                "output": result.stdout})
            scenes.append({"title": title, "duration_ms": duration, "commands": records})
        xml = (sandbox / "demo.Loxone").read_text(encoding="utf-8-sig")
        if 'Title="Studio Licht"' not in xml or 'Type="LightController2"' not in xml:
            raise RuntimeError("Room template was not written to the synthetic config")
        if (scenes[2]["commands"][1]["output"].strip() != "1"
                or scenes[3]["commands"][1]["output"].strip() != "0"):
            raise RuntimeError("Mock light on/off readback failed")
        if "--force" not in scenes[-1]["commands"][0]["output"]:
            raise RuntimeError("Push did not report the expected deployment guard")
        expected = ["/health", "/jdev/sys/date", "/jdev/sys/time",
                    f"/jdev/sps/io/{LIGHT}/on", f"/jdev/sps/io/{LIGHT}",
                    f"/jdev/sps/io/{LIGHT}/off", f"/jdev/sps/io/{LIGHT}"]
        if server.requests != expected or server.unexpected:
            raise RuntimeError(f"Unexpected mock HTTP request sequence: {server.requests}")
    return {
        "provenance": "Actual lox subprocess output; synthetic loopback HTTP mock and local XML.",
        "playback": "30-second editorial timing; typed commands and line wrapping are rendered.",
        "deployment": "Push intentionally exits 1 without --force. No upload/reload/FTP.",
        "requests": expected,
        "scenes": scenes,
    }


def render(transcript, output, font_path=None):
    try:
        from PIL import Image, ImageDraw, ImageFont
    except ImportError as error:
        raise RuntimeError(
            "GIF rendering requires Pillow. In a dedicated virtual environment, "
            "run `python -m pip install Pillow`, then rerun this script."
        ) from error

    try:
        font = ImageFont.truetype(font_path or "DejaVuSansMono.ttf", 19)
        small = ImageFont.truetype(font_path or "DejaVuSansMono.ttf", 15)
    except OSError as error:
        raise RuntimeError(
            "Cannot load the monospace font. Pass --font with a local TrueType font path."
        ) from error
    width, height = 1140, 620
    frames, durations = [], []
    background, foreground = "#101820", "#e4e9ef"
    cell = font.getlength("M")
    columns = int((width - 72) / cell)

    for index, scene in enumerate(transcript["scenes"]):
        # Coarse typing is playback styling; outputs below are unedited captures.
        first = shlex.join(scene["commands"][0]["argv"])
        variants = [(first[:len(first) // 2], False, 180), (first, False, 220),
                    (None, True, scene["duration_ms"] - 400)]
        for typed, show_output, duration in variants:
            image = Image.new("RGB", (width, height), background)
            draw = ImageDraw.Draw(image)
            draw.rounded_rectangle((14, 14, width - 15, height - 15), radius=16,
                                   outline="#334350", width=2)
            draw.text((34, 30), "lox / CONFIG AS CODE", font=small, fill="#8fe0bc")
            draw.text((width - 326, 30), "SIMULATED / NO HARDWARE", font=small, fill="#f3cd81")
            draw.line((34, 66, width - 34, 66), fill="#334350", width=1)
            draw.text((36, 87), f"{index + 1:02d} / 07  {scene['title']}",
                      font=small, fill="#85baf2")
            y = 132
            for command_index, record in enumerate(scene["commands"]):
                if command_index and not show_output:
                    break
                command = shlex.join(record["argv"]) if show_output else typed
                rows = [("$ " + command, "#8fe0bc")]
                if show_output:
                    rows.extend((line, foreground) for line in record["output"].splitlines())
                    rows.append(("", foreground))
                for line, color in rows:
                    # Preserve every character, including CLI table spacing.
                    chunks = [line[n:n + columns] for n in range(0, len(line), columns)] or [""]
                    for chunk in chunks:
                        if y > height - 96:
                            raise RuntimeError(f"Output would be clipped in scene {index + 1}")
                        draw.text((36, y), chunk, font=font, fill=color)
                        y += 25
            draw.text((36, height - 58), "Loopback mock + synthetic XML / real CLI output / no deployment",
                      font=small, fill="#9baebb")
            bar = int((width - 72) * (index + 1) / len(transcript["scenes"]))
            draw.rectangle((36, height - 28, 36 + bar, height - 25), fill="#8fe0bc")
            frames.append(image)
            durations.append(duration)
    output.parent.mkdir(parents=True, exist_ok=True)
    frames[0].save(output, save_all=True, append_images=frames[1:], duration=durations,
                   loop=0, optimize=True, disposal=2)
    with Image.open(output) as gif:
        total = 0
        for frame in range(gif.n_frames):
            gif.seek(frame)
            gif.load()
            total += gif.info["duration"]
        if gif.format != "GIF" or gif.n_frames != 21 or total != 30000:
            raise RuntimeError("Demo must decode as a 21-frame, 30-second GIF")
    if output.stat().st_size > 2_000_000:
        raise RuntimeError("Demo exceeds the 2 MB asset budget")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--lox", type=Path, required=True, help="Freshly built lox executable")
    parser.add_argument("--font", help="Monospace TrueType font; default DejaVuSansMono.ttf")
    args = parser.parse_args()
    binary = args.lox.resolve(strict=True)
    transcript = capture(binary)
    output = ROOT / "docs" / "assets" / "demo.gif"
    render(transcript, output, args.font)
    (output.parent / "demo-transcript.json").write_text(
        json.dumps(transcript, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"{output.relative_to(ROOT)}: 30.00s, 21 frames, {output.stat().st_size:,} bytes")
    print("Verified on/off readback, local template, push refusal and exact loopback request sequence.")


if __name__ == "__main__":
    main()
