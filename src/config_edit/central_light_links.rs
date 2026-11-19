use anyhow::Result;
use lox_core::configuration::editor::{
    self, CentralLightLinkDirection, CentralLightLinkMoveError, CentralLightLinkRemovalError,
    CentralLightLinksError,
};
use serde::Serialize;

use super::{ConfigEditor, editor_error};

#[derive(Debug, Serialize)]
pub struct CentralLightLink {
    pub position: usize,
    pub uuid: String,
    pub title: String,
    pub control_type: String,
}

#[derive(Debug, Serialize)]
pub struct CentralLightLinks {
    pub uuid: String,
    pub title: String,
    pub links: Vec<CentralLightLink>,
}

#[derive(Debug, Serialize)]
pub struct CentralLightLinkMove {
    pub uuid: String,
    pub from_position: usize,
    pub to_position: usize,
    pub changed: bool,
}

#[derive(Debug, Serialize)]
pub struct CentralLightLinkRemoval {
    pub uuid: String,
    pub from_position: usize,
    pub remaining_links: usize,
    pub reference_marker_cleared: bool,
}

impl From<editor::CentralLightLinks> for CentralLightLinks {
    fn from(inspection: editor::CentralLightLinks) -> Self {
        Self {
            uuid: inspection.uuid,
            title: inspection.title,
            links: inspection
                .links
                .into_iter()
                .map(|link| CentralLightLink {
                    position: link.position,
                    uuid: link.uuid,
                    title: link.title,
                    control_type: link.control_type,
                })
                .collect(),
        }
    }
}

impl ConfigEditor {
    pub fn inspect_central_light_links(&self, selector: &str) -> Result<CentralLightLinks> {
        self.central_light_links(selector)
            .map(|(_, inspection)| inspection.into())
    }

    pub fn move_central_light_link(
        &mut self,
        selector: &str,
        uuid: &str,
        direction: CentralLightLinkDirection,
    ) -> Result<(CentralLightLinkMove, CentralLightLinks)> {
        let (movement, inspection) = self
            .kernel
            .move_central_light_link(selector, uuid, direction)
            .map_err(central_light_link_move_error)?;
        Ok((
            CentralLightLinkMove {
                uuid: movement.uuid,
                from_position: movement.from_position,
                to_position: movement.to_position,
                changed: movement.changed,
            },
            inspection.into(),
        ))
    }

    pub fn remove_central_light_link(
        &mut self,
        selector: &str,
        uuid: &str,
    ) -> Result<(CentralLightLinkRemoval, CentralLightLinks)> {
        let (removal, inspection) = self
            .kernel
            .remove_central_light_link(selector, uuid)
            .map_err(central_light_link_removal_error)?;
        Ok((
            CentralLightLinkRemoval {
                uuid: removal.uuid,
                from_position: removal.from_position,
                remaining_links: removal.remaining_links,
                reference_marker_cleared: removal.reference_marker_cleared,
            },
            inspection.into(),
        ))
    }

    fn central_light_links(
        &self,
        selector: &str,
    ) -> Result<(Vec<usize>, editor::CentralLightLinks)> {
        self.kernel
            .match_central_light_links(selector)
            .map_err(central_light_links_error)
    }
}

fn central_light_link_move_error(error: CentralLightLinkMoveError) -> anyhow::Error {
    match error {
        CentralLightLinkMoveError::Edit(error) => editor_error(error),
        CentralLightLinkMoveError::Links(error) => central_light_links_error(error),
        CentralLightLinkMoveError::UnsupportedTarget { uuid } => anyhow::anyhow!(
            "Cannot safely reproduce native Up/Down for linked UUID '{uuid}': this CLI admits only direct ModbusServer targets with a nonempty Title, and every member of this owner's linkC must qualify. Other native-visible families and ancestor fallback are unproven here; nothing was changed."
        ),
        CentralLightLinkMoveError::NotLinked { uuid } => anyhow::anyhow!(
            "'{uuid}' is not an existing linked-function UUID on this CentralLight. Use `lox config central-light inspect FILE OWNER`; positions and names are not identities. Nothing was changed."
        ),
        error => error.into(),
    }
}

