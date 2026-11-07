//! User / user-group object permissions (`<PH>` / `<PRM>` blocks).
//!
//! Loxone stores the objects a user (or user group) may access as a permission list
//! inside the `User` / `UserGroup` element:
//!
//! ```xml
//! <C Type="User" U="..." Title="webhook_api">
//!     <HP .../>
//!     <PH NUM="2">
//!         <PRM REF="<object uuid>" ACT="3"/>
//!         <PRM REF="<object uuid>" ACT="3"/>
//!     </PH>
//! </C>
//! ```
//!
//! `NUM` is the number of `PRM` children; `REF` is the `U` of the permitted object and
//! `ACT` the access level (Loxone Config writes `3` for control access to inputs and
//! blocks). An otherwise unauthorized user calling `/dev/sps/io/<input>/...`
//! receives HTTP 403.
//!
//! These are direct entries, not effective permissions: inherited grants and denies
//! are not evaluated. Removing an entry does not add an explicit deny.

use super::ConfigEditor;
use anyhow::{Result, bail};
use serde::Serialize;
use std::collections::HashMap;
use xmltree::{Element, XMLNode};

/// ACT value Loxone Config writes for "control" access to inputs and blocks.
pub const DEFAULT_PERMISSION_ACT: &str = "3";

/// Whose permission list is edited.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PrincipalKind {
    User,
    UserGroup,
}

impl PrincipalKind {
    /// The `Type` attribute of the principal's `<C>` element.
    pub fn xml_type(self) -> &'static str {
        match self {
            PrincipalKind::User => "User",
            PrincipalKind::UserGroup => "UserGroup",
        }
    }

    /// Human-readable noun for messages.
    pub fn noun(self) -> &'static str {
        match self {
            PrincipalKind::User => "User",
            PrincipalKind::UserGroup => "User group",
        }
    }
}

/// A user or user group.
#[derive(Debug, Clone, Serialize)]
pub struct PrincipalRef {
    pub kind: PrincipalKind,
    pub title: String,
    pub uuid: String,
}

/// An object a permission refers to. `title`/`type` are `None` when the referenced
/// UUID does not exist as an element in the file (dangling or built-in reference).
#[derive(Debug, Clone, Serialize)]
pub struct PermissionObject {
    pub uuid: String,
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub element_type: Option<String>,
}

impl PermissionObject {
    /// `Type 'Title' (uuid:...)` for messages.
    pub fn describe(&self) -> String {
        match (&self.element_type, &self.title) {
            (Some(t), Some(title)) => format!("{} '{}' (uuid:{})", t, title, self.uuid),
            (Some(t), None) => format!("{} (uuid:{})", t, self.uuid),
            (None, Some(title)) => format!("'{}' (uuid:{})", title, self.uuid),
            (None, None) => format!("uuid:{}", self.uuid),
        }
    }
}

/// One `<PRM>` entry, resolved against the file.
#[derive(Debug, Clone, Serialize)]
pub struct PermissionEntry {
    pub uuid: String,
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub element_type: Option<String>,
    pub act: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tim: Option<String>,
}

/// A principal together with its resolved permission list.
#[derive(Debug, Clone, Serialize)]
pub struct PrincipalPermissions {
    pub kind: PrincipalKind,
    pub title: String,
    pub uuid: String,
    pub permissions: Vec<PermissionEntry>,
}

/// What a grant/revoke did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionChange {
    /// A new `PRM` entry was appended.
    Added,
    /// The entry existed; its `ACT` value was changed.
    Updated,
    /// The entry already existed with the requested access — nothing changed.
    Unchanged,
    /// The entry was removed.
    Removed,
    /// There was no entry to remove — nothing changed.
    Absent,
}

/// Result of a grant/revoke.
#[derive(Debug, Clone, Serialize)]
pub struct PermissionEdit {
    pub principal: PrincipalRef,
    pub object: PermissionObject,
    pub change: PermissionChange,
    /// ACT value of the entry after the edit (`None` after a revoke).
    pub act: Option<String>,
    /// ACT value of the entry before the edit (`None` if there was no entry).
    pub previous_act: Option<String>,
}

