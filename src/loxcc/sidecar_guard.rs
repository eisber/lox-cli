//! Fail closed where an XML edit would require native deployment sidecars.
//!
//! No catalogue, permissions, or Touch Pure Display compiler is implemented here.

use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use xmltree::{Element, XMLNode};

#[derive(Debug, PartialEq, Eq)]
struct Shape {
    name: String,
    attributes: BTreeMap<String, String>,
    children: Vec<Shape>,
    text: String,
}

fn shape(element: &Element, include_content: bool) -> Shape {
    let attributes = element
        .attributes
        .iter()
        .filter(|(key, _)| {
            if matches!(key.as_str(), "Px" | "Py" | "Px2" | "Py2") {
                return false;
            }
            if element.attributes.get("Type").map(String::as_str) == Some("Document")
                && matches!(key.as_str(), "Date" | "DateS")
            {
                return false;
            }
            include_content || matches!(key.as_str(), "Type" | "U" | "K")
        })
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    let mut children = Vec::new();
    let mut text = String::new();
    for child in &element.children {
        match child {
            XMLNode::Element(child) if child.name != "C" => {
                if include_content || child.name == "Co" {
                    children.push(shape(child, include_content));
                }
            }
            XMLNode::Text(value) | XMLNode::CData(value)
                if include_content && !value.trim().is_empty() =>
            {
                text.push_str(value);
            }
            _ => {}
        }
    }
    Shape {
        name: element.name.clone(),
        attributes,
        children,
        text,
    }
}

fn identity(value: &str) -> String {
    value
        .strip_prefix("U:")
        .unwrap_or(value)
        .to_ascii_lowercase()
}

fn app_identities(value: &Value, result: &mut BTreeSet<String>) {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                result.insert(identity(key));
                app_identities(value, result);
            }
        }
        Value::Array(values) => {
            for value in values {
                app_identities(value, result);
            }
        }
        Value::String(value) => {
            result.insert(identity(value));
        }
        _ => {}
    }
}

fn runtime_only(element: &Element, app_ids: &BTreeSet<String>) -> bool {
    if element.children.iter().any(|child| {
        child.as_element().is_some_and(|child| {
            !matches!(child.name.as_str(), "Co" | "C")
                || child
                    .attributes
                    .get("U")
                    .is_some_and(|uuid| app_ids.contains(&identity(uuid)))
        })
    }) || element.attributes.contains_key("RightGroup")
        || element.attributes.contains_key("Visu")
        || element
            .attributes
            .get("U")
            .is_some_and(|uuid| app_ids.contains(&identity(uuid)))
    {
        return false;
    }
    // Unknown block types stay protected, including controls whose default
    // visualization does not require an explicit IoData element.
    matches!(
        element.attributes.get("Type").map(String::as_str),
        Some(
            "Program"
                | "Page"
                | "And"
                | "Or"
                | "Not"
                | "Xor"
                | "Add"
                | "Sub"
                | "Mult"
                | "Div"
                | "Mod"
                | "Int"
                | "Avg"
                | "Minmax"
                | "GreaterEqual"
                | "Greater"
                | "Less"
                | "LessEqual"
                | "Equal"
                | "NotEqual"
                | "Constant"
        )
    )
}

fn projection(
    root: &Element,
    app_ids: &BTreeSet<String>,
) -> Result<BTreeMap<String, (String, Shape)>> {
    fn visit(
        element: &Element,
        parent: &str,
        path: &str,
        app_ids: &BTreeSet<String>,
        result: &mut BTreeMap<String, (String, Shape)>,
    ) -> Result<()> {
        let mut next_parent = parent.to_string();
        if element.name == "C" {
            let key = element
                .attributes
                .get("U")
                .map(|uuid| identity(uuid))
                .unwrap_or_else(|| format!("unidentified:{path}"));
            if result
                .insert(
                    key.clone(),
                    (
                        parent.to_string(),
                        shape(element, !runtime_only(element, app_ids)),
                    ),
                )
                .is_some()
            {
                bail!("Cannot check deployment sidecars: duplicate native control identity {key}");
            }
            next_parent = key;
        }
        for (index, child) in element
            .children
            .iter()
            .filter_map(XMLNode::as_element)
            .enumerate()
        {
            visit(
                child,
                &next_parent,
                &format!("{path}/{index}"),
                app_ids,
                result,
            )?;
        }
        Ok(())
    }
    let mut result = BTreeMap::new();
    visit(root, "", "", app_ids, &mut result)?;
    Ok(result)
}

