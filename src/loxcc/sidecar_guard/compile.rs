//! Compile `LoxAPP3.json` and `permissions.bin` changes for a narrow set of XML edits.
//!
//! Every generated catalogue entry and permission record is derived from a native
//! entry of the same type that Loxone Config already compiled into the source
//! archive. Edits without such a template, and every other sidecar-affecting
//! change, fail closed in the caller.
//!
//! Supported edits:
//! - adding a visualized `PushButton2Sel` (`ValueSelector`), `State` (`TextState`)
//!   or `PButtonT` (`Pushbutton`), or a linked `MeterAbsUni` (`Meter`) whose `Rec`
//!   is an existing energy manager;
//! - renaming those controls (`Title`);
//! - adding or renaming load nodes in an energy manager's node tree;
//! - changing an existing visualized `Doorcontroller`'s action labels and links.
//!
//! Untouched catalogue entries and permission records are preserved byte-for-byte.

use super::{Element, XMLNode, shape};
use anyhow::{Context, Result, anyhow, bail, ensure};
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet, HashMap};

type Token = [u8; 16];

const ACTION_END: Token = [0xfe; 16];
const STATES_END: Token = [0xfc; 16];
const OWNER_END: Token = [0xfb; 16];
const RECORD_END: Token = [0xff; 16];

/// Meter catalogue states and the `MeterAbsUni` connectors that provide them.
const METER_STATES: [(&str, &str); 7] = [
    ("jLocked", "R"),
    ("actual", "OPf"),
    ("total", "OMr"),
    ("totalDay", "ORd"),
    ("totalWeek", "ORw"),
    ("totalMonth", "ORm"),
    ("totalYear", "ORy"),
];

/// A control whose sidecar projection differs between the source and edited XML.
pub(super) struct Change<'a> {
    pub(super) key: String,
    pub(super) before: Option<(&'a str, &'a Element)>,
    pub(super) after: Option<(&'a str, &'a Element)>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Selector,
    TextState,
    Pushbutton,
    Meter,
    Intercom,
    EnergyManager,
}

impl Kind {
    fn app_type(self) -> &'static str {
        match self {
            Kind::Selector => "ValueSelector",
            Kind::TextState => "TextState",
            Kind::Pushbutton => "Pushbutton",
            Kind::Meter => "Meter",
            Kind::Intercom => "Intercom",
            Kind::EnergyManager => "EFM",
        }
    }

    /// XML attributes and child elements this compiler reconciles for an existing control.
    fn reconciled(self) -> (&'static [&'static str], &'static [&'static str]) {
        match self {
            Kind::Selector | Kind::TextState | Kind::Pushbutton | Kind::Meter => (&["Title"], &[]),
            Kind::Intercom => (&["OutName1", "OutName2", "OutName3", "linkC"], &[]),
            Kind::EnergyManager => (&[], &["root"]),
        }
    }

    fn can_add(self) -> bool {
        matches!(
            self,
            Kind::Selector | Kind::TextState | Kind::Pushbutton | Kind::Meter
        )
    }
}

fn attribute<'a>(element: &'a Element, name: &str) -> Option<&'a str> {
    element
        .attributes
        .get(name)
        .map(String::as_str)
        .filter(|value| !value.is_empty())
}

fn required<'a>(element: &'a Element, name: &str) -> Result<&'a str> {
    attribute(element, name).with_context(|| {
        format!(
            "{} {} lacks the {name} attribute",
            element.name,
            attribute(element, "U").unwrap_or("<unidentified>")
        )
    })
}

fn children<'a>(element: &'a Element, name: &'a str) -> impl Iterator<Item = &'a Element> {
    element
        .children
        .iter()
        .filter_map(XMLNode::as_element)
        .filter(move |child| child.name == name)
}

fn descendants<'a>(element: &'a Element, name: &str, result: &mut Vec<&'a Element>) {
    if element.name == name {
        result.push(element);
    }
    for child in element.children.iter().filter_map(XMLNode::as_element) {
        descendants(child, name, result);
    }
}

fn visible(element: &Element) -> bool {
    element
        .get_child("IoData")
        .and_then(|io| io.attributes.get("Visu"))
        .is_some_and(|value| value == "true")
}

/// Connector `K` → `U`, requiring both on every connector.
fn connectors(block: &Element) -> Result<HashMap<&str, &str>> {
    children(block, "Co")
        .map(|co| Ok((required(co, "K")?, required(co, "U")?)))
        .collect()
}

fn connector<'a>(connectors: &HashMap<&str, &'a str>, name: &str) -> Result<&'a str> {
    connectors
        .get(name)
        .copied()
        .with_context(|| format!("the native {name} state connector is missing"))
}

