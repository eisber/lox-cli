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
fn virtual_input_format_branch_unicode_preserves_literal_bytes_and_byte_limit() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    for value in [
        "<v.1>°C".to_owned(),
        "<v>µg/m³".to_owned(),
        "<v.2>μg/m²".to_owned(),
        "<v.3>€/kWh".to_owned(),
        "<v>°²³µμ€".to_owned(),
        format!("<v>{}€", "x".repeat(122)),
    ] {
        let source = fixture();
        fs::write(&input, &source).unwrap();
        let output = command(&input, OWNER, &value)
            .args(["-o", "json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let result: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(result["virtual_input_format"]["format"], value);
        let saved = fs::read_to_string(&input).unwrap();
        let document = |text: &str| {
            let start = text.find("<C Type=\"Document\"").unwrap();
            let end = start + text[start..].find('>').unwrap() + 1;
            text[start..end].to_owned()
        };
        assert_eq!(
            saved,
            source
                .replacen(&document(&source), &document(&saved), 1)
                .replace(
                    "Unit=\"&lt;v&gt;\"",
                    &format!("Unit=\"{}\"", quick_xml::escape::escape(&value)),
                )
        );
        command(&input, OWNER, &value)
            .args(["-o", "csv", "--no-header"])
            .assert()
            .success()
            .stdout(format!("{OWNER},\"{value}\",\"{value}\",false\n"));
        assert_eq!(fs::read(&input).unwrap(), saved.as_bytes());
    }
    let original = fixture();
    fs::write(&input, &original).unwrap();
    for value in [
        format!("<v>{}€", "x".repeat(123)),
        "<v>\u{00a0}".to_owned(),
        "<v>Μg".to_owned(),
        "<v>m¹".to_owned(),
        "<v>e\u{0301}".to_owned(),
        "<v>😀".to_owned(),
        "<v>°\u{007f}".to_owned(),
    ] {
        command(&input, OWNER, &value).assert().failure();
        assert_eq!(fs::read(&input).unwrap(), original.as_bytes());
    }
}

#[test]
fn virtual_input_format_branch_explicit_clear_omits_unit_and_preserves_noops() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let existing = dir.path().join("existing.Loxone");
    let new = dir.path().join("new.Loxone");
    for state in ["", "StateOnly=\"true\"", "StateOnly=\"false\""] {
        let original = fixture().replace("StateOnly=\"true\"", state);
        fs::write(&input, &original).unwrap();
        let output = command(&input, OWNER, "--clear")
            .args(["-o", "json", "--save-as"])
            .arg(&existing)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let result: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(result["virtual_input_format"]["previous_format"], "<v>");
        assert_eq!(result["virtual_input_format"]["format"], "");
        assert_eq!(result["virtual_input_format"]["changed"], true);
        assert_eq!(fs::read(&input).unwrap(), original.as_bytes());
        let saved = fs::read_to_string(&existing).unwrap();
        let document = |text: &str| {
            let start = text.find("<C Type=\"Document\"").unwrap();
            let end = start + text[start..].find('>').unwrap() + 1;
            text[start..end].to_owned()
        };
        assert_eq!(
            saved,
            original
                .replacen(&document(&original), &document(&saved), 1)
                .replace("Unit=\"&lt;v&gt;\"", "")
        );
        for noop in [
            original.replace("Unit=\"&lt;v&gt;\"", ""),
            original.replace("Unit=\"&lt;v&gt;\"", "Unit=''"),
        ] {
            fs::write(&input, &noop).unwrap();
            command(&input, OWNER, "--clear")
                .args(["-o", "csv", "--no-header"])
                .assert()
                .success()
                .stdout(format!("{OWNER},\"\",\"\",false\n"));
            assert_eq!(fs::read(&input).unwrap(), noop.as_bytes());
            command(&input, OWNER, "--clear")
                .args(["-q", "--save-as"])
                .arg(&existing)
                .assert()
                .success()
                .stdout("");
            assert_eq!(fs::read(&existing).unwrap(), noop.as_bytes());
        }
    }
    let source = fixture();
    for malformed in [
        source.replace("DateS=\"1\"", "DateS=\"2147483647\""),
        format!("{source}<C/>"),
        source.replace("Analog=\"true\"", "Analog=\"false\""),
        source.replace("Unit=\"&lt;v&gt;\"", "Unit='' Unit=''"),
    ] {
        fs::write(&input, &malformed).unwrap();
        fs::write(&existing, b"preserve").unwrap();
        for destination in [None, Some(&existing), Some(&new)] {
            let mut edit = command(&input, OWNER, "--clear");
            if let Some(destination) = destination {
                edit.arg("--save-as").arg(destination);
            }
            edit.assert().failure().stdout("");
            assert_eq!(fs::read(&input).unwrap(), malformed.as_bytes());
            assert_eq!(fs::read(&existing).unwrap(), b"preserve");
            assert!(!new.exists());
        }
    }
    fs::write(&input, &source).unwrap();
    command(&input, OWNER, "").assert().failure().stdout("");
    command(&input, OWNER, "<v>")
        .arg("--clear")
        .assert()
        .failure();
    assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
}

