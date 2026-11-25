//! Runtime App changes are standalone controls, not updates to the saved program ZIP.
//! See issue #28. Keep untouched XML byte-for-byte, including literal attribute newlines.

use anyhow::{Context, Result, bail};
use lox_core::configuration::runtime_overlay::{self, OverlayError, XmlContext};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{config::Config, ftp, loxcc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub filename: String,
    pub archive: Vec<u8>,
    pub xml: Vec<u8>,
    overlays: BTreeMap<String, Vec<u8>>,
    pub source_archive: Vec<u8>,
}

/// Two identical observations catch changes during acquisition, not changes after it.
/// FTP has no evidenced atomic snapshot or conditional deployment operation.
pub fn acquire(cfg: &Config) -> Result<Snapshot> {
    acquire_with(|| {
        let backups = ftp::list_backups(cfg)?;
        let newest = backups
            .first()
            .context("No configs found on the Miniserver.")?;
        let archive = ftp::download_backup(cfg, &newest.filename)?;
        let mut overlays = BTreeMap::new();
        for name in ftp::list_custom_changes(cfg)? {
            let data = ftp::download_custom_change(cfg, &name)?;
            if overlays.insert(name.clone(), data).is_some() {
                bail!("Duplicate runtime overlay {name}; download with Loxone Config instead.");
            }
        }
        Ok((newest.filename.clone(), archive, overlays))
    })
}

type Capture = (String, Vec<u8>, BTreeMap<String, Vec<u8>>);

fn acquire_with(mut capture: impl FnMut() -> Result<Capture>) -> Result<Snapshot> {
    let first = capture().context("Cannot acquire saved config and /sys/custom runtime changes")?;
    let second = capture().context("Cannot verify config and /sys/custom runtime changes")?;
    if first != second {
        bail!(
            "Miniserver config or /sys/custom changed while downloading. Retry when App edits have stopped; no snapshot was saved."
        );
    }
    let (filename, archive, overlays) = first;
    let base = loxcc::extract_and_decompress(&archive)?;
    let xml = merge_overlays(&base, &overlays)?;
    let effective_archive = if overlays.is_empty() {
        archive.clone()
    } else {
        loxcc::repack_snapshot_zip(&archive, &xml)?
    };
    Ok(Snapshot {
        filename,
        archive: effective_archive,
        xml,
        overlays,
        source_archive: archive,
    })
}

/// Preserve the CLI's pre-Core error messages, contexts and payload types.
fn overlay_error(error: OverlayError) -> anyhow::Error {
    match error {
        OverlayError::DocumentUtf8 { source } => {
            anyhow::Error::new(source).context("Config XML is not valid UTF-8")
        }
        OverlayError::Utf8 { source } => source.into(),
        OverlayError::StringUtf8 { source } => source.into(),
        OverlayError::Xml { source, context } => match context {
            XmlContext::Document => anyhow::Error::new(source).context("Malformed config XML"),
            XmlContext::Bare => source.into(),
        },
        OverlayError::Attribute { source, context } => match context {
            XmlContext::Document => anyhow::Error::new(source).context("Malformed XML attribute"),
            XmlContext::Bare => source.into(),
        },
        OverlayError::Archive { source } => loxcc::archive_error(source),
        OverlayError::FragmentRead { name, source } => {
            overlay_error(*source).context(format!("Cannot read /sys/custom/{name}"))
        }
        OverlayError::UuidConsistency { name, source } => {
            overlay_error(*source).context(format!("UUID consistency failed after {name}"))
        }
        OverlayError::MergedConsistency { source } => {
            overlay_error(*source).context("Merged config has inconsistent UUIDs")
        }
        OverlayError::FilenameSuffix => anyhow::Error::msg("Runtime overlay must end in .LoxCC"),
        OverlayError::MissingControlType => anyhow::Error::msg("Control has no Type"),
        OverlayError::UnexpectedClosingTag => anyhow::Error::msg("Unexpected XML closing tag"),
        OverlayError::Dtd => anyhow::Error::msg("DTD in config XML is not supported."),
        OverlayError::OutsideRoot => {
            anyhow::Error::msg("Unexpected content outside config XML root.")
        }
        OverlayError::IncompleteRoot => {
            anyhow::Error::msg("Config XML must contain exactly one complete root element.")
        }
        OverlayError::FragmentIntegrity => anyhow::Error::msg(
            "Runtime overlay LoxCC length or CRC32 mismatch; refusing a partial/corrupt fragment.",
        ),
        OverlayError::ParentChildOverlap => anyhow::Error::msg(
            "Overlapping parent/child runtime overlays have no evidenced precedence. Reconcile with Loxone Config.",
        ),
        OverlayError::NotControl => anyhow::Error::msg("Runtime overlay is not a control"),
        OverlayError::Placement => anyhow::Error::msg(
            "Three-way merge cannot determine placement of a concurrently added control. Reconcile in Loxone Config.",
        ),
        other => anyhow::Error::msg(other.to_string()),
    }
}

