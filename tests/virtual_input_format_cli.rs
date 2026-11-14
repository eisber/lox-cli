use std::{fs, path::Path};

use quick_xml::{Reader, events::Event};

const OWNER: &str = "10000000-0000-0000-0000000000000004";

fn fixture() -> String {
    format!(
        "\u{feff}<LoxoneApp>\r\n<C Type=\"Document\" V=\"175\" U=\"10000000-0000-0000-0000000000000001\" DateS=\"1\"><C Type=\"LoxLIVE\" V=\"175\" U=\"10000000-0000-0000-0000000000000002\" WF=\"16400\"><C Type=\"VirtualInCaption\" V=\"175\" U=\"10000000-0000-0000-0000000000000003\" WF=\"16384\"><C Type=\"VirtualIn\" V=\"175\" U=\"{OWNER}\" WF=\"16400\" Analog=\"true\" Nio=\"2\" IName=\"VI1\"><Co K=\"Q\" U=\"10000000-0000-0000-0000000000000005\"/><Co K=\"Qm\" U=\"10000000-0000-0000-0000000000000006\"/><IoData Visu=\"true\"/><Display Unit=\"&lt;v&gt;\" StateOnly=\"true\"/></C></C></C></C>\r\n</LoxoneApp>\r\n"
    )
}

fn command(input: &Path, uuid: &str, format: &str) -> assert_cmd::Command {
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("lox");
    command
        .env("LOX_TELEMETRY", "0")
        .args(["config", "set-virtual-input-format"])
        .arg(input)
        .arg(format!("uuid:{uuid}"))
        .arg(format);
    command
}

#[test]
fn virtual_input_format_cli_outputs_noop_and_atomic_refusals() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let existing = dir.path().join("existing.Loxone");
    let new = dir.path().join("new.Loxone");
    let source = fixture();
    fs::write(&input, &source).unwrap();
    let output = command(&input, OWNER, "<v.1>hPa")
        .args(["-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(result["virtual_input_format"]["changed"], true);
    assert_eq!(result["virtual_input_format"]["previous_format"], "<v>");
    let once = fs::read(&input).unwrap();
    command(&input, OWNER, "<v.1>hPa")
        .args(["-q", "--save-as"])
        .arg(&existing)
        .assert()
        .success()
        .stdout("");
    assert_eq!(fs::read(&existing).unwrap(), once);
    command(&input, OWNER, "<v.1>hPa")
        .args(["-o", "csv", "--no-header"])
        .assert()
        .success()
        .stdout(format!("{OWNER},\"<v.1>hPa\",\"<v.1>hPa\",false\n"));
    assert_eq!(fs::read(&input).unwrap(), once);
    for (xml, value) in [
        (source.replace("Analog=\"true\"", "Analog=\"false\""), "<v>"),
        (source.replace("DateS=\"1\"", "DateS=\"2147483647\""), "<v>"),
        (source.clone(), "<v.col>"),
        (source.clone(), ""),
        (
            source.replace(
                "Unit=\"&lt;v&gt;\"",
                "Unit=\"&lt;v&gt;\" Unit=\"&lt;v&gt;\"",
            ),
            "<v>",
        ),
    ] {
        fs::write(&input, &xml).unwrap();
        fs::write(&existing, b"preserve").unwrap();
        for destination in [None, Some(&existing), Some(&new)] {
            let mut edit = command(&input, OWNER, value);
            if let Some(destination) = destination {
                edit.arg("--save-as").arg(destination);
            }
            edit.assert().failure();
            assert_eq!(fs::read(&input).unwrap(), xml.as_bytes());
            assert_eq!(fs::read(&existing).unwrap(), b"preserve");
            assert!(!new.exists());
        }
    }
    command(&input, OWNER, "<v>")
        .args(["--dry-run", "--save-as"])
        .arg(&new)
        .assert()
        .failure()
        .stderr(predicates::str::contains("dry-run not supported"));
    assert!(!new.exists());
}

#[test]
fn virtual_input_format_edits_existing_native_golden_inputs() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("golden.Loxone");
    let configs = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("eval")
        .join("golden")
        .join("configs");
    let mut admitted = 0;
    let mut refused = 0;
    for path in fs::read_dir(configs).unwrap() {
        let path = path.unwrap().path();
        if path
            .extension()
            .is_none_or(|extension| extension != "Loxone")
        {
            continue;
        }
        let original = fs::read(&path).unwrap();
        let mut reader = Reader::from_reader(original.as_slice());
        let mut owners = Vec::new();
        loop {
            match reader.read_event().unwrap() {
                Event::Start(element) | Event::Empty(element)
                    if element.name().as_ref() == b"C" =>
                {
                    let attributes: std::collections::BTreeMap<_, _> = element
                        .attributes()
                        .with_checks(false)
                        .map(|attribute| {
                            let attribute = attribute.unwrap();
                            (
                                attribute.key.as_ref().to_vec(),
                                attribute.unescape_value().unwrap().into_owned(),
                            )
                        })
                        .collect();
                    if attributes.get(b"Type".as_slice()).map(String::as_str) == Some("VirtualIn")
                        && attributes.get(b"Analog".as_slice()).map(String::as_str) == Some("true")
                    {
                        owners.push(attributes[b"U".as_slice()].clone());
                    }
                }
                Event::Eof => break,
                _ => {}
            }
        }
        for owner in owners {
            fs::write(&input, &original).unwrap();
            let result = command(&input, &owner, "<v.1>hPa")
                .arg("-q")
                .output()
                .unwrap();
            if result.status.success() {
                admitted += 1;
                let saved = fs::read(&input).unwrap();
                assert_ne!(saved, original);
                command(&input, &owner, "<v.1>hPa")
                    .arg("-q")
                    .assert()
                    .success()
                    .stdout("");
                assert_eq!(fs::read(&input).unwrap(), saved);
            } else {
                refused += 1;
                assert!(String::from_utf8_lossy(&result.stderr).contains("Virtual Input format"));
                assert_eq!(fs::read(&input).unwrap(), original);
            }
        }
    }
    eprintln!(
        "Existing native analog VirtualIn format edits: {admitted} admitted, {refused} refused"
    );
    assert!(
        admitted > 0,
        "the admitted branch must work on existing real golden inputs"
    );
}
