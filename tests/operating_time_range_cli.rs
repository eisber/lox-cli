use std::{fs, path::Path};

const OWNER: &str = "10000000-0000-0000-0000000000000004";

fn fixture() -> String {
    let modes = (0..10)
        .chain(-6..0)
        .enumerate()
        .map(|(index, mode)| {
            let id = 100 + index * 3;
            format!(
                "<C Type=\"Mode\" V=\"178\" U=\"10000000-0000-0000-{id:016x}\" WF=\"16384\" Mode=\"{mode}\" Title=\"Fake mode {mode}\" Nio=\"2\"><Co K=\"I\" U=\"10000000-0000-0000-{:016x}\"/><Co K=\"Q\" U=\"10000000-0000-0000-{:016x}\"/><IoData/><Display StateOnly=\"true\"/></C>",
                id + 1,
                id + 2,
            )
        })
        .collect::<String>();
    format!(
        "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n<LoxoneApp>\r\n<C Type=\"Document\" V=\"178\" U=\"10000000-0000-0000-0000000000000001\" WF=\"16384\" Title=\"Fake project\" NumO=\"24\" Date=\"old\" DateS=\"1\">\r\n<C Type=\"ModeCaption\" V=\"178\" U=\"10000000-0000-0000-0000000000000002\" WF=\"16384\">{modes}</C>\r\n<C Type=\"CalendarCaption\" V=\"178\" U=\"10000000-0000-0000-0000000000000003\" WF=\"16384\">\r\n<C Type=\"CalendarEntry\" V=\"178\" U=\"{OWNER}\" WF=\"16384\" Title=\"Fake shutdown\" Nio=\"2\" CalType=\"0\" Mode=\"3\" Month=\"1\" Day=\"1\" Month2=\"1\" Day2=\"1\" EasterOffset=\"0\" From=\"6000\" To=\"6001\" Weekday=\"0\" WeekDayInMonth=\"1\" Note=\"keep &#32; this\"><Co K=\"I\" U=\"10000000-0000-0000-0000000000000005\"/><Co K=\"Q\" U=\"10000000-0000-0000-0000000000000006\"/></C>\r\n<C Type=\"CalendarEntry\" V=\"178\" U=\"10000000-0000-0000-0000000000000007\" WF=\"16384\" Title=\"Fake annual period\" CalType=\"1\" Mode=\"4\" Month=\"10\" Day=\"1\" Month2=\"4\" Day2=\"30\" Nio=\"2\"><Co K=\"I\" U=\"10000000-0000-0000-0000000000000008\"/><Co K=\"Q\" U=\"10000000-0000-0000-0000000000000009\"/></C>\r\n</C>\r\n<C Type=\"Program\" V=\"178\" U=\"10000000-0000-0000-000000000000000a\" WF=\"16384\"><C Type=\"Page\" V=\"178\" U=\"10000000-0000-0000-000000000000000b\" WF=\"16384\"><C Type=\"And\" V=\"178\" U=\"10000000-0000-0000-000000000000000c\" WF=\"16384\" Nio=\"3\" Title=\"Fake wired logic\" Px=\"100\" Py=\"100\"><Co K=\"I1\" U=\"10000000-0000-0000-000000000000000d\"><Ref U=\"10000000-0000-0000-0000000000000006\"/></Co><Co K=\"I2\" U=\"10000000-0000-0000-000000000000000e\"/><Co K=\"Q\" U=\"10000000-0000-0000-000000000000000f\"/></C></C></C>\r\n<Opaque Value=\"keep\" Value=\"keep\"><![CDATA[<fake/> & untouched]]></Opaque>\r\n</C>\r\n</LoxoneApp>\r\n"
    )
    .replacen(
        "Date=\"old\" DateS=\"1\"",
        "Date=\"2026-01-01 10:00:00\" DateS=\"536493600\" CDate=\"2026-01-01 10:00:00\" BDate=\"\" Ctry=\"DE\" 12hTF=\"true\"",
        1,
    )
}

fn command(input: &Path, from: &str, to: &str) -> assert_cmd::Command {
    let mut command = assert_cmd::cargo::cargo_bin_cmd!("lox");
    command
        .env("LOX_TELEMETRY", "0")
        .args(["config", "set-operating-time-range"])
        .arg(input)
        .arg(format!("uuid:{OWNER}"))
        .args(["--from", from, "--to", to]);
    command
}

