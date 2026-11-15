use super::EditError;
use std::io::{BufReader, Cursor};
use xmltree::Element;
type Result<T> = std::result::Result<T, EditError>;
pub(super) const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

/// A Loxone config file loaded for editing.
#[doc(hidden)]
pub struct State {
    pub root: Element,
    pub had_bom: bool,
    pub had_crlf: bool,
    /// Attributes renamed from digit-prefixed originals (e.g. `12hTF` → `_12hTF`).
    /// Stored as (sanitized, original) pairs for round-trip restoration in `to_bytes()`.
    pub digit_attr_renames: Vec<(String, String)>,
}

impl State {
    #[cfg(feature = "configuration-xml")]
    pub fn mqtt_topics(&self) -> Vec<super::super::mqtt::MqttTopic> {
        super::super::mqtt::from_root(&self.root)
    }

    /// Load a .Loxone XML file for editing.
    ///
    /// Loxone Config can produce attributes with digit-prefixed names (e.g. `12hTF="true"`)
    /// which are technically invalid XML. We sanitize these before parsing and restore
    /// them on write-back.
    pub fn load(data: &[u8]) -> Result<Self> {
        let had_bom = data.starts_with(UTF8_BOM);
        // Prefer the newline after the XML declaration, or the first outside markup
        // if there is no declaration. Attribute values carry independent line breaks.
        let had_crlf = Self::has_crlf_line_endings(data);
        let xml_data = if had_bom { &data[3..] } else { data };

        let (xml_data_cow, digit_attr_renames) = Self::sanitize_digit_attrs(xml_data);
        // Loxone emits the same attribute twice on some elements (value-identical).
        // A strict parser rejects that; drop the redundant lexical copy before parsing.
        let deduped = Self::dedupe_duplicate_attrs(xml_data_cow.as_ref());
        let reader = BufReader::new(Cursor::new(deduped.as_ref()));
        let root = Element::parse(reader).map_err(EditError::load)?;
        Ok(State {
            root,
            had_bom,
            had_crlf,
            digit_attr_renames,
        })
    }

    fn has_crlf_line_endings(data: &[u8]) -> bool {
        let mut i = 0;
        while i < data.len() {
            if data[i] == b'\n' {
                return i > 0 && data[i - 1] == b'\r';
            }
            if data[i] == b'<' {
                let tail = &data[i..];
                let end = if tail.starts_with(b"<!--") {
                    tail.windows(3)
                        .position(|w| w == b"-->")
                        .map(|end| i + end + 3)
                } else if tail.starts_with(b"<![CDATA[") {
                    tail.windows(3)
                        .position(|w| w == b"]]>")
                        .map(|end| i + end + 3)
                } else if tail.starts_with(b"<?") {
                    tail.windows(2)
                        .position(|w| w == b"?>")
                        .map(|end| i + end + 2)
                } else {
                    Self::xml_tag_end(data, i)
                };
                if let Some(end) = end {
                    i = end;
                    continue;
                }
            }
            i += 1;
        }
        false
    }

    /// Find attribute names starting with a digit and rename them with an underscore prefix.
    /// Returns the (possibly modified) data and a list of (sanitized, original) pairs.
    pub fn sanitize_digit_attrs(
        data: &[u8],
    ) -> (std::borrow::Cow<'_, [u8]>, Vec<(String, String)>) {
        let text = match std::str::from_utf8(data) {
            Ok(s) => s,
            Err(_) => return (std::borrow::Cow::Borrowed(data), Vec::new()),
        };

        // Scan for ` <digit><word-chars>=` patterns (attribute with digit-prefixed name).
        let mut renames: Vec<(String, String)> = Vec::new();
        let bytes = text.as_bytes();
        let len = bytes.len();
        let mut i = 0;
        while i < len {
            // Look for a space/tab/newline followed by a digit
            if (bytes[i] == b' ' || bytes[i] == b'\t' || bytes[i] == b'\n' || bytes[i] == b'\r')
                && i + 1 < len
                && bytes[i + 1].is_ascii_digit()
            {
                // Read the attribute name: digits and word chars until '='
                let start = i + 1;
                let mut end = start;
                while end < len && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
                    end += 1;
                }
                if end < len && bytes[end] == b'=' && end > start {
                    let original = &text[start..end];
                    let sanitized = format!("_{original}");
                    if !renames.iter().any(|(_, o)| o == original) {
                        renames.push((sanitized, original.to_string()));
                    }
                }
            }
            i += 1;
        }

