#!/usr/bin/env python3
"""Execute narrow, offline CLI/unit proof; never run Config GUI or a Miniserver."""

import argparse
from datetime import datetime
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import uuid


ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
FIRST = "74000000-0000-0000-0000-000000000001"
SECOND = "74000000-0000-0000-0000-000000000002"
APPLICATION = {
    "version": "17.2.10.21",
    "sha256": "7b55e3ef18401574ab967c5c6a0140cd5a66dc728521ebba04ad53093577d8b3",
}
UNIT = "config_edit::energy_tests::"
SIM_TEST = "parser::tests::energy_manager_offset_parameter_retains_incoming_wire"
TINY = "0." + "0" * 399 + "1"

# Each row names actual narrow assertions, not a declaration of full capability parity.
DEFINITIONS = [
    ("offset-grid-power",
     "Signed O values -1.25/2.75 and explicit zero; parameter direction; positive/negative nonzero decimal underflow; invalid NaN/scientific input; dry-run.",
     "Set O=-1.25,2.75,0; inspect get-params direction; reject NaN/1e3 and +/-('0.'+399 zeros+'1') with source snapshots; run dry-run. Execute the separate parser regression for incoming parameter wire retention.",
     "O is parameter/P; signed values persist; explicit zero omits Def; nonzero underflow is rejected before omission; unrelated data survives and rejected/dry-run edits write no file. Parser proof is direction/wire retention only, not runtime behavior.",
     ["energy_signed_offset_is_parameter_and_bounds_are_atomic", "energy_scalar_edits_preserve_entire_other_records_and_fail_ambiguous_bindings", "energy_nonzero_decimal_underflow_fails_atomically"],
     ["scientific/GUI numeric input equivalence", "storage/import/export runtime behavior"]),
    ("minimum-state-of-charge",
     "MinSoc zero/100 endpoints, omitted versus explicit-zero inspection, rejection of -1/101 and positive/negative nonzero decimal underflow.",
     "Inspect baseline; set MinSoc=100 then0; reject -1/101 and +/-('0.'+399 zeros+'1') with byte snapshots.",
     "Stored0 differs from omission;100 persists and zero omits Def; out-of-range and nonzero-underflow inputs fail without mutation or default omission.",
     ["energy_signed_offset_is_parameter_and_bounds_are_atomic", "energy_inspection_distinguishes_omission_and_unknown_binding", "energy_nonzero_decimal_underflow_fails_atomically"],
     ["GUI prefix filtering", "Soc/Spwr storage-priority runtime"]),
    ("maximum-storage-power",
     "MaxSpwr5.5/0 storage; negative and positive/negative nonzero-underflow rejection without zero-runtime interpretation.",
     "Set MaxSpwr5.5 then0; reject -1 and +/-('0.'+399 zeros+'1') against byte snapshots.",
     "Nonnegative power persists; explicit zero omits Def; negative/nonzero-underflow input cannot mutate or become a zero sentinel. No runtime meaning is inferred.",
     ["energy_signed_offset_is_parameter_and_bounds_are_atomic", "energy_nonzero_decimal_underflow_fails_atomically"],
     ["MaxSpwr=0 runtime semantics", "GUI negative input filtering"]),
    ("ordered-load-records",
     "Twelve direct existing slots including canonical native 8-4-4-16 IDs; full-record move to2; sparse shape rejection; dry-run.",
     "Inspect12 rows in RFC and canonical synthetic.Loxone native fixtures; edit an existing native Id and move using its RFC alias; compare whole attributes/wires; reject malformed/duplicate/sparse shapes and dry-run move.",
     "Priority follows XML order; complete records/unknown nested children/interleaved node/wire UUIDs survive; no rows created/deleted; unsafe or dry-run edits do not write.",
     ["energy_reorder_preserves_whole_records_unknown_children_and_wire_spellings", "energy_unsupported_shapes_and_ambiguous_ids_do_not_mutate", "energy_native_ids_and_aliases_preserve_records"],
     ["structural add/delete acceptance", "sparse/>12-row native acceptance"]),
    ("load-identity",
     "Native 8-4-4-16/RFC 8-4-4-4-12 identity aliases, case-insensitive selection, exact spelling preservation, duplicate and misplaced-dash rejection.",
     "Use canonical native fixture: edit native Id, reorder through RFC alias, select uppercase hex alias and reject a mixed native/RFC duplicate plus7-5-4-16 malformed selector.",
     "Both strict spellings select the same binary identity; stored native spelling remains exact; mixed-spelling duplicates/malformed dash placement fail atomically. U is never identity.",
     ["energy_reorder_preserves_whole_records_unknown_children_and_wire_spellings", "energy_unsupported_shapes_and_ambiguous_ids_do_not_mutate", "energy_native_ids_and_aliases_preserve_records"],
     ["native malformed/dangling identity acceptance"]),
    ("load-output-used",
     "U=true survives rated-zero edits and reorder; unused slots retain omitted U/false.",
     "Inspect unused slots; configure first load, set rated power0 and reorder it.",
     "U stays exact true on connected zero-rated load, unused rows remain false/omitted, and U is not offered as an enable setter.",
     ["energy_power_zero_substitution_and_above_rated_are_allowed", "energy_unsupported_shapes_and_ambiguous_ids_do_not_mutate"],
     ["GUI connected-state changes", "U editing"]),
    ("load-name",
     "Existing name changes to CLI & renamed with XML escaping and intact unknown/icon/wire data.",
     "Set --name 'CLI & renamed' on the existing Id, then reload/inspect saved XML and reorder.",
     "Na retains the requested string; XML escapes it; other attributes, icon, nested extension and references remain intact.",
     ["energy_native_name_mode_markers_preserve_other_storage"],
     ["complete GUI text-input validation"]),
    ("load-name-customization",
     "Native-backed name setter changes absent Ncu to true; not GUI transition proof.",
     "Set a name on a row whose Ncu is omitted, then inspect stored/effective marker.",
     "Ncu is true after the name setter; only this native/CLI behavior is demonstrated, not the outstanding GUI transition.",
     ["energy_native_name_mode_markers_preserve_other_storage"],
     ["false-to-true official GUI transition"]),
    ("load-analog-mode",
     "Analog setter and digital/default omission on an existing slot.",
     "Set --mode analog, inspect An=true; set digital, inspect An omission/false; restore analog.",
     "Analog is true and digital is false/omitted, without changing U, Id, OIdx or icon.",
     ["energy_native_name_mode_markers_preserve_other_storage"],
     ["other type/mode options"]),
    ("load-mode-customization",
     "Native-backed mode setter changes absent Anc to true; not GUI transition proof.",
     "Set analog on a row whose Anc is omitted and inspect marker.",
     "Anc becomes true after the mode setter; original references survive; no GUI false-to-true proof is asserted.",
     ["energy_native_name_mode_markers_preserve_other_storage"],
     ["false-to-true official GUI transition"]),
    ("load-output-association",
     "OIdx remains0 when native/RFC identity is moved to priority2; twelve explicit unique associations required.",
     "Move first Id to2 in RFC and canonical native fixtures using native/RFC selectors; compare OIdx per Id and reject duplicate OIdx.",
     "First identity still maps to L1 with OIdx0, not priority2; no OIdx renumbering or reference rewriting occurs.",
     ["energy_reorder_preserves_whole_records_unknown_children_and_wire_spellings", "energy_unsupported_shapes_and_ambiguous_ids_do_not_mutate", "energy_native_ids_and_aliases_preserve_records"],
     ["native missing/duplicate output-index acceptance"]),
    ("load-preparation-time",
     "Default RT60 omission, edits30/86400 seconds, rejection29/86401.",
     "Inspect omitted RT; set30 then86400; reject29/86401 with byte snapshots.",
     "RT defaults to60 while retaining omission; accepted endpoints serialize integer seconds; invalid edits leave source unchanged.",
     ["energy_minutes_seconds_negative_runtime_and_overflow", "energy_inspection_distinguishes_omission_and_unknown_binding"],
     ["GUI clamping/prefix filtering equivalence"]),
    ("load-minimum-runtime",
     "Integer2 and -1 minutes to120/-60 seconds; zero omission; checked overflow; dry-run.",
     "Set2 then-1 then0 integer minutes, reject overflow2147483647 minutes, and dry-run -1 minutes.",
     "MinRt stores120/-60 seconds; negative is allowed; zero omits; overflowing/dry-run edits leave source/output unchanged.",
     ["energy_minutes_seconds_negative_runtime_and_overflow"],
     ["fractional GUI rounding", "native overflow/scientific-input semantics", "runtime scheduling"]),
    ("load-daily-minimum-runtime",
     "Daily0/1440 integer minutes to omitted0/86400 seconds; reject -1/1441.",
     "Set1440 then0 then1440 daily minutes; reject -1/1441 against byte snapshots.",
     "MDay uses checked integer seconds, zero omission and 0..1440 minute bounds; invalid edits are atomic.",
     ["energy_minutes_seconds_negative_runtime_and_overflow"],
     ["GUI prefix filtering", "day-cycle runtime"]),
    ("load-daily-deadline",
     "All 28 deadline labels map to selectors0..27; Sunset0 is omitted; invalid hourly labels are rejected.",
     "Set sunset/dusk/sunrise/dawn and hourly01:00..24:00; inspect untouched second-rowtpR27; reject00:00 without writing.",
     "All28choices store the mapped selector, with Sunset omission; valid labels are GUI-save-observed, not runtime-verified; invalid00:00 cannot write.",
     ["energy_all_deadlines_and_invalid_hours_are_atomic", "energy_inspection_distinguishes_omission_and_unknown_binding"],
     ["fresh combined CLI/GUI proof", "invalid native selector acceptance", "deadline execution"]),
    ("load-switch-on-power",
     "Zero SwPwr substitution, rated-zero switch copy, above-rated values, and positive/negative nonzero-underflow rejection before substitution.",
     "Set rated4 from zero switch; set rated2.5/switch0; set switch2.6; reject negative and +/-('0.'+399 zeros+'1') switch in compound patches.",
     "Switch becomes4/2.5;2.6 above rated2.5 stays; nonzero underflow is never treated as switch0/substitution; rejected patches are atomic.",
     ["energy_power_zero_substitution_and_above_rated_are_allowed", "energy_nonzero_decimal_underflow_fails_atomically"],
     ["runtime switching behavior", "GUI filtering"]),
    ("load-rated-power",
     "Rated4 zero-switch copy, explicit rated0 omission with U=true, and positive/negative nonzero-underflow rejection.",
     "Set rated4 then2.5; set switch2.6/rated0; reject +/-('0.'+399 zeros+'1') rated power with staged name and byte snapshots.",
     "Rated-zero omission preserves U=true/SwPwr2.6; nonzero underflow cannot be omitted as rated0; rejected edits are atomic.",
     ["energy_power_zero_substitution_and_above_rated_are_allowed", "energy_nonzero_decimal_underflow_fails_atomically"],
     ["native all-zero runtime behavior", "GUI filtering"]),
    ("load-icon-reference",
     "Bounded exact built-in list/select/clear: 623 loaded filenames/IDs and independent archive provenance; native/RFC case aliases; atomic invalid selection rejection; raw unknown preservation.",
     "List JSON/table/CSV with header flags; select electric-fan.svg by filename/native/RFC alias; reject disabled/absent/malformed/empty/zero/custom compound patches; dry-run/save-as; clear only matched ico retaining other references and opaque definitions. Execute named Rust tests and synthetic ZIP generator tests.",
     "Exact loaded membership stores canonical native fan ID with version/hash; invalid patches cannot write source/destination; explicit clear omits only matched ico, without decoding/validating custom images or orphan cleanup.",
     ["energy_builtin_icon_selection_uses_exact_membership_and_canonical_native_id", "energy_icon_invalid_selections_fail_transactionally_with_actionable_diagnostics", "energy_icon_clear_only_removes_matched_reference_and_preserves_definitions", "energy_unknown_icon_spelling_survives_unrelated_edits_and_inspection", "energy_icon_catalog_corruption_is_an_explicit_error"],
     ["fresh official GUI explicit-clear/save/reopen proof", "custom selection/import/replace/delete and payload validation", "Qt SVG rendering parity", "full configuration/runtime scope"]),
    ("unused-load-feedback",
     "Storage-derived unused condition !U || MaxPwr<=0 on unused/default and connected rated-zero slots.",
     "Inspect unused rows; set connected first rated0 and inspect U and unused_feedback.",
     "Omitted U/false slots and connected zero-rated first slot report unused true; U=true is retained. No GUI color rendering occurs.",
     ["energy_power_zero_substitution_and_above_rated_are_allowed", "energy_inspection_distinguishes_omission_and_unknown_binding"],
     ["GUI rendering equivalence", "connection-state mutation"]),
]