#[test]
fn virtual_input_format_branch_accepts_unmodified_cli_created_input() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("created.Loxone");
    fs::write(&input, fixture()).unwrap();
    assert_cmd::cargo::cargo_bin_cmd!("lox")
        .env("LOX_TELEMETRY", "0")
        .args(["config", "add-virtual-in"])
        .arg(&input)
        .args(["Meter", "--analog", "--unit", "<v>", "-q"])
        .assert()
        .success();
    let created = fs::read(&input).unwrap();
    let mut reader = Reader::from_reader(created.as_slice());
    let owner = loop {
        match reader.read_event().unwrap() {
            Event::Start(element) if element.name().as_ref() == b"C" => {
                let attributes: std::collections::BTreeMap<_, _> = element
                    .attributes()
                    .map(|attribute| {
                        let attribute = attribute.unwrap();
                        (
                            attribute.key.as_ref().to_vec(),
                            attribute.unescape_value().unwrap().into_owned(),
                        )
                    })
                    .collect();
                if attributes.get(b"Title".as_slice()).map(String::as_str) == Some("Meter") {
                    assert_eq!(attributes[b"WF".as_slice()], "16400");
                    break attributes[b"U".as_slice()].clone();
                }
            }
            Event::Eof => panic!("missing CLI-created input"),
            _ => {}
        }
    };
    for value in ["<v.1>°C", "--clear"] {
        command(&input, &owner, value)
            .arg("-q")
            .assert()
            .success()
            .stdout("");
    }
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
fn virtual_input_format_single_root_refuses_hidden_roots_and_trailing_content() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let existing = dir.path().join("existing.Loxone");
    let new = dir.path().join("new.Loxone");
    let source = fixture();
    let cases = [
        format!("{source}<C Type=\"Document\"/>"),
        format!("{source}<C Type=\"VirtualIn\" U=\"{OWNER}\"/>"),
        format!("{source}<C Type=\"VirtualIn\" U=\"10000000-0000-0000-0000-000000000004\"></C>"),
        format!("{source}outside"),
        format!("{source}&amp;"),
        format!("{source}<![CDATA[outside]]>"),
        format!("{source}<![CDATA[ \t\r\n]]>"),
        format!("{source}<C>"),
        source.replace("</LoxoneApp>", ""),
        source.replace("</LoxoneApp>", "</Wrong>"),
        format!("{source}</LoxoneApp>"),
        format!("{source}<!--bad--comment-->"),
        format!("{source}<!--truncated"),
        format!("{source}<?truncated"),
        format!("{source}<?xml version=\"1.0\"?>"),
    ];
    for (index, xml) in cases.iter().enumerate() {
        fs::write(&input, xml).unwrap();
        fs::write(&existing, b"preserve").unwrap();
        for value in ["<v>", "<v.1>hPa"] {
            for destination in [None, Some(&existing), Some(&new)] {
                let mut edit = command(&input, OWNER, value);
                if let Some(destination) = destination {
                    edit.arg("--save-as").arg(destination);
                }
                let output = edit.output().unwrap();
                assert!(!output.status.success(), "malformed XML case {index}");
                assert!(
                    output.stdout.is_empty(),
                    "success-shaped output in case {index}"
                );
                assert_eq!(fs::read(&input).unwrap(), xml.as_bytes());
                assert_eq!(fs::read(&existing).unwrap(), b"preserve");
                assert!(!new.exists());
            }
        }
    }
}

