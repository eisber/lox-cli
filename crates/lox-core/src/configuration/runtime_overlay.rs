//! Existing byte-preserving runtime overlay application and three-way merge.
//!
//! These pure transactions preserve selected source spans, not canonical XML.
//! They neither acquire snapshots nor validate deployment sidecars/permissions.

use super::archive;
use quick_xml::{Reader, events::Event};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::ops::Range;

mod error;
pub use error::{OverlayError, XmlContext};

/// Result of an atomic in-memory operation; input slices are never modified.
pub type Result<T> = std::result::Result<T, OverlayError>;

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
        .ok_or(OverlayError::FilenameSuffix)?;
    let groups: Vec<_> = stem.split('-').map(str::len).collect();
    if !matches!(groups.as_slice(), [8, 4, 4, 16] | [8, 4, 4, 4, 12])
        || !stem.bytes().all(|b| b.is_ascii_hexdigit() || b == b'-')
    {
        return Err(OverlayError::InvalidFilename { name: name.into() });
    }
    Ok(identity(stem))
}

impl<'a> Document<'a> {
    fn parse(bytes: &'a [u8]) -> Result<Self> {
        std::str::from_utf8(bytes).map_err(|source| OverlayError::DocumentUtf8 { source })?;
        let offset = usize::from(bytes.starts_with(b"\xef\xbb\xbf")) * 3;
        let mut reader = Reader::from_reader(&bytes[offset..]);
        let mut controls: Vec<Control> = Vec::new();
        let mut stack: Vec<Option<usize>> = Vec::new();
        let mut ids = HashSet::new();
        let mut roots = 0;
        loop {
            let start = reader.buffer_position() as usize + offset;
            match reader.read_event().map_err(|source| OverlayError::Xml {
                source,
                context: XmlContext::Document,
            })? {
                Event::Start(ref e) | Event::Empty(ref e) => {
                    if stack.is_empty() {
                        roots += 1;
                    }
                    let mut attrs = HashMap::new();
                    for attr in e.attributes() {
                        let attr = attr.map_err(|source| OverlayError::Attribute {
                            source,
                            context: XmlContext::Document,
                        })?;
                        let key = std::str::from_utf8(attr.key.as_ref())?.to_string();
                        let value = attr.unescape_value()?.into_owned();
                        attrs.insert(key, value);
                    }
                    if matches!(e.name().as_ref(), b"C" | b"Co")
                        && let Some(id) = attrs.get("U")
                        && (id.is_empty() || !ids.insert(identity(id)))
                    {
                        return Err(OverlayError::DuplicateUuid { id: id.clone() });
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
                                .ok_or(OverlayError::MissingControlType)?,
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
                    let node = stack.pop().ok_or(OverlayError::UnexpectedClosingTag)?;
                    if let Some(index) = node {
                        controls[index].span.end = reader.buffer_position() as usize + offset;
                        controls[index].close = start;
                    }
                }
                Event::DocType(_) => return Err(OverlayError::Dtd),
                Event::Text(text) if stack.is_empty() => {
                    if !text.unescape()?.trim().is_empty() {
                        return Err(OverlayError::OutsideRoot);
                    }
                }
                Event::Eof => break,
                _ => {}
            }
        }
        if roots != 1 || !stack.is_empty() {
            return Err(OverlayError::IncompleteRoot);
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

/// Apply the same document/UUID checks used by both merge operations.
///
/// This retains the existing preparation check even when no baseline is present.
/// It is not schema, sidecar or deployment validation and exposes no parsed tree.
pub fn validate_document(bytes: &[u8]) -> Result<()> {
    Document::parse(bytes).map(|_| ())
}

fn checked_fragment(data: &[u8]) -> Result<Vec<u8>> {
    let xml = archive::decompress_loxcc(data).map_err(|source| OverlayError::Archive { source })?;
    let field = |offset| u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
    if field(4) != data.len() - 16
        || field(8) != xml.len()
        || field(12) as u32 != crc32fast::hash(&xml)
    {
        return Err(OverlayError::FragmentIntegrity);
    }
    Ok(xml)
}

/// Apply filename-ordered standalone LoxCC controls to exact base XML bytes.
///
/// An empty map bypasses parsing and returns the original bytes. Otherwise UUID
/// aliases/overlaps, corrupt fragments, type changes and unevidenced insertions
/// are refused. Only a new AutopilotRule has the existing unique Page/SpNo=1
/// placement rule. Untouched spans (including BOM and literal newlines) survive.
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
            return Err(OverlayError::DuplicateOverlay { id: expected });
        }
    }
    for control in &document.controls {
        if seen.contains(&control.id) {
            let mut parent = control.parent;
            while let Some(index) = parent {
                if seen.contains(&document.controls[index].id) {
                    return Err(OverlayError::ParentChildOverlap);
                }
                parent = document.controls[index].parent;
            }
        }
    }
    for (name, data) in overlays {
        let expected = overlay_id(name)?;
        let xml = checked_fragment(data).map_err(|source| OverlayError::FragmentRead {
            name: name.clone(),
            source: Box::new(source),
        })?;
        let fragment = Document::parse(&xml)?;
        let root = fragment.controls.first().ok_or(OverlayError::NotControl)?;
        if fragment
            .controls
            .iter()
            .skip(1)
            .any(|c| seen.contains(&c.id))
        {
            return Err(OverlayError::FragmentOverlap { name: name.clone() });
        }
        // The only evidenced fragment format is one standalone C, not a document wrapper.
        let offset = usize::from(xml.starts_with(b"\xef\xbb\xbf")) * 3;
        let mut reader = Reader::from_reader(&xml[offset..]);
        let (root_name, root_start) = loop {
            let start = reader.buffer_position() as usize + offset;
            match reader.read_event()? {
                Event::Start(e) | Event::Empty(e) => break (e.name().as_ref().to_vec(), start),
                Event::Decl(_) | Event::Comment(_) | Event::Text(_) => {}
                _ => return Err(OverlayError::UnsupportedRoot { name: name.clone() }),
            }
        };
        let fragment_id = overlay_id(&format!("{}.LoxCC", root.raw_id))?;
        if root_name != b"C"
            || root.id != expected
            || fragment_id != expected
            || root.parent.is_some()
            || root.span.start != root_start
        {
            return Err(OverlayError::InvalidControlRoot { name: name.clone() });
        }
        let current = Document::parse(&result)?;
        let matches: Vec<_> = current
            .controls
            .iter()
            .filter(|c| c.id == expected)
            .collect();
        if let Some(target) = matches.first() {
            if target.kind != root.kind {
                return Err(OverlayError::TypeChange { name: name.clone() });
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
                return Err(OverlayError::RulePlacement { name: name.clone() });
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
            return Err(OverlayError::UnknownControl {
                name: name.clone(),
                kind: root.kind.clone(),
            });
        }
        Document::parse(&result).map_err(|source| OverlayError::UuidConsistency {
            name: name.clone(),
            source: Box::new(source),
        })?;
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
        return Err(OverlayError::ShellConflict {
            id: li
                .map(|i| local.controls[i].id.as_str())
                .unwrap_or("document")
                .into(),
        });
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
            _ => return Err(OverlayError::ControlConflict { id }),
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
        return Err(OverlayError::Placement);
    }
    Ok(result)
}

/// Merge using the existing semantic comparison and chosen raw shell/child spans.
///
/// Local equal to base chooses live; otherwise live equal to base or local equal
/// to live chooses local. Concurrent incompatible changes/placement are refused.
/// No XML canonicalization, sorting or new conflict-resolution policy is applied.
pub fn three_way(base: &[u8], local: &[u8], live: &[u8]) -> Result<Vec<u8>> {
    let base = Document::parse(base)?;
    let local = Document::parse(local)?;
    let live = Document::parse(live)?;
    let result = merge_node(&base, &local, &live, (None, None, None))?;
    Document::parse(&result).map_err(|source| OverlayError::MergedConsistency {
        source: Box::new(source),
    })?;
    Ok(result)
}
