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
            buf = Self::to_crlf_outside_attrs(&s).into_bytes();
        }

        Ok(buf)
    }

    /// Convert LF to CRLF everywhere except inside start tags.
    ///
    /// Loxone terminates lines with CRLF but writes newlines inside attribute values
    /// (note text, AutoPilot JSON) as bare LF. Converting those too would plant a CR
    /// in the value that xml-rs reads back literally and re-emits as `&#xD;`, adding
    /// one more CR per newline on every save.
    fn to_crlf_outside_attrs(s: &str) -> String {
        let bytes = s.as_bytes();
        let mut out = String::with_capacity(bytes.len() + bytes.len() / 32);
        let mut last = 0;
        let mut i = 0;

        while i < bytes.len() {
            if bytes[i] != b'<' {
                i += 1;
                continue;
            }

            let tail = &s[i..];
            let opaque_end = if tail.starts_with("<!--") {
                tail.find("-->").map(|end| i + end + 3)
            } else if tail.starts_with("<![CDATA[") {
                tail.find("]]>").map(|end| i + end + 3)
            } else if tail.starts_with("<?") {
                tail.find("?>").map(|end| i + end + 2)
            } else {
                None
            };
            if let Some(end) = opaque_end {
                i = end;
                continue;
            }

            let Some(end) = Self::xml_tag_end(bytes, i) else {
                break;
            };
            out.push_str(&s[last..i].replace("\r\n", "\n").replace('\n', "\r\n"));
            out.push_str(&s[i..end]);
            i = end;
            last = i;
        }

        out.push_str(&s[last..].replace("\r\n", "\n").replace('\n', "\r\n"));
        out
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
        if name_end + 2 == bytes.len() && bytes[name_end] == b'/' && bytes[name_end + 1] == b'>' {
            let name = &tag[1..name_end];
            return format!("<{name}></{name}>");
        }

        // xml-rs escapes CR and LF in attributes; Loxone writes them literally (bare LF
        // in note text, CRLF in AutoPilot JSON), so restore whichever was read.
        let mut out = String::with_capacity(tag.len());
        let mut quote = None;
        let mut last = 0;
        let mut i = name_end;
        while i < bytes.len() {
            if let Some(delimiter) = quote {
                let literal = if bytes[i..].starts_with(b"&#xA;") {
                    Some('\n')
                } else if bytes[i..].starts_with(b"&#xD;") {
                    Some('\r')
                } else {
                    None
                };
                if let Some(c) = literal {
                    out.push_str(&tag[last..i]);
                    out.push(c);
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
        assert!(
            out.contains("<IoData></IoData>"),
            "attr-less empty stays expanded"
        );
        assert!(
            out.contains(r#"<Co K="Q" U="b"/>"#),
            "attributed empty self-closes"
        );
        assert!(
            out.contains("line1\nline2"),
            "literal newline in attr value"
        );
        assert!(!out.contains("&#xA;"), "no escaped newline");
        assert!(out.ends_with('\n'), "trailing newline");
    }

    #[test]
    fn test_crlf_file_keeps_lf_in_attr_values_across_saves() {
        // Loxone: CRLF line endings, bare LF in note text, CRLF in AutoPilot JSON.
        let xml = "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n\
<ControlList Version=\"1\">\r\n\
\t<C Type=\"Text\" U=\"a\" Text=\"line1\nline2\" APC=\"{&quot;a&quot;:1},\r\n {&quot;b&quot;:2}\">\r\n\
\t\t<Co K=\"Q\" U=\"b\"/>\r\n\
\t</C>\r\n\
</ControlList>\r\n";
        let first = ConfigEditor::load(xml.as_bytes())
            .unwrap()
            .to_bytes()
            .unwrap();
        let second = ConfigEditor::load(&first).unwrap().to_bytes().unwrap();
        let out = String::from_utf8(second.clone()).unwrap();

        assert_eq!(String::from_utf8(first.clone()).unwrap(), xml, "byte-clean");
        assert_eq!(first, second, "save is idempotent");
        assert!(out.contains("},\r\n {"), "attr CRLF stays CRLF");
        assert!(!out.contains("&#xD;"), "no CR planted in attr values");
        assert!(
            out.contains("Text=\"line1\nline2\""),
            "attr newline stays LF"
        );
        assert!(out.contains("\t</C>\r\n"), "line endings stay CRLF");
        assert!(!out.contains("\r\r"));
    }

    #[test]
    fn test_crlf_conversion_preserves_attrs_and_opaque_markup() {
        let xml = "<?xml version=\"1.0\"?>\r\n\
<C Text='one\ntwo' APC=\"one\r\ntwo\">\r\n\
<!-- <C A=\"x\"> one\r\ntwo -->\r\n\
<![CDATA[<C A=\"x\"> one\r\ntwo]]>\r\n\
<?test <C A=\"x\"> one\r\ntwo?>\r\n\
</C>\r\n";
        assert_eq!(ConfigEditor::to_crlf_outside_attrs(xml), xml);
        assert_eq!(
            ConfigEditor::to_crlf_outside_attrs("<C Text='one\ntwo'>\n</C>\n"),
            "<C Text='one\ntwo'>\r\n</C>\r\n"
        );
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

    #[test]
    fn test_repeated_writes_preserve_multiline_attributes_and_line_endings() {
        for newline in ["\n", "\r\n"] {
            let note = format!("first{newline}second{newline}third");
            let xml = format!(
                "<?xml version=\"1.0\" encoding=\"utf-8\"?>{newline}\
<ControlList Version=\"1\">{newline}\
\t<C Type=\"Text\" Title=\"Note\" Text=\"{note}\"/>{newline}\
</ControlList>{newline}"
            );
            let mut bytes = super::UTF8_BOM.to_vec();
            bytes.extend_from_slice(xml.as_bytes());
            for edit in 0..3 {
                let mut editor = ConfigEditor::load(&bytes).unwrap();
                editor
                    .root
                    .attributes
                    .insert("Unrelated".into(), edit.to_string());
                bytes = editor.to_bytes().unwrap();
                assert!(bytes.starts_with(super::UTF8_BOM));
                let text = std::str::from_utf8(&bytes).unwrap();
                assert!(text.contains(&format!("Text=\"{note}\"")));
                assert!(!text.contains("&#xD;"));
                assert!(!text.contains("\r\r\n"));
                if newline == "\r\n" {
                    assert!(!text.replace("\r\n", "").contains('\n'));
                }
            }
        }
    }

    #[test]
    fn test_write_preserves_lone_attribute_cr_and_literal_entity_text() {
        let xml = b"<ControlList><C Text=\"left&#13;right\" Literal=\"&amp;#xD;\"/></ControlList>";
        let editor = ConfigEditor::load(xml).unwrap();
        let bytes = editor.to_bytes().unwrap();
        let text = std::str::from_utf8(&bytes).unwrap();
        assert!(text.contains("Text=\"left\rright\""));
        assert!(text.contains("Literal=\"&amp;#xD;\""));
        let reloaded = ConfigEditor::load(&bytes).unwrap();
        assert_eq!(
            reloaded.root.get_child("C").unwrap().attributes,
            editor.root.get_child("C").unwrap().attributes
        );
    }
}