fn central_light_link_removal_error(error: CentralLightLinkRemovalError) -> anyhow::Error {
    match error {
        CentralLightLinkRemovalError::Edit(error) => editor_error(error),
        CentralLightLinkRemovalError::Links(error) => central_light_links_error(error),
        CentralLightLinkRemovalError::UnsupportedTarget { uuid } => anyhow::anyhow!(
            "Cannot safely reproduce native removal for linked UUID '{uuid}': every member of this owner's linkC must be a direct ModbusServer target with a nonempty Title. Other native-visible families and ancestor fallback are unproven here; nothing was changed."
        ),
        CentralLightLinkRemovalError::NotLinked { uuid } => anyhow::anyhow!(
            "'{uuid}' is not an existing linked-function UUID on this CentralLight. Use `lox config central-light inspect FILE OWNER`; nothing was changed."
        ),
        CentralLightLinkRemovalError::UnsupportedVersion { .. } => anyhow::anyhow!(
            "CentralLight removal requires explicit decimal V >= 90 on every Document control so WF persistence is known. Missing/legacy versions are not migrated; nothing was changed."
        ),
        CentralLightLinkRemovalError::AffectedFlagsOutOfRange { value } => anyhow::anyhow!(
            "Affected WF '{value}' is outside the admitted nonnegative signed32 format; nothing was changed."
        ),
        error => error.into(),
    }
}

