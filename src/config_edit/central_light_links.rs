use std::collections::{HashMap, HashSet};

use anyhow::{Result, bail};
use serde::Serialize;
use xmltree::Element;

use super::ConfigEditor;

const LINKED: u32 = 0x1000_0000;

#[derive(Debug, Clone, Copy)]
pub enum CentralLightLinkDirection {
    Up,
    Down,
}

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

impl ConfigEditor {
    pub fn inspect_central_light_links(&self, selector: &str) -> Result<CentralLightLinks> {
        let (_, inspection) = self.central_light_links(selector)?;
        Ok(inspection)
    }

    pub fn move_central_light_link(
        &mut self,
        selector: &str,
        uuid: &str,
        direction: CentralLightLinkDirection,
    ) -> Result<CentralLightLinkMove> {
        let selected = identity(uuid)?;
        let (path, inspection) = self.central_light_links(selector)?;
        let index = inspection
            .links
            .iter()
            .position(|link| link.uuid.eq_ignore_ascii_case(&selected))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "'{uuid}' is not an existing linked-function UUID on this CentralLight. Use `lox config central-light inspect FILE OWNER`; positions and names are not identities. Nothing was changed."
                )
            })?;
        let destination = match direction {
            CentralLightLinkDirection::Up => index.saturating_sub(1),
            CentralLightLinkDirection::Down => (index + 1).min(inspection.links.len() - 1),
        };
        let result = CentralLightLinkMove {
            uuid: inspection.links[index].uuid.clone(),
            from_position: index + 1,
            to_position: destination + 1,
            changed: index != destination,
        };
        if result.changed {
            let mut links: Vec<_> = inspection.links.into_iter().map(|link| link.uuid).collect();
            links.swap(index, destination);
            self.get_element_mut(&path)
                .attributes
                .insert("linkC".into(), links.join(","));
        }
        Ok(result)
    }

    fn central_light_links(&self, selector: &str) -> Result<(Vec<usize>, CentralLightLinks)> {
        let path = self.require_one(selector)?;
        let owner = self.get_element(&path);
        if owner.name != "C"
            || owner.attributes.get("Type").map(String::as_str) != Some("CentralLight")
            || owner.namespace.is_some()
            || owner.prefix.is_some()
        {
            bail!(
                "'{selector}' is not a plain CentralLight. Use `lox config controls FILE --type CentralLight`; nothing was changed."
            );
        }
        let documents = self.find_elements("Type:Document");
        if documents.len() != 1 || !path.starts_with(&documents[0]) {
            bail!("CentralLight links require one owning Document; nothing was changed.");
        }
        let document = self.get_element(&documents[0]);
        if document.name != "C" || document.namespace.is_some() || document.prefix.is_some() {
            bail!("CentralLight links require a plain owning Document; nothing was changed.");
        }
        if document
            .children
            .iter()
            .filter_map(|child| child.as_element())
            .any(|child| child.name == "Template")
        {
            bail!(
                "CentralLight links do not support Document Template configurations: native accepted-list propagation is not implemented. Use a standalone document; nothing was changed."
            );
        }
        let io: Vec<_> = owner
            .children
            .iter()
            .filter_map(|child| child.as_element())
            .filter(|child| child.name == "IoData")
            .collect();
        if io.len() != 1
            || io[0].attributes.get("Visu").map(String::as_str) != Some("true")
            || io[0].namespace.is_some()
            || io[0].prefix.is_some()
        {
            bail!(
                "CentralLight links require one direct IoData with explicit Visu=\"true\". Native fallback capability and parent-owner redirection are unsupported; nothing was changed."
            );
        }
        let uuid = owner.attributes.get("U").ok_or_else(|| {
            anyhow::anyhow!("CentralLight has no U identity; nothing was changed.")
        })?;
        let owner_key = identity(uuid)?;
        let mut controls = Vec::new();
        collect_controls(document, &mut controls);
        let mut by_uuid: HashMap<String, Vec<&Element>> = HashMap::new();
        for control in &controls {
            if let Some(key) = control
                .attributes
                .get("U")
                .and_then(|raw| identity(raw).ok())
            {
                by_uuid.entry(key).or_default().push(control);
            }
        }
        resolve(&by_uuid, &owner_key)?;
        let mut referenced = HashSet::new();
        for control in &controls {
            for link in links(control)? {
                let key = identity(link)?;
                resolve(&by_uuid, &key)?;
                referenced.insert(key);
            }
        }
        // Native accept recomputes this marker document-wide. Membership is
        // unchanged by a move, so require a graph needing no incidental repair.
        for control in &controls {
            let marked = flags(control)? & LINKED != 0;
            let expected = control
                .attributes
                .get("U")
                .and_then(|raw| identity(raw).ok())
                .is_some_and(|key| referenced.contains(&key));
            if marked != expected {
                bail!(
                    "Inconsistent document linked-function WF marker on '{}'. Repair the linkC reference graph in Loxone Config first; nothing was changed.",
                    control.attributes.get("U").map_or("(no U)", String::as_str)
                );
            }
        }
        let mut result = Vec::new();
        for (index, link) in links(owner)?.into_iter().enumerate() {
            let control = resolve(&by_uuid, &identity(link)?)?;
            result.push(CentralLightLink {
                position: index + 1,
                uuid: link.into(),
                title: control.attributes.get("Title").cloned().unwrap_or_default(),
                control_type: control.attributes.get("Type").cloned().unwrap_or_default(),
            });
        }
        Ok((
            path,
            CentralLightLinks {
                uuid: uuid.clone(),
                title: owner.attributes.get("Title").cloned().unwrap_or_default(),
                links: result,
            },
        ))
    }
}

