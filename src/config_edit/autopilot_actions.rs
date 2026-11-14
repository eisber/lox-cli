use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    ops::Range,
};

use anyhow::{Context, Result, bail, ensure};
use quick_xml::{Reader, events::Event};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{MapAccess, SeqAccess, Visitor},
};
use serde_json::{Map, Value};
use xmltree::Element;

use super::{ConfigEditor, alarm_clock::entry_name_ordinal, icons::native_identity};

#[derive(Debug, Serialize)]
pub struct AutopilotActionChange {
    pub block_uuid: String,
    pub index: usize,
    pub to_index: Option<usize>,
    pub action: Value,
    pub remaining_actions: usize,
    pub invalid: bool,
    pub changed: bool,
}

// serde_json normally silently keeps the last duplicate object member.
struct ActionJson(Value);

const MAX_EXACT_APC_INTEGER: u64 = (1 << 53) - 1;

impl<'de> Deserialize<'de> for ActionJson {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        struct JsonVisitor;
        impl<'de> Visitor<'de> for JsonVisitor {
            type Value = ActionJson;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("JSON without duplicate object members")
            }
            fn visit_map<A: MapAccess<'de>>(
                self,
                mut access: A,
            ) -> std::result::Result<Self::Value, A::Error> {
                let mut object = Map::new();
                while let Some((key, ActionJson(value))) =
                    access.next_entry::<String, ActionJson>()?
                {
                    if object.insert(key.clone(), value).is_some() {
                        return Err(serde::de::Error::custom(format!(
                            "duplicate APC member '{key}'"
                        )));
                    }
                }
                Ok(ActionJson(Value::Object(object)))
            }
            fn visit_seq<A: SeqAccess<'de>>(
                self,
                mut access: A,
            ) -> std::result::Result<Self::Value, A::Error> {
                let mut values = Vec::new();
                while let Some(ActionJson(value)) = access.next_element()? {
                    values.push(value);
                }
                Ok(ActionJson(Value::Array(values)))
            }
            fn visit_bool<E: serde::de::Error>(
                self,
                value: bool,
            ) -> std::result::Result<Self::Value, E> {
                Ok(ActionJson(value.into()))
            }
            fn visit_i64<E: serde::de::Error>(
                self,
                value: i64,
            ) -> std::result::Result<Self::Value, E> {
                if value.unsigned_abs() > MAX_EXACT_APC_INTEGER {
                    return Err(E::custom(
                        "APC integer exceeds the supported exact native double range",
                    ));
                }
                Ok(ActionJson(value.into()))
            }
            fn visit_u64<E: serde::de::Error>(
                self,
                value: u64,
            ) -> std::result::Result<Self::Value, E> {
                if value > MAX_EXACT_APC_INTEGER {
                    return Err(E::custom(
                        "APC integer exceeds the supported exact native double range",
                    ));
                }
                Ok(ActionJson(value.into()))
            }
            fn visit_f64<E: serde::de::Error>(
                self,
                _value: f64,
            ) -> std::result::Result<Self::Value, E> {
                // This also catches serde_json's lossy fallback for oversized integer tokens.
                Err(E::custom(
                    "APC decimal/exponent/oversized numbers are unsupported; use existing plain integer numeric storage",
                ))
            }
            fn visit_str<E: serde::de::Error>(
                self,
                value: &str,
            ) -> std::result::Result<Self::Value, E> {
                Ok(ActionJson(value.into()))
            }
            fn visit_unit<E: serde::de::Error>(self) -> std::result::Result<Self::Value, E> {
                Ok(ActionJson(Value::Null))
            }
        }
        deserializer.deserialize_any(JsonVisitor)
    }
}

fn serialized_members(value: &Value, allowed: &[&str], section: &str) -> Result<()> {
    let members = value
        .as_object()
        .with_context(|| format!("Autopilot {section} must be an object; nothing was changed."))?;
    for key in members.keys() {
        ensure!(
            allowed.contains(&key.as_str()),
            "Unsupported Autopilot {section} member '{key}'; native accepted serialization is not qualified for it. Nothing was changed."
        );
    }
    for key in ["name", "description"] {
        ensure!(
            value.get(key).is_none_or(Value::is_string),
            "Autopilot {section} member '{key}' must be text; nothing was changed."
        );
    }
    ensure!(
        value
            .get("operator")
            .is_none_or(|value| value.as_u64().is_some_and(|value| value <= i32::MAX as u64)),
        "Autopilot {section} operator must be a nonnegative signed32 integer; nothing was changed."
    );
    Ok(())
}

