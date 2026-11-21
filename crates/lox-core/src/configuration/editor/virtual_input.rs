use super::{SelectionError, legacy::State};
use xmltree::{Element, XMLNode};

/// Legacy creation options. Bounds apply to analog inputs only; absent bounds
/// are omitted. An absent unit defaults to `<v>`.
#[derive(Clone, Default)]
pub struct VirtualInOpts {
    pub analog: bool,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub unit: Option<String>,
}

pub struct VirtualInIds {
    pub object: String,
    pub q: String,
    pub qm: String,
}

pub struct InputRefIds {
    pub object: String,
    pub ai: String,
    pub i: String,
    pub aq: String,
    pub q: String,
}

pub enum LegacyVirtualInputCreation {
    WithInputRef {
        virtual_in_uuid: String,
        input_ref_aq_uuid: String,
    },
    WithoutPage {
        virtual_in_uuid: String,
        unplaced_aq_uuid: String,
    },
}

struct VirtualInputPlan {
    parent_path: Vec<usize>,
    virtual_in: Element,
    input_ref: Element,
    virtual_in_uuid: String,
    input_ref_aq_uuid: String,
}

impl VirtualInputPlan {
    fn apply(self, state: &mut State) -> LegacyVirtualInputCreation {
        state
            .get_element_mut(&self.parent_path)
            .children
            .push(XMLNode::Element(self.virtual_in));
        // The converter's AQ is a wire source only when the converter is on a Page.
        if let Some(page) = state.find_elements("Type:Page").into_iter().next() {
            state
                .get_element_mut(&page)
                .children
                .push(XMLNode::Element(self.input_ref));
            LegacyVirtualInputCreation::WithInputRef {
                virtual_in_uuid: self.virtual_in_uuid,
                input_ref_aq_uuid: self.input_ref_aq_uuid,
            }
        } else {
            LegacyVirtualInputCreation::WithoutPage {
                virtual_in_uuid: self.virtual_in_uuid,
                unplaced_aq_uuid: self.input_ref_aq_uuid,
            }
        }
    }
}

impl State {
    /// Create a VirtualIn and its external converter on the first shared-selector Page.
    ///
    /// This unstable adapter preserves parent selection before ID allocation and
    /// requests the two fixed identity bundles in legacy order. UUID text is used
    /// verbatim. With no Page, only the VirtualIn is appended and the returned AQ
    /// identity is not a wireable output. A caller may still edit this same state
    /// before the normal explicit-clock stamp and serialization.
    pub fn add_virtual_input_legacy(
        &mut self,
        title: &str,
        opts: &VirtualInOpts,
        parent_selector: &str,
        virtual_in_ids: impl FnOnce(&str) -> VirtualInIds,
        input_ref_ids: impl FnOnce(&str) -> InputRefIds,
    ) -> Result<LegacyVirtualInputCreation, SelectionError> {
        let parent_path = self.require_one(parent_selector)?;
        let suffix = self.find_miniserver_serial().unwrap_or_default();
        let vi = virtual_in_ids(&suffix);
        let virtual_in = self.build_virtual_input(&parent_path, title, opts, &vi);
        let converter = input_ref_ids(&suffix);
        let input_ref = build_input_ref(title, opts.analog, &vi, &converter);

        Ok(VirtualInputPlan {
            parent_path,
            virtual_in,
            input_ref,
            virtual_in_uuid: vi.object,
            input_ref_aq_uuid: converter.aq,
        }
        .apply(self))
    }