fn collect_controls<'a>(element: &'a Element, controls: &mut Vec<&'a Element>) {
    if element.name == "C" {
        controls.push(element);
    }
    for child in element
        .children
        .iter()
        .filter_map(|child| child.as_element())
    {
        collect_controls(child, controls);
    }
}

fn resolve<'a>(controls: &HashMap<String, Vec<&'a Element>>, key: &str) -> Result<&'a Element> {
    match controls.get(key).map(Vec::as_slice) {
        Some([control]) if control.namespace.is_none() && control.prefix.is_none() => Ok(control),
        _ => bail!(
            "Linked-function UUID '{key}' does not resolve to exactly one plain control. Dangling/ambiguous references and native refresh repair are unsupported; nothing was changed."
        ),
    }
}

fn links(control: &Element) -> Result<Vec<&str>> {
    let raw = control.attributes.get("linkC").map_or("", String::as_str);
    if raw.is_empty() {
        return Ok(Vec::new());
    }
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for link in raw.split(',') {
        if !seen.insert(identity(link)?) {
            bail!("Duplicate linked-function UUID '{link}'; nothing was changed.");
        }
        result.push(link);
    }
    Ok(result)
}

fn flags(control: &Element) -> Result<u32> {
    match control.attributes.get("WF") {
        None => Ok(0),
        Some(raw) => raw
            .parse()
            .map_err(|_| anyhow::anyhow!("Unsupported decimal WF '{raw}'; nothing was changed.")),
    }
}

fn identity(raw: &str) -> Result<String> {
    if raw.len() == 35
        && raw.bytes().enumerate().all(|(index, byte)| {
            if [8, 13, 18].contains(&index) {
                byte == b'-'
            } else {
                byte.is_ascii_hexdigit()
            }
        })
        && raw.bytes().any(|byte| byte != b'0' && byte != b'-')
    {
        Ok(raw.to_ascii_lowercase())
    } else {
        bail!(
            "'{raw}' is not a nonzero native UUID (8-4-4-16 hexadecimal digits). Select an existing linked-function U, not a row number; nothing was changed."
        )
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
            r#"<ControlList><C Type="Document" U="{}" Date="old"><C Type="CentralLight" Title="Central" U="{}" linkC="{},{},{}" Mystery="keep"><IoData Visu="true"/><Unknown X="keep"/><Co K="I" U="{}"><W C="{}" Io="I" Foo="keep"/></Co></C><C Type="Not" Title="A" U="{}" WF="268451840"/><C Type="Not" Title="B" U="{}" WF="268435456"/><C Type="Not" Title="C" U="{}" WF="268435456"/><C Type="HVACController" U="{}" iObjS="unrelated" WF="16384"/></C></ControlList>"#,
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
        let result = editor
            .move_central_light_link(
                "Central",
                &id(11).to_uppercase(),
                CentralLightLinkDirection::Up,
            )
            .unwrap();
        assert_eq!((result.from_position, result.to_position), (2, 1));
        let mut expected = before;
        expected.children[0].as_mut_element().unwrap().children[0]
            .as_mut_element()
            .unwrap()
            .attributes
            .insert("linkC".into(), format!("{},{},{}", id(11), id(10), id(12)));
        assert_eq!(editor.root, expected);
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
}