fn native_uuid(raw: &str) -> Option<uuid::Uuid> {
    (raw.len() == 35)
        .then(|| native_identity(raw))
        .flatten()
        .filter(|uuid| !uuid.is_nil())
}

fn current_descriptor(entry: &Value) -> Result<()> {
    ensure!(
        entry["descID"]
            .as_u64()
            .is_some_and(|id| (2..=i32::MAX as u64).contains(&id)),
        "Autopilot structural edits require existing descID >= 2; descriptor migration is unsupported. Nothing was changed."
    );
    ensure!(
        entry.get("invalid").is_none_or(|value| value == false),
        "Invalid Autopilot entries are not supported; nothing was changed."
    );
    Ok(())
}

fn native_flags(control: &Element) -> Result<u32> {
    ensure!(
        control
            .attributes
            .get("V")
            .and_then(|v| v.parse::<u32>().ok())
            .is_some_and(|v| v >= 90),
        "Autopilot owner/reference requires explicit V >= 90 to establish effective WF loading; nothing was changed."
    );
    let raw = control
        .attributes
        .get("WF")
        .map(String::as_str)
        .unwrap_or("0");
    ensure!(
        !raw.is_empty() && raw.bytes().all(|byte| byte.is_ascii_digit()),
        "Unsupported Autopilot owner/reference WF; nothing was changed."
    );
    let flags: i32 = raw
        .parse()
        .context("Unsupported signed32 Autopilot WF; nothing was changed")?;
    ensure!(
        flags & 0x0200_0000 == 0,
        "Deleted/protected Autopilot owner/reference is unsupported; nothing was changed."
    );
    Ok(flags as u32)
}

struct ActionStorage {
    apc: Range<usize>,
    sta: Range<usize>,
    dates: BTreeMap<String, Range<usize>>,
    document_insert: usize,
}

