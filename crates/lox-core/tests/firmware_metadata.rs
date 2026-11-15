use lox_core::firmware_metadata::{
    CalendarDate, DeadlineRelation, FirmwareMetadataError, FirmwareVersion, ProgramCompatibility,
    ReleaseStatus, UpdateMetadata,
};

fn version(value: &str) -> FirmwareVersion {
    value.parse().unwrap()
}

fn date(value: &str) -> CalendarDate {
    value.parse().unwrap()
}

fn metadata() -> UpdateMetadata {
    UpdateMetadata::parse(
        r#"<?xml version="1.0"?>
        <Miniserversoftware Version="99">
          <Beta Version="90.0"/>
          <update><Release Version="80.0" Path="ignored"/></update>
          <Release
              Version='26.10.3.7' Path='https://invalid.example/config?a=1&amp;b=2'/>
          <RemoteConnect MinVersion="26.10.1" Deadline="2028-02-29"
                         AutoUpdateAvailable="999.0" Why="ignored"/>
        </Miniserversoftware>"#,
    )
    .unwrap()
}

#[test]
fn selects_direct_release_and_independent_remote_metadata() {
    let metadata = metadata();
    let release = metadata.release.as_ref().unwrap();
    assert_eq!(release.version, version("26.10.3.7"));
    assert_eq!(
        release.config_download.as_deref(),
        Some("https://invalid.example/config?a=1&b=2")
    );
    let assessment = metadata.assess(
        &version("26.9.9.99"),
        Some(&version("26.9.1")),
        date("2028-02-28"),
    );
    assert_eq!(assessment.release_status, ReleaseStatus::Newer);
    assert_eq!(
        assessment.program_compatibility,
        Some(ProgramCompatibility::DifferentMajorMinor)
    );
    let remote = assessment.remote_connect.unwrap();
    assert!(remote.below_minimum && remote.warning_due);
    assert_eq!(remote.deadline, DeadlineRelation::Before);
}

#[test]
fn compares_numeric_components_without_semver_padding_or_malformed_zero_fallback() {
    assert!(version("26.10") > version("26.9.999"));
    assert!(version("26.10.0") > version("26.10"));
    assert_eq!(version("026.010").to_string(), "26.10");
    for bad in [
        "",
        ".1",
        "1.",
        "1..2",
        " 1",
        "+1",
        "-1",
        "1.x",
        "１",
        "4294967296",
    ] {
        assert!(matches!(
            bad.parse::<FirmwareVersion>(),
            Err(FirmwareMetadataError::InvalidVersion { .. })
        ));
    }
}

#[test]
fn distinguishes_missing_equal_and_older_release_without_inventing_current_release() {
    let absent = UpdateMetadata::parse("<Miniserversoftware/>").unwrap();
    let assessment = absent.assess(&version("26.10"), None, date("2028-01-01"));
    assert_eq!(assessment.release_status, ReleaseStatus::Unavailable);
    assert!(assessment.program_compatibility.is_none());
    assert!(assessment.remote_connect.is_none());
    let metadata = metadata();
    for (current, status) in [
        ("26.10.3.7", ReleaseStatus::Equal),
        ("27.0", ReleaseStatus::Older),
    ] {
        let assessment =
            metadata.assess(&version(current), Some(&version("1.0")), date("2028-01-01"));
        assert_eq!(assessment.release_status, status);
        assert!(assessment.program_compatibility.is_none());
    }
    let assessment = metadata.assess(
        &version("26.9"),
        Some(&version("26.10.99.99")),
        date("2028-01-01"),
    );
    assert_eq!(
        assessment.program_compatibility,
        Some(ProgramCompatibility::SameMajorMinor)
    );
}

#[test]
fn preserves_below_minimum_after_deadline_but_matches_upstream_warning_predicate() {
    let metadata = metadata();
    for (as_of, relation, warning_due) in [
        ("2028-02-29", DeadlineRelation::On, true),
        ("2028-03-01", DeadlineRelation::Past, false),
    ] {
        let remote = metadata
            .assess(&version("26.9"), None, date(as_of))
            .remote_connect
            .unwrap();
        assert!(remote.below_minimum);
        assert_eq!(remote.deadline, relation);
        assert_eq!(remote.warning_due, warning_due);
    }
    let remote = metadata
        .assess(&version("26.10.1"), None, date("2028-02-28"))
        .remote_connect
        .unwrap();
    assert!(!remote.below_minimum && !remote.warning_due);
}

#[test]
fn validates_dates_and_does_not_consult_an_ambient_clock() {
    assert_eq!(date("2000-02-29").to_string(), "2000-02-29");
    for bad in [
        "1900-02-29",
        "2027-02-29",
        "2028-04-31",
        "0000-01-01",
        "2028-00-01",
        "2028-13-01",
        "2028-01-00",
        "2028-2-29",
        "2028-02-29Z",
        "２０２８-02-29",
    ] {
        assert!(matches!(
            bad.parse::<CalendarDate>(),
            Err(FirmwareMetadataError::InvalidDate { .. })
        ));
    }
}

