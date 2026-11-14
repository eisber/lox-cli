use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use quick_xml::{Reader, events::Event};
use serde::Serialize;
use xmltree::{Element, XMLNode};

use super::{ConfigEditor, alarm_clock::entry_name_ordinal, icons::native_identity};

#[derive(Debug, Serialize)]
pub struct VirtualInputFormatChange {
    pub block_uuid: String,
    pub previous_format: String,
    pub format: String,
    pub changed: bool,
}

type RawGuards = BTreeMap<usize, (String, BTreeMap<String, String>)>;

fn stored_identity(value: &str) -> Option<uuid::Uuid> {
    (value.len() == 35)
        .then(|| native_identity(value))
        .flatten()
        .filter(|identity| !identity.is_nil())
}

fn kind(element: &Element) -> &str {
    element.attributes.get("Type").map_or("", String::as_str)
}

impl ConfigEditor {
    /// Edit the recovered ordinary analog VirtualIn string-property branch without a DOM resave.
    pub fn edit_virtual_input_format(
        data: &[u8],
        selector: &str,
        format: &str,
    ) -> Result<(Vec<u8>, VirtualInputFormatChange)> {
        let suffix = ["<v>", "<v.0>", "<v.1>", "<v.2>", "<v.3>"]
            .iter()
            .find_map(|prefix| format.strip_prefix(prefix));
        ensure!(
            format.len() <= 128
                && suffix.is_some_and(|suffix| suffix
                    .bytes()
                    .all(|byte| { byte.is_ascii_alphanumeric() || b" %/._-".contains(&byte) })),
            "Virtual Input format supports <v> or <v.0> through <v.3> followed by an optional ASCII unit suffix (letters, digits, space, %, /, ., _, -), at most 128 bytes. Empty/default reset, Unicode and other placeholders are unsupported; nothing was changed."
        );
        let requested = selector.strip_prefix("uuid:").unwrap_or("");
        ensure!(
            stored_identity(requested).is_some(),
            "Virtual Input format requires uuid:<exact nonnil native35 U> (8-4-4-16); nothing was changed."
        );
        ensure!(
            Self::sanitize_digit_attrs(data).1.is_empty(),
            "Virtual Input format refuses shared-loader digit-name rewrites (lox-n37w), including digit-like attribute text; nothing was changed."
        );
        let mut editor = Self::load(data).context(
            "Cannot admit Virtual Input format XML or conflicting duplicate attributes; nothing was changed",
        )?;
        let all = Self::iter_elements(&editor.root);
        ensure!(
            all.iter().all(|element| {
                element.namespace.is_none()
                    && element.prefix.is_none()
                    && !element.attributes.keys().any(|key| key.contains(':'))
                    && element.name != "Template"
                    && kind(element) != "DocumentTemplate"
                    && !element.attributes.contains_key("IfT")
                    && !element.attributes.contains_key("uuidParent")
            }),
            "Virtual Input format does not support namespaces or Template/DocumentTemplate/IfT/uuidParent state; nothing was changed."
        );
        let documents = editor.find_elements("Type:Document");
        let owners = editor.find_elements(selector);
        ensure!(
            documents.len() == 1
                && documents[0].len() <= 1
                && owners.len() == 1
                && owners[0].starts_with(&documents[0])
                && owners[0].len() == documents[0].len() + 3,
            "Virtual Input format requires one exact owner in one ordinary root/direct Document -> LoxLIVE -> VirtualInCaption -> VirtualIn; duplicate owners and other contexts are unsupported. Nothing was changed."
        );
        let owner_path = &owners[0];
        let document = editor.get_element(&documents[0]);
        let live = editor.get_element(&owner_path[..owner_path.len() - 2]);
        let caption = editor.get_element(&owner_path[..owner_path.len() - 1]);
        let owner = editor.get_element(owner_path);
        ensure!(
            kind(live) == "LoxLIVE"
                && kind(caption) == "VirtualInCaption"
                && kind(owner) == "VirtualIn"
                && owner.attributes.get("U").map(String::as_str) == Some(requested)
                && owner.attributes.get("Analog").map(String::as_str) == Some("true")
                && owner.attributes.get("Nio").map(String::as_str) == Some("2")
                && owner
                    .attributes
                    .get("EnoceanSubType")
                    .is_none_or(|value| value == "0")
                && !owner
                    .children
                    .iter()
                    .filter_map(XMLNode::as_element)
                    .any(|child| child.name == "C"),
            "Virtual Input format supports only an existing Analog=true VirtualIn with Nio=2, absent/zero EnoceanSubType and no child controls; nothing was changed."
        );
        ensure!(
            document.attributes.get("DateS").is_none_or(|value| {
                !value.is_empty()
                    && value.bytes().all(|byte| byte.is_ascii_digit())
                    && value
                        .parse::<i32>()
                        .is_ok_and(|seconds| (0..i32::MAX).contains(&seconds))
            }),
            "Virtual Input format requires an absent or incrementable nonnegative signed32 Document DateS; nothing was changed."
        );
        let connectors: Vec<_> = owner
            .children
            .iter()
            .filter_map(XMLNode::as_element)
            .filter(|child| child.name == "Co")
            .collect();
        ensure!(
            connectors.len() == 2
                && connectors[0].attributes.get("K").map(String::as_str) == Some("Q")
                && connectors[1].attributes.get("K").map(String::as_str) == Some("Qm"),
            "Virtual Input format requires exactly the existing Q, Qm connector order, not alias/default layouts; nothing was changed."
        );
        let displays: Vec<_> = owner
            .children
            .iter()
            .filter_map(XMLNode::as_element)
            .filter(|child| child.name == "Display")
            .collect();
        ensure!(
            displays.len() == 1
                && displays[0]
                    .children
                    .iter()
                    .all(|node| node.as_element().is_none())
                && displays[0]
                    .attributes
                    .get("StateOnly")
                    .is_none_or(|value| matches!(value.as_str(), "true" | "false")),
            "Virtual Input format requires one existing direct Display with absent/true/false StateOnly and no nested elements; nothing was changed."
        );
        let display = displays[0];
        let mut guards = RawGuards::new();
        for element in [document, live, caption, owner] {
            let allow_io_flags = matches!(kind(element), "LoxLIVE" | "VirtualIn");
            ensure!(
                element.name == "C"
                    && !element.attributes.contains_key("Dev")
                    && element
                        .attributes
                        .get("V")
                        .and_then(|value| value.parse::<i32>().ok())
                        .is_some_and(|version| version >= 90)
                    && element.attributes.get("WF").is_none_or(|value| {
                        matches!(value.as_str(), "0" | "16384")
                            || (allow_io_flags && value == "16400")
                    }),
                "Virtual Input format requires explicit modern V>=90 and ordinary absent/0/16384 WF (also 16400 on LoxLIVE/VirtualIn), without device bindings, on the owner and ancestors; other flags/contexts are unsupported. Nothing was changed."
            );
        }
        for element in [document, live, caption, owner]
            .into_iter()
            .chain(connectors.iter().copied())
        {
            let identity = element.attributes.get("U").and_then(|value| stored_identity(value))
                .context("Virtual Input format requires existing nonnil native35 ancestor/owner/connector identities; nothing was changed")?;
            ensure!(
                all.iter()
                    .filter(|candidate| matches!(candidate.name.as_str(), "C" | "Co"))
                    .filter(|candidate| candidate
                        .attributes
                        .get("U")
                        .and_then(|value| native_identity(value))
                        == Some(identity))
                    .count()
                    == 1,
                "Virtual Input format requires unique effective identities, including case/RFC-grouping aliases; nothing was changed."
            );
            guard_element(&editor, element, &mut guards);
        }
        guard_element(&editor, display, &mut guards);
        let document_ordinal = entry_name_ordinal(&editor.root, document, &mut 0).unwrap();
        let target = entry_name_ordinal(&editor.root, display, &mut 0).unwrap();
        let change = VirtualInputFormatChange {
            block_uuid: requested.to_owned(),
            previous_format: display.attributes.get("Unit").cloned().unwrap_or_default(),
            format: format.to_owned(),
            changed: display.attributes.get("Unit").map_or("", String::as_str) != format,
        };
        let admitted = format_bytes(data, &guards, target, document_ordinal, None, None)?;
        if !change.changed {
            return Ok((admitted, change));
        }
        ensure!(
            editor.stamp_document(&chrono::Local::now().fixed_offset()),
            "Cannot stamp the admitted Virtual Input format Document; nothing was changed."
        );
        let document = editor.get_element(&documents[0]);
        let stamp = (
            document.attributes["Date"].clone(),
            document.attributes["DateS"].clone(),
        );
        let output = format_bytes(
            data,
            &guards,
            target,
            document_ordinal,
            Some(format),
            Some(&stamp),
        )?;
        Ok((output, change))
    }
}