fn action_storage(
    data: &[u8],
    owner: usize,
    document: usize,
    references: &BTreeSet<usize>,
    block: &Element,
) -> Result<ActionStorage> {
    std::str::from_utf8(data).context("Autopilot edits require UTF-8 XML; nothing was changed")?;
    let offset = usize::from(data.starts_with(b"\xef\xbb\xbf")) * 3;
    let mut reader = Reader::from_reader(&data[offset..]);
    let mut ordinal = 0;
    let mut apc = None;
    let mut sta = None;
    let mut dates = BTreeMap::new();
    let mut document_insert = None;
    loop {
        match reader.read_event()? {
            Event::Start(element) | Event::Empty(element) => {
                let end = reader.buffer_position() as usize + offset;
                let selected = ordinal == owner;
                let stamp = ordinal == document;
                let guarded = selected || stamp || references.contains(&ordinal);
                let control = element.name().as_ref() == b"C";
                ensure!(
                    !element.name().as_ref().contains(&b':'),
                    "Namespaced Autopilot XML is unsupported; nothing was changed."
                );
                let mut seen = BTreeMap::new();
                for attribute in element.attributes().with_checks(false) {
                    let attribute = attribute
                        .context("Cannot inspect Autopilot raw attributes; nothing was changed")?;
                    let key = std::str::from_utf8(attribute.key.as_ref())?;
                    ensure!(
                        !key.contains(':') && key != "xmlns",
                        "Namespaced Autopilot XML is unsupported; nothing was changed."
                    );
                    let global_identity = control && matches!(key, "U" | "Type");
                    if let Some(previous) = seen.insert(key.to_owned(), attribute.value.to_vec()) {
                        ensure!(
                            !guarded && !global_identity && previous == attribute.value.as_ref(),
                            "Autopilot edits refuse duplicate attribute '{key}' on guarded state/identities or conflicting raw values; nothing was changed."
                        );
                        // Match the shared loader's exact raw-value dedupe, without changing bytes.
                        continue;
                    }
                    if global_identity
                        || (guarded
                            && matches!(
                                key,
                                "U" | "Type" | "V" | "WF" | "APC" | "STA" | "Date" | "DateS"
                            ))
                    {
                        ensure!(
                            !attribute.value.windows(2).any(|bytes| bytes == b"&#")
                                && !attribute
                                    .value
                                    .iter()
                                    .any(|byte| matches!(byte, b'\t' | b'\r' | b'\n')),
                            "Numeric XML entities or literal newlines/tabs in Autopilot identity/state/stamp storage are unsupported; nothing was changed."
                        );
                    }
                    let start = (attribute.value.as_ptr() as usize)
                        .checked_sub(data.as_ptr() as usize)
                        .filter(|start| {
                            data.get(*start..*start + attribute.value.len())
                                == Some(attribute.value.as_ref())
                        })
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "Cannot locate original Autopilot attribute; nothing was changed."
                            )
                        })?;
                    let range = start..start + attribute.value.len();
                    if selected && matches!(key, "U" | "Type" | "APC" | "STA") {
                        ensure!(
                            block.attributes.get(key).map(String::as_str)
                                == Some(attribute.unescape_value()?.as_ref()),
                            "Original Autopilot state differs from the admitted DOM; nothing was changed."
                        );
                        match key {
                            "APC" => apc = Some(range),
                            "STA" => sta = Some(range),
                            _ => {}
                        }
                    } else if stamp && matches!(key, "Date" | "DateS") {
                        dates.insert(key.to_owned(), range);
                    }
                }
                if stamp {
                    document_insert = Some(end - if data[end - 2] == b'/' { 2 } else { 1 });
                }
                ordinal += 1;
            }
            Event::Decl(declaration) => {
                if let Some(encoding) = declaration.encoding() {
                    let encoding = encoding?;
                    ensure!(
                        encoding.eq_ignore_ascii_case(b"utf-8")
                            || encoding.eq_ignore_ascii_case(b"utf8"),
                        "Autopilot edits require UTF-8 XML; nothing was changed."
                    );
                }
            }
            Event::DocType(_) => {
                bail!("DTD-dependent Autopilot XML is unsupported; nothing was changed.")
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(ActionStorage {
        apc: apc.context("Missing original APC; nothing was changed")?,
        sta: sta.context("Missing original STA; nothing was changed")?,
        dates,
        document_insert: document_insert
            .context("Missing original Document; nothing was changed")?,
    })
}