impl PermissionEdit {
    /// Whether the document was modified.
    pub fn changed(&self) -> bool {
        matches!(
            self.change,
            PermissionChange::Added | PermissionChange::Updated | PermissionChange::Removed
        )
    }
}

fn is_principal(elem: &Element, kind: PrincipalKind) -> bool {
    elem.name == "C"
        && elem
            .attributes
            .get("Type")
            .is_some_and(|t| t == kind.xml_type())
}

fn prm_ref(node: &XMLNode) -> Option<&str> {
    node.as_element()
        .filter(|e| e.name == "PRM")
        .and_then(|e| e.attributes.get("REF"))
        .map(String::as_str)
}

fn prm_count(ph: &Element) -> usize {
    ph.children
        .iter()
        .filter(|c| c.as_element().is_some_and(|e| e.name == "PRM"))
        .count()
}

fn ph_index(elem: &Element) -> Option<usize> {
    elem.children
        .iter()
        .position(|c| c.as_element().is_some_and(|e| e.name == "PH"))
}

impl ConfigEditor {
    /// Collect paths of all principals of `kind`.
    fn principal_paths(&self, kind: PrincipalKind) -> Vec<Vec<usize>> {
        fn walk(
            elem: &Element,
            kind: PrincipalKind,
            path: &mut Vec<usize>,
            out: &mut Vec<Vec<usize>>,
        ) {
            if is_principal(elem, kind) {
                out.push(path.clone());
            }
            for (i, child) in elem.children.iter().enumerate() {
                if let Some(e) = child.as_element() {
                    path.push(i);
                    walk(e, kind, path, out);
                    path.pop();
                }
            }
        }
        let mut out = Vec::new();
        walk(&self.root, kind, &mut Vec::new(), &mut out);
        out
    }

    /// Find exactly one user / user group by exact (case-insensitive) title or `uuid:<uuid>`.
    ///
    /// Deliberately no fuzzy matching: editing the wrong account's permissions is worse
    /// than an error.
    fn find_principal(&self, name: &str, kind: PrincipalKind) -> Result<Vec<usize>> {
        let all = self.principal_paths(kind);
        let wanted_uuid = name.strip_prefix("uuid:");
        let wanted_lower = name.to_lowercase();
        let matches: Vec<Vec<usize>> = all
            .iter()
            .filter(|path| {
                let e = self.get_element(path);
                match wanted_uuid {
                    Some(u) => e.attributes.get("U").is_some_and(|x| x == u),
                    None => e
                        .attributes
                        .get("Title")
                        .is_some_and(|t| t.to_lowercase() == wanted_lower),
                }
            })
            .cloned()
            .collect();

        match matches.len() {
            0 => {
                let mut titles: Vec<String> = all
                    .iter()
                    .filter_map(|p| self.get_element(p).attributes.get("Title").cloned())
                    .collect();
                titles.sort();
                titles.dedup();
                Err(crate::errors::not_found_error(
                    kind.noun(),
                    name,
                    &titles,
                    "lox config users <file>",
                ))
            }
            1 => Ok(matches.into_iter().next().unwrap()),
            n => {
                let list: Vec<String> = matches
                    .iter()
                    .map(|p| {
                        let u = self.get_element(p).attributes.get("U").cloned();
                        format!("  uuid:{}", u.unwrap_or_default())
                    })
                    .collect();
                bail!(
                    "{} name '{}' matches {} elements (expected 1):\n{}\nUse uuid:<uuid> to pick one.",
                    kind.noun(),
                    name,
                    n,
                    list.join("\n")
                )
            }
        }
    }

    fn principal_ref(&self, path: &[usize], kind: PrincipalKind) -> PrincipalRef {
        let e = self.get_element(path);
        PrincipalRef {
            kind,
            title: e.attributes.get("Title").cloned().unwrap_or_default(),
            uuid: e.attributes.get("U").cloned().unwrap_or_default(),
        }
    }

