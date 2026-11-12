use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
#[cfg(windows)]
use std::os::windows::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use assert_cmd::Command;
use httpmock::{Method::GET, MockServer};
use predicates::prelude::*;
use serde_json::Value;
use tempfile::TempDir;

const ENERGY_XML: &str = include_str!("ux/energy-manager2/cli-synthetic.xml");
const STAT_FILE: &str = "91000000-0000-0000-0000000000000001.202610.xml";

#[test]
fn report_preview_has_no_filesystem_or_network_side_effects() {
    let sandbox = Sandbox::new();
    let backend = sandbox.server.mock(|when, then| {
        when.any_request();
        then.status(500);
    });
    for spelling in [vec!["report"], vec!["feedback", "report"]] {
        let before = snapshot(sandbox.dir.path());
        let output = sandbox
            .command(&["--dry-run", "--json"])
            .env("PATH", sandbox.dir.path())
            .args(spelling)
            .args([
                "--title",
                "Synthetic issue",
                "--body",
                "Synthetic reproduction",
            ])
            .assert()
            .success()
            .stderr("")
            .get_output()
            .stdout
            .clone();
        let result: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(result["dry_run"], true);
        assert_eq!(snapshot(sandbox.dir.path()), before);
    }
    backend.assert_hits(0);
}

#[test]
fn lighting_sid_preview_has_no_filesystem_or_network_side_effects() {
    let sandbox = Sandbox::new();
    let root = sandbox.dir.path();
    let input = root.join("input.Loxone");
    fs::write(
        &input,
        "\u{feff}<ControlList>\r\n<C Type=\"LightController2\" U=\"lighting\"><LightscenesC Num=\"1\"><LightsceneC Name=\"A\" SID=\"+0001\" CID=\"12\"/></LightscenesC><LSConfig OpM=\"0\" ScMv=\"1,opaque\"/></C>\r\n</ControlList>\r\n",
    )
    .unwrap();
    let backend = sandbox.server.mock(|when, then| {
        when.any_request();
        then.status(500);
    });
    for sid in ["1", "32"] {
        for destination in [
            input.clone(),
            root.join("output.Loxone"),
            root.join("missing").join("new.Loxone"),
        ] {
            let before = snapshot(root);
            let result = sandbox
                .command(&["--dry-run", "--json"])
                .args(["config", "lighting", "set-mood-sid"])
                .arg(&input)
                .args([
                    "uuid:lighting",
                    "--mood",
                    "A",
                    "--sid",
                    sid,
                    "--allow-cid-clear",
                    "--save-as",
                ])
                .arg(&destination)
                .assert()
                .success()
                .stderr("")
                .get_output()
                .stdout
                .clone();
            let result: Value = serde_json::from_slice(&result).unwrap();
            assert_eq!(result["dry_run"], true);
            let after = snapshot(root);
            assert!(
                after == before,
                "SID preview wrote local data: {}",
                snapshot_difference(&before, &after)
            );
        }
    }
    backend.assert_hits(0);
}

#[test]
fn lighting_mix_preview_has_no_filesystem_or_network_side_effects() {
    let sandbox = Sandbox::new();
    let root = sandbox.dir.path();
    let input = root.join("input.Loxone");
    let backend = sandbox.server.mock(|when, then| {
        when.any_request();
        then.status(500);
    });
    for (attributes, flag, success) in [
        ("", "--no-mix", true),
        ("", "--mix", true),
        (" MixMod=\"tRuE\" CID=\"+000\"", "--no-mix", true),
        (" CID=\"12\"", "--no-mix", false),
        (" MixMod=\"0\"", "--mix", false),
    ] {
        fs::write(
            &input,
            format!("\u{feff}<ControlList>\r\n<C Type=\"Document\" U=\"doc\" Date=\"2025-01-01 00:00:00\" DateS=\"504921600\"/>\r\n<C Type=\"LightController2\" U=\"lighting\"><LightscenesC Num=\"opaque\"><LightsceneC Name=\"A\" SID=\"+0001\"{attributes}/></LightscenesC><LSConfig OpM=\"opaque\" ScMv=\"opaque\"/></C>\r\n</ControlList>\r\n"),
        ).unwrap();
        for destination in [
            input.clone(),
            root.join("output.Loxone"),
            root.join("missing").join("new.Loxone"),
        ] {
            let before = snapshot(root);
            let assertion = sandbox
                .command(&["--dry-run", "--json"])
                .args(["config", "lighting", "set-mood-mix"])
                .arg(&input)
                .args(["uuid:lighting", "--mood", "A", flag, "--save-as"])
                .arg(&destination)
                .assert();
            let assertion = if success {
                assertion.success()
            } else {
                assertion.failure()
            };
            let result: Value = serde_json::from_slice(&assertion.get_output().stdout).unwrap();
            if success {
                assert_eq!(result["dry_run"], true);
            } else {
                assert_eq!(result["ok"], false);
            }
            let after = snapshot(root);
            assert!(
                after == before,
                "MixMod preview wrote local data: {}",
                snapshot_difference(&before, &after)
            );
        }
    }
    backend.assert_hits(0);
}