impl ConfigEditor {
    /// Remove one existing action, or move it to an existing target action's original index.
    pub fn edit_autopilot_action(
        data: &[u8],
        selector: &str,
        index: usize,
        to_index: Option<usize>,
    ) -> Result<(Vec<u8>, AutopilotActionChange)> {
        let requested = selector.strip_prefix("uuid:").unwrap_or("");
        let identity = native_uuid(requested).context(
            "Autopilot action edits require uuid:<exact nonnil native35 U> (8-4-4-16); nothing was changed.",
        )?;
        let mut editor =
            Self::load(data).context("Autopilot XML admission failed; nothing was changed")?;
        ensure!(
            editor.digit_attr_renames.is_empty(),
            "Autopilot edits refuse input changed by the shared digit-name sanitizer (including digit-like text); nothing was changed."
        );
        let documents = editor.find_elements("Type:Document");
        ensure!(
            documents.len() == 1 && documents[0].len() <= 1,
            "Autopilot action edits require one root/direct-child Document; nothing was changed."
        );
        let document_path = &documents[0];
        let document = editor.get_element(document_path);
        ensure!(
            document.name == "C"
                && document.attributes.get("Type").map(String::as_str) == Some("Document")
                && document.namespace.is_none()
                && document.prefix.is_none(),
            "Autopilot action edits require a plain Document; nothing was changed."
        );
        let elements = Self::iter_elements(&editor.root);
        ensure!(
            !elements.iter().any(|element| {
                element.name == "Template"
                    || element.attributes.contains_key("IfT")
                    || element.attributes.contains_key("uuidParent")
            }),
            "Template/IfT/inherited Autopilot configurations are unsupported; nothing was changed."
        );
        ensure!(
            elements
                .iter()
                .filter(|element| element.name == "C"
                    && element
                        .attributes
                        .get("U")
                        .and_then(|raw| native_identity(raw))
                        == Some(identity))
                .count()
                == 1,
            "Autopilot owner UUID must be effectively unique, including grouping/case aliases; nothing was changed."
        );
        let matches = editor.find_elements(selector);
        ensure!(
            matches.len() == 1 && matches[0].starts_with(document_path),
            "Select one existing AutopilotRule inside its Document; nothing was changed."
        );
        let path = &matches[0];
        let block = editor.get_element(path);
        ensure!(
            block.name == "C"
                && block.attributes.get("Type").map(String::as_str) == Some("AutopilotRule")
                && block.attributes.get("U").map(String::as_str) == Some(requested),
            "Select an exact stored native35 AutopilotRule U, not AutoPilot/name/RFC UUID; nothing was changed."
        );
        native_flags(block)?;
        ensure!(
            block.attributes.get("STA").map(String::as_str) == Some("1"),
            "Autopilot structural edits currently require explicit STA=1 (enabled and valid); nothing was changed."
        );
        let original_apc = block
            .attributes
            .get("APC")
            .context("Missing APC; nothing was changed")?;
        let ActionJson(mut rule) = serde_json::from_str::<ActionJson>(original_apc)
            .context("Unsupported APC JSON; nothing was changed")?;
        serialized_members(
            &rule,
            &[
                "type",
                "id",
                "active",
                "activeIfValid",
                "name",
                "description",
                "operator",
                "invalid",
                "ruleFlags",
                "eventGroups",
                "actions",
            ],
            "rule",
        )?;
        ensure!(
            rule["type"] == 1
                && rule["id"].as_str() == Some(requested)
                && rule["active"] == true
                && rule["activeIfValid"] == true
                && rule.get("invalid").is_none_or(|value| value == false)
                && rule.get("ruleFlags").is_none_or(|value| value == 0),
            "Only existing active, ordinary, unprotected type-1 APC rules with matching native35 id are supported; nothing was changed."
        );
        let groups = rule["eventGroups"]
            .as_array()
            .filter(|groups| !groups.is_empty())
            .context("Autopilot requires existing time eventGroups; nothing was changed")?;
        for group in groups {
            serialized_members(group, &["operator", "events"], "event group")?;
            let events = group["events"]
                .as_array()
                .filter(|events| !events.is_empty())
                .context(
                    "Empty/legacy Autopilot event groups are unsupported; nothing was changed",
                )?;
            for event in events {
                serialized_members(
                    event,
                    &[
                        "type",
                        "descID",
                        "id",
                        "value",
                        "operator",
                        "name",
                        "forceEdge",
                        "invalid",
                    ],
                    "event",
                )?;
                current_descriptor(event)?;
                ensure!(
                    event.get("id").is_none_or(|value| value
                        .as_u64()
                        .is_some_and(|value| value <= i32::MAX as u64))
                        && event.get("forceEdge").is_none_or(Value::is_boolean),
                    "Autopilot time event id/forceEdge storage is unsupported; nothing was changed."
                );
                let value = event["value"].as_f64().filter(|value| value.is_finite())
                    .context("Autopilot time events require existing numeric values; nothing was changed")?;
                ensure!(
                    event.get("arrValues").is_none()
                        && event.get("textValue").is_none()
                        && match event["type"].as_u64() {
                            Some(4) => value > 0.0,
                            Some(5) => (0.0..=1440.0).contains(&value),
                            _ => false,
                        },
                    "Only already-valid numeric time events (type 4/5) are supported, without editing their values; nothing was changed."
                );
            }
        }
        let actions = rule["actions"]
            .as_array()
            .filter(|actions| !actions.is_empty())
            .context("Autopilot requires existing actions; nothing was changed")?;
        let mut reference_ordinals = BTreeSet::new();
        for action in actions {
            serialized_members(
                action,
                &[
                    "type",
                    "descID",
                    "id",
                    "command",
                    "arrValues",
                    "name",
                    "invalid",
                ],
                "action",
            )?;
            current_descriptor(action)?;
            ensure!(
                action["command"].is_string()
                    && action.get("value").is_none()
                    && action.get("arrValues").is_none_or(|values| values
                        .as_array()
                        .is_some_and(|values| values.iter().all(Value::is_string))),
                "Legacy/non-string Autopilot command/value storage is unsupported; nothing was changed."
            );
            match action["type"].as_u64() {
                Some(0 | 8) => {
                    let raw = action["id"]
                        .as_str()
                        .context("Missing action control id; nothing was changed")?;
                    let referenced = native_uuid(raw).context(
                        "Action control id must be nonnil native35; nothing was changed",
                    )?;
                    let references: Vec<_> = elements
                        .iter()
                        .filter(|element| {
                            element.name == "C"
                                && element
                                    .attributes
                                    .get("U")
                                    .and_then(|raw| native_identity(raw))
                                    == Some(referenced)
                        })
                        .collect();
                    ensure!(
                        references.len() == 1
                            && references[0].attributes.get("U").map(String::as_str) == Some(raw),
                        "Action reference is missing, aliased or duplicated; nothing was changed."
                    );
                    let reference_path = editor.find_elements(&format!("uuid:{raw}"));
                    ensure!(
                        reference_path.len() == 1 && reference_path[0].starts_with(document_path),
                        "Action references must belong to the owning Document; nothing was changed."
                    );
                    native_flags(references[0])?;
                    reference_ordinals
                        .insert(entry_name_ordinal(&editor.root, references[0], &mut 0).unwrap());
                }
                Some(5) => ensure!(
                    action["id"]
                        .as_u64()
                        .is_some_and(|id| id <= i32::MAX as u64),
                    "Pause action requires an existing numeric id; no duration conversion is supported. Nothing was changed."
                ),
                _ => bail!(
                    "Only existing control actions (type 0/8) and pauses (type 5) are supported; nothing was changed."
                ),
            }
        }
        ensure!(
            index < actions.len() && to_index.is_none_or(|target| target < actions.len()),
            "Autopilot indices are zero-based existing action rows, not condition rows or an append sentinel; nothing was changed."
        );
        let action = actions[index].clone();
        let remaining_actions = actions.len() - usize::from(to_index.is_none());
        let owner = entry_name_ordinal(&editor.root, block, &mut 0).unwrap();
        let document_ordinal = entry_name_ordinal(&editor.root, document, &mut 0).unwrap();
        let storage = action_storage(data, owner, document_ordinal, &reference_ordinals, block)?;
        let changed = to_index != Some(index);
        let change = AutopilotActionChange {
            block_uuid: requested.into(),
            index,
            to_index,
            action,
            remaining_actions,
            invalid: remaining_actions == 0,
            changed,
        };
        if !changed {
            return Ok((data.to_vec(), change));
        }
        let actions = rule["actions"].as_array_mut().unwrap();
        let moved = actions.remove(index);
        if let Some(target) = to_index {
            actions.insert(target, moved);
        } else if actions.is_empty() {
            // Native serialization creates no actions member for an empty list.
            rule.as_object_mut().unwrap().remove("actions");
        }
        ensure!(
            editor.stamp_document(&chrono::Local::now().fixed_offset()),
            "Cannot stamp the admitted Autopilot Document; nothing was changed."
        );
        let document = editor.get_element(document_path);
        let apc = serde_json::to_string(&rule)?;
        let mut patches = vec![(storage.apc, quick_xml::escape::escape(&apc).into_owned())];
        if change.invalid {
            // accept serializes first; the caller's later validation sets STA's invalid bit.
            patches.push((storage.sta, "5".into()));
        }
        for key in ["Date", "DateS"] {
            let value = quick_xml::escape::escape(&document.attributes[key]).into_owned();
            if let Some(range) = storage.dates.get(key) {
                patches.push((range.clone(), value));
            } else {
                patches.push((
                    storage.document_insert..storage.document_insert,
                    format!(" {key}=\"{value}\""),
                ));
            }
        }
        patches.sort_by_key(|(range, _)| std::cmp::Reverse(range.start));
        let mut output = data.to_vec();
        for (range, value) in patches {
            output.splice(range, value.bytes());
        }
        Ok((output, change))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const OWNER: &str = "11111111-2222-3333-4444555566667777";
    const TARGET: &str = "aaaaaaaa-bbbb-cccc-ddddeeeeffff0001";

    fn rule() -> Value {
        json!({
            "type": 1, "id": OWNER, "active": true, "activeIfValid": true,
            "invalid": false, "name": "Keep me", "description": "Unchanged rule description",
            "eventGroups": [{"operator": 1, "events": [
                {"type": 5, "descID": 88, "id": 0, "value": 700, "name": "absolute"},
                {"type": 4, "descID": 89, "id": 0, "value": 100000, "name": "date"}
            ]}],
            "actions": [
                {"type": 0, "descID": 20, "id": TARGET, "command": "on", "arrValues": ["1"], "name": "A"},
                {"type": 5, "descID": 337705, "id": 0, "command": "1000", "arrValues": ["1000"], "name": "pause"},
                {"type": 8, "descID": 20, "id": TARGET, "command": "off", "arrValues": ["0"], "name": "B"}
            ]
        })
    }

    fn fixture(value: &Value) -> Vec<u8> {
        let json = value.to_string();
        let apc = quick_xml::escape::escape(&json);
        format!(
                        "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n<ControlList><!-- unchanged -->\r\n<C Type=\"Document\" U=\"99999999-2222-3333-4444555566667777\" Date=\"old\" DateS=\"1\"><C Type=\"AutopilotRule\" V=\"175\" U=\"{OWNER}\" Title=\"Rule\" STA=\"1\" APC=\"{apc}\"/><C Type=\"VirtualIn\" V=\"175\" U=\"{TARGET}\" WF=\"16\" Title=\"target\"/><Keep Odd='&amp;&quot;' /></C></ControlList>\r\n"
                    ).into_bytes()
    }

    fn parsed(data: &[u8]) -> (Value, String) {
        let editor = ConfigEditor::load(data).unwrap();
        let path = editor.find_elements(&format!("uuid:{OWNER}")).remove(0);
        let block = editor.get_element(&path);
        (
            serde_json::from_str(&block.attributes["APC"]).unwrap(),
            block.attributes["STA"].clone(),
        )
    }

    #[test]
    fn autopilot_actions_remove_preserves_conditions_adjacent_actions_and_raw_xml() {
        let original = rule();
        let input = fixture(&original);
        let (output, change) =
            ConfigEditor::edit_autopilot_action(&input, &format!("uuid:{OWNER}"), 1, None).unwrap();
        let (actual, sta) = parsed(&output);
        let mut expected = original;
        expected["actions"].as_array_mut().unwrap().remove(1);
        assert_eq!(actual, expected);
        assert_eq!(sta, "1");
        assert_eq!(change.action["type"], 5);
        assert_eq!(change.remaining_actions, 2);
        assert!(!change.invalid);
        assert!(output.starts_with(b"\xef\xbb\xbf"));
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("<!-- unchanged -->\r\n"));
        assert!(output.contains("<Keep Odd='&amp;&quot;' />"));
        assert!(output.ends_with("</C></ControlList>\r\n"));
    }