pub fn merge_overlays(base: &[u8], overlays: &BTreeMap<String, Vec<u8>>) -> Result<Vec<u8>> {
    runtime_overlay::merge_overlays(base, overlays).map_err(overlay_error)
}

pub fn three_way(base: &[u8], local: &[u8], live: &[u8]) -> Result<Vec<u8>> {
    runtime_overlay::three_way(base, local, live).map_err(overlay_error)
}

#[derive(Serialize, Deserialize)]
struct Baseline {
    version: u8,
    host: String,
    serial: String,
    xml: String,
}

fn baseline_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(".lox-base.json");
    PathBuf::from(name)
}

pub fn save_baseline(cfg: &Config, path: &Path, xml: &[u8]) -> Result<bool> {
    let baseline = Baseline {
        version: 1,
        host: cfg.host.clone(),
        serial: cfg.serial.clone(),
        xml: std::str::from_utf8(xml)?.to_string(),
    };
    let path = baseline_path(path);
    let bytes = serde_json::to_vec(&baseline)?;
    match fs::read(&path) {
        Ok(existing) if existing == bytes => return Ok(false),
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e).context("Cannot read existing acquisition baseline"),
    }
    fs::write(path, bytes)
        .context("Cannot save acquisition baseline; do not deploy without a fresh download")?;
    Ok(true)
}

fn load_baseline(cfg: &Config, path: &Path) -> Result<Vec<u8>> {
    let path = baseline_path(path);
    let bytes = fs::read(&path).with_context(|| format!(
        "No trustworthy acquisition baseline at {}. Run `lox config download --extract` and reapply your edits; do not substitute today's live config for the original base.", path.display()
    ))?;
    let baseline: Baseline =
        serde_json::from_slice(&bytes).context("Invalid acquisition baseline")?;
    if baseline.version != 1 || baseline.host != cfg.host || baseline.serial != cfg.serial {
        bail!(
            "Acquisition baseline belongs to another Miniserver or format. Download again and reapply your edits."
        );
    }
    Ok(baseline.xml.into_bytes())
}