#[test]
fn lighting_creation_preview_has_no_filesystem_or_network_side_effects() {
    let sandbox = Sandbox::new();
    let root = sandbox.dir.path();
    let input = root.join("input.Loxone");
    let output = root.join("output.Loxone");
    let backend = sandbox.server.mock(|when, then| {
        when.any_request();
        then.status(500);
    });
    for (collections, name, color, success) in [
        (
            "<LightscenesC Num=\"bad\" Outputs=\"future\"><LightsceneC Name=\"A\" SID=\"2\" CID=\"9\" Q1=\"opaque\" Q2=\"-1\"/></LightscenesC>",
            "New",
            "hsv(120,100,10)",
            true,
        ),
        ("<LightscenesC/>", "µ,\"B", "rgb(0,255,0)", true),
        ("<LightscenesC/><LightscenesC/>", "New", "0", false),
        (
            "<LightscenesC><LightsceneC Name=\"A\" SID=\"2\" Q2=\"opaque\"/></LightscenesC>",
            "New",
            "0",
            false,
        ),
        (
            "<LightscenesC><LightsceneC Name=\"A\" SID=\"2\"/></LightscenesC>",
            "A",
            "0",
            false,
        ),
        ("<LightscenesC/>", "New", "opaque", false),
    ] {
        fs::write(&input, format!(
            "\u{feff}<ControlList>\r\n<C Type=\"Document\" U=\"doc\" Date=\"2025-01-01 00:00:00\" DateS=\"504921600\"/>\r\n<C Type=\"LightController2\" U=\"lighting\" CapAI0=\"opaque\">{collections}<LSConfig OpM=\"opaque\" ScMv=\"opaque\"/><HCL T=\"2\"/></C>\r\n</ControlList>\r\n"
        )).unwrap();
        for destination in [
            None,
            Some(input.clone()),
            Some(output.clone()),
            Some(root.join("missing").join("new.Loxone")),
        ] {
            let before = snapshot(root);
            let mut command = sandbox.command(&["--dry-run", "--json"]);
            command.args(["config", "add-mood"]).arg(&input).args([
                "uuid:lighting",
                "--name",
                name,
                "--color",
                color,
            ]);
            if let Some(destination) = destination {
                command.arg("--save-as").arg(destination);
            }
            let assertion = command.assert();
            let assertion = if success {
                assertion.success().stderr("")
            } else {
                assertion.failure()
            };
            let result: Value = serde_json::from_slice(&assertion.get_output().stdout).unwrap();
            assert_eq!(result[if success { "dry_run" } else { "ok" }], success);
            let after = snapshot(root);
            assert!(
                after == before,
                "Creation preview wrote local data: {}",
                snapshot_difference(&before, &after)
            );
        }
    }
    backend.assert_hits(0);
}

#[test]
fn lighting_raw_color_preview_has_no_filesystem_or_network_side_effects() {
    let sandbox = Sandbox::new();
    let root = sandbox.dir.path();
    let input = root.join("input.Loxone");
    let output = root.join("output.Loxone");
    fs::write(&output, b"existing output").unwrap();
    let backend = sandbox.server.mock(|when, then| {
        when.any_request();
        then.status(500);
    });
    for (attribute, color, index, success) in [
        ("Q2=\"future\"", "hsv(120,100,10)", "2", true),
        ("Q2=\"1610622736\"", "1610622736", "2", true),
        ("Q2=\"+01610622736\"", "rgb(0,255,0)", "2", true),
        ("", "40100", "99", true),
        ("Q2=\"-2147483648\"", "-1000000000", "2", true),
        ("", "4294967295", "2", false),
        ("", "hsv(120,100,10)", "0", false),
        ("", "opaque", "0", false),
    ] {
        fs::write(
            &input,
            format!("\u{feff}<ControlList>\r\n<C Type=\"Document\" U=\"doc\" Date=\"2025-01-01 00:00:00\" DateS=\"504921600\"/>\r\n<C Type=\"LightController2\" U=\"lighting\" CapAI1=\"opaque\"><Unknown><LightsceneC Name=\"A\" SID=\"future\" CID=\"opaque\" Outputs=\"bad\" {attribute}/></Unknown><LightscenesC Num=\"bad\"><LightsceneC Name=\"A\" Q2=\"other\"/></LightscenesC><LSConfig OpM=\"opaque\" ScMv=\"opaque\"/><HCL T=\"2\"/></C>\r\n</ControlList>\r\n"),
        ).unwrap();
        for destination in [
            None,
            Some(input.clone()),
            Some(output.clone()),
            Some(root.join("missing").join("new.Loxone")),
        ] {
            let before = snapshot(root);
            let mut cmd = sandbox.command(&["--dry-run", "--json"]);
            cmd.args(["config", "set-mood-color"])
                .arg(&input)
                .args(["uuid:lighting", "--mood", "A", "--output-index", index])
                .arg(format!("--color={color}"));
            if let Some(destination) = destination {
                cmd.arg("--save-as").arg(destination);
            }
            let assertion = cmd.assert();
            let assertion = if success {
                assertion.success().stderr("")
            } else {
                assertion.failure()
            };
            let result: Value = serde_json::from_slice(&assertion.get_output().stdout).unwrap();
            if success {
                assert_eq!(result["dry_run"], true);
                assert_eq!(result["scope"], "raw_packed_edit");
            } else {
                assert_eq!(result["ok"], false);
            }
            let after = snapshot(root);
            assert!(
                after == before,
                "Raw color preview wrote local data: {}",
                snapshot_difference(&before, &after)
            );
        }
    }
    backend.assert_hits(0);
}