/// Native catalogues expose a control's lock state as its reset connector with the high bit set.
fn lock_state(reset: &str) -> Result<String> {
    let prefix = reset
        .get(..8)
        .context("unsupported native reset connector identity")?;
    let high = u32::from_str_radix(prefix, 16)
        .context("unsupported native reset connector identity")?
        | 0x8000_0000;
    Ok(format!("{high:08x}{}", &reset[8..]))
}

/// Encode a Loxone UUID (`8-4-4-16`) as stored in `permissions.bin`.
fn native_uuid(value: &str) -> Result<Token> {
    let parts: Vec<&str> = value.split('-').collect();
    ensure!(
        parts.len() == 4
            && [8, 4, 4, 16]
                .iter()
                .zip(&parts)
                .all(|(len, part)| part.len() == *len)
            && parts
                .iter()
                .all(|part| part.bytes().all(|byte| byte.is_ascii_hexdigit())),
        "unsupported native permission identity {value}"
    );
    let hex = parts.concat();
    let mut bytes = [0u8; 16];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16)?;
    }
    // The numeric UUID fields are little-endian; the trailing eight bytes are not.
    bytes[..4].reverse();
    bytes[4..6].reverse();
    bytes[6..8].reverse();
    Ok(bytes)
}

/// Convert a Loxone display unit such as `<v.1> %` to the catalogue format `%.1f %%`.
fn display_format(block: &Element) -> Result<String> {
    let unit = match block.get_child("Display") {
        Some(display) => required(display, "Unit")?,
        None => "<v.0>",
    };
    let unsupported = || anyhow!("unsupported display format {unit:?}");
    let start = unit.find('<').ok_or_else(unsupported)?;
    let (prefix, rest) = unit.split_at(start);
    let rest = rest.strip_prefix("<v").ok_or_else(unsupported)?;
    let (precision, rest) = match rest.strip_prefix('.') {
        Some(rest) => {
            let digit = rest.chars().next().filter(|c| ('0'..='6').contains(c));
            (digit.ok_or_else(unsupported)?, &rest[1..])
        }
        None => ('0', rest),
    };
    let suffix = rest.strip_prefix('>').ok_or_else(unsupported)?;
    ensure!(!suffix.contains('<'), unsupported());
    Ok(format!(
        "{}%.{precision}f{}",
        prefix.replace('%', "%%"),
        suffix.replace('%', "%%")
    ))
}

struct Project<'a> {
    /// Every `C` element in document order.
    blocks: Vec<&'a Element>,
    objects: HashMap<&'a str, &'a Element>,
    managers: BTreeSet<&'a str>,
}

impl<'a> Project<'a> {
    fn new(root: &'a Element) -> Self {
        let mut blocks = Vec::new();
        descendants(root, "C", &mut blocks);
        let objects = blocks
            .iter()
            .filter_map(|block| attribute(block, "U").map(|uuid| (uuid, *block)))
            .collect();
        let managers = blocks
            .iter()
            .filter(|block| attribute(block, "Type") == Some("EFM"))
            .filter_map(|block| attribute(block, "U"))
            .collect();
        Self {
            blocks,
            objects,
            managers,
        }
    }

    fn kind(&self, block: &Element) -> Option<Kind> {
        match attribute(block, "Type")? {
            "PushButton2Sel" if visible(block) => Some(Kind::Selector),
            "State" if visible(block) => Some(Kind::TextState),
            "PButtonT" if visible(block) => Some(Kind::Pushbutton),
            "Doorcontroller" if visible(block) => Some(Kind::Intercom),
            "MeterAbsUni"
                if attribute(block, "Rec").is_some_and(|rec| self.managers.contains(rec)) =>
            {
                Some(Kind::Meter)
            }
            "EFM" => Some(Kind::EnergyManager),
            _ => None,
        }
    }

    /// Edited controls of `kind`, in document order.
    fn edited(&self, kind: Kind, edits: &BTreeMap<&'a str, Edit>) -> Vec<(&'a Element, Edit)> {
        self.blocks
            .iter()
            .filter(|block| self.kind(block) == Some(kind))
            .filter_map(|block| Some((*block, *edits.get(attribute(block, "U")?)?)))
            .collect()
    }

    /// Whether a compiled catalogue entry belongs to a block of the same type and version.
    fn same_block_version(&self, entry: &Value, block: &Element) -> bool {
        entry
            .get("uuidAction")
            .and_then(Value::as_str)
            .and_then(|uuid| self.objects.get(uuid))
            .is_some_and(|source| {
                attribute(source, "Type") == attribute(block, "Type")
                    && source.attributes.get("V") == block.attributes.get("V")
            })
    }