    fn build_virtual_input(
        &self,
        parent_path: &[usize],
        title: &str,
        opts: &VirtualInOpts,
        ids: &VirtualInIds,
    ) -> Element {
        let iname = self.next_iname("VI");
        let analog = opts.analog.to_string();
        let mut elem = element(
            "C",
            [
                ("Type", "VirtualIn"),
                ("IName", &iname),
                ("V", "175"),
                ("U", &ids.object),
                ("Title", title),
                ("Cl", "238,238,238"),
                ("Analog", &analog),
                ("Nio", "2"),
                ("WF", "16400"),
                ("Idx", "-1"),
                ("ValOT", "1"),
            ],
        );
        if opts.analog {
            for (key, value) in [
                ("EnVal", "true"),
                ("MinChange", "0.25"),
                ("MinTime", "1000"),
            ] {
                elem.attributes.insert(key.into(), value.into());
            }
            for (key, bound) in [("MinVal", opts.min), ("MaxVal", opts.max)] {
                if let Some(value) = bound {
                    elem.attributes.insert(key.into(), fmt_num(value));
                }
            }
        }
        elem.children
            .push(XMLNode::Element(connector("Q", &ids.q, None)));
        elem.children
            .push(XMLNode::Element(connector("Qm", &ids.qm, None)));

        let mut io_data = element("IoData", [("St", "2"), ("Visu", "true")]);
        if let Some(parent_io) = self
            .get_element(parent_path)
            .children
            .iter()
            .find_map(|child| child.as_element().filter(|elem| elem.name == "IoData"))
        {
            for reference in ["Pr", "Cr"] {
                if let Some(value) = parent_io
                    .attributes
                    .get(reference)
                    .filter(|value| !value.is_empty())
                {
                    io_data.attributes.insert(reference.into(), value.clone());
                }
            }
        }
        for (reference, kind) in [("Pr", "Place"), ("Cr", "Category")] {
            if !io_data.attributes.contains_key(reference)
                && let Some(uuid) = find_first_uuid(&self.root, kind)
            {
                io_data.attributes.insert(reference.into(), uuid.to_owned());
            }
        }
        elem.children.push(XMLNode::Element(io_data));
        elem.children.push(XMLNode::Element(element(
            "Display",
            [
                ("Unit", opts.unit.as_deref().unwrap_or("<v>")),
                ("StateOnly", "true"),
            ],
        )));
        elem
    }

    /// First legacy identity suffix, not necessarily a hardware serial.
    pub fn find_miniserver_serial(&self) -> Option<String> {
        fn scan(elem: &Element) -> Option<String> {
            if let Some(u) = elem.attributes.get("U")
                && let Some((_, serial)) = u.rsplit_once("-ffff")
                && serial.len() >= 12
                && serial.chars().all(|c| c.is_ascii_hexdigit())
            {
                return Some(serial[..12].to_string());
            }
            for child in &elem.children {
                if let XMLNode::Element(c) = child
                    && let Some(s) = scan(c)
                {
                    return Some(s);
                }
            }
            None
        }
        scan(&self.root)
    }

    /// Allocate the index after the highest existing index, not the first gap.
    pub fn next_iname(&self, prefix: &str) -> String {
        let mut max_idx: i32 = -1;
        fn scan(elem: &Element, prefix: &str, max: &mut i32) {
            if let Some(iname) = elem.attributes.get("IName")
                && iname.starts_with(prefix)
                && let Ok(idx) = iname[prefix.len()..].parse::<i32>()
                && idx > *max
            {
                *max = idx;
            }
            for child in &elem.children {
                if let XMLNode::Element(c) = child {
                    scan(c, prefix, max);
                }
            }
        }
        scan(&self.root, prefix, &mut max_idx);
        format!("{}{}", prefix, max_idx + 1)
    }
}

fn build_input_ref(title: &str, analog: bool, vi: &VirtualInIds, ids: &InputRefIds) -> Element {
    let analog = analog.to_string();
    let mut elem = element(
        "C",
        [
            ("Type", "InputRef"),
            ("V", "175"),
            ("U", &ids.object),
            ("Title", title),
            ("Ref", &vi.object),
            ("LinkRefType", "71"),
            ("Analog", &analog),
            ("Nio", "4"),
            ("WF", "18432"),
        ],
    );
    for (key, id, input) in [
        ("AI", ids.ai.as_str(), Some(vi.q.as_str())),
        ("I", ids.i.as_str(), Some(vi.qm.as_str())),
        ("AQ", ids.aq.as_str(), None),
        ("Q", ids.q.as_str(), None),
    ] {
        elem.children
            .push(XMLNode::Element(connector(key, id, input)));
    }
    elem
}