    /// Resolve an object selector to exactly one element with a UUID.
    fn resolve_permission_object(&self, selector: &str) -> Result<PermissionObject> {
        let path = self.require_one(selector)?;
        let e = self.get_element(&path);
        let Some(uuid) = e.attributes.get("U").cloned() else {
            bail!(
                "Element matched by '{}' has no UUID (U attribute), so it cannot be referenced in a permission entry.",
                selector
            );
        };
        Ok(PermissionObject {
            uuid,
            title: e.attributes.get("Title").cloned(),
            element_type: e.attributes.get("Type").cloned(),
        })
    }

    /// Map of element UUID → (Type, Title) for every `<C>` element. First occurrence wins.
    fn uuid_index(&self) -> HashMap<String, (Option<String>, Option<String>)> {
        fn walk(elem: &Element, map: &mut HashMap<String, (Option<String>, Option<String>)>) {
            if elem.name == "C"
                && let Some(u) = elem.attributes.get("U")
            {
                map.entry(u.clone()).or_insert_with(|| {
                    (
                        elem.attributes.get("Type").cloned(),
                        elem.attributes.get("Title").cloned(),
                    )
                });
            }
            for child in &elem.children {
                if let Some(e) = child.as_element() {
                    walk(e, map);
                }
            }
        }
        let mut map = HashMap::new();
        walk(&self.root, &mut map);
        map
    }

    fn collect_permissions(
        &self,
        path: &[usize],
        kind: PrincipalKind,
        index: &HashMap<String, (Option<String>, Option<String>)>,
    ) -> PrincipalPermissions {
        let principal = self.principal_ref(path, kind);
        let elem = self.get_element(path);
        let mut permissions = Vec::new();
        if let Some(ph) = ph_index(elem).and_then(|i| elem.children[i].as_element()) {
            for prm in ph
                .children
                .iter()
                .filter_map(|c| c.as_element())
                .filter(|e| e.name == "PRM")
            {
                let uuid = prm.attributes.get("REF").cloned().unwrap_or_default();
                let (element_type, title) = index.get(&uuid).cloned().unwrap_or((None, None));
                permissions.push(PermissionEntry {
                    uuid,
                    title,
                    element_type,
                    act: prm.attributes.get("ACT").cloned(),
                    tim: prm.attributes.get("TIM").cloned(),
                });
            }
        }
        PrincipalPermissions {
            kind,
            title: principal.title,
            uuid: principal.uuid,
            permissions,
        }
    }

    /// Permission lists of every user (or user group) in the file, in document order.
    pub fn list_permissions(&self, kind: PrincipalKind) -> Vec<PrincipalPermissions> {
        let index = self.uuid_index();
        self.principal_paths(kind)
            .iter()
            .map(|p| self.collect_permissions(p, kind, &index))
            .collect()
    }

    /// Permission list of one user (or user group).
    pub fn principal_permissions(
        &self,
        name: &str,
        kind: PrincipalKind,
    ) -> Result<PrincipalPermissions> {
        let path = self.find_principal(name, kind)?;
        Ok(self.collect_permissions(&path, kind, &self.uuid_index()))
    }