    /// Room and category right groups that own a control's native permissions.
    fn right_groups(&self, block: &Element) -> Result<[Token; 2]> {
        let io = block
            .get_child("IoData")
            .context("a permission owner lacks IoData")?;
        ensure!(
            attribute(block, "RGR").is_none() && attribute(io, "RGR").is_none(),
            "explicit permission overrides (RGR) are unsupported"
        );
        let mut groups = [[0; 16]; 2];
        for (group, (key, kind)) in groups.iter_mut().zip([("Pr", "Place"), ("Cr", "Category")]) {
            let owner = self
                .objects
                .get(required(io, key)?)
                .filter(|owner| attribute(owner, "Type") == Some(kind))
                .with_context(|| format!("the control's {kind} is missing"))?;
            let right = required(owner, "RGR")?;
            ensure!(
                self.objects
                    .get(right)
                    .is_some_and(|object| attribute(object, "Type") == Some("RightGroup")),
                "the control's {kind} needs an existing RightGroup"
            );
            *group = native_uuid(right)?;
        }
        Ok(groups)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Edit {
    Added,
    Modified,
}

/// Classify every changed control, rejecting anything outside the supported edits.
fn classify<'a>(project: &Project<'a>, changes: &[Change<'a>]) -> Result<BTreeMap<&'a str, Edit>> {
    let mut edits = BTreeMap::new();
    for change in changes {
        let (parent, block) = change
            .after
            .with_context(|| format!("removing control {} is unsupported", change.key))?;
        let uuid = required(block, "U")?;
        let kind = project.kind(block);
        let edit = match change.before {
            None => {
                ensure!(
                    kind.is_some_and(Kind::can_add),
                    "adding {} control {uuid} is unsupported",
                    attribute(block, "Type").unwrap_or("untyped")
                );
                Edit::Added
            }
            Some((old_parent, old)) => {
                let kind = kind
                    .filter(|kind| project.kind(old) == Some(*kind))
                    .with_context(|| format!("changing control {uuid} is unsupported"))?;
                let (attributes, elements) = kind.reconciled();
                let strip = |element: &Element| {
                    let mut element = element.clone();
                    element
                        .attributes
                        .retain(|key, _| !attributes.contains(&key.as_str()));
                    element.children.retain(|child| {
                        child
                            .as_element()
                            .is_none_or(|child| !elements.contains(&child.name.as_str()))
                    });
                    shape(&element, true)
                };
                ensure!(
                    parent == old_parent && strip(old) == strip(block),
                    "changing control {uuid} beyond its {} is unsupported",
                    [attributes, elements].concat().join("/")
                );
                Edit::Modified
            }
        };
        edits.insert(uuid, edit);
    }
    Ok(edits)
}

pub(super) fn compile(
    after: &Element,
    changes: &[Change<'_>],
    app: &[u8],
    permissions: Option<&[u8]>,
) -> Result<super::Sidecars> {
    let project = Project::new(after);
    let edits = classify(&project, changes)?;

    let mut data: Value = serde_json::from_slice(app).context("invalid LoxAPP3.json")?;
    let original = data.clone();
    render_selectors(&mut data, &project, &edits)?;
    render_energy(&mut data, &project, &edits)?;
    render_status(&mut data, &project, &edits)?;
    render_intercoms(&mut data, &project, &edits)?;
    let app = if data == original {
        app.to_vec()
    } else {
        let document = project
            .blocks
            .iter()
            .find(|block| attribute(block, "Type") == Some("Document"))
            .context("the project Document is missing")?;
        let date = required(document, "Date")?;
        ensure!(
            data.get("lastModified").and_then(Value::as_str) != Some(date),
            "advance the project Document Date when changing visualized controls"
        );
        data["lastModified"] = json!(date);
        let mut bytes = serde_json::to_vec(&data)?;
        bytes.push(b'\n');
        bytes
    };

    let permissions = if edits.values().any(|edit| *edit == Edit::Added) {
        let source = permissions
            .filter(|data| !data.is_empty())
            .context("source permissions.bin is missing or empty")?;
        let mut records = parse_records(source)?;
        let controls = catalogue(&data)?;
        add_meter_permissions(&mut records, controls, &project, &edits)?;
        add_status_permissions(&mut records, controls, &project, &edits)?;
        add_selector_permissions(&mut records, controls, &project, &edits)?;
        Some(records.concat().concat())
    } else {
        None
    };
    Ok(super::Sidecars { app, permissions })
}

fn catalogue(data: &Value) -> Result<&Map<String, Value>> {
    data.get("controls")
        .and_then(Value::as_object)
        .context("LoxAPP3.json lacks its controls object")
}

fn catalogue_mut(data: &mut Value) -> Result<&mut Map<String, Value>> {
    data.get_mut("controls")
        .and_then(Value::as_object_mut)
        .context("LoxAPP3.json lacks its controls object")
}

fn require_location(data: &Value, io: &Element) -> Result<(String, String)> {
    let room = required(io, "Pr")?;
    let category = required(io, "Cr")?;
    ensure!(
        data.get("rooms")
            .and_then(|rooms| rooms.get(room))
            .is_some()
            && data
                .get("cats")
                .and_then(|cats| cats.get(category))
                .is_some(),
        "a new control needs an existing app room and category"
    );
    Ok((room.to_string(), category.to_string()))
}

fn category_icon(data: &Value, category: &str) -> Result<Value> {
    data["cats"][category]
        .get("image")
        .cloned()
        .context("the app category lacks its image")
}

/// Rename an existing native entry without touching its other metadata.
fn rename(data: &mut Value, block: &Element, kind: Kind) -> Result<()> {
    let uuid = required(block, "U")?;
    let entry = catalogue_mut(data)?
        .get_mut(&format!("U:{uuid}"))
        .and_then(Value::as_object_mut)
        .filter(|entry| entry.get("type").and_then(Value::as_str) == Some(kind.app_type()))
        .with_context(|| format!("an existing native {} entry is required", kind.app_type()))?;
    entry.insert("name".into(), json!(required(block, "Title")?));
    Ok(())
}

fn new_key(data: &Value, uuid: &str) -> Result<String> {
    let key = format!("U:{uuid}");
    ensure!(
        !catalogue(data)?.contains_key(&key),
        "LoxAPP3.json already has a stale entry for new control {uuid}"
    );
    Ok(key)
}

fn render_selectors(
    data: &mut Value,
    project: &Project,
    edits: &BTreeMap<&str, Edit>,
) -> Result<()> {
    for (block, edit) in project.edited(Kind::Selector, edits) {
        if edit == Edit::Modified {
            rename(data, block, Kind::Selector)?;
            continue;
        }
        let uuid = required(block, "U")?;
        let key = new_key(data, uuid)?;
        let template = catalogue(data)?
            .values()
            .find(|entry| {
                entry.get("type").and_then(Value::as_str) == Some("ValueSelector")
                    && project.same_block_version(entry, block)
            })
            .and_then(Value::as_object)
            .cloned()
            .context(
                "a native ValueSelector catalogue template of the same block version is required",
            )?;
        let io = block.get_child("IoData").context("selector lacks IoData")?;
        let (room, category) = require_location(data, io)?;
        let connectors = connectors(block)?;
        let mut details = match template.get("details") {
            None => Map::new(),
            Some(Value::Object(details)) => details.clone(),
            Some(_) => bail!("unsupported native ValueSelector details"),
        };
        details.insert("increaseOnly".into(), json!(false));
        details.insert("format".into(), json!(display_format(block)?));
        let mut entry = template;
        entry.insert("name".into(), json!(required(block, "Title")?));
        entry.insert("uuidAction".into(), json!(uuid));
        entry.insert("room".into(), json!(room));
        entry.insert("cat".into(), json!(category));
        // Explicit visualization belongs in room lists, not only in linked views.
        entry.insert("restrictions".into(), json!(0));
        entry.insert("details".into(), Value::Object(details));
        entry.insert(
            "states".into(),
            json!({
                "jLocked": lock_state(connector(&connectors, "Reset")?)?,
                "min": connector(&connectors, "Min")?,
                "max": connector(&connectors, "Max")?,
                "step": connector(&connectors, "Step")?,
                "value": connector(&connectors, "AQ")?,
            }),
        );
        entry.insert("defaultIcon".into(), category_icon(data, &category)?);
        entry.insert("defaultRating".into(), json!(0));
        entry.insert("isFavorite".into(), json!(false));
        entry.insert("isSecured".into(), json!(false));
        entry.shift_remove("links");
        catalogue_mut(data)?.insert(key, Value::Object(entry));
    }
    Ok(())
}

fn meter_states(block: &Element) -> Result<Map<String, Value>> {
    let connectors = connectors(block)?;
    let mut states = Map::new();
    for (state, port) in METER_STATES {
        let uuid = connector(&connectors, port)?;
        let value = if state == "jLocked" {
            lock_state(uuid)?
        } else {
            uuid.to_string()
        };
        states.insert(state.into(), json!(value));
    }
    Ok(states)
}

fn statistics(block: &Element) -> Vec<BTreeMap<&str, &str>> {
    children(block, "statGrp")
        .map(|group| {
            group
                .attributes
                .iter()
                .map(|(key, value)| (key.as_str(), value.as_str()))
                .collect()
        })
        .collect()
}

fn render_energy(data: &mut Value, project: &Project, edits: &BTreeMap<&str, Edit>) -> Result<()> {
    let before = catalogue(data)?.clone();
    let meters: Vec<&Element> = project
        .blocks
        .iter()
        .copied()
        .filter(|block| project.kind(block) == Some(Kind::Meter))
        .collect();
    for (block, edit) in project.edited(Kind::Meter, edits) {
        if edit == Edit::Modified {
            rename(data, block, Kind::Meter)?;
            continue;
        }
        let uuid = required(block, "U")?;
        let key = new_key(data, uuid)?;
        let states = meter_states(block)?;
        // Copy a compiled meter with the same units and statistics; never invent its schema.
        let mut template = None;
        for source in &meters {
            let Some(entry) =
                attribute(source, "U").and_then(|uuid| before.get(&format!("U:{uuid}")))
            else {
                continue;
            };
            if entry.get("type").and_then(Value::as_str) == Some("Meter")
                && entry["details"].get("type").and_then(Value::as_str) == Some("unidirectional")
                && entry.get("restrictions") == Some(&json!(17))
                && entry.get("states") == Some(&Value::Object(meter_states(source)?))
                && ["PwrN", "Un1", "Un2", "StatsType"]
                    .iter()
                    .all(|key| source.attributes.get(*key) == block.attributes.get(*key))
                && statistics(source) == statistics(block)
                && source.attributes.get("V") == block.attributes.get("V")
            {
                template = entry.as_object().cloned();
                break;
            }
        }
        let mut entry =
            template.context("a matching compiled energy meter template is required")?;
        let io = block
            .get_child("IoData")
            .context("energy meter lacks IoData")?;
        ensure!(
            !visible(block) && attribute(block, "RGR").is_none() && attribute(io, "RGR").is_none(),
            "only linked energy meters without permission overrides are supported"
        );
        let (room, category) = require_location(data, io)?;
        let icon = match attribute(io, "IconV") {
            None => category_icon(data, &category)?,
            Some(icon) => before
                .iter()
                .find(|(key, _)| {
                    project
                        .objects
                        .get(key.strip_prefix("U:").unwrap_or(key))
                        .and_then(|source| source.get_child("IoData"))
                        .is_some_and(|io| attribute(io, "IconV") == Some(icon))
                })
                .map(|(_, entry)| entry.get("defaultIcon").cloned().unwrap_or(Value::Null))
                .context("a compiled energy meter icon is required")?,
        };
        entry.insert("name".into(), json!(required(block, "Title")?));
        entry.insert("uuidAction".into(), json!(uuid));
        entry.insert("room".into(), json!(room));
        entry.insert("cat".into(), json!(category));
        entry.insert("states".into(), Value::Object(states));
        entry.insert("defaultIcon".into(), icon);
        entry.insert("isFavorite".into(), json!(false));
        entry.insert("isSecured".into(), json!(false));
        entry.insert("defaultRating".into(), json!(0));
        entry.shift_remove("links");
        if let Some(links) = attribute(block, "linkC") {
            entry.insert("links".into(), json!(links.split(',').collect::<Vec<_>>()));
        }
        catalogue_mut(data)?.insert(key, Value::Object(entry));
    }
    for (manager, _) in project.edited(Kind::EnergyManager, edits) {
        render_energy_nodes(data, manager)?;
    }
    Ok(())
}

struct EnergyNodes<'a> {
    controls: &'a Map<String, Value>,
    nodes: HashMap<String, Value>,
    declared: BTreeSet<&'a str>,
}

impl EnergyNodes<'_> {
    fn render(&self, node: &Element) -> Result<Value> {
        let uuid = required(node, "u")?;
        let old = self.nodes.get(uuid);
        let mut result = match old {
            None => {
                let meter = self
                    .controls
                    .get(&format!("U:{}", required(node, "ctrlUuid")?))
                    .filter(|meter| meter.get("type").and_then(Value::as_str) == Some("Meter"))
                    .filter(|_| {
                        matches!(attribute(node, "t"), Some("1" | "5"))
                            && attribute(node, "rst") != Some("true")
                    })
                    .context("only new load nodes without residual meters are supported")?;
                json!({
                    "uuid": uuid,
                    "title": required(node, "tit")?,
                    "icon": meter.get("defaultIcon").cloned().context("the node's meter lacks its icon")?,
                    "ctrlUuid": attribute(node, "ctrlUuid"),
                    "nodeType": "Load",
                })
            }
            Some(old) => {
                ensure!(
                    attribute(node, "ctrlUuid") == old.get("ctrlUuid").and_then(Value::as_str),
                    "changing an existing energy node's meter is unsupported"
                );
                let mut result = old.clone();
                if let Some(title) = attribute(node, "tit") {
                    result["title"] = json!(title);
                }
                result
            }
        };
        let mut rendered = children(node, "nd")
            .map(|child| self.render(child))
            .collect::<Result<Vec<_>>>()?;
        // Native Config generates residual nodes outside the XML tree. Preserve them,
        // but never silently preserve a removed real node.
        for child in old
            .and_then(|old| old.get("nodes"))
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let child_uuid = child
                .get("uuid")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if !self.declared.contains(child_uuid) {
                ensure!(
                    attribute(node, "rst") == Some("true")
                        && child.get("ctrlUuid").and_then(Value::as_str) == Some(uuid),
                    "removing compiled energy nodes is unsupported"
                );
                rendered.push(child.clone());
            }
        }
        let object = result
            .as_object_mut()
            .context("invalid compiled energy node")?;
        if rendered.is_empty() {
            object.shift_remove("nodes");
        } else {
            object.insert("nodes".into(), Value::Array(rendered));
        }
        Ok(result)
    }
}

fn index_nodes(items: &[Value], nodes: &mut HashMap<String, Value>) -> Result<()> {
    for node in items {
        let uuid = node
            .get("uuid")
            .and_then(Value::as_str)
            .context("compiled energy node lacks its uuid")?;
        ensure!(
            nodes.insert(uuid.to_string(), node.clone()).is_none(),
            "duplicate compiled energy node {uuid}"
        );
        if let Some(children) = node.get("nodes").and_then(Value::as_array) {
            index_nodes(children, nodes)?;
        }
    }
    Ok(())
}

fn render_energy_nodes(data: &mut Value, manager: &Element) -> Result<()> {
    let key = format!("U:{}", required(manager, "U")?);
    let tree = manager
        .get_child("root")
        .context("a compiled energy manager with its node tree is required")?;
    let controls = catalogue(data)?.clone();
    let original = controls
        .get(&key)
        .filter(|entry| entry.get("type").and_then(Value::as_str) == Some("EFM"))
        .and_then(|entry| entry.get("details")?.get("nodes")?.as_array())
        .context("a compiled energy manager with its node tree is required")?;
    let mut nodes = HashMap::new();
    index_nodes(original, &mut nodes)?;
    let mut declared = Vec::new();
    descendants(tree, "nd", &mut declared);
    let renderer = EnergyNodes {
        controls: &controls,
        nodes,
        declared: declared
            .iter()
            .filter_map(|node| attribute(node, "u"))
            .collect(),
    };
    let rendered = children(tree, "nd")
        .map(|node| renderer.render(node))
        .collect::<Result<Vec<_>>>()?;
    catalogue_mut(data)?[&key]["details"]["nodes"] = Value::Array(rendered);
    Ok(())
}

fn status_states(block: &Element, kind: Kind) -> Result<Map<String, Value>> {
    let connectors = connectors(block)?;
    let mut states = Map::new();
    if kind == Kind::TextState {
        states.insert("textAndIcon".into(), json!(required(block, "U")?));
        states.insert("iconAndColor".into(), json!(connector(&connectors, "I1")?));
    } else {
        states.insert(
            "jLocked".into(),
            json!(lock_state(connector(&connectors, "Reset")?)?),
        );
        states.insert("active".into(), json!(connector(&connectors, "Q")?));
    }
    Ok(states)
}

fn render_status(data: &mut Value, project: &Project, edits: &BTreeMap<&str, Edit>) -> Result<()> {
    let mut blocks = [Kind::TextState, Kind::Pushbutton]
        .into_iter()
        .flat_map(|kind| {
            project
                .edited(kind, edits)
                .into_iter()
                .map(move |(block, edit)| (block, edit, kind))
        })
        .collect::<Vec<_>>();
    let position = |block: &Element| {
        project
            .blocks
            .iter()
            .position(|candidate| std::ptr::eq(*candidate, block))
    };
    blocks.sort_by_key(|(block, _, _)| position(block));
    for (block, edit, kind) in blocks {
        if edit == Edit::Modified {
            rename(data, block, kind)?;
            continue;
        }
        let uuid = required(block, "U")?;
        let key = new_key(data, uuid)?;
        ensure!(
            catalogue(data)?
                .values()
                .any(
                    |entry| entry.get("type").and_then(Value::as_str) == Some(kind.app_type())
                        && project.same_block_version(entry, block)
                ),
            "a native {} catalogue template of the same block version is required",
            kind.app_type()
        );
        let io = block
            .get_child("IoData")
            .context("status control lacks IoData")?;
        ensure!(
            attribute(block, "RGR").is_none() && attribute(io, "RGR").is_none(),
            "explicit status control permission overrides are unsupported"
        );
        let (room, category) = require_location(data, io)?;
        let mut entry = Map::new();
        entry.insert("name".into(), json!(required(block, "Title")?));
        entry.insert("type".into(), json!(kind.app_type()));
        entry.insert("uuidAction".into(), json!(uuid));
        entry.insert("room".into(), json!(room));
        entry.insert("cat".into(), json!(category));
        entry.insert("defaultRating".into(), json!(0));
        entry.insert("isFavorite".into(), json!(false));
        entry.insert("isSecured".into(), json!(false));
        entry.insert("defaultIcon".into(), category_icon(data, &category)?);
        entry.insert("restrictions".into(), json!(0));
        entry.insert("states".into(), Value::Object(status_states(block, kind)?));
        if kind == Kind::Pushbutton {
            entry.insert("details".into(), json!({ "jLockable": true }));
        }
        if let Some(links) = attribute(block, "linkC") {
            entry.insert("links".into(), json!(links.split(',').collect::<Vec<_>>()));
        }
        catalogue_mut(data)?.insert(key, Value::Object(entry));
    }
    Ok(())
}

/// Refresh an existing intercom's action labels and linked controls without changing identities.
fn render_intercoms(
    data: &mut Value,
    project: &Project,
    edits: &BTreeMap<&str, Edit>,
) -> Result<()> {
    for (block, _) in project.edited(Kind::Intercom, edits) {
        let uuid = required(block, "U")?;
        let entry = catalogue_mut(data)?
            .get_mut(&format!("U:{uuid}"))
            .filter(|entry| entry.get("type").and_then(Value::as_str) == Some("Intercom"))
            .context("an existing native Intercom catalogue entry is required")?;
        let outputs = entry["details"]["outputs"]
            .as_array_mut()
            .context("unsupported native Intercom action layout")?;
        let ids: BTreeSet<_> = outputs
            .iter()
            .filter_map(|output| output.get("id")?.as_u64())
            .collect();
        ensure!(
            outputs.len() == 3 && ids == BTreeSet::from([0, 1, 2]),
            "unsupported native Intercom action layout"
        );
        let mut labels = Vec::new();
        for output in outputs.iter_mut() {
            let index = output["id"].as_u64().unwrap_or_default() + 1;
            let label = required(block, &format!("OutName{index}"))?;
            output["name"] = json!(label);
            labels.push((index, label));
        }
        for (index, label) in labels {
            entry
                .get_mut("subControls")
                .and_then(|controls| controls.get_mut(format!("{uuid}/{index}")))
                .and_then(Value::as_object_mut)
                .context("a native Intercom action subcontrol is missing")?
                .insert("name".into(), json!(label));
        }
        let entry = entry.as_object_mut().context("invalid Intercom entry")?;
        match attribute(block, "linkC") {
            Some(links) => {
                entry.insert("links".into(), json!(links.split(',').collect::<Vec<_>>()));
            }
            None => {
                entry.shift_remove("links");
            }
        }
    }
    Ok(())
}

type Record = Vec<Token>;

/// Split `permissions.bin` into records of 16-byte tokens, each ending with `ff×16`.
fn parse_records(data: &[u8]) -> Result<Vec<Record>> {
    ensure!(
        data.len().is_multiple_of(16),
        "unsupported native permission token alignment"
    );
    let mut records = Vec::new();
    let mut record = Vec::new();
    for chunk in data.chunks_exact(16) {
        let token: Token = chunk.try_into()?;
        record.push(token);
        if token == RECORD_END {
            records.push(std::mem::take(&mut record));
        }
    }
    ensure!(
        record.is_empty() && !records.is_empty(),
        "unterminated or empty native permission records"
    );
    Ok(records)
}

fn require_new_record(records: &[Record], action: &Token, uuid: &str) -> Result<()> {
    ensure!(
        !records.iter().any(|record| record[0] == *action),
        "permissions.bin already has a stale record for new control {uuid}"
    );
    Ok(())
}

fn state_tokens(entry: &Value, keys: &[&str]) -> Result<Vec<Token>> {
    keys.iter()
        .map(|key| {
            native_uuid(
                entry["states"][*key]
                    .as_str()
                    .with_context(|| format!("catalogue state {key} is missing"))?,
            )
        })
        .collect()
}

fn entry<'a>(controls: &'a Map<String, Value>, block: &Element) -> Result<&'a Value> {
    controls
        .get(&format!("U:{}", required(block, "U")?))
        .context("the new control's catalogue entry is missing")
}