#[test]
fn rejects_malformed_ambiguous_or_incomplete_metadata_without_partial_success() {
    for xml in [
        "<Miniserversoftware>",
        "<Miniserversoftware><Release Version='2.0'/></wrong>",
        "<Miniserversoftware/><Miniserversoftware/>",
        "<html/>",
        "<!DOCTYPE Miniserversoftware><Miniserversoftware/>",
        "oops<Miniserversoftware/>",
        "<Miniserversoftware/><![CDATA[tail]]>",
        "<Miniserversoftware><Release Version='2.0' Version='3.0'/></Miniserversoftware>",
        "<Miniserversoftware><Release Version='2.0'/><Release Version='3.0'/></Miniserversoftware>",
        "<Miniserversoftware><Release/></Miniserversoftware>",
        "<Miniserversoftware><Release Version='2.bad'/></Miniserversoftware>",
        "<Miniserversoftware><RemoteConnect MinVersion='2.0'/></Miniserversoftware>",
        "<Miniserversoftware><RemoteConnect MinVersion='2.0' Deadline='2027-02-29'/></Miniserversoftware>",
        "<Miniserversoftware><RemoteConnect MinVersion='2.0' Deadline='2028-01-01'/><RemoteConnect MinVersion='3.0' Deadline='2028-01-01'/></Miniserversoftware>",
        "<Miniserversoftware><Release Version='2.0' Path='&undefined;'/></Miniserversoftware>",
    ] {
        assert!(UpdateMetadata::parse(xml).is_err(), "{xml}");
    }
    assert!(matches!(
        UpdateMetadata::parse("<Miniserversoftware><Release/></Miniserversoftware>"),
        Err(FirmwareMetadataError::MissingAttribute {
            element: "Release",
            attribute: "Version"
        })
    ));
    assert!(matches!(
        UpdateMetadata::parse(
            "<Miniserversoftware><Release Version='2.0'/><Release Version='3.0'/></Miniserversoftware>"
        ),
        Err(FirmwareMetadataError::DuplicateElement("Release"))
    ));
}

#[test]
fn rejects_missing_xml_attribute_separators_throughout_document() {
    for xml in [
        "<Miniserversoftware><Release Version='2.0'Path='x'/></Miniserversoftware>",
        "<Miniserversoftware><Release Version=\"2.0\"Path=\"x\"/></Miniserversoftware>",
        "<Miniserversoftware A='a'B='b'/>",
        "<Miniserversoftware><Ignored><Nested A='a'B='b'></Nested></Ignored></Miniserversoftware>",
        "<Miniserversoftware><Release Version='2.0'\u{b}Path='x'/></Miniserversoftware>",
        "<Miniserversoftware><Release Version='2.0'\u{c}Path='x'/></Miniserversoftware>",
    ] {
        assert!(
            matches!(
                UpdateMetadata::parse(xml),
                Err(FirmwareMetadataError::InvalidDocument(_))
            ),
            "{xml}"
        );
    }
}

#[test]
fn rejects_raw_less_than_in_xml_attributes_throughout_document() {
    for xml in [
        "<Miniserversoftware><Release Version='2.0' Path='a<b'/></Miniserversoftware>",
        "<Miniserversoftware><Release Version=\"2.0\" Path=\"a<b\"/></Miniserversoftware>",
        "<Miniserversoftware A='a<b'/>",
        "<Miniserversoftware><Ignored A='a<b'/></Miniserversoftware>",
        "<Miniserversoftware><Ignored><Nested A='a<b'></Nested></Ignored></Miniserversoftware>",
    ] {
        assert!(
            matches!(
                UpdateMetadata::parse(xml),
                Err(FirmwareMetadataError::InvalidDocument(_))
            ),
            "{xml}"
        );
    }
}

#[test]
fn accepts_xml_attribute_whitespace_entities_and_both_quote_styles_without_mutation() {
    for separator in [" ", "\t", "\r", "\n", " \t\r\n"] {
        let xml = format!(
            "<?xml version='1.0'?><Miniserversoftware Source=\"it's &quot;quoted&quot;\">\
             <Release{separator}Version = '2.0'{separator}Path = \"https://invalid.example/a?x=&lt;b&gt;&amp;q='yes'&amp;d=&quot;ok&quot;\"{separator}/>\
             <RemoteConnect{separator}MinVersion='1.9'{separator}Deadline='2028-02-29'/>\
             <Ignored Note='say \"yes\" &amp; it&apos;s valid'></Ignored>\
             </Miniserversoftware>"
        );
        let original = xml.clone();
        let metadata = UpdateMetadata::parse(&xml).unwrap();
        assert_eq!(xml, original);
        let release = metadata.release.unwrap();
        assert_eq!(release.version, version("2.0"));
        assert_eq!(
            release.config_download.as_deref(),
            Some("https://invalid.example/a?x=<b>&q='yes'&d=\"ok\"")
        );
        assert_eq!(
            metadata.remote_connect.unwrap().deadline,
            date("2028-02-29")
        );
    }
}

#[test]
fn does_not_promote_nested_release_or_infer_update_eligibility() {
    let metadata = UpdateMetadata::parse(
        r#"<Miniserversoftware certificate="not-verified">
             <!-- <Release Version="99.0"/> -->
             <update><Release Version="88.0"/></update>
             <RemoteConnect MinVersion="26.0" Deadline="2028-01-01"
                            AutoUpdateAvailable="1.0"/>
           </Miniserversoftware>"#,
    )
    .unwrap();
    let assessment = metadata.assess(&version("25.0"), None, date("2028-01-01"));
    assert_eq!(assessment.release_status, ReleaseStatus::Unavailable);
    assert!(assessment.remote_connect.unwrap().warning_due);
    assert!(metadata.release.is_none());
}