#[test]
fn lighting_numeric_level_preview_has_no_filesystem_or_network_side_effects() {
    let sandbox = Sandbox::new();
    let root = sandbox.dir.path();
    let input = root.join("input.Loxone");
    let backend = sandbox.server.mock(|when, then| {
        when.any_request();
        then.status(500);
    });
    for (raw, capability, level, success) in [
        ("1610622736", "", "0", true),
        ("1610622736", " CapAI1=\"+0032\"", "100", true),
        ("+01610612773", " CapAI1=\"+0006\"", "37", true),
        ("268435465", " CapAI1=\"opaque-unused\"", "37", true),
        ("1610622736", " CapAI1=\"7\"", "37", false),
        ("1610622736", " CapAI1=\"opaque\"", "37", false),
        ("1610622736", "", "101", false),
        ("future", "", "37", false),
    ] {
        fs::write(
            &input,
            format!("\u{feff}<ControlList>\r\n<C Type=\"Document\" U=\"doc\" Date=\"2025-01-01 00:00:00\" DateS=\"504921600\"/>\r\n<C Type=\"LightController2\" U=\"lighting\"{capability}><LightscenesC Num=\"opaque\"><LightsceneC Name=\"A\" SID=\"opaque\" CID=\"future\" MixMod=\"opaque\" SkipNext=\"opaque\" Outputs=\"18\" Q1=\"future\" Q2=\"{raw}\"/><LightsceneC Name=\"B\" Q2=\"-1\"/></LightscenesC><LSConfig OpM=\"opaque\" ScMv=\"opaque\"/><HCL T=\"2\"/></C>\r\n</ControlList>\r\n"),
        ).unwrap();
        for destination in [
            None,
            Some(input.clone()),
            Some(root.join("output.Loxone")),
            Some(root.join("missing").join("new.Loxone")),
        ] {
            let before = snapshot(root);
            let mut cmd = sandbox.command(&["--dry-run", "--json"]);
            cmd.args(["config", "lighting", "set-mood-level"])
                .arg(&input)
                .args([
                    "uuid:lighting",
                    "--mood",
                    "A",
                    "--output-index",
                    "2",
                    "--level",
                    level,
                ]);
            if let Some(destination) = destination {
                cmd.arg("--save-as").arg(destination);
            }
            let assertion = cmd.assert();
            let assertion = if success {
                assertion.success()
            } else {
                assertion.failure()
            };
            let result: Value = serde_json::from_slice(&assertion.get_output().stdout).unwrap();
            if success {
                assert_eq!(result["dry_run"], true);
            } else {
                assert_eq!(result["ok"], false);
            }
            let after = snapshot(root);
            assert!(
                after == before,
                "Numeric level preview wrote local data: {}",
                snapshot_difference(&before, &after)
            );
        }
    }
    backend.assert_hits(0);
}

#[test]
fn lighting_display_name_preview_has_no_filesystem_or_network_side_effects() {
    let sandbox = Sandbox::new();
    let root = sandbox.dir.path();
    let input = root.join("input.Loxone");
    let backend = sandbox.server.mock(|when, then| {
        when.any_request();
        then.status(500);
    });
    for (attribute, flags, success) in [
        ("", vec!["--clear"], true),
        (" SName=\"\"", vec!["--clear"], true),
        (" SName=\"alias\"", vec!["--clear"], true),
        (" SName=\"ABC\"", vec!["--name", "ABC"], true),
        (
            " SName=\"ABC\"",
            vec!["--name", "abc", "--normalize-ascii"],
            true,
        ),
        (" SName=\"alias\"", vec!["--name", "ABC"], true),
        (" SName=\"lower\"", vec!["--name", "lower"], false),
        (
            " SName=\"Grün\"",
            vec!["--name", "Grün", "--normalize-ascii"],
            false,
        ),
    ] {
        fs::write(
            &input,
            format!("\u{feff}<ControlList>\r\n<C Type=\"Document\" U=\"doc\" Date=\"2025-01-01 00:00:00\" DateS=\"504921600\"/>\r\n<C Type=\"LightController2\" U=\"lighting\"><LightscenesC Num=\"opaque\"><LightsceneC Name=\"A\" SID=\"opaque\" CID=\"future\" MixMod=\"opaque\" SkipNext=\"opaque\"{attribute}/></LightscenesC><LSConfig OpM=\"opaque\" ScMv=\"opaque\"/></C>\r\n</ControlList>\r\n"),
        ).unwrap();
        for destination in [
            None,
            Some(input.clone()),
            Some(root.join("output.Loxone")),
            Some(root.join("missing").join("new.Loxone")),
        ] {
            let before = snapshot(root);
            let mut cmd = sandbox.command(&["--dry-run", "--json"]);
            cmd.args(["config", "lighting", "set-mood-display-name"])
                .arg(&input)
                .args(["uuid:lighting", "--mood", "A"])
                .args(&flags);
            if let Some(destination) = destination {
                cmd.arg("--save-as").arg(destination);
            }
            let assertion = cmd.assert();
            let assertion = if success {
                assertion.success()
            } else {
                assertion.failure()
            };
            let result: Value = serde_json::from_slice(&assertion.get_output().stdout).unwrap();
            if success {
                assert_eq!(result["dry_run"], true);
            } else {
                assert_eq!(result["ok"], false);
            }
            let after = snapshot(root);
            assert!(
                after == before,
                "SName preview wrote local data: {}",
                snapshot_difference(&before, &after)
            );
        }
    }
    backend.assert_hits(0);
}

