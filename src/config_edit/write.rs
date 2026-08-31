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

        // Post-process: Loxone writes attribute-less empty elements expanded
        // (`<IoData></IoData>`, never `<IoData/>`). xml-rs always self-closes, so expand
        // the attribute-less self-closing tags to match and keep round-trips byte-clean.
        {
            let s = String::from_utf8(buf).context("XML is not valid UTF-8")?;
            let mut s = Self::expand_attrless_empty_tags(&s);
            // Loxone keeps literal newlines inside attribute values (e.g. multi-line PicoC
            // code, notification texts); xml-rs escapes them to `&#xA;`. Un-escape to match.
            // Loxone never emits `&#xA;`, so this only reverses xml-rs's own escaping.
            s = s.replace("&#xA;", "\n");
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

    /// Expand attribute-less self-closing tags (`<IoData/>` → `<IoData></IoData>`).
    ///
    /// Loxone writes empty elements that have no attributes in expanded form. xml-rs always
    /// self-closes; only tags of the shape `<Name/>` (name immediately followed by `/>`, i.e.
    /// no attributes) are rewritten — attributed empty tags like `<Co K="I" U="…"/>` are left
    /// self-closed, matching Loxone.
    fn expand_attrless_empty_tags(s: &str) -> String {
        let bytes = s.as_bytes();
        let n = bytes.len();
        let mut out = String::with_capacity(n);
        let mut last = 0;
        let mut i = 0;
        while i < n {
            if bytes[i] == b'<'
                && i + 1 < n
                && (bytes[i + 1].is_ascii_alphabetic() || bytes[i + 1] == b'_')
            {
                let name_start = i + 1;
                let mut j = name_start;
                while j < n
                    && (bytes[j].is_ascii_alphanumeric() || matches!(bytes[j], b'_' | b':' | b'-'))
                {
                    j += 1;
                }
                if j + 1 < n && bytes[j] == b'/' && bytes[j + 1] == b'>' {
                    out.push_str(&s[last..i]);
                    let name = &s[name_start..j];
                    out.push('<');
                    out.push_str(name);
                    out.push_str("></");
                    out.push_str(name);
                    out.push('>');
                    i = j + 2;
                    last = i;
                    continue;
                }
                i = j; // skip past the element name
                continue;
            }
            i += 1;
        }
        out.push_str(&s[last..]);
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
}