#[test]
fn operating_time_range_cli_converts_civil_dates_and_preserves_unselected_bytes() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let source = fixture();
    fs::write(&input, &source).unwrap();
    let output = command(&input, "2028-02-28", "2028-03-01")
        .args(["-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let change = &result["operating_time_range"];
    assert_eq!(change["changed"], true);
    assert_eq!(change["previous_from"], "2025-06-06");
    assert_eq!(change["previous_to"], "2025-06-07");
    assert_eq!(change["from_index"], 6997);
    assert_eq!(change["to_index"], 6999);
    let saved = fs::read_to_string(&input).unwrap();
    let doc_tag = |text: &str| {
        let start = text.find("<C Type=\"Document\"").unwrap();
        let end = start + text[start..].find('>').unwrap() + 1;
        text[start..end].to_owned()
    };
    let expected = source
        .replacen(&doc_tag(&source), &doc_tag(&saved), 1)
        .replacen("From=\"6000\" To=\"6001\"", "From=\"6997\" To=\"6999\"", 1);
    assert_eq!(saved, expected);
    command(&input, "2028-02-28", "2028-03-01")
        .arg("-q")
        .assert()
        .success()
        .stdout("");
    assert_eq!(fs::read(&input).unwrap(), saved.as_bytes());
}

#[test]
fn operating_time_range_cli_supports_cross_year_single_day_and_missing_stamps() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("output.Loxone");
    let source = fixture()
        .replace(" Date=\"2026-01-01 10:00:00\" DateS=\"536493600\"", "")
        .replacen(
            &format!("U=\"{OWNER}\" WF=\"16384\""),
            &format!("U=\"{OWNER}\" WF=\"0\""),
            1,
        );
    fs::write(&input, &source).unwrap();
    command(&input, "2026-12-31", "2027-01-02")
        .args(["-q", "--save-as"])
        .arg(&output)
        .assert()
        .success()
        .stdout("");
    assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
    let saved = fs::read_to_string(&output).unwrap();
    assert!(saved.contains(" DateS=\""));
    assert!(saved.contains(" Date=\""));
    assert!(saved.contains(&format!("U=\"{OWNER}\" WF=\"16384\"")));
    let value = command(&output, "2100-12-31", "2100-12-31")
        .args(["-o", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: serde_json::Value = serde_json::from_slice(&value).unwrap();
    assert_eq!(
        result["operating_time_range"]["from_index"],
        result["operating_time_range"]["to_index"]
    );
    command(&output, "2100-12-31", "2100-12-31")
        .args(["-o", "csv", "--no-header"])
        .assert()
        .success()
        .stdout(format!(
            "{OWNER},10000000-0000-0000-0000000000000064,0,2100-12-31,2100-12-31,2100-12-31,2100-12-31,false\n"
        ));
    let once = fs::read(&output).unwrap();
    command(&output, "2100-12-31", "2100-12-31")
        .args(["-q", "--save-as"])
        .arg(&input)
        .assert()
        .success()
        .stdout("");
    assert_eq!(fs::read(&input).unwrap(), once);
}

fn assert_refusal(xml: &str, from: &str, to: &str) {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let existing = dir.path().join("existing.Loxone");
    let new = dir.path().join("new.Loxone");
    fs::write(&input, xml).unwrap();
    fs::write(&existing, b"preserve").unwrap();
    for destination in [None, Some(&existing), Some(&new)] {
        let mut edit = command(&input, from, to);
        if let Some(destination) = destination {
            edit.arg("--save-as").arg(destination);
        }
        edit.assert().failure().stdout("");
        assert_eq!(fs::read(&input).unwrap(), xml.as_bytes());
        assert_eq!(fs::read(&existing).unwrap(), b"preserve");
        assert!(!new.exists());
    }
}

#[test]
fn operating_time_range_cli_refuses_unsupported_dates_and_existing_contexts_atomically() {
    let source = fixture();
    for (from, to) in [
        ("2009-01-01", "2009-01-02"),
        ("2101-01-01", "2101-01-02"),
        ("2100-02-29", "2100-03-01"),
        ("2028-02-30", "2028-03-01"),
        ("2028-2-29", "2028-03-01"),
        ("2028-02-29T00:00:00Z", "2028-03-01"),
        ("2028-03-02", "2028-03-01"),
    ] {
        assert_refusal(&source, from, to);
    }
    let rfc = "10000000-0000-0000-0000-000000000004";
    for xml in [
        source.replacen("Mode=\"3\"", "Mode=\"4\"", 1),
        source.replacen("From=\"6000\"", "From=\"0\"", 1),
        source.replacen("To=\"6001\"", "To=\"0\"", 1),
        source.replacen("From=\"6000\"", "From=\"6002\"", 1),
        source.replacen("CalType=\"0\"", "CalType=\"999\"", 1),
        source.replacen("Mode=\"-1\"", "Mode=\"0\"", 1),
        source.replace("DateS=\"536493600\"", "DateS=\"2147483647\""),
        source.replace("DateS=\"536493600\"", "DateS=\"+1\""),
        source.replace("DateS=\"536493600\"", "DateS=\"-1\""),
        source.replace("DateS=\"536493600\"", "DateS=\"&#49;\""),
        source.replace(
            "DateS=\"536493600\"",
            "DateS=\"536493600\" DateS=\"536493600\"",
        ),
        source.replacen("From=\"6000\"", "From=\"6000\" From=\"6000\"", 1),
        source.replacen("From=\"6000\"", "From=\"&#54;000\"", 1),
        source.replace(
            "</LoxoneApp>",
            &format!("<C Type=\"And\" U=\"{rfc}\"/></LoxoneApp>"),
        ),
        source.replace(
            &format!("U=\"{OWNER}\""),
            &format!("U=\"{OWNER}\" U=\"{OWNER}\""),
        ),
        source.replacen(
            &format!("U=\"{OWNER}\" WF=\"16384\""),
            &format!("U=\"{OWNER}\" WF=\"134234112\""),
            1,
        ),
        source.replacen(
            &format!("U=\"{OWNER}\" WF=\"16384\""),
            &format!("U=\"{OWNER}\" WF=\"536887296\""),
            1,
        ),
        source.replace("NumO=\"24\"", "NumO=\"24\" IfT=\"false\""),
        source.replace("<Opaque", "<Template/><Opaque"),
        source.replace(
            &format!("U=\"{OWNER}\""),
            &format!("U=\"{OWNER}\" uuidParent=\"10000000-0000-0000-0000000000000020\""),
        ),
        source.replace("<Opaque", "<Opaque xmlns=\"urn:fake\""),
        source.replace(
            "Value=\"keep\" Value=\"keep\"",
            "Value=\"keep\" Value=\"different\"",
        ),
        source.replace("<Opaque", "<Fake> 7=\"digit-like text\" </Fake><Opaque"),
        source.replace("CalendarEntry", "Calendar"),
    ] {
        assert_refusal(&xml, "2025-06-06", "2025-06-07");
    }
    assert_refusal(
        &source.replace("DateS=\"536493600\"", "DateS=\"2147483646\""),
        "2028-02-29",
        "2028-03-01",
    );
}

#[test]
fn operating_time_range_cli_validates_single_root_before_noop_and_changed_edits() {
    let source = fixture();
    for xml in [
        format!("{source}<C Type=\"Document\"/>"),
        format!("{source}<C Type=\"CalendarEntry\" U=\"{OWNER}\"/>"),
        format!("{source}outside"),
        format!("{source}<![CDATA[ ]]>"),
        format!("{source}<?xml version=\"1.0\"?>"),
        format!("{source}<!--bad--comment-->"),
        format!("{source}<C>"),
        source.replace("</LoxoneApp>", "</Wrong>"),
        source.replace("</LoxoneApp>", ""),
    ] {
        for (from, to) in [("2025-06-06", "2025-06-07"), ("2028-02-29", "2028-03-01")] {
            assert_refusal(&xml, from, to);
        }
    }
}

#[test]
fn operating_time_range_cli_preserves_safe_markup_and_root_document() {
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let input = dir.path().join("input.Loxone");
    let output = dir.path().join("copy.Loxone");
    let source = fixture();
    let source = source
        .replace("<LoxoneApp>\r\n", "<!-- outside <C/> --><?before fake?>")
        .replace("</LoxoneApp>", "<?after fake?><!-- trailing <C/> -->");
    fs::write(&input, &source).unwrap();
    command(&input, "2025-06-06", "2025-06-07")
        .args(["-q", "--save-as"])
        .arg(&output)
        .assert()
        .success()
        .stdout("");
    assert_eq!(fs::read(&input).unwrap(), source.as_bytes());
    assert_eq!(fs::read(&output).unwrap(), source.as_bytes());
    command(&input, "2028-02-29", "2028-03-01")
        .args(["--dry-run", "--save-as"])
        .arg(&output)
        .assert()
        .failure()
        .stderr(predicates::str::contains("dry-run not supported"));
    assert_eq!(fs::read(&output).unwrap(), source.as_bytes());
}
