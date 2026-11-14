"""Offline behavioral tests for the metadata-only upstream observer."""

import copy
import importlib.util
import io
import json
import os
import unittest
import urllib.error
import urllib.request
from contextlib import redirect_stdout
from email.message import Message
from pathlib import Path
from unittest.mock import MagicMock, patch
from urllib.response import addinfourl

SPEC = importlib.util.spec_from_file_location(
    "check_upstreams", Path(__file__).with_name("check-upstreams.py")
)
CHECKER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECKER)


def registry():
    return {
        "schema_version": 1,
        "repositories": [
            {
                "id": "example/client",
                "ref": "refs/heads/main",
                "reviewed_sha": "a" * 40,
                "latest_seen_sha": "a" * 40,
                "source_url": "https://not-requested.invalid/source",
            }
        ],
    }


def ref_response(sha="a" * 40):
    return {"ref": "refs/heads/main", "object": {"type": "commit", "sha": sha}}


def response(data):
    result = MagicMock()
    result.__enter__.return_value = result
    result.status = 200
    result.geturl.return_value = CHECKER.api_url("example/client", "refs/heads/main")
    result.read.return_value = json.dumps(data).encode()
    return result


class UpstreamCheckTests(unittest.TestCase):
    def run_check(self, data, catalog=None):
        with patch.object(CHECKER.urllib.request, "build_opener") as build:
            build.return_value.open.return_value = response(data)
            report = CHECKER.check(catalog or registry(), timeout=3)
            return report, build.return_value.open

    def test_unchanged_and_changed_ref_preserve_pins_and_input(self):
        for sha, expected in (("a" * 40, "unchanged"), ("b" * 40, "needs_review")):
            with self.subTest(expected=expected):
                catalog = registry()
                before = copy.deepcopy(catalog)
                report, opened = self.run_check(ref_response(sha), catalog)
                row = report["repositories"][0]
                self.assertEqual(report["status"], expected)
                self.assertEqual(row["reviewed_sha"], "a" * 40)
                self.assertEqual(row["latest_seen_sha"], sha)
                self.assertEqual(row["needs_review"], expected == "needs_review")
                self.assertTrue(row["compare_url"].endswith(f"{'a' * 40}...{sha}"))
                self.assertEqual(catalog, before)
                self.assertEqual(opened.call_count, 1)

    def test_already_observed_unreviewed_commit_still_needs_review(self):
        catalog = registry()
        catalog["repositories"][0]["latest_seen_sha"] = "b" * 40
        report, _ = self.run_check(ref_response("b" * 40), catalog)
        self.assertEqual(report["status"], "needs_review")

    def test_only_constructed_github_api_url_and_bounded_timeout_are_used(self):
        with patch.object(CHECKER.urllib.request, "build_opener") as build:
            build.return_value.open.return_value = response(ref_response())
            CHECKER.check(registry(), timeout=2, token="test-private-token")
            request = build.return_value.open.call_args.args[0]
            self.assertEqual(
                request.full_url,
                "https://api.github.com/repos/example/client/git/ref/heads/main",
            )
            self.assertEqual(request.get_method(), "GET")
            self.assertEqual(
                request.get_header("Authorization"), "Bearer test-private-token"
            )
            self.assertEqual(build.return_value.open.call_args.kwargs, {"timeout": 2})
            self.assertEqual(
                build.return_value.open.return_value.read.call_args.args,
                (CHECKER.MAX_RESPONSE_BYTES + 1,),
            )

    def test_invalid_catalog_rejected_before_any_request(self):
        cases = [
            ("id", "https://attacker.invalid/owner/repo"),
            ("id", "owner/repo?redirect=elsewhere"),
            ("id", "owner/../repo"),
            ("id", "owner/repo%2Fmore"),
            ("ref", "refs/heads/../../other"),
            ("ref", "refs/heads/main?redirect=elsewhere"),
            ("ref", "refs/heads/main#fragment"),
            ("ref", "refs/heads/main%2Fmore"),
            ("ref", "refs/heads/a//b"),
            ("ref", "refs/heads/a.lock"),
            ("ref", "refs/tags/v1"),
            ("reviewed_sha", "abc"),
            ("latest_seen_sha", "g" * 40),
        ]
        for field, value in cases:
            with self.subTest(field=field, value=value):
                catalog = registry()
                catalog["repositories"][0][field] = value
                with patch.object(CHECKER.urllib.request, "build_opener") as build:
                    with self.assertRaises(CHECKER.CheckError):
                        CHECKER.check(catalog)
                    build.assert_not_called()
        for timeout in (0, 31, float("nan")):
            with self.assertRaises(CHECKER.CheckError):
                CHECKER.check(registry(), timeout=timeout)
        self.assertEqual(
            CHECKER.api_url("example/client", "refs/heads/release/v1.2"),
            "https://api.github.com/repos/example/client/git/ref/heads/release/v1.2",
        )

    def test_unexpected_api_shapes_are_errors_not_unchanged(self):
        cases = [
            [],
            {},
            {"ref": "refs/heads/other", "object": {"type": "commit", "sha": "a" * 40}},
            {"ref": "refs/heads/main", "object": {"type": "tag", "sha": "a" * 40}},
            {"ref": "refs/heads/main", "object": {"type": "commit", "sha": "abc"}},
            {"ref": "refs/heads/main", "object": None},
        ]
        for data in cases:
            with self.subTest(data=data):
                report, _ = self.run_check(data)
                self.assertEqual(report["status"], "error")
                self.assertIsNone(report["repositories"][0]["latest_seen_sha"])
                self.assertIsNone(report["repositories"][0]["needs_review"])

    def test_network_and_http_errors_are_nonzero_and_do_not_leak_details(self):
        errors = [
            urllib.error.URLError("test-private-token"),
            TimeoutError("test-private-token"),
            urllib.error.HTTPError(
                "https://api.github.com/", 403, "test-private-token", {}, None
            ),
        ]
        for error in errors:
            with self.subTest(error=type(error).__name__):
                output = io.StringIO()
                with (
                    patch.object(
                        CHECKER.Path, "read_text", return_value=json.dumps(registry())
                    ),
                    patch.object(CHECKER.urllib.request, "build_opener") as build,
                    patch.dict(
                        os.environ, {"GH_TOKEN": "test-private-token"}, clear=True
                    ),
                    redirect_stdout(output),
                ):
                    build.return_value.open.side_effect = error
                    self.assertEqual(CHECKER.main([]), 1)
                text = output.getvalue()
                self.assertNotIn("test-private-token", text)
                report = json.loads(text)
                self.assertEqual(report["status"], "error")
                self.assertEqual(report["repositories"][0]["status"], "error")

    def test_invalid_json_oversize_or_redirected_responses_are_errors(self):
        for raw, status, url in [
            (b"not json", 200, CHECKER.api_url("example/client", "refs/heads/main")),
            (
                b" " * (CHECKER.MAX_RESPONSE_BYTES + 1),
                200,
                CHECKER.api_url("example/client", "refs/heads/main"),
            ),
            (b"{}", 200, "https://attacker.invalid"),
            (b"{}", 302, CHECKER.api_url("example/client", "refs/heads/main")),
        ]:
            with self.subTest(status=status, length=len(raw), url=url):
                with patch.object(CHECKER.urllib.request, "build_opener") as build:
                    reply = response({})
                    reply.read.return_value = raw
                    reply.status = status
                    reply.geturl.return_value = url
                    build.return_value.open.return_value = reply
                    self.assertEqual(CHECKER.check(registry())["status"], "error")

    def test_real_opener_refuses_redirect_without_second_request(self):
        headers = Message()
        headers["Location"] = "https://attacker.invalid/steal"
        url = CHECKER.api_url("example/client", "refs/heads/main")
        reply = addinfourl(io.BytesIO(b""), headers, url, 302)
        reply.msg = "Found"
        with patch.object(
            urllib.request.HTTPSHandler, "https_open", return_value=reply
        ) as opened:
            report = CHECKER.check(registry(), token="test-private-token")
            self.assertEqual(report["status"], "error")
            self.assertEqual(report["repositories"][0]["error"], "GitHub HTTP 302")
            self.assertEqual(opened.call_count, 1)

    def test_partial_failure_keeps_successful_observations_but_fails_report(self):
        catalog = registry()
        second = copy.deepcopy(catalog["repositories"][0])
        second["id"] = "example/another"
        catalog["repositories"].append(second)
        with patch.object(CHECKER.urllib.request, "build_opener") as build:
            build.return_value.open.side_effect = [
                response(ref_response("b" * 40)),
                TimeoutError(),
            ]
            report = CHECKER.check(catalog)
        self.assertEqual(report["status"], "error")
        self.assertEqual(report["repositories"][0]["status"], "needs_review")
        self.assertEqual(report["repositories"][1]["status"], "error")

    def test_invalid_registry_cli_reports_failure_without_network(self):
        for text in ("{", json.dumps({"schema_version": 1, "repositories": []})):
            output = io.StringIO()
            with (
                patch.object(CHECKER.Path, "read_text", return_value=text),
                patch.object(CHECKER.urllib.request, "build_opener") as build,
                redirect_stdout(output),
            ):
                self.assertEqual(CHECKER.main([]), 1)
                build.assert_not_called()
            self.assertEqual(json.loads(output.getvalue())["status"], "error")

    def test_maintained_registry_is_accepted_offline(self):
        catalog = json.loads(CHECKER.DEFAULT_REGISTRY.read_text(encoding="utf-8"))
        self.assertEqual(len(CHECKER.validate_registry(catalog)), 7)


if __name__ == "__main__":
    unittest.main()