fn add_meter_permissions(
    records: &mut Vec<Record>,
    controls: &Map<String, Value>,
    project: &Project,
    edits: &BTreeMap<&str, Edit>,
) -> Result<()> {
    let keys: Vec<&str> = METER_STATES.iter().map(|(state, _)| *state).collect();
    let meters: Vec<&Element> = project
        .blocks
        .iter()
        .copied()
        .filter(|block| project.kind(block) == Some(Kind::Meter))
        .collect();
    let mut additions = Vec::new();
    for (block, edit) in project.edited(Kind::Meter, edits) {
        if edit != Edit::Added {
            continue;
        }
        let uuid = required(block, "U")?;
        let action = native_uuid(uuid)?;
        require_new_record(records, &action, uuid)?;
        // The FB section grants access through the linked energy manager,
        // independently of the meter's own room and category.
        let mut template = false;
        for source in &meters {
            let mut prefix = vec![native_uuid(required(source, "U")?)?, ACTION_END];
            for value in meter_states(source)?.values() {
                prefix.push(native_uuid(value.as_str().unwrap_or_default())?);
            }
            prefix.push(STATES_END);
            if records.iter().any(|record| {
                record.len() == 16 && record[..10] == prefix[..] && record[12] == OWNER_END
            }) {
                template = true;
                break;
            }
        }
        ensure!(
            template,
            "a matching compiled meter permission template is required"
        );
        let entry = entry(controls, block)?;
        ensure!(
            entry.get("states") == Some(&Value::Object(meter_states(block)?)),
            "energy meter state identities differ from XML"
        );
        let manager = project
            .objects
            .get(required(block, "Rec")?)
            .filter(|manager| attribute(manager, "Type") == Some("EFM"))
            .context("the energy meter's manager is missing")?;
        let mut record = vec![action, ACTION_END];
        record.extend(state_tokens(entry, &keys)?);
        record.push(STATES_END);
        record.extend(project.right_groups(block)?);
        record.push(OWNER_END);
        record.extend(project.right_groups(manager)?);
        record.push(RECORD_END);
        additions.push(record);
    }
    records.extend(additions);
    Ok(())
}