    #[test]
    fn autopilot_actions_move_uses_original_target_index_in_both_directions() {
        for (from, target, order) in [(0, 2, [1, 2, 0]), (2, 0, [2, 0, 1])] {
            let original = rule();
            let (output, change) = ConfigEditor::edit_autopilot_action(
                &fixture(&original),
                &format!("uuid:{OWNER}"),
                from,
                Some(target),
            )
            .unwrap();
            let (actual, sta) = parsed(&output);
            let mut expected = original.clone();
            expected["actions"] = order
                .into_iter()
                .map(|index| original["actions"][index].clone())
                .collect();
            assert_eq!(actual, expected);
            assert_eq!(sta, "1");
            assert_eq!(change.remaining_actions, 3);
        }
        let input = fixture(&rule());
        let (output, change) =
            ConfigEditor::edit_autopilot_action(&input, &format!("uuid:{OWNER}"), 1, Some(1))
                .unwrap();
        assert_eq!(output, input);
        assert!(!change.changed);
    }

    #[test]
    fn autopilot_actions_last_removal_matches_accept_then_validity_update() {
        let mut original = rule();
        original["actions"] = json!([original["actions"][1]]);
        let (output, change) = ConfigEditor::edit_autopilot_action(
            &fixture(&original),
            &format!("uuid:{OWNER}"),
            0,
            None,
        )
        .unwrap();
        let (actual, sta) = parsed(&output);
        original.as_object_mut().unwrap().remove("actions");
        assert_eq!(actual, original);
        assert_eq!(sta, "5");
        assert!(change.invalid);
        assert_eq!(change.remaining_actions, 0);
    }