        if renames.is_empty() {
            return (std::borrow::Cow::Borrowed(data), renames);
        }

        // Apply replacements
        let mut result = text.to_string();
        for (sanitized, original) in &renames {
            // Legacy global replacement is not quote/tag-aware (lox-n37w).
            // Preserve its compatibility limitation: quoted/text content can change too.
            let from = format!(" {original}=");
            let to = format!(" {sanitized}=");
            result = result.replace(&from, &to);
        }
        (std::borrow::Cow::Owned(result.into_bytes()), renames)
    }

    /// Remove **value-identical** duplicate attributes from element start tags.
    ///
    /// Loxone Config emits the same attribute twice on some elements (observed on API
    /// Connector elements: `Title` appears twice with the same value). A strict XML parser
    /// rejects that ("Attribute 'Title' is redefined"). Since the duplicates are
    /// value-identical, dropping the redundant copy is lossless. Duplicates with *different*
    /// values are left untouched so a genuine conflict still surfaces at parse time.
    ///
    /// Both XML quote styles are supported; non-element markup is left untouched.
    pub fn dedupe_duplicate_attrs(data: &[u8]) -> std::borrow::Cow<'_, [u8]> {
        let text = match std::str::from_utf8(data) {
            Ok(s) => s,
            Err(_) => return std::borrow::Cow::Borrowed(data),
        };
        let bytes = text.as_bytes();
        let len = bytes.len();
        let ws = |b: u8| matches!(b, b' ' | b'\t' | b'\r' | b'\n');
        let mut deletes: Vec<(usize, usize)> = Vec::new();
        let mut i = 0;
        while i < len {
            if bytes[i] != b'<' {
                i += 1;
                continue;
            }
            let tail = &text[i..];
            let skipped_end = if tail.starts_with("<!--") {
                tail.find("-->").map(|end| i + end + 3)
            } else if tail.starts_with("<![CDATA[") {
                tail.find("]]>").map(|end| i + end + 3)
            } else if tail.starts_with("<?") {
                tail.find("?>").map(|end| i + end + 2)
            } else if tail.starts_with("<!") {
                // A declaration can contain an internal subset with tag-like text.
                let mut quote = None;
                let mut depth: usize = 0;
                let mut end = None;
                for (offset, &byte) in bytes.iter().enumerate().skip(i + 2) {
                    if let Some(delimiter) = quote {
                        if byte == delimiter {
                            quote = None;
                        }
                    } else {
                        match byte {
                            b'"' | b'\'' => quote = Some(byte),
                            b'[' => depth += 1,
                            b']' => depth = depth.saturating_sub(1),
                            b'>' if depth == 0 => {
                                end = Some(offset + 1);
                                break;
                            }
                            _ => {}
                        }
                    }
                }
                end
            } else if tail.starts_with("</") {
                Self::xml_tag_end(bytes, i)
            } else {
                None
            };
            if let Some(end) = skipped_end {
                i = end;
                continue;
            }
            if i + 1 < len && !matches!(bytes[i + 1], b'/' | b'?' | b'!') {
                // Skip the element name.
                let mut j = i + 1;
                while j < len && !ws(bytes[j]) && !matches!(bytes[j], b'>' | b'/') {
                    j += 1;
                }
                // Scan attributes until the end of the start tag.
                let mut seen: Vec<(&str, &str)> = Vec::new();
                loop {
                    while j < len && ws(bytes[j]) {
                        j += 1;
                    }
                    if j >= len || matches!(bytes[j], b'>' | b'/') {
                        break;
                    }
                    let name_start = j;
                    while j < len && !ws(bytes[j]) && !matches!(bytes[j], b'=' | b'>' | b'/') {
                        j += 1;
                    }
                    let name_end = j;
                    while j < len && ws(bytes[j]) {
                        j += 1;
                    }
                    if j >= len || bytes[j] != b'=' {
                        break; // not `name=…`, stop scanning this tag
                    }
                    j += 1;
                    while j < len && ws(bytes[j]) {
                        j += 1;
                    }
                    if j >= len || !matches!(bytes[j], b'"' | b'\'') {
                        break;
                    }
                    let delimiter = bytes[j];
                    let val_start = j + 1;
                    j = val_start;
                    while j < len && bytes[j] != delimiter {
                        j += 1;
                    }
                    if j >= len {
                        break;
                    }
                    let val_end = j;
                    j += 1; // past closing quote
                    let attr_end = j;
                    let name = &text[name_start..name_end];
                    let value = &text[val_start..val_end];
                    if let Some(&(_, prev)) = seen.iter().find(|(n, _)| *n == name) {
                        if prev == value {
                            // Redundant copy → delete it plus one leading whitespace char.
                            let del_start = if name_start > 0 && ws(bytes[name_start - 1]) {
                                name_start - 1
                            } else {
                                name_start
                            };
                            deletes.push((del_start, attr_end));
                        }
                        // Different value: leave it, let the parser flag the conflict.
                    } else {
                        seen.push((name, value));
                    }
                }
                i = j;
                continue;
            }
            i += 1;
        }
        if deletes.is_empty() {
            return std::borrow::Cow::Borrowed(data);
        }
        let mut out = String::with_capacity(text.len());
        let mut last = 0;
        for (s, e) in deletes {
            if s >= last {
                out.push_str(&text[last..s]);
                last = e;
            }
        }
        out.push_str(&text[last..]);
        std::borrow::Cow::Owned(out.into_bytes())
    }

    /// Get a mutable reference to an element by path.
    pub fn get_element_mut(&mut self, path: &[usize]) -> &mut Element {
        let mut current = &mut self.root;
        for &idx in path {
            current = current.children[idx].as_mut_element().unwrap();
        }
        current
    }

    /// Get a reference to an element by path.
    pub fn get_element(&self, path: &[usize]) -> &Element {
        let mut current = &self.root;
        for &idx in path {
            current = current.children[idx].as_element().unwrap();
        }
        current
    }
}

