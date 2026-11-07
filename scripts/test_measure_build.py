import contextlib
import importlib.util
import io
import os
from pathlib import Path
from types import SimpleNamespace
import unittest
from unittest import mock


spec = importlib.util.spec_from_file_location(
    "measure_build", Path(__file__).with_name("measure-build.py")
)
measure = importlib.util.module_from_spec(spec)
spec.loader.exec_module(measure)


class ToolchainMetadataTests(unittest.TestCase):
    def setUp(self):
        self.cwd = Path.cwd() / ".reverse-engineering" / "workload-source"
        self.env = {"PATH": "workload-path", "RUSTUP_TOOLCHAIN": "workload-toolchain"}
        self.cargo = str(Path("proxies") / "cargo.exe")

    def probe(self, command, *, cwd=None, env=None, text=False):
        version = (
            "workload-toolchain"
            if cwd == self.cwd and env is self.env
            else "caller-toolchain"
        )
        if command[:2] == ["rustup", "run"]:
            version = command[2]
            command = command[3:]
        elif len(command) > 2 and command[1].startswith("+"):
            version = command[1][1:]
        if command[0] == self.env.get("RUSTC"):
            version = "custom-compiler"
        return f"{command[0]} {version}\n"

    def test_caller_and_workload_contexts_are_distinct(self):
        with mock.patch.dict(os.environ, {"RUSTUP_TOOLCHAIN": "caller-toolchain"}):
            with mock.patch.object(measure.subprocess, "check_output", side_effect=self.probe) as probe:
                result = measure.toolchain_metadata(
                    ["cargo", "check"], self.cargo, cwd=self.cwd, env=self.env
                )
        self.assertIn("workload-toolchain", result["rustc"])
        self.assertIn("workload-toolchain", result["cargo"])
        for call in probe.call_args_list:
            self.assertEqual(call.kwargs["cwd"], self.cwd)
            self.assertIs(call.kwargs["env"], self.env)

    def test_explicit_selector_overrides_context_for_both_probes(self):
        with mock.patch.object(measure.subprocess, "check_output", side_effect=self.probe) as probe:
            result = measure.toolchain_metadata(
                ["cargo", "+1.91.0", "check"], self.cargo, cwd=self.cwd, env=self.env
            )
        self.assertIn("1.91.0", result["rustc"])
        self.assertIn("1.91.0", result["cargo"])
        self.assertEqual(result["toolchain_selector"], "+1.91.0")
        self.assertEqual(
            probe.call_args_list[0].args[0],
            ["rustup", "run", "1.91.0", "rustc", "-Vv"],
        )
        self.assertEqual(probe.call_args_list[1].args[0], [self.cargo, "+1.91.0", "-V"])
        self.assertEqual(self.env["RUSTUP_TOOLCHAIN"], "workload-toolchain")
        for call in probe.call_args_list:
            self.assertEqual(call.kwargs["cwd"], self.cwd)
            self.assertIs(call.kwargs["env"], self.env)

    def test_selector_inside_opaque_wrapper_is_not_guessed(self):
        with mock.patch.object(measure.subprocess, "check_output", side_effect=self.probe) as probe:
            result = measure.toolchain_metadata(
                ["wrapper", "cargo", "+nightly", "check"], "wrapper",
                cwd=self.cwd, env=self.env,
            )
        self.assertIsNone(result["toolchain_selector"])
        self.assertEqual(result["toolchain_metadata_scope"], "workload-cwd/environment-context")
        self.assertIn("not introspected", result["toolchain_metadata_note"])
        self.assertEqual(probe.call_args_list[0].args[0], ["rustc", "-Vv"])
        self.assertEqual(probe.call_args_list[1].args[0], ["cargo", "-V"])

    def test_explicit_compiler_override_is_not_given_rustup_selector(self):
        self.env["RUSTC"] = str(Path("custom tools") / "compiler.exe")
        with mock.patch.object(measure.subprocess, "check_output", side_effect=self.probe) as probe:
            result = measure.toolchain_metadata(
                ["cargo", "+nightly", "check"], self.cargo, cwd=self.cwd, env=self.env
            )
        self.assertIn("custom-compiler", result["rustc"])
        self.assertIn("nightly", result["cargo"])
        self.assertEqual(
            probe.call_args_list[0].args[0],
            ["rustup", "run", "nightly", self.env["RUSTC"], "-Vv"],
        )

    def test_main_uses_same_probe_and_workload_environment(self):
        root = Path.cwd() / ".reverse-engineering"
        args = SimpleNamespace(
            label="metadata-test", cwd=self.cwd, target_dir=root / "probe-target",
            output=root / "probe-results.jsonl", jobs=3, sample_seconds=0.1,
            command=["--", "cargo", "+nightly", "check"],
        )
        process = mock.Mock(returncode=0)
        process.poll.return_value = 0

        def git_output(command, **kwargs):
            if command[1:3] == ["diff", "--binary"]:
                return b""
            if "--git-common-dir" in command:
                return str(Path.cwd() / ".git")
            return ""

        with contextlib.ExitStack() as stack:
            stack.enter_context(mock.patch.object(
                measure.argparse.ArgumentParser, "parse_args", return_value=args
            ))
            stack.enter_context(mock.patch.object(Path, "mkdir"))
            stack.enter_context(mock.patch.object(Path, "exists", return_value=False))
            stack.enter_context(mock.patch.object(Path, "iterdir", return_value=iter(())))
            stack.enter_context(mock.patch.object(Path, "write_text"))
            stack.enter_context(mock.patch.object(Path, "write_bytes"))
            stack.enter_context(mock.patch.object(Path, "open", mock.mock_open()))
            stack.enter_context(mock.patch.object(measure.shutil, "which", return_value=self.cargo))
            stack.enter_context(mock.patch.object(
                measure.subprocess, "check_output", side_effect=git_output
            ))
            stack.enter_context(mock.patch.object(
                measure, "windows_family_sampler",
                return_value=(lambda _: None, lambda: {}),
            ))
            probes = stack.enter_context(mock.patch.object(
                measure, "toolchain_metadata", return_value={"rustc": "workload", "cargo": "workload"}
            ))
            workload = stack.enter_context(mock.patch.object(
                measure.subprocess, "Popen", return_value=process
            ))
            stack.enter_context(contextlib.redirect_stdout(io.StringIO()))
            self.assertEqual(measure.main(), 0)
        self.assertEqual(probes.call_args.kwargs["cwd"], workload.call_args.kwargs["cwd"])
        self.assertIs(probes.call_args.kwargs["env"], workload.call_args.kwargs["env"])
        self.assertEqual(probes.call_args.kwargs["env"]["CARGO_BUILD_JOBS"], "3")
        self.assertEqual(
            workload.call_args.args[0], [self.cargo, "+nightly", "check"]
        )


if __name__ == "__main__":
    unittest.main()