#[test]
fn virtual_input_format_single_root_preserves_document_markup_and_safe_cdata() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let wrapper = fixture();
    let document = wrapper
        .trim_start_matches('\u{feff}')
        .replace("<LoxoneApp>\r\n", "")
        .replace("</LoxoneApp>\r\n", "");
    for body in [wrapper.trim_start_matches('\u{feff}').to_owned(), document] {
        let body = body.replace(
            "<IoData",
            "<Evidence><![CDATA[<C Type='Document'/> & <fake/>]]>&amp;&#32;<Empty/></Evidence><IoData",
        );
        let xml = format!(
            "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n<!-- <C Type='Document'/> --><?before <fake/>?>\r\n{body}<?after <fake/>?><!-- trailing <C/> --> \t\r\n"
        );
        fs::write(&input, &xml).unwrap();
        command(&input, OWNER, "<v.1>hPa")
            .arg("-q")
            .assert()
            .success()
            .stdout("");
        let saved = fs::read_to_string(&input).unwrap();
        let doc_tag = |text: &str| {
            let start = text.find("<C Type=\"Document\"").unwrap();
            let end = start + text[start..].find('>').unwrap() + 1;
            text[start..end].to_owned()
        };
        let expected = xml.replacen(&doc_tag(&xml), &doc_tag(&saved), 1).replacen(
            "Unit=\"&lt;v&gt;\"",
            "Unit=\"&lt;v.1&gt;hPa\"",
            1,
        );
        assert_eq!(saved, expected);
        command(&input, OWNER, "<v.1>hPa")
            .arg("-q")
            .assert()
            .success()
            .stdout("");
        assert_eq!(fs::read(&input).unwrap(), expected.as_bytes());
    }
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
    let mut fixtures = 0;
    let mut fixtures_with_inputs = 0;
    let mut admitted = 0;
    let mut sanitizer_refused = 0;
    let mut flags_refused = 0;
    for path in fs::read_dir(configs).unwrap() {
        let path = path.unwrap().path();
        if !path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("loxone"))
        {
            continue;
        }
        fixtures += 1;
        let filename = path.file_name().unwrap().to_str().unwrap();
        let expected_refusal = match filename {
            "UseCase-wallbox-charging-modes.loxone"
            | "UseCase_espresso-machine-preheat-alarm.Loxone"
            | "usecase-keyless-locker-management.Loxone" => {
                Some("shared-loader digit-name rewrites (lox-n37w)")
            }
            "Netzgebu\u{0308}hr reduziert.Loxone" => {
                Some("requires explicit modern V>=90 and ordinary absent/0/16384 WF")
            }
            _ => None,
        };
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
        if !owners.is_empty() {
            fixtures_with_inputs += 1;
            eprintln!(
                "Golden fixture {filename}: {} existing analog inputs; expected {}",
                owners.len(),
                expected_refusal.unwrap_or("admitted")
            );
        }
        for owner in owners {
            fs::write(&input, &original).unwrap();
            let result = command(&input, &owner, "<v.1>hPa")
                .arg("-q")
                .output()
                .unwrap();
            if let Some(reason) = expected_refusal {
                assert!(!result.status.success(), "expected refusal in {filename}");
                assert!(
                    String::from_utf8_lossy(&result.stderr).contains(reason),
                    "wrong refusal reason in {filename}: {}",
                    String::from_utf8_lossy(&result.stderr)
                );
                if reason.contains("lox-n37w") {
                    sanitizer_refused += 1;
                } else {
                    flags_refused += 1;
                }
                assert!(result.stdout.is_empty());
                assert_eq!(fs::read(&input).unwrap(), original);
            } else {
                assert!(
                    result.status.success(),
                    "unexpected refusal in {filename}: {}",
                    String::from_utf8_lossy(&result.stderr)
                );
                admitted += 1;
                let saved = fs::read(&input).unwrap();
                assert_ne!(saved, original);
                command(&input, &owner, "<v.1>hPa")
                    .arg("-q")
                    .assert()
                    .success()
                    .stdout("");
                assert_eq!(fs::read(&input).unwrap(), saved);
                for value in ["<v.1>µg/m³", "--clear"] {
                    command(&input, &owner, value)
                        .arg("-q")
                        .assert()
                        .success()
                        .stdout("");
                    let once = fs::read(&input).unwrap();
                    command(&input, &owner, value)
                        .arg("-q")
                        .assert()
                        .success()
                        .stdout("");
                    assert_eq!(fs::read(&input).unwrap(), once);
                }
            }
        }
    }
    eprintln!(
        "Offline golden format edits: {fixtures} fixtures scanned, {fixtures_with_inputs} with analog inputs, {} inputs; {admitted} admitted, {sanitizer_refused} sanitizer refusals, {flags_refused} WF refusals; not a native application pass",
        admitted + sanitizer_refused + flags_refused,
    );
    assert!(
        admitted > 0,
        "the admitted branch must work on existing real golden inputs"
    );
}