fn add_status_permissions(
    records: &mut Vec<Record>,
    controls: &Map<String, Value>,
    project: &Project,
    edits: &BTreeMap<&str, Edit>,
) -> Result<()> {
    let blocks: Vec<(&Element, Kind)> = project
        .blocks
        .iter()
        .copied()
        .filter_map(|block| {
            let kind = project.kind(block)?;
            let uuid = attribute(block, "U")?;
            (matches!(kind, Kind::TextState | Kind::Pushbutton)
                && edits.get(uuid) == Some(&Edit::Added))
            .then_some((block, kind))
        })
        .collect();
    for (block, kind) in blocks {
        let uuid = required(block, "U")?;
        let action = native_uuid(uuid)?;
        require_new_record(records, &action, uuid)?;
        let keys: &[&str] = if kind == Kind::TextState {
            &["textAndIcon", "iconAndColor"]
        } else {
            &["jLocked", "active"]
        };
        // Match a native compiled record, including state ordering and group layout.
        let mut template = false;
        for candidate in controls
            .values()
            .filter(|entry| entry.get("type").and_then(Value::as_str) == Some(kind.app_type()))
        {
            let Some(candidate_action) = candidate.get("uuidAction").and_then(Value::as_str) else {
                continue;
            };
            let candidate_action = native_uuid(candidate_action)?;
            let states = state_tokens(candidate, keys)?;
            if records.iter().any(|record| {
                record.len() == 8
                    && record[0] == candidate_action
                    && record[1] == ACTION_END
                    && record[4] == STATES_END
                    && record[2..4] == states[..]
            }) {
                template = true;
                break;
            }
        }
        ensure!(
            template,
            "a matching native {} permission template is required",
            kind.app_type()
        );
        let mut record = vec![action, ACTION_END];
        record.extend(state_tokens(entry(controls, block)?, keys)?);
        record.push(STATES_END);
        record.extend(project.right_groups(block)?);
        record.push(RECORD_END);
        records.push(record);
    }
    Ok(())
}

