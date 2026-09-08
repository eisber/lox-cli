use super::{ConfigEditor, UTF8_BOM};
use anyhow::{Context, Result};

impl ConfigEditor {
    /// Write the edited XML back to bytes, preserving BOM and line endings.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();

        // Write XML declaration
        buf.extend_from_slice(b"<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");

        // Write DOM tree
        let config = xmltree::EmitterConfig::new()
            .perform_indent(true)
            .indent_string("\t")
            .write_document_declaration(false)
            // Loxone writes self-closing tags without a leading space (`<C/>`, not `<C />`).
            // xml-rs pads by default, which makes every self-closing element differ on
            // round-trip and drowns real changes in formatting noise.
            .pad_self_closing(false);
        self.root
            .write_with_config(&mut buf, config)
            .context("Failed to write XML")?;

        // Post-process: restore digit-prefixed attribute names
        if !self.digit_attr_renames.is_empty() {
            let mut s = String::from_utf8(buf).context("XML is not valid UTF-8")?;
            for (sanitized, original) in &self.digit_attr_renames {
                let from = format!(" {sanitized}=");
                let to = format!(" {original}=");
                s = s.replace(&from, &to);
            }
            buf = s.into_bytes();
        }

        // Post-process XML syntax emitted by xml-rs to match Loxone's formatting.
        {
            let s = String::from_utf8(buf).context("XML is not valid UTF-8")?;
            let mut s = Self::normalize_loxone_xml(&s);
            // Loxone terminates the file with a trailing newline; xml-rs does not.
            if !s.ends_with('\n') {
                s.push('\n');
            }
            buf = s.into_bytes();
        }

        // Post-process: restore BOM
        if self.had_bom {
            let mut result = Vec::with_capacity(3 + buf.len());
            result.extend_from_slice(UTF8_BOM);
            result.extend_from_slice(&buf);
            buf = result;
        }

        // Post-process: restore CRLF line endings
        if self.had_crlf {
            let s = String::from_utf8(buf).context("XML is not valid UTF-8")?;
            buf = s.replace('\n', "\r\n").into_bytes();
        }

        Ok(buf)
    }

    /// Match Loxone's empty-element and attribute-newline formatting.
    ///
    /// Only start tags are rewritten so XML-like text in comments, CDATA, and processing
    /// instructions remains unchanged.
    fn normalize_loxone_xml(s: &str) -> String {
        let bytes = s.as_bytes();
        let mut out = String::with_capacity(bytes.len());
        let mut last = 0;
        let mut i = 0;

        while i < bytes.len() {
            if bytes[i] != b'<' {
                i += 1;
                continue;
            }

            let tail = &s[i..];
            let skipped_end = if tail.starts_with("<!--") {
                tail.find("-->").map(|end| i + end + 3)
            } else if tail.starts_with("<![CDATA[") {
                tail.find("]]>").map(|end| i + end + 3)
            } else if tail.starts_with("<?") {
                tail.find("?>").map(|end| i + end + 2)
            } else if tail.starts_with("</") || tail.starts_with("<!") {
                Self::xml_tag_end(bytes, i)
            } else {
                None
            };
            if let Some(end) = skipped_end {
                i = end;
                continue;
            }

            let Some(end) = Self::xml_tag_end(bytes, i) else {
                break;
            };
            out.push_str(&s[last..i]);
            out.push_str(&Self::normalize_start_tag(&s[i..end]));
            i = end;
            last = i;
        }

        out.push_str(&s[last..]);
        out
    }

    fn xml_tag_end(bytes: &[u8], start: usize) -> Option<usize> {
        let mut quote = None;
        for (i, &byte) in bytes.iter().enumerate().skip(start + 1) {
            if let Some(delimiter) = quote {
                if byte == delimiter {
                    quote = None;
                }
            } else if matches!(byte, b'"' | b'\'') {
                quote = Some(byte);
            } else if byte == b'>' {
                return Some(i + 1);
            }
        }
        None
    }

    fn normalize_start_tag(tag: &str) -> String {
        let bytes = tag.as_bytes();
        let mut name_end = 1;
        while name_end < bytes.len()
            && (bytes[name_end].is_ascii_alphanumeric()
                || matches!(bytes[name_end], b'_' | b':' | b'-' | b'.'))
        {
            name_end += 1;
        }

        // Loxone expands attribute-less empty elements but keeps attributed empties closed.
        if name_end + 2 == bytes.len()
            && bytes[name_end] == b'/'
            && bytes[name_end + 1] == b'>'
        {
            let name = &tag[1..name_end];
            return format!("<{name}></{name}>");
        }

        // xml-rs escapes newlines in attributes; Loxone writes them literally.
        let mut out = String::with_capacity(tag.len());
        let mut quote = None;
        let mut last = 0;
        let mut i = name_end;
        while i < bytes.len() {
            if let Some(delimiter) = quote {
                if bytes[i..].starts_with(b"&#xA;") {
                    out.push_str(&tag[last..i]);
                    out.push('\n');
                    i += 5;
                    last = i;
                    continue;
                }
                if bytes[i] == delimiter {
                    quote = None;
                }
            } else if matches!(bytes[i], b'"' | b'\'') {
                quote = Some(bytes[i]);
            }
            i += 1;
        }
        out.push_str(&tag[last..]);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::ConfigEditor;

    #[test]
    fn test_write_matches_loxone_formatting() {
        // Loxone's on-disk conventions the emitter must reproduce for byte-clean round-trips:
        // no space before '/>', attribute-less empties expanded, attributed empties
        // self-closed, literal newlines in attribute values, trailing newline.
        let xml = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
<ControlList Version=\"1\">\n\
\t<C U=\"a\" NTXT=\"line1\nline2\">\n\
\t\t<IoData></IoData>\n\
\t\t<Co K=\"Q\" U=\"b\"></Co>\n\
\t</C>\n\
</ControlList>\n";
        let editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let out = String::from_utf8(editor.to_bytes().unwrap()).unwrap();
        assert!(!out.contains(" />"), "no padded self-close");
        assert!(out.contains("<IoData></IoData>"), "attr-less empty stays expanded");
        assert!(out.contains(r#"<Co K="Q" U="b"/>"#), "attributed empty self-closes");
        assert!(out.contains("line1\nline2"), "literal newline in attr value");
        assert!(!out.contains("&#xA;"), "no escaped newline");
        assert!(out.ends_with('\n'), "trailing newline");
    }

    #[test]
    fn test_write_preserves_comment_and_cdata_payloads() {
        let xml = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
<ControlList Version=\"1\">\n\
\t<!-- preserve <IoData/> and &#xA; literally -->\n\
\t<![CDATA[preserve <IoData/> and &#xA; literally]]>\n\
</ControlList>\n";
        let editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let out = String::from_utf8(editor.to_bytes().unwrap()).unwrap();

        assert!(out.contains("<!-- preserve <IoData/> and &#xA; literally -->"));
        assert!(out.contains("<![CDATA[preserve <IoData/> and &#xA; literally]]>"));
    }
}
