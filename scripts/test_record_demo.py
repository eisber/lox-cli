import importlib.util
import json
import os
from pathlib import Path
import shutil
import unittest
from unittest.mock import patch
from urllib.error import HTTPError
from urllib.request import build_opener, ProxyHandler


SPEC = importlib.util.spec_from_file_location(
    "record_demo", Path(__file__).with_name("record-demo.py"))
demo = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(demo)


class DemoTests(unittest.TestCase):
    def test_environment_discards_inherited_connection_and_proxies(self):
        sandbox = demo.ROOT / "target" / "readme-demo-env-test"
        sandbox.mkdir(parents=True, exist_ok=False)
        try:
            with patch.dict(os.environ, {"LOX_HOST": "not-a-demo-endpoint",
                                        "LOX_PASS": "not-a-real-secret",
                                        "HTTPS_PROXY": "not-a-demo-proxy",
                                        "all_proxy": "not-a-demo-proxy"}):
                env = demo.isolated_environment(sandbox)
            self.assertNotIn("LOX_HOST", env)
            self.assertNotIn("LOX_PASS", env)
            self.assertNotIn("HTTPS_PROXY", env)
            self.assertNotIn("all_proxy", env)
            self.assertEqual(env["LOX_CONFIG"], str(sandbox / "connection.yaml"))
            self.assertEqual(env["LOX_TELEMETRY"], "0")
            self.assertEqual(env["USERPROFILE"], str(sandbox / "home"))
        finally:
            shutil.rmtree(sandbox)

    def test_mock_mutates_only_allowlisted_synthetic_control(self):
        with demo.sandbox_server() as (sandbox, server, _env):
            self.assertEqual(server.server_address[0], "127.0.0.1")
            host = f"http://127.0.0.1:{server.server_port}"
            opener = build_opener(ProxyHandler({}))
            for command, expected in (("on", "1"), ("off", "0")):
                path = f"/jdev/sps/io/{demo.LIGHT}/{command}"
                with opener.open(host + path, timeout=5) as response:
                    self.assertEqual(json.load(response)["LL"]["value"], expected)
            with self.assertRaises(HTTPError) as error:
                opener.open(host + "/dev/sys/reboot", timeout=5)
            self.assertEqual(error.exception.code, 404)
            error.exception.close()
            self.assertEqual(server.unexpected, ["/dev/sys/reboot"])
            self.assertEqual(server.state, 0)
        self.assertFalse(sandbox.exists())

    def test_renderer_refuses_to_hide_clipped_output(self):
        try:
            from PIL import ImageFont
        except ImportError:
            self.skipTest("Optional GIF rendering needs Pillow; stdlib safety tests still run")
        try:
            ImageFont.truetype("DejaVuSansMono.ttf", 19)
        except OSError:
            self.skipTest("Optional GIF rendering needs the DejaVu Sans Mono font")
        transcript = {"scenes": [{
            "title": "TOO MUCH OUTPUT", "duration_ms": 30000,
            "commands": [{"argv": ["lox", "test"], "output": "line\n" * 100}],
        }]}
        output = demo.ROOT / "target" / "demo-should-not-exist.gif"
        with self.assertRaisesRegex(RuntimeError, "clipped"):
            demo.render(transcript, output)
        self.assertFalse(output.exists())

    def test_renderer_reports_actionable_missing_dependency(self):
        output = demo.ROOT / "target" / "demo-should-not-exist.gif"
        with patch.dict("sys.modules", {"PIL": None}):
            with self.assertRaisesRegex(
                    RuntimeError, "dedicated virtual environment.*python -m pip install Pillow"):
                demo.render({}, output)
        self.assertFalse(output.exists())


if __name__ == "__main__":
    unittest.main()
