//! Runtime App changes are standalone controls, not updates to the saved program ZIP.
//! See issue #28. Keep untouched XML byte-for-byte, including literal attribute newlines.

use anyhow::{Context, Result, bail};
use quick_xml::{Reader, events::Event};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::ops::Range;
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

#[derive(Debug)]
struct Control {
    id: String,
    raw_id: String,
    kind: String,
    sp_no: Option<String>,
    span: Range<usize>,
    close: usize,
    empty: bool,
    parent: Option<usize>,
}

struct Document<'a> {
    bytes: &'a [u8],
    controls: Vec<Control>,
}

fn identity(value: &str) -> String {
    value.replace('-', "").to_ascii_lowercase()
}

fn overlay_id(name: &str) -> Result<String> {
    let stem = name
        .strip_suffix(".LoxCC")
        .context("Runtime overlay must end in .LoxCC")?;
    let groups: Vec<_> = stem.split('-').map(str::len).collect();
    if !matches!(groups.as_slice(), [8, 4, 4, 16] | [8, 4, 4, 4, 12])
        || !stem.bytes().all(|b| b.is_ascii_hexdigit() || b == b'-')
    {
        bail!("Invalid runtime overlay filename {name}; expected a control UUID.LoxCC.");
    }
    Ok(identity(stem))
}

impl<'a> Document<'a> {
    fn parse(bytes: &'a [u8]) -> Result<Self> {
        std::str::from_utf8(bytes).context("Config XML is not valid UTF-8")?;
        let offset = usize::from(bytes.starts_with(b"\xef\xbb\xbf")) * 3;
        let mut reader = Reader::from_reader(&bytes[offset..]);
        let mut controls: Vec<Control> = Vec::new();
        let mut stack: Vec<Option<usize>> = Vec::new();
        let mut ids = HashSet::new();
        let mut roots = 0;
        loop {
            let start = reader.buffer_position() as usize + offset;
            match reader.read_event().context("Malformed config XML")? {
                Event::Start(ref e) | Event::Empty(ref e) => {
                    if stack.is_empty() {
                        roots += 1;
                    }
                    let mut attrs = HashMap::new();
                    for attr in e.attributes() {
                        let attr = attr.context("Malformed XML attribute")?;
                        let key = std::str::from_utf8(attr.key.as_ref())?.to_string();
                        let value = attr.unescape_value()?.into_owned();
                        attrs.insert(key, value);
                    }
                    // Only definitions own identities; other U attributes include
                    // boolean flags (MO/AF/FS) and references to definitions (SObj).
                    if matches!(e.name().as_ref(), b"C" | b"Co")
                        && let Some(id) = attrs.get("U")
                        && (id.is_empty() || !ids.insert(identity(id)))
                    {
                        bail!("Empty or duplicate UUID {id:?} in config XML.");
                    }
                    let empty = bytes[reader.buffer_position() as usize + offset - 2] == b'/';
                    let node = if e.name().as_ref() == b"C" && attrs.contains_key("U") {
                        let index = controls.len();
                        controls.push(Control {
                            id: identity(&attrs["U"]),
                            raw_id: attrs["U"].clone(),
                            kind: attrs
                                .get("Type")
                                .filter(|s| !s.is_empty())
                                .cloned()
                                .context("Control has no Type")?,
                            sp_no: attrs.get("SpNo").cloned(),
                            span: start..reader.buffer_position() as usize + offset,
                            close: reader.buffer_position() as usize + offset,
                            empty,
                            parent: stack.iter().rev().find_map(|i| *i),
                        });
                        Some(index)
                    } else {
                        None
                    };
                    if !empty {
                        stack.push(node);
                    }
                }
                Event::End(_) => {
                    let node = stack.pop().context("Unexpected XML closing tag")?;
                    if let Some(index) = node {
                        controls[index].span.end = reader.buffer_position() as usize + offset;
                        controls[index].close = start;
                    }
                }
                Event::DocType(_) => bail!("DTD in config XML is not supported."),
                Event::Text(text) if stack.is_empty() => {
                    if !text.unescape()?.trim().is_empty() {
                        bail!("Unexpected content outside config XML root.");
                    }
                }
                Event::Eof => break,
                _ => {}
            }
        }
        if roots != 1 || !stack.is_empty() {
            bail!("Config XML must contain exactly one complete root element.");
        }
        Ok(Self { bytes, controls })
    }

    fn control_bytes(&self, index: usize) -> &[u8] {
        &self.bytes[self.controls[index].span.clone()]
    }

    fn children(&self, parent: Option<usize>) -> Vec<usize> {
        self.controls
            .iter()
            .enumerate()
            .filter_map(|(i, c)| (c.parent == parent).then_some(i))
            .collect()
    }
}