fn find_first_uuid<'a>(elem: &'a Element, kind: &str) -> Option<&'a str> {
    if elem.name == "C" && elem.attributes.get("Type").map(String::as_str) == Some(kind) {
        return elem.attributes.get("U").map(String::as_str);
    }
    elem.children
        .iter()
        .filter_map(XMLNode::as_element)
        .find_map(|child| find_first_uuid(child, kind))
}

fn connector(key: &str, id: &str, input: Option<&str>) -> Element {
    let mut co = element("Co", [("K", key), ("U", id)]);
    if let Some(input) = input {
        co.children
            .push(XMLNode::Element(element("In", [("Input", input)])));
    }
    co
}

fn element<const N: usize>(name: &str, attributes: [(&str, &str); N]) -> Element {
    let mut elem = Element::new(name);
    for (key, value) in attributes {
        elem.attributes.insert(key.into(), value.into());
    }
    elem
}

fn fmt_num(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    fn by_uuid<'a>(state: &'a State, uuid: &str) -> &'a Element {
        state.get_element(&state.require_one(&format!("uuid:{uuid}")).unwrap())
    }

    const SUFFIX: &str = "AbCd01234567";

    fn identity(index: u8) -> String {
        format!("{index:08x}-0000-0000-ffff{SUFFIX}")
    }

    #[test]
    fn page_present_plan_preserves_distinct_identity_roles_and_explicit_stamp() {
        let mut state = State::load(
            br#"<ControlList NextObj="29" NextMem="1">
  <C Type="Document" U="0000002a-ffff-0000-ffffAbCd01234567" DateS="1">
    <C Type="Place" U="room-first"/>
    <C Type="Category" U="category-first"/>
    <C Type="VirtualInCaption" U="caption">
      <IoData Pr="room-parent" Cr=""/>
      <C Type="VirtualIn" U="old" IName="VI7"/>
    </C>
    <C Type="VirtualInCaption" U="caption"/>
    <C Type="Program"><C Type="pAgE" U="first-page"/></C>
    <C Type="Page" U="last-page"/>
  </C>
</ControlList>"#,
        )
        .unwrap();
        let phase = Cell::new(0);
        let result = state
            .add_virtual_input_legacy(
                "A & B",
                &VirtualInOpts {
                    analog: true,
                    min: Some(-1.5),
                    ..Default::default()
                },
                "Type:VirtualInCaption",
                |suffix| {
                    assert_eq!(suffix, SUFFIX);
                    assert_eq!(phase.replace(1), 0);
                    VirtualInIds {
                        object: identity(1),
                        q: identity(2),
                        qm: identity(3),
                    }
                },
                |suffix| {
                    assert_eq!(suffix, SUFFIX);
                    assert_eq!(phase.replace(2), 1);
                    InputRefIds {
                        object: identity(4),
                        ai: identity(5),
                        i: identity(6),
                        aq: identity(7),
                        q: identity(8),
                    }
                },
            )
            .unwrap();
        let LegacyVirtualInputCreation::WithInputRef {
            virtual_in_uuid,
            input_ref_aq_uuid,
        } = result
        else {
            panic!("Page-present creation did not place the InputRef");
        };
        assert_eq!(virtual_in_uuid, identity(1));
        assert_eq!(input_ref_aq_uuid, identity(7));

        assert_eq!(
            state
                .get_element(&state.require_one("Type:Document").unwrap())
                .attributes["DateS"],
            "1"
        );
        let instant = chrono::DateTime::parse_from_rfc3339("2009-01-01T01:00:00+01:00").unwrap();
        assert!(state.stamp_document(&instant));
        let output = state.to_bytes().unwrap();
        let document = std::str::from_utf8(&output).unwrap();
        assert!(document.contains(r#"NextObj="29" NextMem="1""#));
        assert!(document.contains(r#"DateS="2" Date="2009-01-01 01:00:02""#));

        let state = State::load(&output).unwrap();
        let vi = by_uuid(&state, &virtual_in_uuid);
        assert_eq!(vi.attributes["IName"], "VI8");
        assert_eq!(vi.attributes["Title"], "A & B");
        assert_eq!(vi.attributes["MinVal"], "-1.5");
        assert!(!vi.attributes.contains_key("MaxVal"));
        let io = vi.get_child("IoData").unwrap();
        assert_eq!(io.attributes["Pr"], "room-parent");
        assert_eq!(io.attributes["Cr"], "category-first");
        let display = vi.get_child("Display").unwrap();
        assert_eq!(display.attributes["Unit"], "<v>");
        assert_eq!(
            by_uuid(&state, "caption")
                .children
                .last()
                .and_then(XMLNode::as_element)
                .unwrap()
                .attributes["U"],
            virtual_in_uuid
        );
        let reference = by_uuid(&state, "first-page")
            .get_child("C")
            .expect("InputRef belongs on the first shared-selector Page");
        assert!(by_uuid(&state, "last-page").get_child("C").is_none());
        assert_eq!(reference.attributes["U"], identity(4));
        assert_eq!(reference.attributes["Ref"], virtual_in_uuid);
        let connectors = |parent: &Element| {
            parent
                .children
                .iter()
                .filter_map(XMLNode::as_element)
                .filter(|child| child.name == "Co")
                .map(|child| {
                    (
                        child.attributes["K"].clone(),
                        child.attributes["U"].clone(),
                        child
                            .get_child("In")
                            .map(|input| input.attributes["Input"].clone()),
                    )
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(
            connectors(vi),
            vec![
                ("Q".to_owned(), identity(2), None),
                ("Qm".to_owned(), identity(3), None)
            ]
        );
        assert_eq!(
            connectors(reference),
            vec![
                ("AI".to_owned(), identity(5), Some(identity(2))),
                ("I".to_owned(), identity(6), Some(identity(3))),
                ("AQ".to_owned(), input_ref_aq_uuid, None),
                ("Q".to_owned(), identity(8), None),
            ]
        );
    }

    #[test]
    fn absence_of_page_is_only_a_legacy_unplaced_aq_outcome() {
        let mut state =
            State::load(br#"<ControlList><C Type="VirtualInCaption" U="caption"/></ControlList>"#)
                .unwrap();
        let result = state
            .add_virtual_input_legacy(
                "Unplaced converter",
                &VirtualInOpts::default(),
                "uuid:caption",
                |suffix| {
                    assert!(suffix.is_empty());
                    VirtualInIds {
                        object: identity(1),
                        q: identity(2),
                        qm: identity(3),
                    }
                },
                |_| InputRefIds {
                    object: identity(4),
                    ai: identity(5),
                    i: identity(6),
                    aq: identity(7),
                    q: identity(8),
                },
            )
            .unwrap();
        let LegacyVirtualInputCreation::WithoutPage {
            virtual_in_uuid,
            unplaced_aq_uuid,
        } = result
        else {
            panic!("No Page must not report a placed InputRef");
        };
        assert_eq!(virtual_in_uuid, identity(1));
        assert_eq!(unplaced_aq_uuid, identity(7));
        let output = state.to_bytes().unwrap();
        let document = std::str::from_utf8(&output).unwrap();
        assert!(document.contains(&virtual_in_uuid));
        assert!(!document.contains(&unplaced_aq_uuid));
        assert!(!document.contains("InputRef"));
    }

    #[test]
    fn ambiguous_parent_rejects_before_requesting_identities_or_editing() {
        let mut state = State::load(
            br#"<ControlList><C Type="Page" U="a"/><C Type="Page" U="b"/></ControlList>"#,
        )
        .unwrap();
        let baseline = state.to_bytes().unwrap();
        assert!(matches!(
            state.add_virtual_input_legacy(
                "Untouched",
                &VirtualInOpts::default(),
                "Type:Page",
                |_| panic!("unexpected VirtualIn allocation"),
                |_| panic!("unexpected InputRef allocation")
            ),
            Err(SelectionError::Ambiguous { .. })
        ));
        assert_eq!(state.to_bytes().unwrap(), baseline);
    }
}