    /// Add or update `principal`'s direct permission entry for the matched object.
    ///
    /// Appends `<PRM REF="<uuid>" ACT="<act>"/>` to the principal's `<PH>` (created if
    /// missing) and updates `NUM`. Idempotent: an existing entry is left alone unless
    /// `act` is given and differs, in which case only its `ACT` attribute is updated.
    /// `act` defaults to [`DEFAULT_PERMISSION_ACT`] for new entries.
    pub fn grant_permission(
        &mut self,
        principal: &str,
        kind: PrincipalKind,
        object_selector: &str,
        act: Option<&str>,
    ) -> Result<PermissionEdit> {
        let object = self.resolve_permission_object(object_selector)?;
        let path = self.find_principal(principal, kind)?;
        let principal = self.principal_ref(&path, kind);
        let elem = self.get_element_mut(&path);

        let existing_ph = ph_index(elem);
        if let Some(idx) = existing_ph {
            let ph = elem.children[idx]
                .as_mut_element()
                .expect("ph_index points at an element");
            if let Some(prm) = ph
                .children
                .iter_mut()
                .filter_map(|c| c.as_mut_element())
                .find(|e| {
                    e.name == "PRM" && e.attributes.get("REF").is_some_and(|r| *r == object.uuid)
                })
            {
                let previous_act = prm.attributes.get("ACT").cloned();
                let (change, act_now) = match act {
                    Some(a) if previous_act.as_deref() != Some(a) => {
                        prm.attributes.insert("ACT".to_string(), a.to_string());
                        (PermissionChange::Updated, Some(a.to_string()))
                    }
                    _ => (PermissionChange::Unchanged, previous_act.clone()),
                };
                return Ok(PermissionEdit {
                    principal,
                    object,
                    change,
                    act: act_now,
                    previous_act,
                });
            }
        }

        let act_value = act.unwrap_or(DEFAULT_PERMISSION_ACT).to_string();
        let mut prm = Element::new("PRM");
        prm.attributes
            .insert("REF".to_string(), object.uuid.clone());
        prm.attributes.insert("ACT".to_string(), act_value.clone());

        match existing_ph {
            Some(idx) => {
                let ph = elem.children[idx]
                    .as_mut_element()
                    .expect("ph_index points at an element");
                ph.children.push(XMLNode::Element(prm));
                let num = prm_count(ph);
                ph.attributes.insert("NUM".to_string(), num.to_string());
            }
            None => {
                let mut ph = Element::new("PH");
                ph.attributes.insert("NUM".to_string(), "1".to_string());
                ph.children.push(XMLNode::Element(prm));
                elem.children.push(XMLNode::Element(ph));
            }
        }

        Ok(PermissionEdit {
            principal,
            object,
            change: PermissionChange::Added,
            act: Some(act_value),
            previous_act: None,
        })
    }