fn checked_fragment(data: &[u8]) -> Result<Vec<u8>> {
    let xml = loxcc::decompress_loxcc(data)?;
    let field = |offset| u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
    if field(4) != data.len() - 16
        || field(8) != xml.len()
        || field(12) as u32 != crc32fast::hash(&xml)
    {
        bail!(
            "Runtime overlay LoxCC length or CRC32 mismatch; refusing a partial/corrupt fragment."
        );
    }
    Ok(xml)
}

pub fn merge_overlays(base: &[u8], overlays: &BTreeMap<String, Vec<u8>>) -> Result<Vec<u8>> {
    if overlays.is_empty() {
        return Ok(base.to_vec());
    }
    let mut result = base.to_vec();
    let document = Document::parse(base)?;
    let mut seen = HashSet::new();
    for name in overlays.keys() {
        let expected = overlay_id(name)?;
        if !seen.insert(expected.clone()) {
            bail!("Multiple runtime overlays identify UUID {expected}.");
        }
    }
    for control in &document.controls {
        if seen.contains(&control.id) {
            let mut parent = control.parent;
            while let Some(index) = parent {
                if seen.contains(&document.controls[index].id) {
                    bail!(
                        "Overlapping parent/child runtime overlays have no evidenced precedence. Reconcile with Loxone Config."
                    );
                }
                parent = document.controls[index].parent;
            }
        }
    }
    for (name, data) in overlays {
        let expected = overlay_id(name)?;
        let xml =
            checked_fragment(data).with_context(|| format!("Cannot read /sys/custom/{name}"))?;
        let fragment = Document::parse(&xml)?;
        let root = fragment
            .controls
            .first()
            .context("Runtime overlay is not a control")?;
        if fragment
            .controls
            .iter()
            .skip(1)
            .any(|c| seen.contains(&c.id))
        {
            bail!(
                "Runtime overlay {name} overlaps another control's fragment. Reconcile with Loxone Config."
            );
        }
        // The only evidenced fragment format is one standalone C, not a document wrapper.
        let offset = usize::from(xml.starts_with(b"\xef\xbb\xbf")) * 3;
        let mut reader = Reader::from_reader(&xml[offset..]);
        let (root_name, root_start) = loop {
            let start = reader.buffer_position() as usize + offset;
            match reader.read_event()? {
                Event::Start(e) | Event::Empty(e) => break (e.name().as_ref().to_vec(), start),
                Event::Decl(_) | Event::Comment(_) | Event::Text(_) => {}
                _ => bail!("Unsupported runtime overlay root in {name}."),
            }
        };
        let fragment_id = overlay_id(&format!("{}.LoxCC", root.raw_id))?;
        if root_name != b"C"
            || root.id != expected
            || fragment_id != expected
            || root.parent.is_some()
            || root.span.start != root_start
        {
            bail!("Runtime overlay {name} must be a standalone C with the filename's UUID.");
        }
        let current = Document::parse(&result)?;
        let matches: Vec<_> = current
            .controls
            .iter()
            .filter(|c| c.id == expected)
            .collect();
        if let Some(target) = matches.first() {
            if target.kind != root.kind {
                bail!(
                    "Runtime overlay {name} changes control Type; refusing ambiguous replacement."
                );
            }
            result.splice(
                target.span.clone(),
                fragment.control_bytes(0).iter().copied(),
            );
        } else if root.kind == "AutopilotRule" {
            let pages: Vec<_> = current
                .controls
                .iter()
                .filter(|c| c.kind == "Page" && c.sp_no.as_deref() == Some("1"))
                .collect();
            if pages.len() != 1 {
                bail!(
                    "Cannot place new AutopilotRule {name}: expected exactly one Page with SpNo=\"1\". Use Loxone Config to reconcile."
                );
            }
            let page = pages[0];
            let insertion = fragment.control_bytes(0);
            if page.empty {
                let mut replacement = result[page.span.start..page.span.end - 2].to_vec();
                replacement.push(b'>');
                replacement.extend_from_slice(insertion);
                replacement.extend_from_slice(b"</C>");
                result.splice(page.span.clone(), replacement);
            } else {
                result.splice(page.close..page.close, insertion.iter().copied());
            }
        } else {
            bail!(
                "Runtime overlay {name} has no matching control (Type {}). Only new AutopilotRule placement is evidenced; use Loxone Config.",
                root.kind
            );
        }
        Document::parse(&result)
            .with_context(|| format!("UUID consistency failed after {name}"))?;
    }
    Ok(result)
}