#[test]
fn lighting_rename_preview_has_no_filesystem_or_network_side_effects() {
    let sandbox = Sandbox::new();
    let root = sandbox.dir.path();
    let input = root.join("input.Loxone");
    let backend = sandbox.server.mock(|when, then| {
        when.any_request();
        then.status(500);
    });
    for (stored, reference, name, success) in [
        ("A", "A", "A", true),
        ("Grün &amp; &quot;A,B&quot;", "opaque", "New Name", true),
        ("Grün", "Grün", "Grün", false),
        ("A", "missing", "New", false),
    ] {
        fs::write(
            &input,
            format!("\u{feff}<ControlList>\r\n<C Type=\"Document\" U=\"doc\" Date=\"2025-01-01 00:00:00\" DateS=\"504921600\"/>\r\n<C Type=\"LightController2\" U=\"lighting\"><LightscenesC Num=\"opaque\"><LightsceneC Name=\"{stored}\" SID=\"opaque\" CID=\"future\" MixMod=\"opaque\" SkipNext=\"opaque\"/></LightscenesC><LSConfig OpM=\"opaque\" ScMv=\"opaque\"/></C>\r\n</ControlList>\r\n"),
        ).unwrap();
        for destination in [
            None,
            Some(input.clone()),
            Some(root.join("output.Loxone")),
            Some(root.join("missing").join("new.Loxone")),
        ] {
            let before = snapshot(root);
            let mut cmd = sandbox.command(&["--dry-run", "--json"]);
            cmd.args(["config", "lighting", "rename-mood"])
                .arg(&input)
                .args(["uuid:lighting", "--mood", reference, "--name", name]);
            if let Some(destination) = destination {
                cmd.arg("--save-as").arg(destination);
            }
            let assertion = cmd.assert();
            let assertion = if success {
                assertion.success()
            } else {
                assertion.failure()
            };
            let result: Value = serde_json::from_slice(&assertion.get_output().stdout).unwrap();
            if success {
                assert_eq!(result["dry_run"], true);
            } else {
                assert_eq!(result["ok"], false);
            }
            let after = snapshot(root);
            assert!(
                after == before,
                "Name preview wrote local data: {}",
                snapshot_difference(&before, &after)
            );
        }
    }
    backend.assert_hits(0);
}

#[test]
fn lighting_skip_preview_has_no_filesystem_or_network_side_effects() {
    let sandbox = Sandbox::new();
    let root = sandbox.dir.path();
    let input = root.join("input.Loxone");
    let backend = sandbox.server.mock(|when, then| {
        when.any_request();
        then.status(500);
    });
    for (sid, attributes, flag, success) in [
        ("+000777", "", "--no-skip", true),
        ("+000777", "", "--skip", true),
        ("32", " SkipNext=\"tRuE\" CID=\"12\"", "--no-skip", true),
        (
            "+0001",
            " SkipNext=\"FaLsE\" CID=\"opaque\"",
            "--no-skip",
            true,
        ),
        ("+0001", " SkipNext=\"0\"", "--skip", false),
        ("0", "", "--no-skip", false),
    ] {
        fs::write(
            &input,
            format!("\u{feff}<ControlList>\r\n<C Type=\"Document\" U=\"doc\" Date=\"2025-01-01 00:00:00\" DateS=\"504921600\"/>\r\n<C Type=\"LightController2\" U=\"lighting\"><LightscenesC Num=\"opaque\"><LightsceneC Name=\"A\" SID=\"{sid}\"{attributes}/></LightscenesC><LSConfig OpM=\"opaque\" ScMv=\"opaque\"/></C>\r\n</ControlList>\r\n"),
        ).unwrap();
        for destination in [
            None,
            Some(input.clone()),
            Some(root.join("output.Loxone")),
            Some(root.join("missing").join("new.Loxone")),
        ] {
            let before = snapshot(root);
            let mut cmd = sandbox.command(&["--dry-run", "--json"]);
            cmd.args(["config", "lighting", "set-mood-skip"])
                .arg(&input)
                .args(["uuid:lighting", "--mood", "A", flag]);
            if let Some(destination) = destination {
                cmd.arg("--save-as").arg(destination);
            }
            let assertion = cmd.assert();
            let assertion = if success {
                assertion.success()
            } else {
                assertion.failure()
            };
            let result: Value = serde_json::from_slice(&assertion.get_output().stdout).unwrap();
            if success {
                assert_eq!(result["dry_run"], true);
            } else {
                assert_eq!(result["ok"], false);
            }
            let after = snapshot(root);
            assert!(
                after == before,
                "SkipNext preview wrote local data: {}",
                snapshot_difference(&before, &after)
            );
        }
    }
    backend.assert_hits(0);
}