    #[test]
    fn autopilot_actions_refuse_unsupported_models_even_before_noop() {
        let mut cases = Vec::new();
        for (key, value) in [
            ("type", json!(2)),
            ("active", json!(false)),
            ("activeIfValid", json!(false)),
            ("invalid", json!(true)),
            ("ruleFlags", json!(1)),
            ("permissions", json!({})),
            ("eventGroups", json!([])),
        ] {
            let mut value_rule = rule();
            value_rule[key] = value;
            cases.push(value_rule);
        }
        for field in ["type", "descID", "value"] {
            let mut value_rule = rule();
            value_rule["eventGroups"][0]["events"][0][field] = json!(-1);
            cases.push(value_rule);
        }
        for (field, value) in [
            ("type", json!(6)),
            ("descID", json!(1)),
            ("invalid", json!(true)),
            ("id", json!("00000000-0000-0000-0000000000000000")),
            ("arrValues", json!([7])),
            ("value", json!("legacy")),
        ] {
            let mut value_rule = rule();
            value_rule["actions"][0][field] = value;
            cases.push(value_rule);
        }
        for value in cases {
            let input = fixture(&value);
            let before = input.clone();
            assert!(
                ConfigEditor::edit_autopilot_action(&input, &format!("uuid:{OWNER}"), 1, Some(1),)
                    .is_err(),
                "{value}"
            );
            assert_eq!(input, before);
        }
        for (from, to) in [(3, None), (0, Some(3)), (usize::MAX, Some(0))] {
            assert!(
                ConfigEditor::edit_autopilot_action(
                    &fixture(&rule()),
                    &format!("uuid:{OWNER}"),
                    from,
                    to,
                )
                .is_err()
            );
        }
    }

