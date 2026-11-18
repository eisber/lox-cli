import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest import mock


SPEC = importlib.util.spec_from_file_location(
    "validate_toolkit", Path(__file__).with_name("validate-toolkit.py")
)
VALIDATE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(VALIDATE)


class ValidationReuseTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.folder = Path(self.temporary.name)
        self.root = self.folder / "source"
        self.root.mkdir()
        self.source = self.root / "source.rs"
        self.source.write_bytes(b"first source")
        self.results = self.folder / "results" / "checks.json"
        self.target = self.folder / "target"
        self.patch_root = mock.patch.object(VALIDATE, "ROOT", self.root)
        self.patch_root.start()
        self.addCleanup(self.patch_root.stop)

    def test_reuse_requires_retained_successful_commands_and_unchanged_artifacts(self):
        log = self.folder / "successful.log"
        log.write_bytes(b"actual successful command")
        record = {"key": "same", "status": "passed", "commands": [{"exit_code": 0}],
                  "artifacts": {str(log): VALIDATE.digest(log)}}
        self.assertTrue(VALIDATE.valid_result(record, "same", 1))
        self.assertFalse(VALIDATE.valid_result(record, "changed", 1))
        self.assertFalse(VALIDATE.valid_result(record, "same", 4))
        for status in ("running", "failed"):
            self.assertFalse(VALIDATE.valid_result({**record, "status": status}, "same", 1))
        self.assertFalse(VALIDATE.valid_result({**record, "artifacts": {}}, "same", 1))
        log.write_bytes(b"changed after validation")
        self.assertFalse(VALIDATE.valid_result(record, "same", 1))
        log.unlink()
        self.assertFalse(VALIDATE.valid_result(record, "same", 1))

    def test_source_environment_toolchain_and_command_changes_invalidate(self):
        tool = self.folder / "compiler"
        tool.write_bytes(b"compiler version one")
        env = {"CARGO_HOME": str(self.folder / "cargo"), "RUSTFLAGS": "-C debuginfo=1"}
        command = [["cargo", "check", "--locked"]]
        with mock.patch.object(VALIDATE, "source_files", return_value=[self.source]), \
                mock.patch.object(VALIDATE, "executable", return_value=tool), \
                mock.patch.object(VALIDATE.subprocess, "check_output", return_value="toolchain"):
            original = VALIDATE.identity(VALIDATE.inputs(env, command))
            self.source.write_bytes(b"second source, same crate version")
            self.assertNotEqual(original, VALIDATE.identity(VALIDATE.inputs(env, command)))
            self.source.unlink()
            self.assertNotEqual(original, VALIDATE.identity(VALIDATE.inputs(env, command)))
            self.source.write_bytes(b"first source")
            self.assertNotEqual(original, VALIDATE.identity(VALIDATE.inputs({**env, "RUSTFLAGS": ""}, command)))
            self.assertNotEqual(original, VALIDATE.identity(VALIDATE.inputs(env, [["cargo", "test"]])))
            tool.write_bytes(b"compiler version two")
            self.assertNotEqual(original, VALIDATE.identity(VALIDATE.inputs(env, command)))

    def test_external_cargo_configuration_change_invalidates(self):
        cargo = self.folder / "cargo"
        cargo.mkdir()
        config = cargo / "config.toml"
        config.write_bytes(b"[build]\njobs=2\n")
        with mock.patch.object(VALIDATE, "source_files", return_value=[self.source]), \
                mock.patch.object(VALIDATE, "executable", return_value=self.source), \
                mock.patch.object(VALIDATE.subprocess, "check_output", return_value="toolchain"):
            before = VALIDATE.identity(VALIDATE.inputs({"CARGO_HOME": str(cargo)}, []))
            config.write_bytes(b"[build]\njobs=3\n")
            self.assertNotEqual(before, VALIDATE.identity(VALIDATE.inputs({"CARGO_HOME": str(cargo)}, [])))

    def test_configured_execution_overrides_disable_result_reuse(self):
        cargo = self.folder / "cargo"
        cargo.mkdir()
        (cargo / "config.toml").write_text('[build]\nrustc = "external/compiler"\n')
        blockers = VALIDATE.reuse_blockers({"CARGO_HOME": str(cargo)})
        self.assertTrue(any("build.rustc" in reason for reason in blockers))
        (cargo / "config.toml").write_text('[target.x86_64-pc-windows-msvc]\nrunner = "external/runner"\n')
        blockers = VALIDATE.reuse_blockers({"CARGO_HOME": str(cargo)})
        self.assertTrue(any("linker/runner" in reason for reason in blockers))
        (cargo / "config.toml").write_text("")
        for key in ("CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUNNER",
                    "CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER"):
            blockers = VALIDATE.reuse_blockers({"CARGO_HOME": str(cargo), key: "external/tool"})
            self.assertTrue(any(key in reason for reason in blockers))

    def test_kotlin_compiler_payload_not_just_launcher_identifies_toolchain(self):
        kotlin = self.folder / "kotlin"
        launcher = kotlin / "bin" / "kotlinc.bat"
        launcher.parent.mkdir(parents=True)
        launcher.write_bytes(b"unchanged launcher")
        jar = kotlin / "lib" / "kotlin-compiler.jar"
        jar.parent.mkdir()
        jar.write_bytes(b"first compiler")
        java = self.folder / "jdk"
        for name in ("lib/modules", "bin/server/jvm.dll"):
            path = java / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b"runtime")
        before = VALIDATE.native_tool_payloads(java, launcher)
        jar.write_bytes(b"changed compiler, unchanged launcher")
        self.assertNotEqual(before, VALIDATE.native_tool_payloads(java, launcher))

    def fake_inputs(self, env, commands, paths=None, tools=()):
        return {"source": VALIDATE.digest(self.source), "commands": commands,
                "target": env["CARGO_TARGET_DIR"]}

    def test_gate_entrypoint_runs_all_four_then_reuses_only_identical_success(self):
        calls = []

        def execute(command, env, log):
            calls.append((list(command), dict(env)))
            log.write_bytes(b"successful output\n")
            return {"command": list(command), "exit_code": 0, "seconds": .01, "log": str(log)}

        arguments = ["gates", "--target-dir", str(self.target), "--results", str(self.results), "--offline"]
        with mock.patch.object(VALIDATE, "inputs", side_effect=self.fake_inputs), \
                mock.patch.object(VALIDATE, "execute", side_effect=execute):
            self.assertEqual(VALIDATE.main(arguments), 0)
            self.assertEqual([row[0][1] for row in calls], ["fmt", "clippy", "build", "test"])
            self.assertEqual(calls[-1][1]["RUST_MIN_STACK"], "16777216")
            self.assertEqual(calls[-1][1]["NO_PROXY"], "127.0.0.1,localhost,::1")
            self.assertTrue(all("RUST_MIN_STACK" not in row[1] for row in calls[:-1]))
            self.assertEqual(VALIDATE.main(arguments), 0)
            self.assertEqual(len(calls), 4)
            self.source.write_bytes(b"changed relevant source")
            self.assertEqual(VALIDATE.main(arguments), 0)
            self.assertEqual(len(calls), 8)
            retained = list(self.results.parent.glob("validation-runs/gates-*/result.json"))
            self.assertEqual(len(retained), 2)
            self.assertTrue(all(json.loads(path.read_text())["status"] == "passed" for path in retained))

    def test_failed_phase_is_not_cached_as_success_and_stops_before_later_gates(self):
        calls = []

        def execute(command, env, log):
            calls.append(command[1])
            log.write_bytes(b"lint failed")
            return {"command": list(command), "exit_code": 1 if command[1] == "clippy" else 0,
                    "seconds": .01, "log": str(log)}

        args = ["gates", "--target-dir", str(self.target), "--results", str(self.results)]
        with mock.patch.object(VALIDATE, "inputs", side_effect=self.fake_inputs), \
                mock.patch.object(VALIDATE, "execute", side_effect=execute):
            self.assertEqual(VALIDATE.main(args), 1)
            self.assertEqual(calls, ["fmt", "clippy"])
            self.assertEqual(json.loads(self.results.read_text())["gates"]["status"], "failed")

    def test_mutation_during_commands_never_records_success(self):
        def execute(command, env, log):
            log.write_bytes(b"checked old input")
            self.source.write_bytes(b"mutation while command ran")
            return {"command": list(command), "exit_code": 0, "seconds": .01, "log": str(log)}

        with mock.patch.object(VALIDATE, "inputs", side_effect=self.fake_inputs), \
                mock.patch.object(VALIDATE, "execute", side_effect=execute):
            with self.assertRaisesRegex(RuntimeError, "Inputs changed"):
                VALIDATE.main(["dev", "--target-dir", str(self.target), "--results", str(self.results)])
            self.assertNotEqual(json.loads(self.results.read_text())["dev"]["status"], "passed")

    def test_successful_pretty_json_package_helpers_run_and_packages_always_get_fresh_targets(self):
        calls = []

        def execute(command, env, log):
            calls.append((list(command), env["CARGO_TARGET_DIR"]))
            if command[:2] == ["cargo", "package"]:
                archive = Path(env["CARGO_TARGET_DIR"]) / "package" / "same-0.1.0.crate"
                archive.parent.mkdir(parents=True)
                archive.write_bytes(b"actual archive inventory is tested separately")
                log.write_bytes(b"packaging complete")
            else:
                log.write_text(json.dumps({"packages": [], "status": "ok"}, indent=2))
            return {"command": list(command), "exit_code": 0, "seconds": .01, "log": str(log)}

        args = ["packages", "--target-dir", str(self.target), "--results", str(self.results), "--offline"]
        with mock.patch.object(VALIDATE, "inputs", side_effect=self.fake_inputs), \
                mock.patch.object(VALIDATE, "execute", side_effect=execute):
            self.assertEqual(VALIDATE.main(args), 0)
            self.assertEqual(VALIDATE.main(args), 0)
        self.assertEqual(len(calls), 6)
        self.assertNotEqual(calls[0][1], calls[3][1])
        self.assertEqual(calls[0][0][:2], ["cargo", "package"])
        self.assertTrue(calls[1][0][1].endswith("verify-packages.py"))
        self.assertTrue(calls[2][0][1].endswith("test-simulator-package.py"))

    def test_private_environment_disables_telemetry_and_does_not_inherit_connection_secrets(self):
        self.results.parent.mkdir()
        with mock.patch.dict(os.environ, {"LOX_PASS": "never retain", "GH_TOKEN": "never retain",
                                          "RUST_MIN_STACK": "old"}, clear=False):
            env = VALIDATE.environment(self.results, self.target)
        self.assertNotIn("LOX_PASS", env)
        self.assertNotIn("GH_TOKEN", env)
        self.assertNotIn("RUST_MIN_STACK", env)
        self.assertEqual(env["LOX_TELEMETRY"], "0")
        self.assertEqual(env["DO_NOT_TRACK"], "1")
        self.assertEqual(Path(env["LOX_CONFIG"]).read_bytes(), b"telemetry: false\n")

    @unittest.skipUnless(os.name == "nt", "Existing native proof builder is Windows AMD64 only")
    def test_native_reuse_builds_current_library_and_invalidates_changed_library_or_generated_output(self):
        library = self.target / "release" / "lox_archive.dll"
        library.parent.mkdir(parents=True)
        library.write_bytes(b"first native library")
        calls = []
        commands = []

        def execute(command, env, log):
            calls.append(command[0])
            commands.append(list(command))
            if command[0] == "cargo":
                log.write_text(json.dumps({"reason": "compiler-artifact",
                                           "filenames": [str(library)]}) + "\n")
            else:
                output = Path(command[command.index("-OutputDirectory") + 1])
                actual = output / "venv" / "Lib" / "site-packages" / "lox_archive" / "lox_archive.dll"
                actual.parent.mkdir(parents=True)
                actual.write_bytes(library.read_bytes())
                generated = output / "generated-java" / "Record.java"
                generated.parent.mkdir()
                generated.write_bytes(b"generated API")
                (output / "ARTIFACTS.json").write_text(json.dumps({"artifact_manifest": [
                    {"path": str(path.relative_to(output)), "sha256": VALIDATE.digest(path)}
                    for path in (actual, generated)]}))
                (output / "commands.json").write_text(json.dumps([{"exit_code": 0}]))
                (output / "java-allocation-smoke.log").write_bytes(b"live buffers released")
                log.write_bytes(b"all native error/disposal checks passed")
            return {"command": list(command), "exit_code": 0, "seconds": .01, "log": str(log)}

        args = ["native", "--target-dir", str(self.target), "--results", str(self.results),
                "--maturin", "maturin", "--java-generator", "generator", "--java-home", "jdk",
                "--kotlin-compiler", "kotlinc"]
        with mock.patch.object(VALIDATE, "inputs", side_effect=self.fake_inputs), \
                mock.patch.object(VALIDATE, "source_files", return_value=[self.source]), \
                mock.patch.object(VALIDATE, "native_tool_payloads", return_value={"compiler": "identity"}), \
                mock.patch.object(VALIDATE, "execute", side_effect=execute):
            self.assertEqual(VALIDATE.main(args), 0)
            self.assertEqual(commands[0][1], "rustc")
            self.assertEqual(commands[0][commands[0].index("--crate-type") + 1], "cdylib")
            first = json.loads(self.results.read_text())["native"]
            retained_dll = Path(first["run_directory"]) / "current-library.dll"
            self.assertEqual(retained_dll.read_bytes(), library.read_bytes())
            self.assertEqual(VALIDATE.main(args), 0)
            self.assertEqual(calls, ["cargo", "pwsh", "cargo"])
            library.write_bytes(b"changed native library, identical bindings source")
            self.assertEqual(VALIDATE.main(args), 0)
            self.assertEqual(calls[-2:], ["cargo", "pwsh"])
            self.source.write_bytes(b"changed Core test assertions, unchanged library")
            self.assertEqual(VALIDATE.main(args), 0)
            self.assertEqual(calls[-2:], ["cargo", "pwsh"])
            for name in ("java-allocation-smoke.log", "commands.json", "current-library.log"):
                record = json.loads(self.results.read_text())["native"]
                proof = next(Path(path) for path in record["artifacts"] if path.endswith(name))
                proof.unlink()
                self.assertEqual(VALIDATE.main(args), 0)
                self.assertEqual(calls[-2:], ["cargo", "pwsh"])
            record = json.loads(self.results.read_text())["native"]
            generated = next(Path(path) for path in record["artifacts"] if path.endswith("Record.java"))
            generated.write_bytes(b"modified generated API")
            self.assertEqual(VALIDATE.main(args), 0)
            self.assertEqual(calls[-2:], ["cargo", "pwsh"])

    @unittest.skipUnless(os.name == "nt", "Existing native proof builder is Windows AMD64 only")
    def test_native_mismatch_is_failed_and_preserves_actual_prebuild(self):
        library = self.target / "release" / "lox_archive.dll"
        library.parent.mkdir(parents=True)
        library.write_bytes(b"original producer")

        def execute(command, env, log):
            if command[0] == "cargo":
                log.write_text(json.dumps({"reason": "compiler-artifact",
                                           "filenames": [str(library)]}) + "\n")
            else:
                output = Path(command[command.index("-OutputDirectory") + 1])
                actual = output / "venv" / "Lib" / "site-packages" / "lox_archive" / "lox_archive.dll"
                actual.parent.mkdir(parents=True)
                actual.write_bytes(b"different actual wheel")
                library.write_bytes(b"builder overwrote producer slot")
                (output / "commands.json").write_text(json.dumps([{"exit_code": 0}]))
                (output / "ARTIFACTS.json").write_text(json.dumps({"artifact_manifest": [
                    {"path": str(actual.relative_to(output)), "sha256": VALIDATE.digest(actual)}]}))
                log.write_bytes(b"native checks completed")
            return {"command": list(command), "exit_code": 0, "seconds": .01, "log": str(log)}

        args = ["native", "--target-dir", str(self.target), "--results", str(self.results),
                "--maturin", "maturin", "--java-generator", "generator", "--java-home", "jdk",
                "--kotlin-compiler", "kotlinc"]
        with mock.patch.object(VALIDATE, "inputs", side_effect=self.fake_inputs), \
                mock.patch.object(VALIDATE, "source_files", return_value=[self.source]), \
                mock.patch.object(VALIDATE, "native_tool_payloads", return_value={"compiler": "identity"}), \
                mock.patch.object(VALIDATE, "execute", side_effect=execute):
            with self.assertRaisesRegex(RuntimeError, "Qualified native library differs"):
                VALIDATE.main(args)
        record = json.loads(self.results.read_text())["native"]
        self.assertEqual(record["status"], "failed")
        self.assertEqual((Path(record["run_directory"]) / "current-library.dll").read_bytes(),
                         b"original producer")


if __name__ == "__main__":
    unittest.main()