fn central_light_links_error(error: CentralLightLinksError) -> anyhow::Error {
    use CentralLightLinksError::*;

    let detail = match &error {
        NotPlainCentralLight { .. } => ". Use `lox config controls FILE --type CentralLight`",
        DocumentTemplate => {
            ": native accepted-list propagation is not implemented. Use a standalone document"
        }
        NotDirectVisibleOwner => {
            ". Native fallback capability and parent-owner redirection are unsupported"
        }
        InvalidIdentity { .. } => ". Select an existing linked-function U, not a row number",
        UnresolvedControl { .. } => {
            ". Dangling/ambiguous references and native refresh repair are unsupported"
        }
        InconsistentMarker { .. } => ". Repair the linkC reference graph in Loxone Config first",
        _ => "",
    };
    match error {
        Edit(error) => editor_error(error),
        error => anyhow::anyhow!("{error}{detail}; nothing was changed."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u32) -> String {
        format!("{n:08x}-0000-0000-0000000000000000")
    }

    fn fixture() -> String {
        format!(
            r#"<ControlList><C Type="Document" U="{}" Date="old"><C Type="CentralLight" Title="Central" U="{}" linkC="{},{},{}" Mystery="keep"><IoData Visu="true"/><Unknown X="keep"/><Co K="I" U="{}"><W C="{}" Io="I" Foo="keep"/></Co></C><C Type="ModbusServer" Title="A" U="{}" WF="268451840"/><C Type="ModbusServer" Title="B" U="{}" WF="268435456"/><C Type="ModbusServer" Title="C" U="{}" WF="268435456"/><C Type="HVACController" U="{}" iObjS="unrelated" WF="16384"/></C></ControlList>"#,
            id(1),
            id(2),
            id(10),
            id(11),
            id(12),
            id(20),
            id(21),
            id(10),
            id(11),
            id(12),
            id(30)
        )
    }

    #[test]
    fn central_light_link_uuid_move_preserves_other_configuration_and_unrelated_iobjs() {
        let xml = fixture();
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = editor.root.clone();
        let (result, inspection) = editor
            .move_central_light_link(
                "Central",
                &id(11).to_uppercase(),
                CentralLightLinkDirection::Up,
            )
            .unwrap();
        assert_eq!((result.from_position, result.to_position), (2, 1));
        assert_eq!(inspection.links[0].uuid, id(11));
        let mut expected = before;
        expected.children[0].as_mut_element().unwrap().children[0]
            .as_mut_element()
            .unwrap()
            .attributes
            .insert("linkC".into(), format!("{},{},{}", id(11), id(10), id(12)));
        assert_eq!(editor.root, expected);

        let now = chrono::DateTime::parse_from_rfc3339("2025-01-03T04:05:06+01:00").unwrap();
        let transaction = editor::move_central_light_link(
            xml.as_bytes(),
            "Central",
            &id(11).to_uppercase(),
            CentralLightLinkDirection::Up,
            &now,
        )
        .unwrap();
        assert!(transaction.document_stamped && editor.stamp_document(&now));
        assert_eq!(editor.to_bytes().unwrap(), transaction.bytes);

        editor
            .move_central_light_link("Central", &id(11), CentralLightLinkDirection::Down)
            .unwrap();
        assert_eq!(
            editor.inspect_central_light_links("Central").unwrap().links[1].uuid,
            id(11)
        );
    }

    #[test]
    fn central_light_link_boundaries_are_noops_and_empty_list_cannot_move() {
        let xml = fixture();
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        let before = editor.root.clone();
        for (uuid, direction) in [
            (id(10), CentralLightLinkDirection::Up),
            (id(12), CentralLightLinkDirection::Down),
        ] {
            assert!(
                !editor
                    .move_central_light_link("Central", &uuid, direction)
                    .unwrap()
                    .0
                    .changed
            );
            assert_eq!(editor.root, before);
        }
        let empty = xml
            .replace(
                &format!(r#"linkC="{},{},{}""#, id(10), id(11), id(12)),
                r#"linkC="""#,
            )
            .replace("268451840", "16384")
            .replace("268435456", "0");
        let mut editor = ConfigEditor::load(empty.as_bytes()).unwrap();
        assert!(
            editor
                .inspect_central_light_links("Central")
                .unwrap()
                .links
                .is_empty()
        );
        assert!(
            editor
                .move_central_light_link("Central", &id(10), CentralLightLinkDirection::Down)
                .is_err()
        );
    }

    #[test]
    fn central_light_link_unsupported_coupling_and_bad_identities_are_atomic() {
        let xml = fixture();
        let variants = [
            xml.replace(r#"Visu="true""#, r#"Visu="false""#),
            xml.replace(r#"Type="CentralLight""#, r#"Type="CentralShade""#),
            xml.replace(r#"Date="old">"#, r#"Date="old"><Template IfT="true"/>"#),
            xml.replace("268451840", "16384"),
            xml.replace(r#"Mystery="keep""#, r#"Mystery="keep" WF="268435456""#),
            xml.replace(
                r#"iObjS="unrelated""#,
                &format!(r#"iObjS="unrelated" linkC="{}""#, id(99)),
            ),
            xml.replace(
                &format!("{},{},{}", id(10), id(11), id(12)),
                &format!("{},{},{}", id(10), id(10), id(12)),
            ),
            xml.replace(
                &format!("{},{},{}", id(10), id(11), id(12)),
                &format!("{},{},{}", id(10), id(11), id(99)),
            ),
            xml.replace(
                r#"<IoData Visu="true"/>"#,
                r#"<IoData Visu="true"/><IoData Visu="true"/>"#,
            ),
            xml.replace(
                r#"<C Type="HVACController""#,
                &format!(r#"<C Type="Not" U="{}"/><C Type="HVACController""#, id(10)),
            ),
        ];
        for invalid in variants {
            let mut editor = ConfigEditor::load(invalid.as_bytes()).unwrap();
            let before = editor.root.clone();
            assert!(
                editor
                    .move_central_light_link("Central", &id(11), CentralLightLinkDirection::Up)
                    .is_err()
            );
            assert_eq!(editor.root, before);
        }
        for invalid in [
            "2",
            "A",
            "00000000-0000-0000-0000000000000000",
            "uuid:0000000a-0000-0000-0000000000000000",
            &id(99),
        ] {
            let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
            let before = editor.root.clone();
            assert!(
                editor
                    .move_central_light_link("Central", invalid, CentralLightLinkDirection::Up)
                    .is_err()
            );
            assert_eq!(editor.root, before);
        }
    }

    #[test]
    fn central_light_link_move_refuses_hidden_or_unproven_members_not_just_selection() {
        let xml = fixture();
        let hidden_first = xml.replacen(r#"Type="ModbusServer""#, r#"Type="Not""#, 1);
        for (uuid, direction) in [
            (id(10), CentralLightLinkDirection::Down),
            (id(11), CentralLightLinkDirection::Up),
            (id(12), CentralLightLinkDirection::Down),
        ] {
            let mut editor = ConfigEditor::load(hidden_first.as_bytes()).unwrap();
            let before = editor.root.clone();
            assert_eq!(
                editor
                    .inspect_central_light_links("Central")
                    .unwrap()
                    .links
                    .len(),
                3
            );
            assert!(
                editor
                    .move_central_light_link("Central", &uuid, direction)
                    .unwrap_err()
                    .to_string()
                    .contains("native Up/Down")
            );
            assert_eq!(editor.root, before);
        }
        let empty_title = xml.replacen(r#"Title="A""#, r#"Title="""#, 1);
        let mut editor = ConfigEditor::load(empty_title.as_bytes()).unwrap();
        assert!(
            editor
                .move_central_light_link("Central", &id(11), CentralLightLinkDirection::Up)
                .is_err()
        );
        let unrelated_hidden = xml.replace(
            r#"iObjS="unrelated" WF="16384"/>"#,
            &format!(
                r#"iObjS="unrelated" WF="16384" linkC="{}"/><C Type="Not" U="{}" Title="Unrelated" WF="268435456"/>"#,
                id(13), id(13)
            ),
        );
        let mut editor = ConfigEditor::load(unrelated_hidden.as_bytes()).unwrap();
        assert!(
            editor
                .move_central_light_link("Central", &id(11), CentralLightLinkDirection::Up)
                .unwrap()
                .0
                .changed
        );
    }

    #[test]
    fn central_light_link_removal_clears_only_last_incoming_reference_bit() {
        let xml = fixture().replace("<C ", r#"<C V="178" "#);
        for shared in [false, true] {
            let source = if shared {
                xml.replace(
                    r#"iObjS="unrelated""#,
                    &format!(r#"iObjS="unrelated" linkC="{}""#, id(10).to_uppercase()),
                )
            } else {
                xml.clone()
            };
            let mut editor = ConfigEditor::load(source.as_bytes()).unwrap();
            let mut expected = editor.root.clone();
            let document = expected.children[0].as_mut_element().unwrap();
            document.children[0]
                .as_mut_element()
                .unwrap()
                .attributes
                .insert("linkC".into(), format!("{},{}", id(11), id(12)));
            if !shared {
                document.children[1]
                    .as_mut_element()
                    .unwrap()
                    .attributes
                    .insert("WF".into(), "16384".into());
            }
            let (result, inspection) = editor
                .remove_central_light_link("Central", &id(10).to_uppercase())
                .unwrap();
            assert_eq!(result.from_position, 1);
            assert_eq!(result.remaining_links, 2);
            assert_eq!(result.reference_marker_cleared, !shared);
            assert_eq!(inspection.links[0].uuid, id(11));
            assert_eq!(editor.root, expected);

            let now = chrono::DateTime::parse_from_rfc3339("2025-01-03T04:05:06+01:00").unwrap();
            let transaction = editor::remove_central_light_link(
                source.as_bytes(),
                "Central",
                &id(10).to_uppercase(),
                &now,
            )
            .unwrap();
            assert!(transaction.document_stamped && editor.stamp_document(&now));
            assert_eq!(editor.to_bytes().unwrap(), transaction.bytes);
        }
    }

    #[test]
    fn central_light_link_removal_allows_last_entry_and_omits_empty_defaults() {
        let xml = fixture().replace("<C ", r#"<C V="178" "#);
        let mut editor = ConfigEditor::load(xml.as_bytes()).unwrap();
        for n in [10, 11, 12] {
            editor.remove_central_light_link("Central", &id(n)).unwrap();
        }
        let owner = editor.require_one("Central").unwrap();
        assert!(!editor.get_element(&owner).attributes.contains_key("linkC"));
        for n in [11, 12] {
            let path = editor.require_one(&format!("uuid:{}", id(n))).unwrap();
            assert!(!editor.get_element(&path).attributes.contains_key("WF"));
        }
        let bytes = editor.to_bytes().unwrap();
        let reloaded = ConfigEditor::load(&bytes).unwrap();
        assert!(
            reloaded
                .inspect_central_light_links("Central")
                .unwrap()
                .links
                .is_empty()
        );
        let before = editor.root.clone();
        assert!(
            editor
                .remove_central_light_link("Central", &id(12))
                .is_err()
        );
        assert_eq!(editor.root, before);
    }

    #[test]
    fn central_light_link_removal_refuses_unproven_persistence_or_coupling_atomically() {
        let xml = fixture().replace("<C ", r#"<C V="178" "#);
        for invalid in [
            fixture(),
            xml.replacen(r#"V="178""#, r#"V="89""#, 1),
            xml.replacen(r#"V="178""#, r#"V="future""#, 1),
            xml.replace("268451840", "2415935488"),
            xml.replace("268451840", "16384"),
            xml.replacen(r#"Type="ModbusServer""#, r#"Type="Not""#, 1),
            xml.replace(r#"Date="old">"#, r#"Date="old"><Template IfT="true"/>"#),
        ] {
            let mut editor = ConfigEditor::load(invalid.as_bytes()).unwrap();
            let before = editor.root.clone();
            assert!(
                editor
                    .remove_central_light_link("Central", &id(10))
                    .is_err()
            );
            assert_eq!(editor.root, before);
        }
    }

    #[test]
    fn central_light_link_removal_restores_ordered_attributes_on_post_edit_refusal() {
        let xml = fixture()
            .replace("<C ", r#"<C V="178" "#)
            .replace(&format!("{},{},{}", id(10), id(11), id(12)), &id(11))
            .replace("268451840", "16384")
            .replace(
                &format!(r#"U="{}" WF="268435456""#, id(12)),
                &format!(r#"U="{}""#, id(12)),
            );
        let shadowed = xml.replacen(
            "<ControlList>",
            &format!(r#"<ControlList><C U="{}" WF="77" Mystery="keep"/>"#, id(11)),
            1,
        );
        let mut editor = ConfigEditor::load(shadowed.as_bytes()).unwrap();
        let before = editor.to_bytes().unwrap();
        let error = editor
            .remove_central_light_link("Central", &id(11))
            .unwrap_err();
        assert!(error.to_string().contains(&format!(
            "Inconsistent document linked-function WF marker on '{}'",
            id(11)
        )));
        assert_eq!(editor.to_bytes().unwrap(), before);
    }
}