    #[test]
    fn autopilot_actions_refuse_identity_aliases_templates_and_loader_rewrites() {
        let input = String::from_utf8(fixture(&rule())).unwrap();
        let mut cases = vec![
            input.replace("Date=\"old\"", "Date=\"old\" IfT=\"false\""),
            input.replace("<Keep ", "<Template IfT=\"false\"/><Keep "),
            input.replace("STA=\"1\"", "STA=\"5\""),
            input.replace("Title=\"Rule\"", "Title=\"Wake 7am=ok\""),
            input.replace("V=\"175\"", "V=\"89\""),
            input.replace("WF=\"16\"", "WF=\"33554432\""),
            input.replace(
                "Title=\"target\"",
                "Title=\"target\" uuidParent=\"inherited\"",
            ),
        ];
        for alias in [
            OWNER.to_string(),
            OWNER.to_uppercase(),
            "11111111-2222-3333-4444-555566667777".to_string(),
            TARGET.to_uppercase(),
        ] {
            cases.push(input.replace("<Keep ", &format!("<C Type=\"Not\" U=\"{alias}\"/><Keep ")));
        }
        for value in cases {
            assert!(
                ConfigEditor::edit_autopilot_action(
                    value.as_bytes(),
                    &format!("uuid:{OWNER}"),
                    1,
                    Some(1),
                )
                .is_err(),
                "{value}"
            );
        }
        for selector in [
            "Rule".to_string(),
            "uuid:00000000-0000-0000-0000000000000000".to_string(),
            "uuid:11111111-2222-3333-4444-555566667777".to_string(),
        ] {
            assert!(
                ConfigEditor::edit_autopilot_action(input.as_bytes(), &selector, 0, None).is_err()
            );
        }
    }

    #[test]
    fn autopilot_actions_review_document_type_before_noop_and_edit() {
        let input = String::from_utf8(fixture(&rule()))
            .unwrap()
            .replace("Type=\"Document\"", "Type=\"document\"");
        for target in [None, Some(0), Some(2)] {
            assert!(
                ConfigEditor::edit_autopilot_action(
                    input.as_bytes(),
                    &format!("uuid:{OWNER}"),
                    0,
                    target,
                )
                .is_err(),
                "noncanonical Document admitted for {target:?}",
            );
        }
    }