UNIT_CASES = {
    "offset-grid-power": (
        "Scalar API signed/zero O, metadata=P, positive/negative nonzero-underflow rejection, duplicate bindings; simulator parser O parameter incoming wire.",
        "Signed O/default omission roundtrip; +/-('0.'+399 zeros+'1') fails numeric_underflow without DOM mutation; parser O is Parameter retaining -1.25 default and Source.AQ incoming wire, without runtime simulation."),
    "minimum-state-of-charge": (
        "MinSoc0/100, -1/101, omitted storage and positive/negative nonzero-underflow.",
        "Endpoints succeed, invalid bounds/nonzero-underflow preserve bytes, and stored0 remains distinct from omitted default0."),
    "maximum-storage-power": (
        "MaxSpwr5.5/0, negative input and positive/negative nonzero-underflow.",
        "5.5 reads back, explicit0 omits; negative/nonzero-underflow fail without changing bytes or applying a zero sentinel; runtime remains untested."),
    "ordered-load-records": (
        "Complete record moves around interleaved nodes plus canonical native fixture edit and native/RFC alias reorder; invalid shapes.",
        "Full XMLNodes/attributes/native Id spellings retain identity and OIdx; reverse reorder restores bytes; malformed/mixed-duplicate/sparse edits cannot mutate."),
    "load-identity": (
        "Canonical native Id selected natively, by RFC alias and uppercase hex; malformed7-5-4-16 and mixed native/RFC duplicate rejection.",
        "Accepted aliases resolve one persistent identity while native spelling remains exact; malformed dash placement and binary-equivalent duplicates reject atomically."),
    "load-output-used": (
        "Connected U=true remains after MaxPwr0; omitted U becomes effective false; UUID-valued U shape is rejected.",
        "Used true spelling survives rated-zero edit, omission remains false, and invalid U cannot mutate the editor."),
    "load-name": (
        "Native API sets 'Native CLI & <name>' and reloads serialized XML; invalid XML NUL text is rejected.",
        "The exact name roundtrips XML escaping; unknown child/icon attributes remain; NUL name fails without changing bytes."),
    "load-name-customization": (
        "Direct native name setter starts from omitted Ncu.",
        "Ncu becomes effective true after the name edit; this is a Rust/native setter assertion, not a GUI transition observation."),
    "load-analog-mode": (
        "Direct native setter switches omitted/digital->analog->digital.",
        "Analog effective true serializes, then digital removes An and restores default false while preserving unrelated storage."),
    "load-mode-customization": (
        "Direct native mode setter starts from omitted Anc.",
        "Anc becomes effective true after setting analog; no official GUI transition is tested."),
    "load-output-association": (
        "Priority moves in RFC/canonical native fixtures retain OIdx; duplicate/missing/out-of-range association fails.",
        "OIdx0/L1 remains on original native Id after selection via RFC alias and reorder; no association renumbering or invalid-shape mutation."),
    "load-preparation-time": (
        "RT omission defaults60, 30/86400 seconds accepted,29/86401 rejected.",
        "Both endpoints read back as integer seconds and invalid preparation patches leave bytes unchanged, even after a staged name edit."),
    "load-minimum-runtime": (
        "MinRt -1/0 minutes and upper checked integer-minute conversion, including i64 and signed32 overflow.",
        "-1 stores-60, zero omits,35791394 minutes stores2147483640 seconds;35791395/i64 extrema fail atomically without a nonnegative rule."),
    "load-daily-minimum-runtime": (
        "MDay1440/0 integer minutes accepted; -1/1441 rejected.",
        "1440 stores86400 seconds, zero omits, and invalid daily-minute patches preserve editor bytes."),
    "load-daily-deadline": (
        "All28deadline edits, invalid parser spellings and atomic rejection of programmatic invalid hours.",
        "Sunset omits/defaults0; Dusk/Sunrise/Dawn and hourly01:00..24:00 mapto1..27; native27 displays24:00; invalid hours leave editorbytes unchanged."),
    "load-switch-on-power": (
        "Zero-switch copy, zero substitution, above-rated2.6, negative and positive/negative nonzero-underflow compound edits.",
        "Switch4/2.5 substitutions and above-rated2.6 are retained; nonzero-underflow never triggers substitution and failed patches leave identical bytes."),
    "load-rated-power": (
        "Rated4 zero-switch copy, explicit rated0 with U=true, and positive/negative nonzero-underflow.",
        "Explicit rated0 omits MaxPwr preserving U/SwPwr2.6; nonzero-underflow cannot be mistaken for rated0 and leaves bytes unchanged."),
    "load-icon-reference": (
        "Exact built-in fan selection by filename/native/RFC case alias; disabled/absent/malformed/empty/zero/custom rejection; clear closure; opaque reference preservation; embedded catalog errors.",
        "Only exact loaded members are newly assigned in canonical native spelling; invalid compound patches and corrupt catalogs fail explicitly; clear changes only matched ico, retaining definitions/other references; unknown existing storage is not sanitized. No GUI/custom payload validity is claimed."),
    "unused-load-feedback": (
        "Inspect false/default-U slots and true-U rated-zero slot.",
        "Both omitted-U slots and connected rated-zero slot expose unused_feedback=true; no GUI rendering assertion."),
}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def save_json(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def write_definitions():
    mappings, cases = [], []
    for capability, condition, procedure, expected, unit_names, omissions in DEFINITIONS:
        test_names = [UNIT + name for name in unit_names]
        if capability == "offset-grid-power":
            test_names.append(SIM_TEST)
        mappings.append({
            "capability_id": capability,
            "condition_scope": condition,
            "omissions": omissions,
            "required_case_ids": [capability + ".implementation", capability + ".unit_test"],
            "rust_unit_tests": test_names,
        })
        cases.append({
            "id": capability + ".implementation", "capability_ids": [capability],
            "condition": condition, "evidence_kind": "implementation",
            "procedure": "Run tests/ux/energy-manager2/run-cli-cases.py on the built local lox binary: " + procedure,
            "expected": expected,
        })
        unit_condition, unit_expected = UNIT_CASES[capability]
        cases.append({
            "id": capability + ".unit_test", "capability_ids": [capability],
            "condition": unit_condition,
            "evidence_kind": "unit_test",
            "procedure": "Execute cargo test --bin lox --test config_integration energy_ -- --nocapture" + (" and cargo test -p lox-sim --lib " + SIM_TEST + " -- --nocapture" if capability == "offset-grid-power" else "") + "; inspect named assertions: " + ", ".join(test_names),
            "expected": unit_expected,
        })
    path = HERE / "CLI-case-definitions.json"
    save_json(path, {
        "schema_version": 1, "application": APPLICATION, "block_type": "EnergyManager2",
        "control_type": 515, "status": "incomplete_cli_unit_subset_only",
        "description": "Case definitions for a NEW CLI snapshot, not mutations of the original recovery contract/GUI results. Each mapping is explicitly narrower than complete capability parity. Execution uses independent synthetic input and actual CLI/unit processes; no official Config GUI is run.",
        "capability_mappings": mappings, "cases": cases,
    })
    return path


class Runner:
    def __init__(self, lox, cargo):
        self.lox, self.cargo = str(lox), cargo
        self.started_at = datetime.now().astimezone().isoformat()
        self.id = "energy-cli-" + datetime.now().strftime("%Y%m%dT%H%M%S") + "-" + uuid.uuid4().hex[:8]
        self.output = ROOT / ".reverse-engineering" / "ux" / "energy-manager2" / "cli" / self.id
        self.output.mkdir(parents=True, exist_ok=False)
        self.commands, self.observations, self.snapshots = [], [], {}
        self.cli_version = "unavailable"
        self.env = dict(os.environ, LOX_TELEMETRY="0", LOX_CONFIG=str(self.output / "absent.yaml"))
        self.before_definition = write_definitions()

    def artifact(self, path):
        return {"path": str(path.relative_to(ROOT)), "sha256": digest(path)}

    def snapshot(self, path):
        relative = path.relative_to(ROOT)
        if relative not in self.snapshots:
            copied = self.output / "sources" / relative
            copied.parent.mkdir(parents=True, exist_ok=True)
            copied.write_bytes(path.read_bytes())
            self.snapshots[relative] = copied
        return self.artifact(self.snapshots[relative])

    def execute(self, label, argv, expected_exit=0, parse_json=True):
        before = time.monotonic()
        process = subprocess.run(argv, cwd=ROOT, env=self.env, capture_output=True)
        stdout = self.output / (label + ".stdout.txt")
        stderr = self.output / (label + ".stderr.txt")
        stdout.write_bytes(process.stdout)
        stderr.write_bytes(process.stderr)
        entry = {
            "label": label, "argv": argv, "cwd": str(ROOT),
            "explicit_environment": {"LOX_TELEMETRY": "0", "LOX_CONFIG": self.env["LOX_CONFIG"]},
            "exit_code": process.returncode, "expected_exit": expected_exit,
            "elapsed_seconds": time.monotonic() - before,
            "stdout": self.artifact(stdout), "stderr": self.artifact(stderr),
        }
        self.commands.append(entry)
        if process.returncode != expected_exit:
            raise AssertionError(f"{label}: exit {process.returncode}, expected {expected_exit}; inspect {stderr}")
        return json.loads(process.stdout) if parse_json else process.stdout.decode("utf-8", errors="replace")

    def cli(self, label, arguments, expected_exit=0):
        return self.execute(label, [self.lox, "--quiet", "--output", "json", *map(str, arguments)], expected_exit)

    def fixture(self, name):
        path = self.output / (name + ".Loxone")
        text = (HERE / "cli-synthetic.xml").read_text(encoding="utf-8")
        path.write_bytes(b"\xef\xbb\xbf" + text.replace("\r\n", "\n").replace("\n", "\r\n").encode("utf-8"))
        return path

    def inspect(self, label, path):
        return self.cli(label, ["config", "energy", "inspect", path, "CLI Energy"])["energy"]

    def scalar(self, label, path, key, value, expected_exit=0):
        return self.cli(label, ["config", "energy", "set", path, "CLI Energy", key, value], expected_exit)

    def load(self, label, path, *flags, expected_exit=0):
        return self.cli(label, ["config", "energy", "load", "set", path, "CLI Energy", FIRST, *flags], expected_exit)

    def prove(self, capabilities, start, actual):
        report = self.output / ("assertions-" + capabilities[0] + ".json")
        save_json(report, {"capability_ids": capabilities, "assertions_passed": actual})
        artifacts = [self.artifact(report)]
        for entry in self.commands[start:]:
            artifacts.extend([entry["stdout"], entry["stderr"]])
        for capability in capabilities:
            self.observations.append({
                "id": capability + ".implementation", "outcome": "pass",
                "evidence_kind": "implementation", "actual": actual, "artifacts": list(artifacts),
            })

    def run(self):
        unit_log = self.execute(
            "targeted-rust-tests",
            [self.cargo, "test", "--bin", "lox", "--test", "config_integration", "energy_", "--", "--nocapture"],
            parse_json=False,
        )
        assert "test result: ok." in unit_log, "Targeted Cargo tests did not report success."
        unit_entry = self.commands[-1]
        sim_log = self.execute("targeted-simulator-parser-tests",
            [self.cargo, "test", "-p", "lox-sim", "--lib", SIM_TEST, "--", "--nocapture"],
            parse_json=False)
        assert "1 passed; 0 failed" in sim_log and SIM_TEST + " ... ok" in sim_log
        sim_artifacts = [self.commands[-1]["stdout"], self.commands[-1]["stderr"],
                         self.snapshot(ROOT / "lox-sim" / "src" / "parser.rs")]
        unit_artifacts = [unit_entry["stdout"], unit_entry["stderr"],
                          self.snapshot(ROOT / "src" / "config_edit" / "energy_tests.rs")]
        for capability, condition, _, _, names, _ in DEFINITIONS:
            for name in names:
                assert UNIT + name + " ... ok" in unit_log, name
            self.observations.append({
                "id": capability + ".unit_test", "outcome": "pass", "evidence_kind": "unit_test",
                "actual": "Actually executed named Rust tests passed: " + ", ".join(UNIT + name for name in names) + ("; " + SIM_TEST if capability == "offset-grid-power" else "") + ". Assertions: " + UNIT_CASES[capability][1],
                "artifacts": list(unit_artifacts) + (sim_artifacts if capability == "offset-grid-power" else []),
            })
        self.cli_version = self.execute("cli-version", [self.lox, "--version"], parse_json=False).strip()

        start = len(self.commands)
        path = self.fixture("scalar")
        original = path.read_bytes()
        baseline = self.inspect("baseline", path)
        assert len(baseline["loads"]) == 12
        assert baseline["scalar_parameters"][0]["value"] == {"stored": None, "effective": 0.0, "default": 0.0}
        assert baseline["scalar_parameters"][1]["value"]["stored"] == "0"
        for value in ["-1.25", "2.75", "0"]:
            state = self.scalar("offset-" + value.replace("-", "negative-"), path, "O", value)["energy"]
            assert state["scalar_parameters"][0]["value"]["effective"] == float(value)
            assert state["loads"] == baseline["loads"]
        assert state["scalar_parameters"][0]["value"]["stored"] is None
        directions = self.cli("get-params", ["config", "get-params", path, "CLI Energy"])
        assert next(connector for connector in directions["connectors"] if connector["key"] == "O")["direction"] == "P"
        for invalid in ["NaN", "1e3"]:
            before = path.read_bytes()
            self.scalar("offset-invalid-" + invalid, path, "O", invalid, 1)
            assert path.read_bytes() == before
        for sign, value in [("positive", TINY), ("negative", "-" + TINY)]:
            before = path.read_bytes()
            result = self.scalar("offset-underflow-" + sign, path, "O", value, 1)
            assert result["error"] == "numeric_underflow" and path.read_bytes() == before
        destination = self.output / "dry-run-not-created.Loxone"
        before = path.read_bytes()
        proposal = self.cli("scalar-dry-run", ["--dry-run", "config", "energy", "set", path, "CLI Energy", "O", "-1.25", "--save-as", destination])
        assert proposal["dry_run"] and proposal["saved_to"] is None
        assert path.read_bytes() == before and not destination.exists()
        self.prove(["offset-grid-power"], start, "Signed -1.25/2.75 retained; explicit zero omitted; direction P; NaN/1e3 and positive/negative decimal underflow rejected; rejected/dry-run source/output nonmutation. Separate actual parser unit test retains the incoming O parameter wire without runtime behavior.")

        for key, capability, high, invalids in [
            ("MinSoc", "minimum-state-of-charge", "100", ["-1", "101"]),
            ("MaxSpwr", "maximum-storage-power", "5.5", ["-1"]),
        ]:
            start = len(self.commands)
            for value in [high, "0"]:
                state = self.scalar(key + "-" + value, path, key, value)["energy"]
                parameter = next(parameter for parameter in state["scalar_parameters"] if parameter["key"] == key)
                assert parameter["value"]["effective"] == float(value)
                if value == "0":
                    assert parameter["value"]["stored"] is None
            for invalid in invalids:
                before = path.read_bytes()
                self.scalar(key + "-invalid-" + invalid.replace("-", "negative-"), path, key, invalid, 1)
                assert path.read_bytes() == before
            for sign, value in [("positive", TINY), ("negative", "-" + TINY)]:
                before = path.read_bytes()
                result = self.scalar(key + "-underflow-" + sign, path, key, value, 1)
                assert result["error"] == "numeric_underflow" and path.read_bytes() == before
            self.prove([capability], start, key + " accepted " + high + "/0; zero omitted; negative/out-of-range AND positive/negative nonzero-underflow rejected with identical source. No runtime interpretation.")
        assert path.read_bytes().startswith(b"\xef\xbb\xbf")
        assert b"12hTF=" in path.read_bytes() and b"_12hTF=" not in path.read_bytes()
        assert b"\n" not in path.read_bytes().replace(b"\r\n", b"")
        assert original != path.read_bytes()  # Explicit baseline MinSoc0 was intentionally omitted.

        path = self.fixture("load-fields")
        start = len(self.commands)
        state = self.load("name-mode", path, "--name", "CLI & renamed", "--mode", "analog")["energy"]
        first = state["loads"][0]
        assert first["name"]["effective"] == "CLI & renamed"
        assert first["analog"]["effective"] and first["name_customized"]["effective"] and first["mode_customized"]["effective"]
        assert first["icon"] == "75000000-0000-0000-0000-000000000001"
        state = self.load("mode-digital", path, "--mode", "digital")["energy"]
        assert state["loads"][0]["analog"]["stored"] is None and not state["loads"][0]["analog"]["effective"]
        self.load("mode-analog-restored", path, "--mode", "analog")
        self.prove(["load-name", "load-name-customization", "load-analog-mode", "load-mode-customization"], start, "Native name/mode setters saved XML-escaped 'CLI & renamed', analog true, digital omission/false and Ncu/Anc true; exact icon/unknown fields survived. Marker transitions are CLI/native-only, not GUI proof.")

        start = len(self.commands)
        state = self.inspect("preparation-default", path)
        assert state["loads"][0]["preparation_seconds"]["stored"] is None
        assert state["loads"][0]["preparation_seconds"]["effective"] == 60
        for value in ["30", "86400"]:
            state = self.load("preparation-" + value, path, "--preparation-seconds", value)["energy"]
            assert state["loads"][0]["preparation_seconds"]["effective"] == int(value)
        for value in ["29", "86401"]:
            before = path.read_bytes()
            self.load("preparation-invalid-" + value, path, "--preparation-seconds", value, expected_exit=1)
            assert path.read_bytes() == before
        self.prove(["load-preparation-time"], start, "Omitted RT remained default60 until touched; 30/86400 seconds accepted,29/86401 rejected atomically.")

        start = len(self.commands)
        for value in ["2", "-1", "0"]:
            state = self.load("minimum-runtime-" + value.replace("-", "negative-"), path, "--min-runtime-minutes", value)["energy"]
            assert state["loads"][0]["min_runtime_seconds"]["effective"] == int(value) * 60
        assert state["loads"][0]["min_runtime_seconds"]["stored"] is None
        before = path.read_bytes()
        self.load("minimum-runtime-overflow", path, "--min-runtime-minutes", "2147483647", expected_exit=1)
        assert path.read_bytes() == before
        proposal = self.cli("minimum-runtime-dry-run", ["--dry-run", "config", "energy", "load", "set", path, "CLI Energy", FIRST, "--min-runtime-minutes", "-1", "--save-as", destination])
        assert proposal["energy"]["loads"][0]["min_runtime_seconds"]["effective"] == -60
        assert path.read_bytes() == before and not destination.exists()
        self.prove(["load-minimum-runtime"], start, "Integer2/-1/0 minutes saved120/-60/omitted0 seconds; signed overflow rejected; negative dry-run proposal did not mutate source/destination.")

        start = len(self.commands)
        for value in ["1440", "0", "1440"]:
            state = self.load("daily-" + value + "-" + str(len(self.commands)), path, "--daily-minutes", value)["energy"]
            assert state["loads"][0]["daily_seconds"]["effective"] == int(value) * 60
            if value == "0":
                assert state["loads"][0]["daily_seconds"]["stored"] is None
        for value in ["-1", "1441"]:
            before = path.read_bytes()
            self.load("daily-invalid-" + value.replace("-", "negative-"), path, "--daily-minutes", value, expected_exit=1)
            assert path.read_bytes() == before
        self.prove(["load-daily-minimum-runtime"], start, "Daily1440/0 minutes saved86400/omitted0 seconds; -1/1441 rejected atomically; no day-cycle runtime asserted.")

        start = len(self.commands)
        labels = ["sunset", "dusk", "sunrise", "dawn"] + [f"{hour:02}:00" for hour in range(1, 25)]
        for value, label in enumerate(labels):
            state = self.load("deadline-" + label.replace(":", "-"), path, "--deadline", label)["energy"]
            assert state["loads"][0]["deadline"]["effective"] == value
            assert state["loads"][0]["deadline_label"] == label
            assert state["loads"][0]["deadline_gui_verified"]
            if value == 0:
                assert state["loads"][0]["deadline"]["stored"] is None
        assert state["loads"][1]["deadline"]["stored"] == "27"
        assert state["loads"][1]["deadline_label"] == "24:00" and state["loads"][1]["deadline_gui_verified"]
        before = path.read_bytes()
        self.execute("deadline-invalid-hour", [self.lox, "--quiet", "config", "energy", "load", "set", str(path), "CLI Energy", FIRST, "--deadline", "00:00"], 2, False)
        assert path.read_bytes() == before
        self.prove(["load-daily-deadline"], start, "All28deadline labels mappedto0..27 with Sunset omission; untouchedtpR27 retained24:00; invalid00:00 rejected without writing. GUI-observed labels are not runtime execution proof.")

        start = len(self.commands)
        state = self.load("rated-four-zero-switch", path, "--rated-power", "4")["energy"]
        assert state["loads"][0]["switch_on_power"]["effective"] == 4.0
        state = self.load("switch-zero-substitution", path, "--rated-power", "2.5", "--switch-on-power", "0")["energy"]
        assert state["loads"][0]["switch_on_power"]["effective"] == 2.5
        state = self.load("switch-above-rated", path, "--switch-on-power", "2.6")["energy"]
        assert state["loads"][0]["rated_power"]["effective"] == 2.5
        assert state["loads"][0]["switch_on_power"]["effective"] == 2.6
        state = self.load("rated-zero-connected", path, "--rated-power", "0")["energy"]
        assert state["loads"][0]["rated_power"]["stored"] is None
        assert state["loads"][0]["used"]["stored"] == "true" and state["loads"][0]["unused_feedback"]
        assert state["loads"][0]["switch_on_power"]["effective"] == 2.6
        assert state["loads"][2]["used"]["stored"] is None and state["loads"][2]["unused_feedback"]
        before = path.read_bytes()
        self.load("compound-power-error", path, "--name", "discard", "--rated-power", "8", "--switch-on-power", "-1", expected_exit=1)
        assert path.read_bytes() == before
        for flag in ["--rated-power", "--switch-on-power"]:
            for sign, value in [("positive", TINY), ("negative", "-" + TINY)]:
                before = path.read_bytes()
                result = self.load(flag[2:] + "-underflow-" + sign, path, "--name", "discard", flag, value, expected_exit=1)
                assert result["error"] == "numeric_underflow" and path.read_bytes() == before
        self.prove(["load-switch-on-power", "load-rated-power", "load-output-used", "unused-load-feedback"], start, "Rated4 zero-switch copy, switch0 substitution2.5, above-rated2.6 and explicit rated0/U=true preservation verified. Positive/negative rated/switch underflow rejected before any zero/default/substitution rule; compound failures leave source unchanged; storage-derived unused feedback verified.")

        start = len(self.commands)
        baseline = self.inspect("before-reorder", path)
        state = self.cli("move-priority-two", ["config", "energy", "load", "move", path, "CLI Energy", FIRST, "2"])["energy"]
        assert state["loads"][0]["id"] == SECOND and state["loads"][1]["id"] == FIRST
        for prior in baseline["loads"]:
            current = next(row for row in state["loads"] if row["id"] == prior["id"])
            assert current["attributes"] == prior["attributes"]
            assert current["child_elements"] == prior["child_elements"]
        assert state["loads"][1]["output"] == "L1" and state["loads"][1]["output_index"]["effective"] == 0
        text = path.read_text(encoding="utf-8-sig")
        assert '<Unknown value="nested"/>' in text and '<Interleaved sentinel="position"/>' in text
        assert 'Input="73000000-0000-0000-0000-000000000001"' in text
        assert state["loads"][1]["icon"] == "75000000-0000-0000-0000-000000000001"
        before = path.read_bytes()
        self.cli("move-dry-run", ["--dry-run", "config", "energy", "load", "move", path, "CLI Energy", FIRST, "12", "--save-as", destination])
        assert path.read_bytes() == before and not destination.exists()
        for label, modified, expected_error in [
            ("duplicate-id", (HERE / "cli-synthetic.xml").read_text().replace(SECOND, FIRST), "ambiguous_load_id"),
            ("duplicate-output", (HERE / "cli-synthetic.xml").read_text().replace('OIdx="1"', 'OIdx="0"'), "ambiguous_output_association"),
            ("sparse", (HERE / "cli-synthetic.xml").read_text().replace('<Load Id="74000000-0000-0000-0000-00000000000c" OIdx="11"/>', ""), "unsupported_load_shape"),
        ]:
            malformed = self.output / (label + ".Loxone")
            malformed.write_text(modified, encoding="utf-8")
            before_malformed = malformed.read_bytes()
            result = self.cli(label, ["config", "energy", "load", "move", malformed, "CLI Energy", FIRST, "2"], 1)
            assert result["error"] == expected_error and malformed.read_bytes() == before_malformed
        native_path = self.output / "native-ids.Loxone"
        native_bytes = (HERE / "synthetic.Loxone").read_bytes()
        native_path.write_bytes(native_bytes)
        native_id = "30000000-0000-0000-0000000000000001"
        native_second = "30000000-0000-0000-0000000000000002"
        rfc_alias = "30000000-0000-0000-0000-000000000001"
        block = "Synthetic Energy"
        native = self.cli("canonical-native-inspect", ["config", "energy", "inspect", native_path, block])["energy"]
        assert native["load_edit_supported"] and len(native["loads"]) == 12
        native = self.cli("canonical-native-edit", ["config", "energy", "load", "set", native_path, block,
            native_id, "--name", "Native CLI", "--min-runtime-minutes", "-1"])["energy"]
        assert native["loads"][0]["id"] == native_id and native["loads"][0]["min_runtime_seconds"]["effective"] == -60
        moved = self.cli("canonical-native-rfc-alias-move", ["config", "energy", "load", "move", native_path, block, rfc_alias, "2"])["energy"]
        assert moved["loads"][1]["id"] == native_id and moved["loads"][1]["output"] == "L1"
        for prior in native["loads"]:
            current = next(row for row in moved["loads"] if row["id"] == prior["id"])
            assert current["attributes"] == prior["attributes"]
        native_text = native_path.read_text(encoding="utf-8-sig")
        assert 'Input="23000000-0000-0000-0000000000000001"' in native_text
        assert 'PilotUnknown="retain"' in native_text and "PilotExtension" in native_text
        uppercase = self.cli("canonical-native-uppercase-alias", ["config", "energy", "load", "set", native_path, block,
            "30000000-0000-0000-000000000000000A", "--rated-power", "1"])["energy"]
        assert next(row for row in uppercase["loads"] if row["output"] == "L10")["id"] == "30000000-0000-0000-000000000000000a"
        malformed_selector = "3000000-00000-0000-0000000000000001"
        before = native_path.read_bytes()
        invalid = self.cli("canonical-native-malformed-selector", ["config", "energy", "load", "move",
            native_path, block, malformed_selector, "2"], 1)
        assert invalid["error"] == "invalid_load_selector" and native_path.read_bytes() == before
        for label, text, expected in [
            ("native-rfc-duplicate", native_bytes.decode("utf-8-sig").replace(native_second, rfc_alias), "ambiguous_load_id"),
            ("native-misplaced-dashes", native_bytes.decode("utf-8-sig").replace(native_id, malformed_selector), "invalid_load_id"),
        ]:
            malformed = self.output / (label + ".Loxone")
            malformed.write_text(text, encoding="utf-8")
            before = malformed.read_bytes()
            invalid = self.cli(label, ["config", "energy", "load", "move", malformed, block, native_id, "2"], 1)
            assert invalid["error"] == expected and malformed.read_bytes() == before
        self.prove(["ordered-load-records", "load-identity", "load-output-association"], start, "RFC fixture full12-record preservation/reorder/dry-run passed. Canonical native8-4-4-16 fixture edited and reordered via RFC alias; exact native IDs/OIdx/L1/icon/unknown/wire data preserved, uppercase selector matched lowercase native Id. Malformed dash placement and binary-equivalent native/RFC duplicates rejected atomically.")
        self.run_icons()

    def run_icons(self):
        start = len(self.commands)
        self.execute("icon-generator-tests",
            [sys.executable, "-m", "unittest", "discover", "-s", "scripts", "-p", "test_inventory_loxone_icons.py"],
            parse_json=False)
        metadata = json.loads((ROOT / "docs" / "schemas" / "loxone-icons.json").read_text())
        listed = self.cli("icon-list-json", ["config", "energy", "icons"])
        assert listed["source"] == "builtin" and listed["catalog"] == metadata["catalog"]
        assert listed["count"] == len(listed["icons"]) == 623
        assert {"native_id": "00000094-00ff-0000-0000000000000000", "filename": "electric-fan.svg"} in listed["icons"]
        assert not any(row["filename"] == "awning-close.svg" for row in listed["icons"])
        for output in ["table", "csv"]:
            for no_header in [False, True]:
                text = self.execute(f"icon-list-{output}-header-{not no_header}",
                    [self.lox, "--quiet", "--output", output, *(["--no-header"] if no_header else []),
                     "config", "energy", "icons"], parse_json=False)
                assert text.startswith("NAME" if output == "table" else "SOURCE,NAME,NATIVE_ID") == (not no_header)
                assert "electric-fan.svg" in text
                assert listed["catalog"]["archive_sha256"] in text
                if output == "csv":
                    assert len(text.splitlines()) == 623 + int(not no_header)

        path = self.fixture("icon-fields")
        baseline = self.inspect("icon-baseline", path)
        raw = baseline["loads"][0]["icon"]
        preserved = self.load("icon-unknown-unrelated-name-edit", path, "--name", "Unrelated")["energy"]
        assert preserved["loads"][0]["icon"] == raw
        assert preserved["loads"][0]["icon_resolution"]["status"] == "unresolved"
        for index, selection in enumerate([
            "electric-fan.svg", "builtin:electric-fan.svg",
            "00000094-00FF-0000-0000000000000000", "00000094-00FF-0000-0000-000000000000",
        ]):
            before = self.inspect("icon-before-selection-" + str(index), path)
            state = self.load("icon-selection-" + str(index), path, "--icon", selection)["energy"]
            expected = dict(before["loads"][0]["attributes"], ico="00000094-00ff-0000-0000000000000000")
            assert state["loads"][0]["attributes"] == expected
            assert state["loads"][1:] == before["loads"][1:]
            assert state["loads"][0]["icon_resolution"]["filename"] == "electric-fan.svg"
            assert state["builtin_icon_catalog"] == listed["catalog"]

        destination = self.output / "icon-must-not-overwrite.Loxone"
        destination.write_bytes(b"independent output sentinel, not XML")
        original_destination = destination.read_bytes()
        for index, (selection, expected_error) in enumerate([
            ("", "invalid_icon_selection"), ("builtin:", "invalid_icon_selection"),
            ("awning-close.svg", "unknown_builtin_icon"),
            ("0000001d-00ff-0000-0000000000000000", "unknown_builtin_icon"),
            ("000000a0-00ff-0000-0000000000000000", "unknown_builtin_icon"),
            ("0000094-000ff-0000-0000000000000000", "unknown_builtin_icon"),
            ("Electric-fan.svg", "unknown_builtin_icon"),
            ("00000000-0000-0000-0000000000000000", "explicit_icon_clear_required"),
            ("project:" + raw, "unsupported_custom_icon"), (raw, "unknown_builtin_icon"),
        ]):
            before = path.read_bytes()
            error = self.load("icon-invalid-" + str(index), path, "--name", "discard",
                "--rated-power", "8", "--icon", selection, "--save-as", destination, expected_exit=1)
            assert error["error"] == expected_error and path.read_bytes() == before
            assert destination.read_bytes() == original_destination
        self.execute("icon-mutually-exclusive-clear",
            [self.lox, "--quiet", "config", "energy", "load", "set", str(path),
             "CLI Energy", FIRST, "--icon", "electric-fan.svg", "--no-icon"], 2, False)
        assert path.read_bytes() == before

        source = self.fixture("icon-closure")
        custom = "75000000-0000-0000-0000-000000000001"
        caption = '<C Type="LoxCaption" CaptionType="5" SubType="5" U="caption">'
        icon = f'<C Type="Icon" U="{custom}" User="1" Ix="4" Icon="000102" Future="opaque"/>'
        definition = caption + icon + "</C>"
        text = source.read_bytes().decode("utf-8-sig").replace(
            f'<Load Id="{SECOND}" OIdx="1"', f'<Load Id="{SECOND}" OIdx="1" ico="{custom}"'
        ).replace("</ControlList>", definition + f'<C Type="StateV" U="other" ico="{custom}"/></ControlList>')
        source.write_bytes(b"\xef\xbb\xbf" + text.encode("utf-8"))
        original = source.read_bytes()
        before_state = self.inspect("icon-closure-before", source)
        output = self.output / "icon-save-as.Loxone"
        proposed = self.cli("icon-select-dry-run", ["--dry-run", "config", "energy", "load", "set",
            source, "CLI Energy", FIRST, "--room", "Pilot Room", "--icon", "electric-fan.svg", "--save-as", output])
        assert proposed["saved_to"] is None and proposed["energy"]["loads"][0]["icon_resolution"]["status"] == "known_builtin"
        assert source.read_bytes() == original and not output.exists()
        selected = self.cli("icon-select-save-as", ["config", "energy", "load", "set",
            source, "CLI Energy [Pilot Room]", FIRST, "--icon", "electric-fan.svg", "--save-as", output])
        assert source.read_bytes() == original
        saved = output.read_bytes()
        proposal = self.cli("icon-clear-dry-run", ["--dry-run", "config", "energy", "load", "set",
            output, "CLI Energy", FIRST, "--no-icon"])
        assert proposal["energy"]["loads"][0]["icon"] is None and output.read_bytes() == saved
        cleared = self.load("icon-explicit-clear", output, "--no-icon")["energy"]
        expected = dict(before_state["loads"][0]["attributes"])
        del expected["ico"]
        assert cleared["loads"][0]["attributes"] == expected
        assert cleared["loads"][1:] == before_state["loads"][1:]
        assert cleared["loads"][0]["icon_resolution"]["status"] == "none"
        assert cleared["builtin_icon_catalog"] == selected["energy"]["builtin_icon_catalog"] == listed["catalog"]
        saved = output.read_bytes()
        assert saved.startswith(b"\xef\xbb\xbf") and b"\n" not in saved.replace(b"\r\n", b"")
        text = saved.decode("utf-8-sig")
        assert caption in text and icon in text and text.count(custom) == 3  # Definition + two other references.
        assert 'Input="73000000-0000-0000-0000-000000000001"' in text and '12hTF="true"' in text
        assert '<Unknown value="nested"/>' in text
        self.prove(["load-icon-reference"], start,
            "Actual builtin list JSON/table/CSV/header flags and independent archive version/hash passed; exact fan name/native/RFC aliases stored canonical native ID; disabled/absent/malformed/empty/zero/custom compound edits rejected without changing source/existing destination. Dry-run/save-as/room selectors and explicit clear retained Id/OIdx, children, wires, opaque custom definitions and other references. Synthetic ZIP generator tests passed. No GUI clear/custom-save/reopen, custom payload validation or full-scope/runtime parity is claimed.")

    def finish(self, failure=None):
        save_json(self.output / "commands.json", self.commands)
        save_json(self.output / "metadata.json", {
            "runner_id": self.id, "failure": failure,
            "cli_executable": self.lox, "cli_version": self.cli_version, "cli_sha256": digest(Path(self.lox)),
            "case_definitions": self.snapshot(self.before_definition),
            "source_artifacts": [
                self.snapshot(ROOT / path) for path in [
                    "src/config_edit/energy.rs", "src/config_edit/icons.rs", "src/config_edit/schema.rs", "src/config_edit/energy_tests.rs",
                    "src/config_edit/write.rs",
                    "src/commands/config_cmd/energy.rs", "tests/config_integration.rs",
                    "lox-sim/src/parser.rs", "tests/ux/energy-manager2/synthetic.Loxone",
                    "tests/ux/energy-manager2/cli-synthetic.xml", "tests/ux/energy-manager2/run-cli-cases.py",
                    "docs/schemas/loxone-icons.json", "scripts/inventory-loxone-icons.py", "scripts/test_inventory_loxone_icons.py",
                ]
            ],
            "fixture_origin": "independently authored synthetic XML; actual CLI/unit execution, no Config GUI",
            "binding_target_application": APPLICATION,
            "application_warning": "Application identity is the recovered binding target; that official executable is NOT executed by this CLI runner.",
        })
        context_artifacts = [
            self.artifact(self.output / "commands.json"),
            self.artifact(self.output / "metadata.json"),
            self.snapshot(self.before_definition),
            self.snapshot(HERE / "cli-synthetic.xml"),
        ]
        for observation in self.observations:
            observation["artifacts"].extend(context_artifacts)
            paths = [artifact["path"] for artifact in observation["artifacts"]]
            if len(paths) != len(set(paths)):
                raise ValueError("Duplicate artifact path in observation " + observation["id"])
        record = {
            "schema_version": 1, "id": self.id,
            "runner": "run-cli-cases.py: " + self.cli_version + ", actual offline CLI and cargo unit/integration execution",
            "started_at": self.started_at, "finished_at": datetime.now().astimezone().isoformat(),
            "application": APPLICATION, "synthetic": False, "kind": "cli_execution",
            "operation": "Actually execute the locally built lox CLI and named Cargo tests against disposable independently authored synthetic fixtures; record narrow implementation/unit outcomes only, including exact builtin list/select/clear. The application identity is the recovered Config binding TARGET, not an executed GUI. No GUI observations, past-run merging, server connection/write, custom mutation or runtime simulation. See adjacent commands.json and metadata.json for exact invocations/digests" + ("; failed: " + failure if failure else "."),
            "observations": self.observations,
        }
        save_json(self.output / "runner-record.json", record)
        print(json.dumps({
            "ok": failure is None, "runner_record": str((self.output / "runner-record.json").relative_to(ROOT)),
            "observations": len(self.observations), "commands": len(self.commands), "failure": failure,
        }, indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--definitions-only", action="store_true")
    parser.add_argument("--lox", type=Path, default=ROOT / "target" / "debug" / ("lox.exe" if os.name == "nt" else "lox"))
    parser.add_argument("--cargo", default="cargo")
    args = parser.parse_args()
    if args.definitions_only:
        print(write_definitions())
        return 0
    runner = Runner(args.lox.resolve(), args.cargo)
    try:
        runner.run()
    except Exception as error:
        runner.finish(f"{type(error).__name__}: {error}")
        return 1
    runner.finish()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