fn has_display(element: &Element) -> bool {
    element.attributes.get("Type").map(String::as_str) == Some("TPDC")
        || (element.attributes.get("Type").map(String::as_str) == Some("TreeDevice")
            && element.attributes.get("SubType").map(String::as_str) == Some("32811"))
        || element
            .children
            .iter()
            .filter_map(XMLNode::as_element)
            .any(has_display)
}

fn has_visualization(element: &Element) -> bool {
    element.name == "IoData"
        || element
            .children
            .iter()
            .filter_map(XMLNode::as_element)
            .any(has_visualization)
}

pub(super) fn validate(
    before: &[u8],
    after: &[u8],
    app: Option<&[u8]>,
    permissions: Option<&[u8]>,
) -> Result<()> {
    let before =
        Element::parse(before).context("Cannot check sidecars: invalid source project XML")?;
    let after =
        Element::parse(after).context("Cannot check sidecars: invalid edited project XML")?;
    if before.name != "ControlList" || after.name != "ControlList" {
        bail!(
            "Cannot check deployment sidecars: expected a native ControlList project. \
             Save and deploy this project with Loxone Config."
        );
    }
    let mut app_ids = BTreeSet::new();
    let mut has_app_controls = false;
    if let Some(app) = app {
        let app: Value =
            serde_json::from_slice(app).context("Cannot check sidecars: invalid LoxAPP3.json")?;
        if !app.is_object() {
            bail!("Cannot check deployment sidecars: LoxAPP3.json must be a JSON object");
        }
        if let Some(controls) = app.get("controls") {
            let controls = controls.as_object().context(
                "Cannot check deployment sidecars: LoxAPP3.json controls must be an object",
            )?;
            has_app_controls = !controls.is_empty();
        }
        app_identities(&app, &mut app_ids);
    }
    if has_visualization(&before) || has_app_controls {
        if app.is_none() {
            bail!(
                "Cannot safely deploy this project: source LoxAPP3.json is missing. \
                 Download a complete native backup or save and deploy with Loxone Config."
            );
        }
        if permissions.is_none_or(|data| data.is_empty()) {
            bail!(
                "Cannot safely deploy this project: source permissions.bin is missing or empty, \
                 even if LoxAPP3.json already matches. \
                 Download a complete native backup or save and deploy with Loxone Config."
            );
        }
    }

    let before_shape = projection(&before, &app_ids)?;
    let after_shape = projection(&after, &app_ids)?;
    let root_shape = |root: &Element| {
        let mut result = shape(root, true);
        result.attributes.retain(|key, _| {
            !matches!(
                key.as_str(),
                "Version" | "NextMem" | "NextConst" | "NextNote" | "NextObj"
            )
        });
        result
    };
    if before_shape != after_shape || root_shape(&before) != root_shape(&after) {
        if has_display(&before) || has_display(&after) {
            bail!(
                "Deployment requires native sidecar compilation, including TPDC/Touch Pure Display \
                 TPDISP payloads, Displayconf checksums and managed API wiring. \
                 lox cannot safely generate or validate those device artifacts. \
                 Save and deploy this edit with Loxone Config; --force cannot bypass this check."
            );
        }
        let missing = match (app, permissions) {
            (None, _) => " (source LoxAPP3.json is missing)",
            (_, None | Some([])) => " (source permissions.bin is missing or empty)",
            _ => "",
        };
        bail!(
            "Deployment would change the app catalogue or native permission identities{missing}. \
             lox does not compile LoxAPP3.json or permissions.bin and will not upload stale sidecars. \
             Save and deploy this edit with Loxone Config; --force cannot bypass this check."
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loxcc::{
        compress_loxcc, extract_and_decompress, repack_snapshot_zip, repack_zip,
        validate_archive_deployment,
    };
    use std::io::{Cursor, Read, Write};

    const SELECTOR: &str = r#"<C Type="PushButton2Sel" V="177" U="selector" Title="Level" Format="&lt;v&gt;">
      <Co K="Min" U="min"/><Co K="Max" U="max"/><Co K="Step" U="step"/><Co K="AQ" U="value"/>
      <IoData Visu="true" Pr="room" Cr="category"/>
    </C>"#;
    const DISPLAY: &str = r#"<C Type="TreeDevice" U="device" SubType="32811">
      <C Type="ApiActor" U="actor"><Co K="I" U="actor-in"><In Input="ref-out"/></Co></C>
    </C>
    <C Type="TPDC" U="display">
      <Co K="OutputAPI" U="api"><In Input="ref-in" FLG="1"/></Co>
      <Displayconf CRCS="-123" CRCR="456" CRCC="789" CRCI="0" CO="selector"/>
    </C>
    <C Type="OutputRef" U="ref">
      <Co K="AI" U="ref-in"><In Input="api"/></Co><Co K="AQ" U="ref-out"/>
    </C>"#;

    fn project(controls: &str) -> Vec<u8> {
        format!(
            r#"<ControlList Version="10">
            <C Type="Document" U="doc" Title="Project" Date="old" DateS="1"/>
            <C Type="Place" U="room" Title="Room" RightGroup="room-right"/>
            <C Type="Category" U="category" Title="Category" RightGroup="cat-right"/>
            <C Type="User" U="user"><PH NUM="1"><PRM REF="room" ACT="3"/></PH></C>
            <C Type="And" U="gate" Title="Gate"><Co K="I1" U="gate-in" V="1"/></C>
            {controls}</ControlList>"#
        )
        .into_bytes()
    }

    fn archive(xml: &[u8], permissions: Option<&[u8]>) -> Vec<u8> {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        writer.set_raw_comment(b"native archive metadata".as_slice().into());
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .last_modified_time(zip::DateTime::from_date_and_time(2024, 3, 5, 6, 8, 10).unwrap())
            .unix_permissions(0o640);
        writer.start_file("sps0.LoxCC", options).unwrap();
        writer.write_all(&compress_loxcc(xml)).unwrap();
        writer.start_file("LoxAPP3.json", options).unwrap();
        writer
            .write_all(
                br#"{"lastModified":"native-revision","controls":{"U:selector":{"uuidAction":"selector","details":{"format":"%.1f %%","native":"keep"},"isFavorite":true,"restrictions":17}}}"#,
            )
            .unwrap();
        if let Some(permissions) = permissions {
            writer.start_file("permissions.bin", options).unwrap();
            writer.write_all(permissions).unwrap();
        }
        writer.add_directory("TPDISP/", options).unwrap();
        writer.start_file("TPDISP/display.json", options).unwrap();
        writer
            .write_all(b"[{\"U\":0,\"T\":484}]\n\x03[]\n\x03[]\n")
            .unwrap();
        writer.start_file("unrelated.bin", options).unwrap();
        writer.write_all(b"\x00\x01\xffunchanged").unwrap();
        writer.finish().unwrap().into_inner()
    }

    fn replace(xml: &[u8], from: &str, to: &str) -> Vec<u8> {
        String::from_utf8(xml.to_vec())
            .unwrap()
            .replace(from, to)
            .into_bytes()
    }

    fn replace_member(source: &[u8], name: &str, replacement: Option<&[u8]>) -> Vec<u8> {
        let mut archive = zip::ZipArchive::new(Cursor::new(source)).unwrap();
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default();
        let mut replaced = false;
        for index in 0..archive.len() {
            let mut file = archive.by_index(index).unwrap();
            if file.name() == name {
                replaced = true;
                if let Some(replacement) = replacement {
                    writer.start_file(name, options).unwrap();
                    writer.write_all(replacement).unwrap();
                }
            } else {
                let mut bytes = Vec::new();
                file.read_to_end(&mut bytes).unwrap();
                if file.is_dir() {
                    writer.add_directory(file.name(), options).unwrap();
                } else {
                    writer.start_file(file.name(), options).unwrap();
                    writer.write_all(&bytes).unwrap();
                }
            }
        }
        if !replaced && let Some(replacement) = replacement {
            writer.start_file(name, options).unwrap();
            writer.write_all(replacement).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    #[test]
    fn sidecars_raw_upload_requires_complete_matching_candidate_sidecars() {
        let xml = project(SELECTOR);
        let source = archive(&xml, Some(b"native permissions"));
        for member in ["LoxAPP3.json", "permissions.bin", "TPDISP/display.json"] {
            let candidate = replace_member(&source, member, None);
            assert!(
                validate_archive_deployment(&source, &candidate, &xml).is_err(),
                "missing {member}"
            );
        }
        for (member, bytes) in [
            ("LoxAPP3.json", br#"{"controls":{}}"#.as_slice()),
            ("permissions.bin", b"unverified permissions".as_slice()),
            ("TPDISP/display.json", b"[]\n\x03[]\n\x03[]\n".as_slice()),
            ("TPDISP/new-display.json", b"new payload".as_slice()),
        ] {
            let candidate = replace_member(&source, member, Some(bytes));
            let error = validate_archive_deployment(&source, &candidate, &xml)
                .unwrap_err()
                .to_string();
            assert!(error.contains("replacement sidecars"), "{member}: {error}");
        }
    }

    #[test]
    fn sidecars_raw_upload_allows_neutral_xml_and_unrelated_members() {
        let xml = project(SELECTOR);
        let source = archive(&xml, Some(b"native permissions"));
        assert!(validate_archive_deployment(&source, &source, &xml).is_ok());
        let edited = replace(&xml, "V=\"1\"", "V=\"0\"");
        let candidate = repack_snapshot_zip(&source, &edited).unwrap();
        assert!(validate_archive_deployment(&source, &candidate, &edited).is_ok());
        let candidate =
            replace_member(&candidate, "unrelated.bin", Some(b"updated unrelated data"));
        assert!(validate_archive_deployment(&source, &candidate, &edited).is_ok());
    }

    #[test]
    fn sidecars_reject_catalogue_changes_without_compiling_speculative_payloads() {
        let original = project(SELECTOR);
        let source = archive(&original, Some(b"opaque native permissions"));
        let edits = [
            replace(&original, "Title=\"Level\"", "Title=\"Renamed\""),
            replace(&original, "Visu=\"true\"", "Visu=\"false\""),
            replace(&original, "Pr=\"room\"", "Pr=\"other-room\""),
            replace(&original, "Cr=\"category\"", "Cr=\"other-category\""),
            replace(&original, "&lt;v&gt;", "&lt;v&gt; %"),
            replace(&original, "U=\"min\"", "U=\"new-min\""),
            replace(&original, SELECTOR, ""),
            project(&format!("{SELECTOR}<C Type=\"FutureControl\" U=\"new\"/>")),
        ];
        for edit in edits {
            let error = repack_zip(&source, &edit).unwrap_err().to_string();
            assert!(error.contains("LoxAPP3.json"), "{error}");
            assert!(error.contains("permissions.bin"), "{error}");
            assert!(error.contains("Loxone Config"), "{error}");
        }
    }

    #[test]
    fn sidecars_reject_new_visible_control_even_with_matching_source_app_entry() {
        // The synthetic catalogue already mentions selector. App-entry equality
        // must not suppress the XML/permissions impact check.
        let source = archive(&project(""), Some(b"opaque native permissions"));
        assert!(repack_zip(&source, &project(SELECTOR)).is_err());
    }

    #[test]
    fn sidecars_check_permissions_independently_of_catalogue_equality() {
        let original = project(SELECTOR);
        for (from, to) in [
            ("room-right", "new-room-right"),
            ("cat-right", "new-cat-right"),
            ("ACT=\"3\"", "ACT=\"0\""),
        ] {
            let source = archive(&original, Some(b"opaque native permissions"));
            let error = repack_zip(&source, &replace(&original, from, to))
                .unwrap_err()
                .to_string();
            assert!(error.contains("permissions.bin"), "{error}");
        }
    }

    #[test]
    fn sidecars_missing_permissions_do_not_bypass_change_guard() {
        for permissions in [None, Some(b"".as_slice())] {
            let source = archive(&project(""), permissions);
            let error = repack_zip(&source, &project(SELECTOR))
                .unwrap_err()
                .to_string();
            assert!(
                error.contains("permissions.bin is missing or empty"),
                "{error}"
            );
        }
    }

    #[test]
    fn sidecars_preserve_untouched_unverified_native_versions_and_metadata() {
        let original = project(SELECTOR);
        let source = archive(&original, Some(b"opaque native permissions"));
        let edited = replace(&original, "V=\"1\"", "V=\"0\"");
        let repacked = repack_zip(&source, &edited).unwrap();
        assert_eq!(extract_and_decompress(&repacked).unwrap(), edited);
        let mut before = zip::ZipArchive::new(Cursor::new(&source)).unwrap();
        let mut after = zip::ZipArchive::new(Cursor::new(&repacked)).unwrap();
        assert_eq!(before.comment(), after.comment());
        for name in [
            "LoxAPP3.json",
            "permissions.bin",
            "TPDISP/",
            "TPDISP/display.json",
            "unrelated.bin",
        ] {
            let mut original = before.by_name(name).unwrap();
            let mut actual = after.by_name(name).unwrap();
            assert_eq!(original.compression(), actual.compression(), "{name}");
            assert_eq!(
                original.compressed_size(),
                actual.compressed_size(),
                "{name}"
            );
            assert_eq!(original.last_modified(), actual.last_modified(), "{name}");
            assert_eq!(original.unix_mode(), actual.unix_mode(), "{name}");
            let mut expected = Vec::new();
            let mut result = Vec::new();
            original.read_to_end(&mut expected).unwrap();
            actual.read_to_end(&mut result).unwrap();
            assert_eq!(expected, result, "{name}");
        }
        let unchanged = repack_zip(&source, &original).unwrap();
        assert_eq!(unchanged, source);
        assert_eq!(extract_and_decompress(&unchanged).unwrap(), original);
    }

    #[test]
    fn sidecars_allow_save_stamps_layout_and_xml_serialization_changes() {
        let original = project(SELECTOR);
        let edited = replace(
            &original,
            "Date=\"old\" DateS=\"1\"",
            "DateS=\"2\" Date=\"new\"",
        );
        let edited = replace(
            &edited,
            "U=\"selector\"",
            "Px=\"100\" Py=\"200\" U=\"selector\"",
        );
        let edited = replace(&edited, "Version=\"10\"", "Version=\"11\"");
        let edited = replace(
            &edited,
            "<Co K=\"Min\"",
            "<!-- annotation -->\n<Co K=\"Min\"",
        );
        assert!(repack_zip(&archive(&original, Some(b"native")), &edited).is_ok());
    }

    #[test]
    fn sidecars_protect_catalogue_references_without_explicit_visualization() {
        let original = project("");
        let app = br#"{"controls":{"gate":{"uuidAction":"gate"}}}"#;
        let edited = replace(&original, "V=\"1\"", "V=\"2\"");
        assert!(validate(&original, &edited, Some(app), Some(b"native")).is_err());
    }

    #[test]
    fn sidecars_matching_app_still_rejects_missing_permissions() {
        let xml = project(SELECTOR);
        for permissions in [None, Some(b"".as_slice())] {
            let error = repack_zip(&archive(&xml, permissions), &xml)
                .unwrap_err()
                .to_string();
            assert!(
                error.contains("permissions.bin is missing or empty"),
                "{error}"
            );
        }
        let error = validate(&xml, &xml, None, Some(b"native"))
            .unwrap_err()
            .to_string();
        assert!(error.contains("LoxAPP3.json is missing"), "{error}");
    }

    #[test]
    fn sidecars_allow_existing_unvisualized_logic_wiring_edits() {
        let original = project(SELECTOR);
        let edited = replace(
            &original,
            "<Co K=\"I1\" U=\"gate-in\" V=\"1\"/>",
            "<Co K=\"I1\" U=\"gate-in\" V=\"1\"><In Input=\"value\"/></Co>",
        );
        assert_ne!(original, edited);
        assert!(repack_zip(&archive(&original, Some(b"native")), &edited).is_ok());
    }

    #[test]
    fn sidecars_tpdc_edits_fail_closed_without_claiming_native_support() {
        let original = project(DISPLAY);
        let source = archive(&original, Some(b"native"));
        for (from, to) in [
            ("FLG=\"1\"", "FLG=\"0\""),
            ("CRCS=\"-123\"", "CRCS=\"999\""),
            ("CO=\"selector\"", "CO=\"other-control\""),
            ("Input=\"api\"", "Input=\"other-api\""),
            ("Input=\"ref-out\"", "Input=\"other-output\""),
            ("SubType=\"32811\"", "SubType=\"32812\""),
            ("Title=\"Room\"", "Title=\"New Room\""),
        ] {
            let error = repack_zip(&source, &replace(&original, from, to))
                .unwrap_err()
                .to_string();
            assert!(error.contains("TPDC/Touch Pure Display"), "{error}");
            assert!(error.contains("Loxone Config"), "{error}");
        }
        assert!(repack_zip(&archive(&project(""), Some(b"native")), &original).is_err());
        assert!(repack_zip(&source, &project("")).is_err());
        assert!(repack_zip(&source, &replace(&original, "V=\"1\"", "V=\"0\"")).is_ok());
    }

    #[test]
    fn sidecars_malformed_inputs_are_actionable_errors() {
        let xml = project(SELECTOR);
        for (before, after, app) in [
            (b"<".as_slice(), xml.as_slice(), b"{}".as_slice()),
            (xml.as_slice(), b"<".as_slice(), b"{}".as_slice()),
            (xml.as_slice(), xml.as_slice(), b"{".as_slice()),
            (xml.as_slice(), xml.as_slice(), b"[]".as_slice()),
        ] {
            assert!(validate(before, after, Some(app), Some(b"native")).is_err());
        }
    }
}