    #[test]
    fn autopilot_actions_review_unrelated_duplicate_bytes_and_guarded_refusals() {
        let input = String::from_utf8(fixture(&rule())).unwrap();
        let unrelated = r#"<C Type="APIConnector" U="bbbbbbbb-2222-3333-4444555566667777" Title="x&amp;y" Title='x&amp;y' APC="{&quot;text&quot;:&quot;&#65;&quot;}"/>"#;
        let with_unrelated = input.replace("<Keep ", &format!("{unrelated}<Keep "));
        let (output, _) = ConfigEditor::edit_autopilot_action(
            with_unrelated.as_bytes(),
            &format!("uuid:{OWNER}"),
            1,
            None,
        )
        .unwrap();
        assert!(String::from_utf8(output).unwrap().contains(unrelated));

        for refused in [
            input.replace("STA=\"1\"", "STA=\"1\" STA=\"1\""),
            input.replace("Date=\"old\"", "Date=\"old\" Date=\"old\""),
            input.replace("Title=\"target\"", "Title=\"target\" Title=\"target\""),
            input.replace(
                "<Keep ",
                &format!(r#"<C Type="APIConnector" U="{OWNER}" U="{OWNER}"/><Keep "#),
            ),
            input.replace("<Keep ", r#"<APIConnector Title="x" Title="y"/><Keep "#),
        ] {
            let error = ConfigEditor::edit_autopilot_action(
                refused.as_bytes(),
                &format!("uuid:{OWNER}"),
                0,
                Some(0),
            )
            .unwrap_err();
            let message = format!("{error:#}").to_ascii_lowercase();
            assert!(
                message.contains("autopilot") && message.contains("nothing was changed"),
                "{message}"
            );
        }
    }

    #[test]
    fn autopilot_actions_review_refuses_load_only_and_unknown_members() {
        let mut cases = Vec::new();
        for key in ["visu", "permissions", "future"] {
            let mut value = rule();
            value[key] = json!({});
            cases.push((value, key));
        }
        for container in ["action", "group", "event"] {
            let mut value = rule();
            match container {
                "action" => value["actions"][0]["future"] = json!(7),
                "group" => value["eventGroups"][0]["future"] = json!(7),
                _ => value["eventGroups"][0]["events"][0]["future"] = json!(7),
            }
            cases.push((value, "future"));
        }
        for (value, member) in cases {
            for target in [None, Some(0), Some(2)] {
                let error = ConfigEditor::edit_autopilot_action(
                    &fixture(&value),
                    &format!("uuid:{OWNER}"),
                    0,
                    target,
                )
                .unwrap_err();
                assert!(format!("{error:#}").contains(member), "{error:#}");
            }
        }
    }

    #[test]
    fn autopilot_actions_review_refuses_lossy_or_unsupported_numbers() {
        let original = String::from_utf8(fixture(&rule())).unwrap();
        for (before, after) in [
            ("&quot;value&quot;:700", "&quot;value&quot;:7e2"),
            (
                "&quot;value&quot;:700",
                "&quot;value&quot;:700.0000000000000000000000000000000001",
            ),
            (
                "&quot;value&quot;:100000",
                "&quot;value&quot;:123456789012345678901234567890",
            ),
            (
                "&quot;value&quot;:100000",
                "&quot;value&quot;:9007199254740993",
            ),
        ] {
            let input = original.replace(before, after);
            assert_ne!(input, original);
            for target in [None, Some(0), Some(2)] {
                assert!(
                    ConfigEditor::edit_autopilot_action(
                        input.as_bytes(),
                        &format!("uuid:{OWNER}"),
                        0,
                        target,
                    )
                    .is_err(),
                    "accepted {after} for {target:?}"
                );
            }
        }
    }

    #[test]
    fn autopilot_actions_reject_duplicate_json_and_ambiguous_raw_storage() {
        let input = String::from_utf8(fixture(&rule())).unwrap();
        for value in [
            input.replace(
                "&quot;active&quot;:true",
                "&quot;active&quot;:false,&quot;active&quot;:true",
            ),
            input.replace("STA=\"1\"", "STA=\"1\" STA=\"1\""),
            input.replace("&quot;pause&quot;", "&quot;&#112;ause&quot;"),
            input.replace("APC=\"{", "APC=\"{\n "),
        ] {
            assert!(
                ConfigEditor::edit_autopilot_action(
                    value.as_bytes(),
                    &format!("uuid:{OWNER}"),
                    1,
                    Some(1),
                )
                .is_err()
            );
        }
    }
}