fn add_selector_permissions(
    records: &mut Vec<Record>,
    controls: &Map<String, Value>,
    project: &Project,
    edits: &BTreeMap<&str, Edit>,
) -> Result<()> {
    const KEYS: [&str; 5] = ["jLocked", "min", "max", "step", "value"];
    let added: Vec<&Element> = project
        .edited(Kind::Selector, edits)
        .into_iter()
        .filter(|(_, edit)| *edit == Edit::Added)
        .map(|(block, _)| block)
        .collect();
    if added.is_empty() {
        return Ok(());
    }
    // Confirm the native token layout against an existing compiled selector.
    let mut template = false;
    for candidate in controls
        .values()
        .filter(|entry| entry.get("type").and_then(Value::as_str) == Some("ValueSelector"))
    {
        let Some(action) = candidate.get("uuidAction").and_then(Value::as_str) else {
            continue;
        };
        let action = native_uuid(action)?;
        let states = state_tokens(candidate, &KEYS)?;
        if records.iter().any(|record| {
            record.len() == 11
                && record[0] == action
                && record[1] == ACTION_END
                && record[2..7] == states[..]
                && record[7] == STATES_END
        }) {
            template = true;
            break;
        }
    }
    ensure!(
        template,
        "a matching native ValueSelector permission template is required"
    );
    for block in added {
        let uuid = required(block, "U")?;
        let action = native_uuid(uuid)?;
        require_new_record(records, &action, uuid)?;
        let entry = entry(controls, block)?;
        ensure!(
            entry.get("restrictions") == Some(&json!(0)),
            "an explicitly visualized selector must be visible in the app catalogue"
        );
        let mut record = vec![action, ACTION_END];
        record.extend(state_tokens(entry, &KEYS)?);
        record.push(STATES_END);
        record.extend(project.right_groups(block)?);
        record.push(RECORD_END);
        records.push(record);
    }
    Ok(())
}

#[cfg(test)]
mod tests;