pub fn copy_baseline(source: &Path, destination: &Path) -> Result<()> {
    let path = baseline_path(source);
    if path.try_exists()? {
        let bytes = fs::read(&path).context("Cannot read acquisition baseline")?;
        let _: Baseline = serde_json::from_slice(&bytes).context("Invalid acquisition baseline")?;
        fs::write(baseline_path(destination), bytes).context("Cannot copy acquisition baseline")?;
    } else {
        match fs::remove_file(baseline_path(destination)) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e).context("Cannot discard stale extraction baseline"),
        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct Prepared {
    pub snapshot: Snapshot,
    pub xml: Vec<u8>,
}

pub fn prepare(cfg: &Config, path: &Path, local: &[u8]) -> Result<Prepared> {
    let live = acquire(cfg)?;
    let base = if baseline_path(path).try_exists()? {
        Some(load_baseline(cfg, path)?)
    } else {
        None
    };
    prepare_with_base(base.as_deref(), local, live)
}

fn prepare_with_base(base: Option<&[u8]>, local: &[u8], live: Snapshot) -> Result<Prepared> {
    runtime_overlay::validate_document(local).map_err(overlay_error)?;
    let xml = if let Some(base) = base {
        three_way(base, local, &live.xml)?
    } else if live.overlays.is_empty() {
        // Backward compatibility: a legacy local file with no observed pending App changes.
        local.to_vec()
    } else {
        bail!(
            "Cannot preserve pending /sys/custom changes without the original acquisition baseline. Run `lox config download --extract`, keep its .lox-base.json file, and reapply your edits, or reconcile with Loxone Config. No config was uploaded; --force does not choose an overwrite policy."
        );
    };
    Ok(Prepared {
        snapshot: live,
        xml,
    })
}

/// Best-effort race detection only: no evidenced FTP transaction covers upload and reload.
pub fn recheck(cfg: &Config, expected: &Snapshot) -> Result<()> {
    check_unchanged(expected, &acquire(cfg)?)?;
    eprintln!(
        "⚠ Runtime snapshot rechecked, not locked: avoid App edits until reload completes; edits after this check (including before a deferred reboot) can still be lost."
    );
    Ok(())
}

fn check_unchanged(expected: &Snapshot, current: &Snapshot) -> Result<()> {
    if expected != current {
        bail!(
            "Live config or /sys/custom changed during deployment preparation. No config was uploaded. Download again and reconcile App changes before retrying."
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};

    const A: &str = "00000000-0000-0000-ffff000000000001";
    const B: &str = "00000000-0000-0000-ffff000000000002";
    const RULE: &str = "00000000-0000-0000-ffff000000000003";

    fn control(id: &str, value: &str) -> String {
        format!(r#"<C Type="DayTimer" U="{id}" Title="{value}"/>"#)
    }

    fn config(a: &str, b: &str) -> Vec<u8> {
        format!("\u{feff}<?xml version=\"1.0\"?>\r\n<Loxone>\r\n<C Type=\"Page\" U=\"page\" SpNo=\"1\">\r\n{}\r\n{}\r\n</C>\r\n</Loxone>\r\n",
            control(A, a), control(B, b)).into_bytes()
    }

    fn overlay(id: &str, xml: &str) -> BTreeMap<String, Vec<u8>> {
        BTreeMap::from([(format!("{id}.LoxCC"), loxcc::compress_loxcc(xml.as_bytes()))])
    }

    #[test]
    fn runtime_overlay_legacy_message_payload_downcasts() {
        let invalid = BTreeMap::from([("bad.LoxCC".to_owned(), Vec::new())]);
        let error = merge_overlays(b"<Root/>", &invalid).unwrap_err();
        let message = error.to_string();
        assert_eq!(error.downcast_ref::<String>(), Some(&message));
        assert!(error.downcast_ref::<&'static str>().is_none());
        assert!(error.downcast_ref::<OverlayError>().is_none());
        assert_eq!(error.chain().count(), 1);

        let suffix = BTreeMap::from([("bad.loxcc".to_owned(), Vec::new())]);
        let error = merge_overlays(b"<Root/>", &suffix).unwrap_err();
        assert_eq!(
            error.downcast_ref::<&'static str>().copied(),
            Some("Runtime overlay must end in .LoxCC")
        );
        assert!(error.downcast_ref::<String>().is_none());
        assert!(error.downcast_ref::<OverlayError>().is_none());
        assert_eq!(error.chain().count(), 1);

        for (xml, expected) in [
            (
                br#"<C U="missing-type"/>"#.as_slice(),
                "Control has no Type",
            ),
            (
                b"<!DOCTYPE Root><Root/>".as_slice(),
                "DTD in config XML is not supported.",
            ),
            (
                b"outside<Root/>".as_slice(),
                "Unexpected content outside config XML root.",
            ),
            (
                b"".as_slice(),
                "Config XML must contain exactly one complete root element.",
            ),
        ] {
            let error = merge_overlays(xml, &invalid).unwrap_err();
            assert_eq!(
                error.downcast_ref::<&'static str>().copied(),
                Some(expected)
            );
            assert!(error.downcast_ref::<String>().is_none());
            assert_eq!(error.chain().count(), 1);
        }
    }

    #[test]
    fn runtime_overlay_byte_and_error_characterization() {
        fn observe(name: &str, result: Result<Vec<u8>>) {
            let value = match result {
                Ok(bytes) => serde_json::json!({"bytes": hex::encode(bytes)}),
                Err(error) => serde_json::json!({
                    "message": error.to_string(),
                    "chain": error.chain().map(ToString::to_string).collect::<Vec<_>>(),
                    "utf8": error.downcast_ref::<std::str::Utf8Error>().is_some(),
                    "xml": error.downcast_ref::<quick_xml::Error>().is_some(),
                    "attribute": error.downcast_ref::<quick_xml::events::attributes::AttrError>().is_some(),
                    "archive": error.downcast_ref::<lox_core::configuration::archive::ArchiveError>().is_some(),
                }),
            };
            println!(
                "OVERLAY_PROOF {}",
                serde_json::json!({"case":name,"result":value})
            );
        }
        let base = config("before", "literal\r\nattribute<!--not markup-->");
        observe(
            "empty-invalid-base",
            merge_overlays(b"\xff<", &BTreeMap::new()),
        );
        observe(
            "replace",
            merge_overlays(&base, &overlay(A, &control(A, "after"))),
        );
        let rule = format!(r#"<C Type="AutopilotRule" U="{RULE}"/>"#);
        for (name, bytes) in [
            ("new-rule", base.as_slice()),
            (
                "new-rule-empty-page",
                br#"<Root><!--keep--><C Type="Page" U="p" SpNo="1"/></Root>"#.as_slice(),
            ),
            ("new-rule-no-page", b"<Root/>".as_slice()),
            (
                "new-rule-ambiguous",
                br#"<Root><C Type="Page" U="p" SpNo="1"/><C Type="Page" U="q" SpNo="1"/></Root>"#
                    .as_slice(),
            ),
        ] {
            observe(name, merge_overlays(bytes, &overlay(RULE, &rule)));
        }
        for (name, fragment) in [
            ("wrong-uuid", control(B, "wrong")),
            (
                "wrapped-root",
                format!("<Wrapper>{}</Wrapper>", control(A, "wrapped")),
            ),
            ("wrong-type", format!(r#"<C Type="Not" U="{A}"/>"#)),
            ("missing-type", format!(r#"<C U="{A}"/>"#)),
            (
                "malformed-fragment",
                format!(r#"<C Type="DayTimer" U="{A}">"#),
            ),
            (
                "multiple-roots",
                format!("{}{}", control(A, "a"), control(B, "b")),
            ),
            ("fragment-dtd", format!("<!DOCTYPE C>{}", control(A, "a"))),
            (
                "fragment-duplicate-attribute",
                format!(r#"<C Type="DayTimer" Type="DayTimer" U="{A}"/>"#),
            ),
            (
                "fragment-entity",
                format!(r#"<C Type="DayTimer" U="{A}" Title="&unknown;"/>"#),
            ),
            (
                "result-duplicate-connector",
                format!(r#"<C Type="DayTimer" U="{A}"><Co U="{B}"/></C>"#),
            ),
        ] {
            observe(name, merge_overlays(&base, &overlay(A, &fragment)));
        }
        for (name, bytes) in [
            ("invalid-base-utf8", b"\xff".as_slice()),
            ("base-dtd", b"<!DOCTYPE Root><Root/>".as_slice()),
            ("base-no-root", b"<!--only-->".as_slice()),
            ("base-outside-root", b"text<Root/>".as_slice()),
            ("base-empty-uuid", br#"<C Type="Page" U=""/>"#.as_slice()),
            ("base-duplicate-attr", br#"<Root a="1" a="2"/>"#.as_slice()),
            ("base-broken-tag", b"<Root></Other>".as_slice()),
        ] {
            observe(name, merge_overlays(bytes, &overlay(A, &control(A, "a"))));
        }
        for name in [
            "wrong.loxcc",
            "bad.LoxCC",
            "00000000-0000-0000-ffff00000000000g.LoxCC",
        ] {
            observe(
                name,
                merge_overlays(&base, &BTreeMap::from([(name.into(), vec![])])),
            );
        }
        let valid = loxcc::compress_loxcc(control(A, "after").as_bytes());
        for (name, mut bytes, index) in [
            ("crc", valid.clone(), Some(12)),
            ("compressed-length", valid.clone(), Some(4)),
            ("uncompressed-length", valid.clone(), Some(8)),
            ("magic", valid.clone(), Some(0)),
            ("header-short", vec![0; 15], None),
            ("truncated", valid[..valid.len() - 1].to_vec(), None),
        ] {
            if let Some(index) = index {
                bytes[index] ^= 1;
            }
            observe(
                name,
                merge_overlays(&base, &BTreeMap::from([(format!("{A}.LoxCC"), bytes)])),
            );
        }
        let alias = "00000000-0000-0000-ffff-000000000001";
        let mut duplicate = overlay(A, &control(A, "one"));
        duplicate.extend(overlay(alias, &control(alias, "two")));
        observe("alias-overlay", merge_overlays(&base, &duplicate));
        let nested_base = format!(
            r#"<Root><C Type="Page" U="{A}" SpNo="1">{}</C></Root>"#,
            control(B, "old")
        );
        let mut nested = overlay(
            A,
            &format!(
                r#"<C Type="Page" U="{A}" SpNo="1">{}</C>"#,
                control(B, "nested")
            ),
        );
        nested.extend(overlay(B, &control(B, "direct")));
        observe(
            "parent-child",
            merge_overlays(nested_base.as_bytes(), &nested),
        );
        let mut cross = overlay(
            A,
            &format!(r#"<C Type="DayTimer" U="{A}">{}</C>"#, control(B, "nested")),
        );
        cross.extend(overlay(B, &control(B, "direct")));
        observe("cross-fragment", merge_overlays(&base, &cross));
        let mut ordered = overlay(B, &control(B, "second"));
        ordered.extend(overlay(A, &control(A, "first")));
        observe("source-order", merge_overlays(&base, &ordered));
        let local = config("local", "literal\r\nattribute<!--not markup-->");
        let live = config("before", "live");
        for (name, b, l, r) in [
            ("three-local", base.as_slice(), local.as_slice(), base.as_slice()),
            ("three-live", base.as_slice(), base.as_slice(), live.as_slice()),
            ("three-same", base.as_slice(), local.as_slice(), local.as_slice()),
            ("three-disjoint", base.as_slice(), local.as_slice(), live.as_slice()),
            ("three-conflict", base.as_slice(), local.as_slice(), config("conflict", "live").as_slice()),
            ("three-format", b"<Root><C Type=\"And\" U=\"a\"/></Root>", b"\xef\xbb\xbf<?xml version=\"1.0\"?><Root>\r\n<C U=\"a\" Type=\"And\"></C>\r\n</Root>", b"<Root><C Type=\"Or\" U=\"a\"/></Root>"),
            ("three-comment-conflict", b"<Root><!--a--></Root>", b"<Root><!--b--></Root>", b"<Root><!--c--></Root>"),
            ("three-invalid", b"<Root/>", b"\xff", b"<Root/>"),
            ("three-delete", b"<Root><C Type=\"And\" U=\"a\"/></Root>", b"<Root/>", b"<Root><C Type=\"And\" U=\"a\"/></Root>"),
            ("three-both-delete", b"<Root><C Type=\"And\" U=\"a\"/></Root>", b"<Root/>", b"<Root/>"),
            ("three-delete-modify", b"<Root><C Type=\"And\" U=\"a\"/></Root>", b"<Root/>", b"<Root><C Type=\"Or\" U=\"a\"/></Root>"),
            ("three-live-add", b"<Root/>", b"<Root/>", b"<Root><C Type=\"And\" U=\"a\"/></Root>"),
            ("three-same-add", b"<Root/>", b"<Root><C Type=\"And\" U=\"a\"/></Root>", b"<Root><C U=\"a\" Type=\"And\"></C></Root>"),
            ("three-conflicting-add", b"<Root/>", b"<Root><C Type=\"And\" U=\"a\"/></Root>", b"<Root><C Type=\"Or\" U=\"a\"/></Root>"),
        ] {
            observe(name, three_way(b, l, r));
        }
    }

    fn archive(xml: &[u8]) -> Vec<u8> {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        writer
            .start_file("sps0.LoxCC", zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(&loxcc::compress_loxcc(xml)).unwrap();
        writer
            .start_file("unrelated.bin", zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(&[0, 255, 7]).unwrap();
        writer.finish().unwrap().into_inner()
    }

    fn snapshot(xml: &[u8], overlays: BTreeMap<String, Vec<u8>>) -> Snapshot {
        let capture = ("sps_1_20260101120000.zip".into(), archive(xml), overlays);
        acquire_with(|| Ok(capture.clone())).unwrap()
    }

    #[test]
    fn overlay_replaces_uuid_without_rewriting_unrelated_bytes() {
        let base = config("before", "keep&#xD;&#xA;text");
        let replacement = format!(
            r#"<?xml version="1.0"?><C Title="live" U="{A}" Type="DayTimer"><Co U="connector" K="I"/></C>"#
        );
        let result = merge_overlays(&base, &overlay(A, &replacement)).unwrap();
        let expected = String::from_utf8(base).unwrap().replace(
            &control(A, "before"),
            &replacement[replacement.find("<C ").unwrap()..],
        );
        assert_eq!(result, expected.as_bytes());
    }

    #[test]
    fn no_overlays_preserves_archive_and_xml_exactly() {
        let xml = config("literal\r\nattribute", "unchanged");
        let zip = archive(&xml);
        let mut calls = 0;
        let result = acquire_with(|| {
            calls += 1;
            Ok(("saved.zip".into(), zip.clone(), BTreeMap::new()))
        })
        .unwrap();
        assert_eq!(calls, 2);
        assert_eq!(result.archive, zip);
        assert_eq!(result.xml, xml);
    }

    #[test]
    fn no_overlay_repository_fixture_is_preserved() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("eval")
            .join("fixture.Loxone");
        let xml = fs::read(path).unwrap();
        assert_eq!(merge_overlays(&xml, &BTreeMap::new()).unwrap(), xml);
    }

    #[test]
    fn acquisition_merges_runtime_changes_into_saved_zip() {
        let xml = config("old", "keep");
        let result = snapshot(&xml, overlay(A, &control(A, "App")));
        assert_eq!(
            loxcc::extract_and_decompress(&result.archive).unwrap(),
            result.xml
        );
        assert!(
            String::from_utf8(result.xml)
                .unwrap()
                .contains("Title=\"App\"")
        );
        let mut zip = zip::ZipArchive::new(Cursor::new(result.archive)).unwrap();
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(&mut zip.by_name("unrelated.bin").unwrap(), &mut bytes).unwrap();
        assert_eq!(bytes, [0, 255, 7]);
    }

    #[test]
    fn acquisition_refuses_observed_race_and_failed_overlay_listing() {
        let zip = archive(&config("old", "keep"));
        let mut calls = 0;
        let err = acquire_with(|| {
            calls += 1;
            Ok((
                "saved.zip".into(),
                zip.clone(),
                if calls == 1 {
                    BTreeMap::new()
                } else {
                    overlay(A, &control(A, "App"))
                },
            ))
        })
        .unwrap_err();
        assert!(err.to_string().contains("changed while downloading"));
        let err = acquire_with(|| bail!("FTP LIST /sys/custom denied")).unwrap_err();
        assert!(format!("{err:#}").contains("FTP LIST /sys/custom denied"));
    }

    #[test]
    fn malformed_unknown_and_mismatched_fragments_fail_closed() {
        let xml = config("old", "keep");
        for fragment in [
            format!("<C Type=\"DayTimer\" U=\"{A}\">"),
            control(B, "wrong UUID"),
            format!("<Wrapper>{}</Wrapper>", control(A, "wrapped")),
            format!("<C Type=\"Not\" U=\"{A}\"/>"),
            format!("<C Type=\"DayTimer\">{}</C>", control(A, "nested")),
        ] {
            assert!(
                merge_overlays(&xml, &overlay(A, &fragment)).is_err(),
                "{fragment}"
            );
        }
        assert!(merge_overlays(&xml, &overlay(RULE, &control(RULE, "unknown"))).is_err());
        let mut corrupt = overlay(A, &control(A, "App"));
        corrupt.values_mut().next().unwrap()[12] ^= 1;
        assert!(
            merge_overlays(&xml, &corrupt)
                .unwrap_err()
                .to_string()
                .contains("Cannot read")
        );
        let mut truncated = overlay(A, &control(A, "App"));
        truncated.values_mut().next().unwrap().pop();
        assert!(merge_overlays(&xml, &truncated).is_err());
    }

    #[test]
    fn native_flags_and_references_preserve_upload_and_overlay_edits() {
        for metadata in [
            r#"<MO U="true"/><AF U="true"/><FS U="true"/>"#.to_string(),
            format!(r#"<SObj U="{A}"/><SObj U="{A}"/><SObj U="{RULE}"/>"#),
        ] {
            let before = format!(
                r#"<Root>{metadata}<C Type="DayTimer" U="{A}" Title="before"><Co U="{RULE}" K="I"/></C></Root>"#
            );
            let edited = before.replace("Title=\"before\"", "Title=\"edited\"");
            let prepared = prepare_with_base(
                None,
                edited.as_bytes(),
                snapshot(before.as_bytes(), BTreeMap::new()),
            )
            .unwrap();
            assert_eq!(prepared.xml, edited.as_bytes());

            let fragment =
                format!(r#"<C Type="DayTimer" U="{A}" Title="edited"><Co U="{RULE}" K="I"/></C>"#);
            assert_eq!(
                merge_overlays(before.as_bytes(), &overlay(A, &fragment)).unwrap(),
                edited.as_bytes()
            );
        }
    }

    #[test]
    fn empty_control_and_connector_uuids_block_upload_preparation() {
        let base = config("before", "keep");
        for invalid in [
            r#"<Root><C Type="DayTimer" U=""/></Root>"#.to_string(),
            format!(r#"<Root><C Type="DayTimer" U="{A}"><Co K="I" U=""/></C></Root>"#),
        ] {
            let error =
                prepare_with_base(None, invalid.as_bytes(), snapshot(&base, BTreeMap::new()))
                    .unwrap_err();
            assert!(error.to_string().contains("Empty or duplicate UUID"));
        }
    }

    #[test]
    fn duplicate_control_and_connector_uuids_are_rejected() {
        let xml = config("old", "keep");
        let fragment = format!(r#"<C Type="DayTimer" U="{A}"><Co U="{B}" K="I"/></C>"#);
        assert!(merge_overlays(&xml, &overlay(A, &fragment)).is_err());
        let duplicate = format!("<Root>{}{}</Root>", control(A, "one"), control(A, "two"));
        assert!(merge_overlays(duplicate.as_bytes(), &overlay(A, &control(A, "App"))).is_err());
        let mut overlays = overlay(A, &control(A, "one"));
        let alternate = "00000000-0000-0000-ffff-000000000001";
        overlays.insert(
            format!("{alternate}.LoxCC"),
            loxcc::compress_loxcc(control(alternate, "two").as_bytes()),
        );
        assert!(merge_overlays(&xml, &overlays).is_err());
    }

    #[test]
    fn overlapping_runtime_fragments_are_not_order_dependent() {
        let base = format!(
            "<Root><C Type=\"Page\" U=\"{A}\" SpNo=\"1\">{}</C></Root>",
            control(B, "old")
        );
        let mut overlays = overlay(
            A,
            &format!(
                "<C Type=\"Page\" U=\"{A}\" SpNo=\"1\">{}</C>",
                control(B, "parent copy")
            ),
        );
        overlays.extend(overlay(B, &control(B, "child edit")));
        assert!(
            merge_overlays(base.as_bytes(), &overlays)
                .unwrap_err()
                .to_string()
                .contains("Overlapping")
        );
    }

    #[test]
    fn opaque_existing_control_replacement_is_preserved_but_new_unknown_type_refused() {
        let base = format!("<Root><C Type=\"FutureControl\" U=\"{A}\" Old=\"1\"/></Root>");
        let replacement =
            format!("<C Type=\"FutureControl\" U=\"{A}\"><Unknown Payload=\"opaque\"/></C>");
        assert_eq!(
            merge_overlays(base.as_bytes(), &overlay(A, &replacement)).unwrap(),
            format!("<Root>{replacement}</Root>").as_bytes()
        );
        assert!(merge_overlays(b"<Root/>", &overlay(A, &replacement)).is_err());
    }

    #[test]
    fn new_autopilot_rule_requires_unique_evidenced_page() {
        let rule = format!(r#"<C Type="AutopilotRule" U="{RULE}" APC="{{}}"/>"#);
        let overlays = overlay(RULE, &rule);
        let xml = config("old", "keep");
        let merged = merge_overlays(&xml, &overlays).unwrap();
        assert!(String::from_utf8(merged).unwrap().contains(&rule));
        let empty = br#"<Root><C Type="Page" U="p" SpNo="1"/></Root>"#;
        assert!(merge_overlays(empty, &overlays).is_ok());
        assert!(merge_overlays(b"<Root/>", &overlays).is_err());
        let ambiguous =
            br#"<Root><C Type="Page" U="p" SpNo="1"/><C Type="Page" U="p2" SpNo="1"/></Root>"#;
        assert!(merge_overlays(ambiguous, &overlays).is_err());
    }

    #[test]
    fn three_way_preserves_independent_local_and_concurrent_app_edits() {
        let base = config("old A", "old B");
        let local = config("local A", "old B");
        let live = config("old A", "App B");
        assert_eq!(
            three_way(&base, &local, &live).unwrap(),
            config("local A", "App B")
        );
        assert_eq!(three_way(&base, &base, &live).unwrap(), live);
        assert_eq!(three_way(&base, &local, &local).unwrap(), local);
    }

    #[test]
    fn three_way_conflicts_on_same_control_edit_or_delete_modify() {
        let base = config("old A", "old B");
        let local = config("local A", "old B");
        let live = config("App A", "old B");
        assert!(
            three_way(&base, &local, &live)
                .unwrap_err()
                .to_string()
                .contains("conflict")
        );
        let deleted = String::from_utf8(base.clone())
            .unwrap()
            .replace(&control(A, "old A"), "")
            .into_bytes();
        assert!(three_way(&base, &deleted, &live).is_err());
        assert!(three_way(&base, &live, &deleted).is_err());
        assert_eq!(three_way(&base, &deleted, &base).unwrap(), deleted);
    }

    #[test]
    fn three_way_handles_additions_and_rejects_conflicting_insertion_layout() {
        let base = config("a", "b");
        let local = config("local", "b");
        let rule = format!(r#"<C Type="AutopilotRule" U="{RULE}"/>"#);
        let live = merge_overlays(&base, &overlay(RULE, &rule)).unwrap();
        let expected = merge_overlays(&local, &overlay(RULE, &rule)).unwrap();
        assert_eq!(three_way(&base, &local, &live).unwrap(), expected);
        let live = String::from_utf8(live)
            .unwrap()
            .replace(&control(A, "a"), "");
        assert!(three_way(&base, &local, live.as_bytes()).is_err());
    }

    #[test]
    fn three_way_ignores_formatting_but_not_literal_attribute_newlines() {
        let base = br#"<Root><C Type="DayTimer" U="a" Title="before"/></Root>"#;
        let local = br#"<Root>
  <C U="a" Title="before" Type="DayTimer"></C>
</Root>"#;
        let live = br#"<Root><C Type="DayTimer" U="a" Title="after"/></Root>"#;
        assert_eq!(three_way(base, local, live).unwrap(), live);
        let local = b"<Root><C Type=\"DayTimer\" U=\"a\" Title=\"line\none\"/></Root>";
        let live = b"<Root><C Type=\"DayTimer\" U=\"a\" Title=\"line one\"/></Root>";
        assert!(three_way(base, local, live).is_err());
    }

    #[test]
    fn missing_baseline_only_refuses_observed_pending_overlays() {
        let base = config("old", "keep");
        let local = config("local", "keep");
        let live = snapshot(&base, BTreeMap::new());
        assert_eq!(prepare_with_base(None, &local, live).unwrap().xml, local);
        let live = snapshot(&base, overlay(B, &control(B, "App")));
        assert!(
            prepare_with_base(None, &local, live.clone())
                .unwrap_err()
                .to_string()
                .contains("baseline")
        );
        assert_eq!(
            prepare_with_base(Some(&base), &local, live).unwrap().xml,
            config("local", "App")
        );
    }

    #[test]
    fn final_recheck_detects_new_or_deleted_overlays_and_saved_config_changes() {
        let base = config("old", "keep");
        let initial = snapshot(&base, BTreeMap::new());
        assert!(check_unchanged(&initial, &initial).is_ok());
        let app = snapshot(&base, overlay(A, &control(A, "App")));
        assert!(check_unchanged(&initial, &app).is_err());
        assert!(check_unchanged(&app, &initial).is_err());
        assert!(
            check_unchanged(
                &initial,
                &snapshot(&config("saved edit", "keep"), BTreeMap::new())
            )
            .is_err()
        );
    }

    fn native_archive_with_sidecars(xml: &[u8]) -> Vec<u8> {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default();
        for (name, data) in [
            ("sps0.LoxCC", loxcc::compress_loxcc(xml)),
            (
                "LoxAPP3.json",
                br#"{"lastModified":"preserve-native-revision","controls":{"unrelated":{"uuidAction":"unrelated","details":{"format":"%.1f %%"},"isFavorite":true}}}"#.to_vec(),
            ),
            ("permissions.bin", b"\x00\xffopaque native permissions".to_vec()),
            ("unrelated.bin", b"\x12\x34preserve archive member".to_vec()),
        ] {
            writer.start_file(name, options).unwrap();
            writer.write_all(&data).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    fn native_project(controls: &str) -> Vec<u8> {
        format!(
            r#"<ControlList Version="10"><C Type="Page" U="page" SpNo="1">{controls}</C></ControlList>"#
        )
        .into_bytes()
    }

    #[test]
    fn deployment_sidecars_reject_visibility_hidden_by_effective_overlay_snapshot() {
        let original_control = format!(
            r#"<C Type="PushButton2Sel" V="178" U="{A}" Title="Selector"><Co K="AQ" U="selector-value"/><IoData Visu="false"/></C>"#
        );
        let base = native_project(&original_control);
        let source_archive = native_archive_with_sidecars(&base);
        let live_control = original_control.replace("Visu=\"false\"", "Visu=\"true\"");
        let capture = (
            "saved.zip".to_string(),
            source_archive.clone(),
            overlay(A, &live_control),
        );
        let snapshot = acquire_with(|| Ok(capture.clone())).unwrap();
        let prepared = prepare_with_base(Some(&base), &base, snapshot).unwrap();
        assert_eq!(prepared.snapshot.source_archive, source_archive);
        assert_eq!(
            loxcc::extract_and_decompress(&prepared.snapshot.source_archive).unwrap(),
            base
        );
        assert_ne!(prepared.xml, base);
        assert_eq!(
            loxcc::extract_and_decompress(&prepared.snapshot.archive).unwrap(),
            prepared.xml,
            "the archival snapshot already contains the newly visualized selector"
        );
        let error = loxcc::repack_zip(&prepared.snapshot.source_archive, &prepared.xml)
            .unwrap_err()
            .to_string();
        assert!(error.contains("LoxAPP3.json"), "{error}");
        assert!(error.contains("permissions.bin"), "{error}");
    }

    #[test]
    fn deployment_sidecars_preserve_disjoint_local_and_live_custom_parameters() {
        let local_control =
            format!(r#"<C Type="And" U="{A}"><Co K="I1" U="local-input" V="1"/></C>"#);
        let live_control = format!(r#"<C Type="Or" U="{B}"><Co K="I1" U="live-input" V="2"/></C>"#);
        let base = native_project(&format!("{local_control}{live_control}"));
        let local = native_project(&format!(
            "{}{live_control}",
            local_control.replace("V=\"1\"", "V=\"3\"")
        ));
        let capture = (
            "saved.zip".to_string(),
            native_archive_with_sidecars(&base),
            overlay(B, &live_control.replace("V=\"2\"", "V=\"4\"")),
        );
        let snapshot = acquire_with(|| Ok(capture.clone())).unwrap();
        let prepared = prepare_with_base(Some(&base), &local, snapshot).unwrap();
        let repacked = loxcc::repack_zip(&prepared.snapshot.source_archive, &prepared.xml).unwrap();
        let actual = loxcc::extract_and_decompress(&repacked).unwrap();
        let expected = native_project(&format!(
            "{}{}",
            local_control.replace("V=\"1\"", "V=\"3\""),
            live_control.replace("V=\"2\"", "V=\"4\"")
        ));
        assert_eq!(actual, expected);
        let mut original =
            zip::ZipArchive::new(Cursor::new(&prepared.snapshot.source_archive)).unwrap();
        let mut deployed = zip::ZipArchive::new(Cursor::new(&repacked)).unwrap();
        for name in ["LoxAPP3.json", "permissions.bin", "unrelated.bin"] {
            let mut expected = Vec::new();
            let mut actual = Vec::new();
            std::io::Read::read_to_end(&mut original.by_name(name).unwrap(), &mut expected)
                .unwrap();
            std::io::Read::read_to_end(&mut deployed.by_name(name).unwrap(), &mut actual).unwrap();
            assert_eq!(actual, expected, "{name}");
        }
    }

    #[test]
    fn acquisition_baseline_roundtrips_is_target_bound_and_survives_extract() {
        let dir = tempfile::Builder::new()
            .prefix("overlay-test-")
            .tempdir_in(".")
            .unwrap();
        let cfg = Config {
            host: "http://synthetic".into(),
            serial: "TEST".into(),
            ..Default::default()
        };
        let zip = dir.path().join("download.zip");
        let xml_path = dir.path().join("download.Loxone");
        let xml = config("before", "keep");
        assert!(load_baseline(&cfg, &zip).is_err());
        assert!(save_baseline(&cfg, &zip, &xml).unwrap());
        assert!(!save_baseline(&cfg, &zip, &xml).unwrap());
        assert_eq!(load_baseline(&cfg, &zip).unwrap(), xml);
        copy_baseline(&zip, &xml_path).unwrap();
        assert_eq!(load_baseline(&cfg, &xml_path).unwrap(), xml);
        let wrong = Config {
            serial: "OTHER".into(),
            ..cfg
        };
        assert!(load_baseline(&wrong, &zip).is_err());
        fs::write(baseline_path(&zip), b"malformed").unwrap();
        assert!(load_baseline(&wrong, &zip).is_err());
        fs::remove_file(baseline_path(&zip)).unwrap();
        copy_baseline(&zip, &xml_path).unwrap();
        assert!(load_baseline(&wrong, &xml_path).is_err());
    }
}