    /// Remove `principal`'s direct permission entry for the matched object.
    ///
    /// Removes the matching `<PRM>` entry and updates `NUM`; an emptied `<PH>` is removed.
    /// A `uuid:<uuid>`
    /// selector also works for entries whose object no longer exists in the file.
    /// Does not deny inherited access. Idempotent: an absent entry changes nothing.
    pub fn revoke_permission(
        &mut self,
        principal: &str,
        kind: PrincipalKind,
        object_selector: &str,
    ) -> Result<PermissionEdit> {
        let object = match self.resolve_permission_object(object_selector) {
            Ok(o) => o,
            Err(e) => match object_selector.strip_prefix("uuid:") {
                Some(u) if !u.is_empty() => PermissionObject {
                    uuid: u.to_string(),
                    title: None,
                    element_type: None,
                },
                _ => return Err(e),
            },
        };
        let path = self.find_principal(principal, kind)?;
        let principal = self.principal_ref(&path, kind);
        let elem = self.get_element_mut(&path);

        let mut previous_act = None;
        let mut removed = false;
        let mut remove_ph = None;
        if let Some(idx) = ph_index(elem) {
            let ph = elem.children[idx]
                .as_mut_element()
                .expect("ph_index points at an element");
            ph.children.retain(|c| {
                if prm_ref(c) == Some(object.uuid.as_str()) {
                    if !removed {
                        previous_act = c
                            .as_element()
                            .and_then(|e| e.attributes.get("ACT").cloned());
                    }
                    removed = true;
                    false
                } else {
                    true
                }
            });
            if removed {
                let num = prm_count(ph);
                if num == 0 && ph.children.iter().all(|c| c.as_element().is_none()) {
                    remove_ph = Some(idx);
                } else {
                    ph.attributes.insert("NUM".to_string(), num.to_string());
                }
            }
        }
        if let Some(idx) = remove_ph {
            elem.children.remove(idx);
        }

        Ok(PermissionEdit {
            principal,
            object,
            change: if removed {
                PermissionChange::Removed
            } else {
                PermissionChange::Absent
            },
            act: None,
            previous_act,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Loxone-formatted fixture (BOM, CRLF, tabs) that round-trips byte-identically.
    fn fixture() -> Vec<u8> {
        let xml = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
<ControlList Version=\"267\">\n\
\t<C Type=\"Document\" V=\"178\" U=\"doc-1\" Title=\"Test\">\n\
\t\t<C Type=\"VirtualInCaption\" V=\"178\" U=\"vic-1\" Title=\"Virtual Inputs\">\n\
\t\t\t<C Type=\"VirtualIn\" V=\"178\" U=\"vi-1\" Title=\"UniFi Doorbell\" WF=\"16384\"/>\n\
\t\t\t<C Type=\"VirtualIn\" V=\"178\" U=\"vi-2\" Title=\"UniFi Gate\" WF=\"16384\"/>\n\
\t\t</C>\n\
\t\t<C Type=\"UserCaption\" V=\"178\" U=\"uc-1\" Title=\"Users\">\n\
\t\t\t<C Type=\"User\" V=\"178\" U=\"user-1\" Title=\"webhook_api\" WF=\"16384\">\n\
\t\t\t\t<HP H=\"h\" Sc=\"2\" S=\"s\" Alg=\"1\"/>\n\
\t\t\t\t<PH NUM=\"1\">\n\
\t\t\t\t\t<PRM REF=\"vi-2\" ACT=\"3\"/>\n\
\t\t\t\t</PH>\n\
\t\t\t</C>\n\
\t\t\t<C Type=\"User\" V=\"178\" U=\"user-2\" Title=\"Guest\" WF=\"16384\">\n\
\t\t\t\t<HP H=\"\" Sc=\"-1\" S=\"\" Alg=\"1\"/>\n\
\t\t\t</C>\n\
\t\t\t<C Type=\"UserGroup\" V=\"178\" U=\"grp-1\" Title=\"Webhooks\" WF=\"16384\" User=\"\"/>\n\
\t\t</C>\n\
\t</C>\n\
</ControlList>\n";
        let mut out = vec![0xEF, 0xBB, 0xBF];
        out.extend_from_slice(xml.replace('\n', "\r\n").as_bytes());
        out
    }

    fn load() -> (Vec<u8>, ConfigEditor) {
        let data = fixture();
        let editor = ConfigEditor::load(&data).unwrap();
        assert_eq!(
            editor.to_bytes().unwrap(),
            data,
            "fixture must round-trip byte-identically"
        );
        (data, editor)
    }

    fn text(editor: &ConfigEditor) -> String {
        String::from_utf8(editor.to_bytes().unwrap()).unwrap()
    }

    #[test]
    fn grant_appends_prm_and_bumps_num() {
        let (_, mut editor) = load();
        let edit = editor
            .grant_permission("webhook_api", PrincipalKind::User, "UniFi Doorbell", None)
            .unwrap();
        assert_eq!(edit.change, PermissionChange::Added);
        assert_eq!(edit.object.uuid, "vi-1");
        assert_eq!(edit.act.as_deref(), Some("3"));
        let out = text(&editor);
        assert!(out.contains(
            "<PH NUM=\"2\">\r\n\t\t\t\t\t<PRM REF=\"vi-2\" ACT=\"3\"/>\r\n\t\t\t\t\t<PRM REF=\"vi-1\" ACT=\"3\"/>\r\n\t\t\t\t</PH>"
        ));
        // Password hashes untouched
        assert!(out.contains("<HP H=\"h\" Sc=\"2\" S=\"s\" Alg=\"1\"/>"));
    }

    #[test]
    fn grant_is_idempotent() {
        let (data, mut editor) = load();
        let edit = editor
            .grant_permission("WEBHOOK_API", PrincipalKind::User, "uuid:vi-2", None)
            .unwrap();
        assert_eq!(edit.change, PermissionChange::Unchanged);
        assert!(!edit.changed());
        assert_eq!(editor.to_bytes().unwrap(), data);
    }

    #[test]
    fn grant_with_different_act_updates_entry() {
        let (_, mut editor) = load();
        let edit = editor
            .grant_permission("webhook_api", PrincipalKind::User, "uuid:vi-2", Some("1"))
            .unwrap();
        assert_eq!(edit.change, PermissionChange::Updated);
        assert_eq!(edit.previous_act.as_deref(), Some("3"));
        assert!(
            text(&editor).contains("<PH NUM=\"1\">\r\n\t\t\t\t\t<PRM REF=\"vi-2\" ACT=\"1\"/>")
        );
    }

    #[test]
    fn grant_preserves_existing_raw_act_and_schedule() {
        let data = String::from_utf8(fixture())
            .unwrap()
            .replace(
                r#"<PRM REF="vi-2" ACT="3"/>"#,
                r#"<PRM REF="vi-2" TIM="schedule-1" ACT="0"/>"#,
            )
            .into_bytes();
        let mut editor = ConfigEditor::load(&data).unwrap();
        let edit = editor
            .grant_permission("uuid:user-1", PrincipalKind::User, "uuid:vi-2", None)
            .unwrap();
        assert_eq!(edit.change, PermissionChange::Unchanged);
        assert_eq!(edit.act.as_deref(), Some("0"));
        assert_eq!(editor.to_bytes().unwrap(), data);

        editor
            .grant_permission("webhook_api", PrincipalKind::User, "uuid:vi-2", Some("1"))
            .unwrap();
        assert!(text(&editor).contains(r#"<PRM REF="vi-2" TIM="schedule-1" ACT="1"/>"#));
        editor
            .grant_permission("webhook_api", PrincipalKind::User, "UniFi Doorbell", None)
            .unwrap();
        let perms = editor
            .principal_permissions("webhook_api", PrincipalKind::User)
            .unwrap();
        assert_eq!(perms.permissions.len(), 2);
        assert_eq!(perms.permissions[0].tim.as_deref(), Some("schedule-1"));
        assert_eq!(perms.permissions[0].act.as_deref(), Some("1"));
    }

    #[test]
    fn revoke_only_removes_direct_entry_not_group_permissions() {
        let (_, mut editor) = load();
        editor
            .grant_permission("Webhooks", PrincipalKind::UserGroup, "UniFi Gate", None)
            .unwrap();
        editor
            .revoke_permission("webhook_api", PrincipalKind::User, "UniFi Gate")
            .unwrap();
        let user = editor
            .principal_permissions("webhook_api", PrincipalKind::User)
            .unwrap();
        let group = editor
            .principal_permissions("Webhooks", PrincipalKind::UserGroup)
            .unwrap();
        assert!(user.permissions.is_empty());
        assert_eq!(group.permissions.len(), 1);
        assert_eq!(group.permissions[0].uuid, "vi-2");
    }

    #[test]
    fn ambiguous_principal_is_rejected_without_changes() {
        let data = String::from_utf8(fixture())
            .unwrap()
            .replace(r#"Title="Guest""#, r#"Title="WEBHOOK_API""#)
            .into_bytes();
        let mut editor = ConfigEditor::load(&data).unwrap();
        let err = editor
            .grant_permission("webhook_api", PrincipalKind::User, "UniFi Doorbell", None)
            .unwrap_err()
            .to_string();
        assert!(err.contains("expected 1"), "{err}");
        assert_eq!(editor.to_bytes().unwrap(), data);
        editor
            .grant_permission("uuid:user-2", PrincipalKind::User, "UniFi Doorbell", None)
            .unwrap();
        assert_eq!(
            editor
                .principal_permissions("uuid:user-1", PrincipalKind::User)
                .unwrap()
                .permissions
                .len(),
            1
        );
    }

    #[test]
    fn grant_creates_ph_when_missing() {
        let (_, mut editor) = load();
        editor
            .grant_permission("Guest", PrincipalKind::User, "UniFi Doorbell", None)
            .unwrap();
        let out = text(&editor);
        assert!(out.contains(
            "<HP H=\"\" Sc=\"-1\" S=\"\" Alg=\"1\"/>\r\n\t\t\t\t<PH NUM=\"1\">\r\n\t\t\t\t\t<PRM REF=\"vi-1\" ACT=\"3\"/>\r\n\t\t\t\t</PH>"
        ));
    }

    #[test]
    fn grant_then_revoke_is_byte_identical() {
        // Existing PH, user without PH, and a self-closing group without children.
        for (principal, kind) in [
            ("webhook_api", PrincipalKind::User),
            ("Guest", PrincipalKind::User),
            ("Webhooks", PrincipalKind::UserGroup),
        ] {
            let (data, mut editor) = load();
            let g = editor
                .grant_permission(principal, kind, "UniFi Doorbell", None)
                .unwrap();
            assert_eq!(g.change, PermissionChange::Added);
            assert_ne!(editor.to_bytes().unwrap(), data);
            let r = editor
                .revoke_permission(principal, kind, "UniFi Doorbell")
                .unwrap();
            assert_eq!(r.change, PermissionChange::Removed);
            assert_eq!(
                editor.to_bytes().unwrap(),
                data,
                "grant+revoke for {principal} must restore the original bytes"
            );
        }
    }

    #[test]
    fn revoke_absent_is_noop() {
        let (data, mut editor) = load();
        let edit = editor
            .revoke_permission("webhook_api", PrincipalKind::User, "UniFi Doorbell")
            .unwrap();
        assert_eq!(edit.change, PermissionChange::Absent);
        assert!(!edit.changed());
        assert_eq!(editor.to_bytes().unwrap(), data);
    }

    #[test]
    fn revoke_dangling_uuid() {
        let (_, mut editor) = load();
        editor
            .grant_permission("webhook_api", PrincipalKind::User, "uuid:vi-1", None)
            .unwrap();
        editor.remove_element("vi-1").unwrap();
        let edit = editor
            .revoke_permission("webhook_api", PrincipalKind::User, "uuid:vi-1")
            .unwrap();
        assert_eq!(edit.change, PermissionChange::Removed);
        assert!(edit.object.title.is_none());
    }

    #[test]
    fn ambiguous_object_is_rejected() {
        let (_, mut editor) = load();
        let err = editor
            .grant_permission("webhook_api", PrincipalKind::User, "UniFi", None)
            .unwrap_err()
            .to_string();
        assert!(err.contains("elements (expected 1)"), "{err}");
    }

    #[test]
    fn principal_requires_exact_title() {
        let (_, mut editor) = load();
        let err = editor
            .grant_permission("Unifi", PrincipalKind::User, "UniFi Doorbell", None)
            .unwrap_err()
            .to_string();
        assert!(err.contains("not found"), "{err}");
        // A group is not a user
        assert!(
            editor
                .grant_permission("Webhooks", PrincipalKind::User, "UniFi Doorbell", None)
                .is_err()
        );
    }

    #[test]
    fn list_permissions_resolves_titles() {
        let (_, editor) = load();
        let p = editor
            .principal_permissions("webhook_api", PrincipalKind::User)
            .unwrap();
        assert_eq!(p.uuid, "user-1");
        assert_eq!(p.permissions.len(), 1);
        assert_eq!(p.permissions[0].uuid, "vi-2");
        assert_eq!(p.permissions[0].title.as_deref(), Some("UniFi Gate"));
        assert_eq!(p.permissions[0].element_type.as_deref(), Some("VirtualIn"));
        assert_eq!(p.permissions[0].act.as_deref(), Some("3"));

        let all = editor.list_permissions(PrincipalKind::User);
        assert_eq!(all.len(), 2);
        assert!(all[1].permissions.is_empty());
        let groups = editor.list_permissions(PrincipalKind::UserGroup);
        assert_eq!(groups.len(), 1);
    }
}