/// Check if an element matches a selector string.
pub(super) fn matches_selector(elem: &Element, selector: &str) -> bool {
    if elem.name != "C" {
        return false;
    }

    if let Some(uuid) = selector.strip_prefix("uuid:") {
        return elem.attributes.get("U").map(|u| u == uuid).unwrap_or(false);
    }
    if let Some(gid) = selector.strip_prefix("gid:") {
        return elem
            .attributes
            .get("gid")
            .map(|g| g.eq_ignore_ascii_case(gid))
            .unwrap_or(false);
    }
    if let Some(type_name) = selector.strip_prefix("Type:") {
        return elem
            .attributes
            .get("Type")
            .map(|t| t.eq_ignore_ascii_case(type_name))
            .unwrap_or(false);
    }

    // Bracket syntax: "Title [Room]" — match by title AND room name
    if let Some(bracket_start) = selector.rfind('[')
        && selector.ends_with(']')
    {
        let title_part = selector[..bracket_start].trim();
        let _room_part = &selector[bracket_start + 1..selector.len() - 1];
        let title_match = elem
            .attributes
            .get("Title")
            .map(|t| t.to_lowercase().contains(&title_part.to_lowercase()))
            .unwrap_or(false);
        if !title_match {
            return false;
        }
        // Check room via IoData Pr attribute — walk children for IoData
        let room_uuid = elem.children.iter().find_map(|c| {
            c.as_element()
                .filter(|e| e.name == "IoData")
                .and_then(|e| e.attributes.get("Pr").cloned())
        });
        if let Some(_ru) = room_uuid {
            // We need to check if this room UUID matches the room name.
            // Store the room_part for post-filtering in require_one.
            // For now, embed room UUID match in selector via a static approach:
            // We can't access the full tree here, so use a simpler heuristic:
            // the room_part will be checked by the caller after find_elements.
            return true; // Title matches, room will be filtered by caller
        }
        return false;
    }

    // Default: match by Title (case-insensitive contains)
    elem.attributes
        .get("Title")
        .map(|t| t.to_lowercase().contains(&selector.to_lowercase()))
        .unwrap_or(false)
}