struct Sandbox {
    dir: TempDir,
    server: MockServer,
}

impl Sandbox {
    fn new() -> Self {
        let sandbox = Self {
            dir: tempfile::Builder::new()
                .prefix(".dry-run-cli-")
                .tempdir_in(env!("CARGO_MANIFEST_DIR"))
                .unwrap(),
            server: MockServer::start(),
        };
        let root = sandbox.dir.path();
        fs::create_dir_all(root.join(".lox").join("cache")).unwrap();
        fs::create_dir_all(root.join("home").join(".lox")).unwrap();
        fs::create_dir(root.join("repo")).unwrap();
        let config = format!(
            "host: {}\nuser: synthetic\npass: synthetic\nserial: SYNTHETIC\n\
             telemetry: true\nconfig_repo: '{}'\n",
            sandbox.server.base_url(),
            root.join("repo").display()
        );
        fs::write(root.join(".lox").join("config.yaml"), &config).unwrap();
        fs::write(root.join("home").join(".lox").join("config.yaml"), &config).unwrap();
        fs::write(root.join(".lox").join("cache").join("structure.json"), "{}").unwrap();
        fs::write(
            root.join(".lox").join("token.json"),
            r#"{"token":"synthetic","key":"00","valid_until":4102444800}"#,
        )
        .unwrap();
        fs::write(root.join("input.Loxone"), ENERGY_XML).unwrap();
        fs::write(root.join("malformed.Loxone"), "<ControlList><C").unwrap();
        fs::write(root.join("output.Loxone"), "keep existing output").unwrap();
        fs::write(root.join("repo").join("config.Loxone"), ENERGY_XML).unwrap();

        // Build a genuine, synthetic upload archive through the normal local path.
        sandbox
            .command(&[
                "--quiet",
                "config",
                "compress",
                "input.Loxone",
                "--save-as",
                "input.LoxCC",
            ])
            .env("LOX_TELEMETRY", "0")
            .assert()
            .success();
        let mut zip = zip::ZipWriter::new(fs::File::create(root.join("input.zip")).unwrap());
        zip.start_file("sps0.LoxCC", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(&fs::read(root.join("input.LoxCC")).unwrap())
            .unwrap();
        zip.finish().unwrap();
        sandbox
    }

    fn command(&self, args: &[&str]) -> Command {
        let root = self.dir.path();
        let mut command = Command::cargo_bin("lox").unwrap();
        command
            .current_dir(root)
            .env("LOX_CONFIG", root.join(".lox").join("config.yaml"))
            .env("LOX_HOME", root.join("home"))
            .env("HOME", root.join("home"))
            .env("USERPROFILE", root.join("home"))
            .env_remove("HOMEDRIVE")
            .env_remove("HOMEPATH")
            .env_remove("LOX_HOST")
            .env_remove("LOX_USER")
            .env_remove("LOX_PASS")
            .env_remove("LOX_SERIAL")
            .env("LOX_TELEMETRY", "1")
            // Catch accidental analytics requests locally, never at the real endpoint.
            .env("HTTP_PROXY", self.server.base_url())
            .env("HTTPS_PROXY", self.server.base_url())
            .env("ALL_PROXY", self.server.base_url())
            .env("NO_PROXY", "127.0.0.1,localhost")
            .timeout(Duration::from_secs(10))
            .args(args);
        command
    }

    fn assert_unsupported(&self, args: &[&str]) {
        let backend = self.server.mock(|when, then| {
            when.any_request();
            then.status(500);
        });
        let before = snapshot(self.dir.path());
        let output = self
            .command(&["--dry-run", "--json"])
            .args(args)
            .assert()
            .code(1)
            .stderr("")
            .get_output()
            .stdout
            .clone();
        let error: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(error["ok"], false, "{args:?}");
        assert_eq!(error["error"], "dry_run_not_supported", "{args:?}");
        assert!(
            error["message"]
                .as_str()
                .unwrap()
                .contains("inputs were not fully validated"),
            "{args:?}: {error}"
        );
        backend.assert_hits(0);
        let after = snapshot(self.dir.path());
        assert!(
            after == before,
            "{args:?} wrote local data: {}",
            snapshot_difference(&before, &after)
        );
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Entry {
    contents: Option<Vec<u8>>,
    modified: SystemTime,
    readonly: bool,
}

fn snapshot_difference(
    before: &BTreeMap<PathBuf, Entry>,
    after: &BTreeMap<PathBuf, Entry>,
) -> String {
    let paths: std::collections::BTreeSet<_> = before.keys().chain(after.keys()).collect();
    paths
        .into_iter()
        .filter_map(|path| match (before.get(path), after.get(path)) {
            (Some(old), Some(new)) if old != new => Some(format!(
                "{path:?}: contents_changed={}, modified={:?} -> {:?}, readonly={} -> {}",
                old.contents != new.contents,
                old.modified,
                new.modified,
                old.readonly,
                new.readonly
            )),
            (Some(_), None) => Some(format!("{path:?}: removed")),
            (None, Some(_)) => Some(format!("{path:?}: created")),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn snapshot(root: &Path) -> BTreeMap<PathBuf, Entry> {
    fn visit(root: &Path, path: &Path, entries: &mut BTreeMap<PathBuf, Entry>) {
        let mut options = fs::OpenOptions::new();
        options.read(true);
        #[cfg(windows)]
        options.custom_flags(0x0200_0000); // FILE_FLAG_BACKUP_SEMANTICS opens directories too.
        let mut file = options.open(path).unwrap();
        // Windows path queries can return stale directory-entry mtimes. Use the
        // same opened handle for both metadata and bytes, including directories.
        let metadata = file.metadata().unwrap();
        let contents = metadata.is_file().then(|| {
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).unwrap();
            bytes
        });
        entries.insert(
            path.strip_prefix(root).unwrap().to_path_buf(),
            Entry {
                contents,
                modified: metadata.modified().unwrap(),
                readonly: metadata.permissions().readonly(),
            },
        );
        if metadata.is_dir() {
            for entry in fs::read_dir(path).unwrap() {
                visit(root, &entry.unwrap().path(), entries);
            }
        }
    }
    let mut entries = BTreeMap::new();
    visit(root, root, &mut entries);
    entries
}

#[test]
fn snapshot_detects_timestamp_only_mutations() {
    let dir = tempfile::Builder::new()
        .prefix(".dry-run-snapshot-")
        .tempdir_in(env!("CARGO_MANIFEST_DIR"))
        .unwrap();
    let path = dir.path().join("structure.json");
    fs::write(&path, "{}").unwrap();
    let before = snapshot(dir.path());
    fs::File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(86_400))
        .unwrap();
    let after = snapshot(dir.path());
    let key = Path::new("structure.json");
    assert_eq!(before[key].contents, after[key].contents);
    assert_ne!(before[key].modified, after[key].modified);
    assert_ne!(before, after);
}

macro_rules! unsupported_deployment {
    ($name:ident, $($arg:expr),+ $(,)?) => {
        #[test]
        fn $name() {
            Sandbox::new().assert_unsupported(&[$($arg),+]);
        }
    };
}

unsupported_deployment!(
    dry_run_config_push_force_never_mutates,
    "config",
    "push",
    "input.Loxone",
    "--force",
    "--reboot"
);
unsupported_deployment!(
    dry_run_config_upload_force_never_mutates,
    "config",
    "upload",
    "input.zip",
    "--force"
);
unsupported_deployment!(
    dry_run_config_patch_force_never_mutates,
    "config",
    "patch",
    "--replace",
    "Alpha",
    "Beta",
    "--force",
    "--reboot"
);
unsupported_deployment!(
    dry_run_config_push_http_force_never_mutates,
    "config",
    "push-http",
    "input.zip",
    "--force"
);
unsupported_deployment!(
    dry_run_config_restore_force_never_mutates,
    "config",
    "restore",
    "HEAD",
    "--force"
);
unsupported_deployment!(
    dry_run_live_set_write_never_mutates,
    "live",
    "set",
    "91000000-0000-0000-0000000000000001",
    "1",
    "--write"
);

#[test]
fn dry_run_other_local_mutations_and_flag_positions_are_guarded() {
    let sandbox = Sandbox::new();
    for args in [
        vec!["cache", "clear"],
        vec!["cache", "refresh"],
        vec!["token", "clear"],
        vec!["token", "refresh"],
        vec!["token", "revoke"],
        vec!["telemetry", "enable"],
        vec!["ctx", "init"],
        vec!["completions", "bash", "--install"],
        vec!["config", "download", "--extract"],
        vec![
            "config",
            "set-param",
            "input.Loxone",
            "CLI Energy",
            "MinSoc",
            "20",
            "--save-as",
            "output.Loxone",
        ],
        vec![
            "config",
            "extract",
            "input.zip",
            "--save-as",
            "output.Loxone",
        ],
        vec!["config", "snapshot", "input.Loxone", "--before"],
        vec!["config", "report", "input.Loxone", "--open-issue"],
        vec!["live", "get", "unresolved name"],
    ] {
        sandbox.assert_unsupported(&args);
    }
    let before = snapshot(sandbox.dir.path());
    sandbox
        .command(&[
            "config",
            "push",
            "input.Loxone",
            "--force",
            "--dry-run",
            "--quiet",
        ])
        .assert()
        .code(1)
        .stdout("")
        .stderr(predicate::str::contains("dry-run not supported"));
    assert_eq!(snapshot(sandbox.dir.path()), before);
}

#[test]
fn dry_run_unsupported_inputs_do_not_claim_validation_or_success() {
    let sandbox = Sandbox::new();
    for file in ["missing.Loxone", "malformed.Loxone"] {
        sandbox.assert_unsupported(&["config", "push", file, "--force"]);
    }
    sandbox.assert_unsupported(&["live", "set", "", "", "--write"]);
    let before = snapshot(sandbox.dir.path());
    sandbox
        .command(&[
            "--dry-run",
            "config",
            "patch",
            "--replace",
            "incomplete",
            "--force",
        ])
        .assert()
        .code(2)
        .stdout("")
        .stderr(predicate::str::contains("values required"));
    assert_eq!(snapshot(sandbox.dir.path()), before);
}

#[test]
fn dry_run_energy_plan_validates_without_output_or_telemetry_writes() {
    let sandbox = Sandbox::new();
    let backend = sandbox.server.mock(|when, then| {
        when.any_request();
        then.status(500);
    });
    let before = snapshot(sandbox.dir.path());
    let new_output = Path::new("new").join("output.Loxone");
    for output_path in ["output.Loxone", new_output.to_str().unwrap()] {
        let result = sandbox
            .command(&[
                "--dry-run",
                "--json",
                "config",
                "energy",
                "set",
                "input.Loxone",
                "CLI Energy",
                "MinSoc",
                "50",
                "--save-as",
                output_path,
            ])
            .assert()
            .success()
            .stderr("")
            .get_output()
            .stdout
            .clone();
        let plan: Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(plan["ok"], true);
        assert_eq!(plan["dry_run"], true);
        assert!(plan["saved_to"].is_null());
        let scalars = plan["energy"]["scalar_parameters"].as_array().unwrap();
        assert!(
            scalars
                .iter()
                .any(|scalar| { scalar["key"] == "MinSoc" && scalar["value"]["stored"] == "50" })
        );
        assert_eq!(snapshot(sandbox.dir.path()), before);
    }
    backend.assert_hits(0);
}

#[test]
fn dry_run_energy_invalid_inputs_fail_without_writes() {
    let sandbox = Sandbox::new();
    let backend = sandbox.server.mock(|when, then| {
        when.any_request();
        then.status(500);
    });
    let before = snapshot(sandbox.dir.path());
    for (file, selector, value) in [
        ("input.Loxone", "CLI Energy", "101"),
        ("input.Loxone", "not a block", "50"),
        ("malformed.Loxone", "CLI Energy", "50"),
        ("missing.Loxone", "CLI Energy", "50"),
    ] {
        let result = sandbox
            .command(&[
                "--dry-run",
                "--json",
                "config",
                "energy",
                "set",
                file,
                selector,
                "MinSoc",
                value,
                "--save-as",
                "output.Loxone",
            ])
            .assert()
            .code(1)
            .get_output()
            .stdout
            .clone();
        let error: Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(error["ok"], false);
        assert_ne!(error["error"], "dry_run_not_supported");
        assert_eq!(snapshot(sandbox.dir.path()), before);
    }
    backend.assert_hits(0);
}

#[test]
fn dry_run_stats_download_force_only_reads_index_and_preserves_output_formats() {
    let sandbox = Sandbox::new();
    let index = sandbox.server.mock(|when, then| {
        when.method(GET).path("/stats/");
        then.status(200).body(format!(
            "<li><a href=\"{STAT_FILE}\">Synthetic Statistic 202610</a></li>"
        ));
    });
    let files = sandbox.server.mock(|when, then| {
        when.path(format!("/stats/{STAT_FILE}"));
        then.status(500);
    });
    let before = snapshot(sandbox.dir.path());
    for format in ["json", "csv", "table"] {
        let output = sandbox
            .command(&[
                "--dry-run",
                "-o",
                format,
                "stats",
                "download",
                "--dir",
                "stats",
                "--force",
            ])
            .assert()
            .success()
            .stderr("")
            .get_output()
            .stdout
            .clone();
        let text = String::from_utf8(output).unwrap();
        match format {
            "json" => {
                let plan: Value = serde_json::from_str(&text).unwrap();
                assert_eq!(plan["dry_run"], true);
                assert_eq!(plan["would_download"], serde_json::json!([STAT_FILE]));
            }
            "csv" => assert_eq!(text, format!("FILE,STATUS\n{STAT_FILE},would_download\n")),
            _ => {
                assert!(text.contains(STAT_FILE));
                assert!(text.contains("1 would be downloaded, 0 skipped (dry run)"));
            }
        }
        assert_eq!(snapshot(sandbox.dir.path()), before);
    }
    index.assert_hits(3);
    files.assert_hits(0);
}

#[test]
fn dry_run_stats_filters_and_named_reads_preserve_cold_contexts() {
    let sandbox = Sandbox::new();
    let root = sandbox.dir.path();
    fs::remove_dir_all(root.join(".lox").join("cache")).unwrap();
    fs::write(
        root.join(".lox").join("token.json"),
        r#"{"token":"expired-synthetic","key":"00","valid_until":0}"#,
    )
    .unwrap();
    // Exercise normal disk-cache eligibility, not the client's localhost exemption.
    let host = sandbox.server.base_url().replace("127.0.0.1", "2130706433");
    assert!(!host.contains("127.0.0.1") && !host.contains("localhost"));
    assert_eq!(
        reqwest::Url::parse(&host).unwrap().host_str(),
        Some("127.0.0.1")
    );
    fs::write(
        root.join(".lox").join("config.yaml"),
        format!("host: {host}\nuser: synthetic\npass: synthetic\ntelemetry: true\n"),
    )
    .unwrap();
    let cold_context = root.join("home").join(".lox").join("contexts").join("cold");
    fs::create_dir_all(&cold_context).unwrap();
    let cold_config = cold_context.join("config.yaml");
    fs::copy(root.join(".lox").join("config.yaml"), &cold_config).unwrap();
    assert!(!cold_context.join("cache").exists());
    assert!(!cold_context.join("token.json").exists());

    let file = STAT_FILE.replace(".202610", "_2.202610");
    let previous = file.replace(".202610", ".202609");
    let other = "92000000-0000-0000-0000000000000001.202610.xml";
    let index = sandbox.server.mock(|when, then| {
        when.method(GET).path("/stats/");
        then.status(200).body(format!(
            "<li><a href=\"{file}\">Synthetic Statistic (Statistic Group: 2) 202610</a></li>\
             <li><a href=\"{previous}\">Synthetic Statistic (Statistic Group: 2) 202609</a></li>\
             <li><a href=\"{other}\">Other Statistic 202610</a></li>"
        ));
    });
    let series = sandbox.server.mock(|when, then| {
        when.method(GET).path(format!("/stats/{file}"));
        then.status(200).body(
            r#"<Statistics Name="Synthetic Statistic" StatsGroup="2" Outputs="Value">
                <S T="2026-10-01 00:00:00" V="7"/>
               </Statistics>"#,
        );
    });
    let structure = sandbox.server.mock(|when, then| {
        when.path("/data/LoxApp3.json");
        then.status(200).json_body(serde_json::json!({}));
    });
    let auth = sandbox.server.mock(|when, then| {
        when.path_contains("/jdev/sys/");
        then.status(400);
    });
    let unexpected = sandbox.server.mock(|when, then| {
        when.any_request();
        then.status(400);
    });
    let before = snapshot(root);
    for context in [None, Some("cold")] {
        for action in [
            vec![
                "download",
                "--dir",
                "stats",
                "--filter",
                "synthetic",
                "--from",
                "2026-10",
                "--to",
                "2026-10",
                "--force",
            ],
            vec![
                "get",
                "synthetic statistic",
                "--group",
                "2",
                "--from",
                "2026-10",
                "--to",
                "2026-10",
            ],
            vec![
                "ls",
                "--filter",
                "synthetic",
                "--month",
                "2026-10",
                "--latest",
            ],
        ] {
            let mut command = sandbox.command(&["--dry-run", "--json"]);
            if let Some(context) = context {
                // LOX_CONFIG overrides --ctx and safely isolates Windows known-folder lookup.
                command.env("LOX_CONFIG", &cold_config);
                command.args(["--ctx", context]);
            }
            let output = command
                .arg("stats")
                .args(&action)
                .assert()
                .success()
                .stderr("")
                .get_output()
                .stdout
                .clone();
            let result: Value = serde_json::from_slice(&output).unwrap();
            match action[0] {
                "download" => assert_eq!(result["would_download"], serde_json::json!([file])),
                "get" => assert_eq!(result["rows"][0]["values"]["Value"], 7.0),
                _ => assert_eq!(result.as_array().unwrap().len(), 1),
            }
            assert_eq!(snapshot(root), before, "{context:?}: {action:?}");
            assert!(!cold_context.join("cache").exists());
            assert!(!cold_context.join("token.json").exists());
        }
    }
    sandbox
        .command(&[
            "--dry-run",
            "--json",
            "config",
            "controls",
            "input.Loxone",
            "--room",
            "Pilot Room",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("CLI Energy"))
        .stderr("");
    assert_eq!(snapshot(root), before);
    index.assert_hits(6);
    series.assert_hits(2);
    structure.assert_hits(0);
    auth.assert_hits(0);
    unexpected.assert_hits(0);
}

#[test]
fn dry_run_read_only_commands_validate_inputs_without_startup_writes() {
    let sandbox = Sandbox::new();
    let before = snapshot(sandbox.dir.path());
    sandbox
        .command(&["--dry-run", "color", "encode", "--rgb", "100,40,0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("40100"))
        .stderr("");
    sandbox
        .command(&["--dry-run", "color", "encode", "--rgb", "not-a-color"])
        .assert()
        .failure()
        .stdout("")
        .stderr(predicate::str::contains("dry-run not supported").not());
    assert_eq!(snapshot(sandbox.dir.path()), before);
}
