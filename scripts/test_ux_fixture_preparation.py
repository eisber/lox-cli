"""Fixture preparation contracts only; no SDK, SSH or native execution."""

import base64
import json
from pathlib import Path
import random
import shutil
import unittest
import uuid
import zlib

import ux_property_executor as EX


class PreparationSDK:
    def __init__(self, raw, destination):
        self.raw, self.destination = raw, destination
        self.calls = []
        self.encoded = ""
        self.failure = None
        self.bad_status = False
        self.corrupt = False

    def request(self, method, params, timeout):
        assert method == "tools/call" and timeout == 45
        self.calls.append(params)
        name, arguments = params["name"], params["arguments"]
        if self.failure == len(self.calls):
            raise TimeoutError("Inert chunk uncertainty")
        if name == "FileSystem":
            if arguments["append"]:
                self.encoded += arguments["content"]
            else:
                self.encoded = arguments["content"]
            result = ("Appended to " if arguments["append"] else "Written to ")
            result += arguments["path"] + " (4,096 bytes)"
        elif self.bad_status:
            result = ("Response: Command execution failed: FileNotFoundError: "
                      "[WinError 206] The filename or extension is too long\nStatus Code: 1")
        else:
            if "CreateDirectory" in arguments["command"]:
                from pathlib import PureWindowsPath
                value = {"created": str(PureWindowsPath(self.destination).parent)}
            else:
                if "FromBase64String" in arguments["command"]:
                    assert base64.b64decode(self.encoded, validate=True) == self.raw
                value = {"path": self.destination, **EX.byte_layout(self.raw)}
                if self.corrupt:
                    value["sha256"] = "0" * 64
            result = "Response: " + json.dumps(value) + "\r\n\nStatus Code: 0"
        return {"isError": False, "structuredContent": {"result": result}}


class FixturePreparationTests(unittest.TestCase):
    def setUp(self):
        self.root = (Path(__file__).resolve().parents[1] / ".reverse-engineering" /
                     "preparation-tests" / uuid.uuid4().hex)
        self.root.mkdir(parents=True)
        self.fixture = self.root / "source.Loxone"
        self.destination = r"C:\Owned\FreshPreparation\Input.Loxone"
        self.raw = b"\xef\xbb\xbf<R>\r\n mixed\n lone\r</R>"
        self.sdk = PreparationSDK(self.raw, self.destination)

    def tearDown(self):
        shutil.rmtree(self.root)

    def prepare(self, **kwargs):
        self.fixture.write_bytes(self.raw)
        self.sdk.raw = self.raw
        return EX.prepare_fixture(self.sdk, self.fixture, EX.sha(self.raw),
                                  self.destination, self.root, "one-case", **kwargs)

    def test_large_incompressible_payload_never_enters_command_line(self):
        self.raw = b"\xef\xbb\xbf\r\n" + random.Random(37).randbytes(300000)
        self.assertGreater(len(zlib.compress(self.raw)), len(self.raw) * .99)
        result = self.prepare()
        self.assertEqual(result["outcome"], "prepared")
        self.assertEqual(result["remote"], {"path": self.destination, **EX.byte_layout(self.raw)})
        chunks = [call["arguments"] for call in self.sdk.calls if call["name"] == "FileSystem"]
        self.assertGreater(len(chunks), 50)
        self.assertTrue(all(len(chunk["content"]) <= 4096 for chunk in chunks))
        self.assertFalse(chunks[0]["append"])
        self.assertTrue(all(chunk["append"] for chunk in chunks[1:]))
        for call in self.sdk.calls:
            if call["name"] == "PowerShell":
                command = call["arguments"]["command"]
                budget = EX.powershell_budget(command)
                self.assertLess(budget["encoded_command_characters"], 30000)
                self.assertNotIn(chunks[0]["content"], command)
        directory = self.root / "preparation-one-case"
        self.assertEqual(len(list(directory.glob("*-intent.json"))), len(self.sdk.calls))
        self.assertEqual(len(list(directory.glob("*-ack.json"))), len(self.sdk.calls))
        self.assertEqual((directory / "input.fixture").read_bytes(), self.raw)
        self.assertEqual(result["native_completion_credit"], 0)

    def test_status_one_even_with_iserror_false_is_not_preparation_success(self):
        self.sdk.bad_status = True
        result = self.prepare()
        self.assertEqual(result["outcome"], "blocked")
        self.assertEqual(result["code"], "PREPARATION_POWERSHELL_STATUS_NOT_ZERO")
        self.assertEqual(len(self.sdk.calls), 1)
        self.assertFalse(any(call["name"] == "FileSystem" for call in self.sdk.calls))
        self.assertIn("Status Code: 1",
                      (self.root / "preparation-one-case" / "0000-ack.json").read_text())

    def test_unknown_chunk_is_retained_and_never_automatically_replayed(self):
        self.raw = random.Random(11).randbytes(16000)
        self.sdk.failure = 3
        result = self.prepare()
        self.assertEqual(result["outcome"], "UNKNOWN")
        self.assertEqual(result["pending_step"], 2)
        count = len(self.sdk.calls)
        directory = self.root / "preparation-one-case"
        self.assertTrue((directory / "0002-intent.json").exists())
        self.assertFalse((directory / "0002-ack.json").exists())
        with self.assertRaises(FileExistsError):
            self.prepare()
        self.assertEqual(len(self.sdk.calls), count)
        self.assertEqual(result["native_inputs"], 0)

    def test_mismatching_final_raw_hash_is_not_prepared(self):
        self.sdk.corrupt = True
        result = self.prepare()
        self.assertEqual(result["outcome"], "blocked")
        self.assertEqual(result["code"], "PREPARATION_REMOTE_RAW_BYTES_MISMATCH")

    def test_existing_strict_transfer_receives_file_paths_not_embedded_payload(self):
        calls = []
        class ExistingTransfer:
            def push(inner, local, remote):
                calls.append((local, remote))
                self.assertEqual(Path(local).read_bytes(), self.raw)
        result = self.prepare(transfer=ExistingTransfer())
        self.assertEqual(result["outcome"], "prepared")
        self.assertEqual(len(calls), 1)
        self.assertEqual(calls[0][1], self.destination)
        self.assertEqual([call["name"] for call in self.sdk.calls], ["PowerShell", "PowerShell"])
        commands = [call["arguments"]["command"] for call in self.sdk.calls]
        self.assertFalse(any("FromBase64String" in command for command in commands))

    def test_windows_encoded_command_budget_is_checked_before_sdk_dispatch(self):
        with self.assertRaises(EX.Stop) as caught:
            EX.powershell_budget("x" * 46249)
        self.assertEqual(caught.exception.code, "WINDOWS_ENCODED_COMMAND_BUDGET")
        self.assertGreater(caught.exception.details["encoded_command_characters"], 30000)
        with self.assertRaises(EX.Stop):
            EX.powershell_budget("\U0001f600" * 6000)
        executor = object.__new__(EX.Executor)
        executor.stage = "before"
        executor.client = self.sdk
        with self.assertRaises(EX.Stop):
            executor.tool("PowerShell", {"command": "x" * 46249})
        self.assertEqual(self.sdk.calls, [])


if __name__ == "__main__":
    unittest.main()