/// Compare XML without confusing formatting/attribute order with a config edit.
fn semantic(bytes: &[u8]) -> Result<Vec<String>> {
    let bytes = bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(bytes);
    let mut reader = Reader::from_reader(bytes);
    let mut tokens = Vec::new();
    loop {
        match reader.read_event()? {
            Event::Start(e) | Event::Empty(e) => {
                let empty = bytes[reader.buffer_position() as usize - 2] == b'/';
                let mut attrs = BTreeMap::new();
                for a in e.attributes() {
                    let a = a?;
                    attrs.insert(a.key.as_ref().to_vec(), a.unescape_value()?.into_owned());
                }
                let name = String::from_utf8(e.name().as_ref().to_vec())?;
                tokens.push(format!("start:{name}:{attrs:?}"));
                if empty {
                    tokens.push(format!("end:{name}"));
                }
            }
            Event::End(e) => tokens.push(format!(
                "end:{}",
                String::from_utf8_lossy(e.name().as_ref())
            )),
            Event::Text(e) => {
                let text = e.unescape()?;
                if !text.trim().is_empty() {
                    tokens.push(format!("text:{text}"));
                }
            }
            Event::Decl(_) => {}
            Event::Eof => break,
            e => tokens.push(format!("{e:?}")),
        }
    }
    Ok(tokens)
}

fn shell(doc: &Document<'_>, index: Option<usize>) -> Vec<u8> {
    let range = index
        .map(|i| doc.controls[i].span.clone())
        .unwrap_or(0..doc.bytes.len());
    let mut result = doc.bytes[range.clone()].to_vec();
    for child in doc.children(index).into_iter().rev() {
        let c = &doc.controls[child];
        let slot = format!("<OverlaySlot U=\"{}\"/>", c.id);
        result.splice(
            c.span.start - range.start..c.span.end - range.start,
            slot.bytes(),
        );
    }
    result
}

fn merge_node(
    base: &Document<'_>,
    local: &Document<'_>,
    live: &Document<'_>,
    indexes: (Option<usize>, Option<usize>, Option<usize>),
) -> Result<Vec<u8>> {
    let (bi, li, ri) = indexes;
    let bs = shell(base, bi);
    let ls = shell(local, li);
    let rs = shell(live, ri);
    let b = semantic(&bs)?;
    let l = semantic(&ls)?;
    let r = semantic(&rs)?;
    let (chosen, index) = if l == b {
        (live, ri)
    } else if r == b || l == r {
        (local, li)
    } else {
        bail!(
            "Three-way merge conflict at UUID {} (local and live both changed). Reconcile in Loxone Config; no deployment was performed.",
            li.map(|i| local.controls[i].id.as_str())
                .unwrap_or("document")
        );
    };
    let child_map = |doc: &Document<'_>, parent| -> HashMap<String, usize> {
        doc.children(parent)
            .into_iter()
            .map(|i| (doc.controls[i].id.clone(), i))
            .collect()
    };
    let bm = child_map(base, bi);
    let lm = child_map(local, li);
    let rm = child_map(live, ri);
    let mut merged = HashMap::new();
    let ids: HashSet<_> = bm
        .keys()
        .chain(lm.keys())
        .chain(rm.keys())
        .cloned()
        .collect();
    for id in ids {
        let b = bm.get(&id).copied();
        let l = lm.get(&id).copied();
        let r = rm.get(&id).copied();
        let value = match (b, l, r) {
            (Some(b), Some(l), Some(r)) => {
                Some(merge_node(base, local, live, (Some(b), Some(l), Some(r)))?)
            }
            (None, Some(l), None) => Some(local.control_bytes(l).to_vec()),
            (None, None, Some(r)) => Some(live.control_bytes(r).to_vec()),
            (None, Some(l), Some(r))
                if semantic(local.control_bytes(l))? == semantic(live.control_bytes(r))? =>
            {
                Some(local.control_bytes(l).to_vec())
            }
            (Some(b), None, Some(r))
                if semantic(base.control_bytes(b))? == semantic(live.control_bytes(r))? =>
            {
                None
            }
            (Some(b), Some(l), None)
                if semantic(base.control_bytes(b))? == semantic(local.control_bytes(l))? =>
            {
                None
            }
            (Some(_), None, None) => None,
            _ => bail!(
                "Three-way merge conflict for UUID {id}: concurrent addition, deletion, or modification. Reconcile in Loxone Config."
            ),
        };
        merged.insert(id, value);
    }
    let range = index
        .map(|i| chosen.controls[i].span.clone())
        .unwrap_or(0..chosen.bytes.len());
    let mut result = chosen.bytes[range.clone()].to_vec();
    for child in chosen.children(index).into_iter().rev() {
        let c = &chosen.controls[child];
        let value = merged.remove(&c.id).flatten().unwrap_or_default();
        result.splice(c.span.start - range.start..c.span.end - range.start, value);
    }
    if merged.values().any(Option::is_some) {
        bail!(
            "Three-way merge cannot determine placement of a concurrently added control. Reconcile in Loxone Config."
        );
    }
    Ok(result)
}

pub fn three_way(base: &[u8], local: &[u8], live: &[u8]) -> Result<Vec<u8>> {
    let base = Document::parse(base)?;
    let local = Document::parse(local)?;
    let live = Document::parse(live)?;
    let result = merge_node(&base, &local, &live, (None, None, None))?;
    Document::parse(&result).context("Merged config has inconsistent UUIDs")?;
    Ok(result)
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
    Document::parse(local)?;
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