fn guard_element(editor: &ConfigEditor, element: &Element, guards: &mut RawGuards) {
    guards.insert(
        entry_name_ordinal(&editor.root, element, &mut 0).unwrap(),
        (
            element.name.clone(),
            element
                .attributes
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
        ),
    );
}

fn raw_span(data: &[u8], value: &[u8]) -> Result<std::ops::Range<usize>> {
    let start = (value.as_ptr() as usize)
        .checked_sub(data.as_ptr() as usize)
        .context("Cannot locate original Virtual Input format storage; nothing was changed")?;
    let end = start
        .checked_add(value.len())
        .context("Virtual Input format storage range overflow")?;
    ensure!(
        data.get(start..end) == Some(value),
        "Virtual Input format storage must borrow original XML; nothing was changed."
    );
    Ok(start..end)
}

fn format_bytes(
    data: &[u8],
    guards: &RawGuards,
    target: usize,
    document: usize,
    format: Option<&str>,
    stamp: Option<&(String, String)>,
) -> Result<Vec<u8>> {
    std::str::from_utf8(data)?;
    let offset = usize::from(data.starts_with(b"\xef\xbb\xbf")) * 3;
    let mut reader = Reader::from_reader(&data[offset..]);
    let mut ordinal = 0;
    let mut found = 0;
    let mut patches = Vec::new();
    loop {
        match reader.read_event()? {
            Event::Start(element) | Event::Empty(element) => {
                let end = reader.buffer_position() as usize + offset;
                let guarded = guards.get(&ordinal);
                let name = element.name();
                let mut seen = BTreeMap::new();
                let mut attributes = Vec::new();
                for attribute in element.attributes().with_checks(false) {
                    let attribute = attribute?;
                    let key = std::str::from_utf8(attribute.key.as_ref())?;
                    let identity = (matches!(name.as_ref(), b"C" | b"Co" | b"Ref") && key == "U")
                        || (name.as_ref() == b"C" && key == "Type")
                        || (name.as_ref() == b"Co" && key == "K");
                    if let Some(previous) = seen.insert(key.to_owned(), attribute.value.to_vec()) {
                        ensure!(
                            guarded.is_none() && !identity && previous == attribute.value.as_ref(),
                            "Virtual Input format refuses guarded/identity duplicate attributes or conflicting raw values; nothing was changed."
                        );
                        continue;
                    }
                    raw_span(data, attribute.key.as_ref())?;
                    let value = raw_span(data, attribute.value.as_ref())?;
                    ensure!(
                        matches!(data.get(value.end), Some(b'\'' | b'"')),
                        "Cannot locate original attribute delimiter; nothing was changed."
                    );
                    attributes.push(attribute);
                }
                ensure!(
                    !name.as_ref().contains(&b':')
                        && attributes
                            .iter()
                            .all(|attribute| attribute.key.as_ref() != b"xmlns"
                                && !attribute.key.as_ref().contains(&b':')),
                    "Virtual Input format does not support XML namespaces; nothing was changed."
                );
                if let Some((tag, expected)) = guarded {
                    ensure!(
                        name.as_ref() == tag.as_bytes() && attributes.len() == expected.len(),
                        "Virtual Input format raw/DOM guarded element differs; nothing was changed."
                    );
                    for attribute in &attributes {
                        let key = std::str::from_utf8(attribute.key.as_ref())?;
                        ensure!(
                            !attribute.value.windows(2).any(|bytes| bytes == b"&#")
                                && expected.get(key).map(String::as_str)
                                    == Some(attribute.unescape_value()?.as_ref()),
                            "Virtual Input format refuses numeric-reference admission metadata or raw/DOM disagreement; nothing was changed."
                        );
                    }
                    found += 1;
                }
                let mut replacements = Vec::new();
                if ordinal == target
                    && let Some(format) = format
                {
                    replacements.push(("Unit", format));
                }
                if ordinal == document
                    && let Some((date, seconds)) = stamp
                {
                    replacements.extend([("Date", date.as_str()), ("DateS", seconds.as_str())]);
                }
                for (key, value) in replacements {
                    let escaped = quick_xml::escape::escape(value).into_owned();
                    if let Some(attribute) = attributes
                        .iter()
                        .find(|attribute| attribute.key.as_ref() == key.as_bytes())
                    {
                        patches.push((raw_span(data, attribute.value.as_ref())?, escaped));
                    } else {
                        let insertion = end - if data[end - 2] == b'/' { 2 } else { 1 };
                        patches.push((insertion..insertion, format!(" {key}=\"{escaped}\"")));
                    }
                }
                ordinal += 1;
            }
            Event::Decl(declaration) => {
                if let Some(encoding) = declaration.encoding() {
                    let encoding = encoding?;
                    ensure!(
                        encoding.eq_ignore_ascii_case(b"utf-8")
                            || encoding.eq_ignore_ascii_case(b"utf8"),
                        "Virtual Input format supports UTF-8 XML only; nothing was changed."
                    );
                }
            }
            Event::DocType(_) => anyhow::bail!(
                "Virtual Input format does not support DTD-dependent storage; nothing was changed."
            ),
            Event::Eof => break,
            _ => {}
        }
    }
    ensure!(
        found == guards.len(),
        "Cannot locate all admitted Virtual Input format state in original XML; nothing was changed."
    );
    patches.sort_by_key(|patch| std::cmp::Reverse(patch.0.start));
    let mut output = data.to_vec();
    for (range, value) in patches {
        output.splice(range, value.bytes());
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::super::ConfigEditor;

    const OWNER: &str = "10000000-0000-0000-0000000000000004";
    const XML: &str = r#"<LoxoneApp>
<C Type="Document" V="175" U="10000000-0000-0000-0000000000000001" Date="old" DateS="1">
  <C Type="LoxLIVE" V="175" U="10000000-0000-0000-0000000000000002" WF="16400">
    <C Type="VirtualInCaption" V="175" U="10000000-0000-0000-0000000000000003" WF="16384">
      <C Type="VirtualIn" V="175" U="10000000-0000-0000-0000000000000004" WF="16400" Analog="true" Nio="2" IName="VI1" Title="Pressure">
        <Co K="Q" U="10000000-0000-0000-0000000000000005"/>
        <Co K="Qm" U="10000000-0000-0000-0000000000000006"/>
        <IoData Visu="true" St="2" Note="keep" Note="keep"/>
        <Display Unit='&lt;v&gt;' StateOnly="false" Type="2" Mult="1"/>
      </C>
    </C>
  </C>
</C>
</LoxoneApp>"#;

    #[test]
    fn virtual_input_format_preserves_every_other_byte_and_state_only() {
        let input = format!("\u{feff}{}", XML.replace('\n', "\r\n"));
        let (output, change) = ConfigEditor::edit_virtual_input_format(
            input.as_bytes(),
            &format!("uuid:{OWNER}"),
            "<v.1>hPa",
        )
        .unwrap();
        assert!(change.changed);
        assert_eq!(change.previous_format, "<v>");
        let editor = ConfigEditor::load(&output).unwrap();
        let document = editor.get_element(&editor.find_elements("Type:Document")[0]);
        let expected = input
            .replace("Unit='&lt;v&gt;'", "Unit='&lt;v.1&gt;hPa'")
            .replace(
                "Date=\"old\"",
                &format!("Date=\"{}\"", document.attributes["Date"]),
            )
            .replace(
                "DateS=\"1\"",
                &format!("DateS=\"{}\"", document.attributes["DateS"]),
            );
        assert_eq!(output, expected.as_bytes());
        let (noop, change) =
            ConfigEditor::edit_virtual_input_format(&output, &format!("uuid:{OWNER}"), "<v.1>hPa")
                .unwrap();
        assert!(!change.changed);
        assert_eq!(noop, output);
    }

    #[test]
    fn virtual_input_format_missing_unit_is_not_constructor_default() {
        for state in ["", " StateOnly=\"true\"", " StateOnly=\"false\""] {
            let input = XML.replace("Unit='&lt;v&gt;' StateOnly=\"false\"", state.trim_start());
            let (output, change) = ConfigEditor::edit_virtual_input_format(
                input.as_bytes(),
                &format!("uuid:{OWNER}"),
                "<v.2> %",
            )
            .unwrap();
            assert_eq!(change.previous_format, "");
            assert!(String::from_utf8(output).unwrap().contains(&format!(
                "<Display {} Type=\"2\" Mult=\"1\" Unit=\"&lt;v.2&gt; %\"/>",
                state.trim_start()
            )));
        }
    }

    #[test]
    fn virtual_input_format_refuses_unsupported_values_and_noop_contexts() {
        for value in ["", " ", "<v.4>", "<v.col>", "<v.1>°C", "<v>\n", "<script>"] {
            assert!(
                ConfigEditor::edit_virtual_input_format(
                    XML.as_bytes(),
                    &format!("uuid:{OWNER}"),
                    value,
                )
                .is_err(),
                "{value:?}"
            );
        }
        let cases = [
            XML.replace("Analog=\"true\"", "Analog=\"false\""),
            XML.replace("Analog=\"true\"", "Analog=\"&#116;rue\""),
            XML.replace("Type=\"VirtualIn\"", "Type=\"VirtualOut\""),
            XML.replace("WF=\"16400\" Analog", "WF=\"134234128\" Analog"),
            XML.replace("<LoxoneApp>", "<LoxoneApp xmlns=\"urn:test\">"),
            XML.replace("IName=\"VI1\"", "IName=\"VI1\" IfT=\"0\""),
            XML.replace("IName=\"VI1\"", "IName=\"VI1\" EnoceanSubType=\"1\""),
            XML.replace("Type=\"Document\"", "Type=\"DocumentTemplate\""),
            XML.replace(
                "StateOnly=\"false\"",
                "StateOnly=\"false\" StateOnly=\"false\"",
            ),
            XML.replace("StateOnly=\"false\"", "StateOnly=\"0\""),
            XML.replace("Unit='&lt;v&gt;'", "Unit='&lt;v&gt;' Unit='&lt;v&gt;'"),
            XML.replace(
                "Note=\"keep\" Note=\"keep\"",
                "Note=\"keep\" Note=\"other\"",
            ),
            XML.replace("Note=\"keep\" Note=\"keep\"", "Note=\"Wake 7am=ok\""),
            XML.replace(
                "</LoxoneApp>",
                r#"<C Type="And" U="10000000-0000-0000-0000-000000000004"/></LoxoneApp>"#,
            ),
            XML.replace("<Display ", "<Display xmlns=\"\" "),
            XML.replace(
                "<Display ",
                r#"<C Type="VirtualIn" WF="134217728"/><Display "#,
            ),
            XML.replace("DateS=\"1\"", "DateS=\"2147483647\""),
        ];
        for (index, input) in cases.iter().enumerate() {
            assert!(
                ConfigEditor::edit_virtual_input_format(
                    input.as_bytes(),
                    &format!("uuid:{OWNER}"),
                    "<v>",
                )
                .is_err(),
                "case {index}"
            );
        }
    }
}
