//! Offline MQTT topic catalog using the existing configuration editor loader.
//!
//! Only exact `C`/`GenTSensor` and `C`/`GenTActor` elements are cataloged.
//! Source order, duplicates and raw topic/QoS strings are retained; this does not
//! validate a broker or MQTT values. Loading retains the legacy digit-name and
//! quoted-text limitation (lox-n37w), rather than broadening XML admission.

use super::editor::{EditError, legacy::State};
use xmltree::Element;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MqttDirection {
    Subscribe,
    Publish,
}

impl MqttDirection {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Subscribe => "subscribe",
            Self::Publish => "publish",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MqttTopic {
    pub title: String,
    pub direction: MqttDirection,
    pub topic: String,
    /// Raw stored value, including empty or nonnumeric strings.
    pub qos: String,
}

/// Catalog topics without file I/O, terminal output or input mutation.
///
/// Every direct `SET` contributes its direct properties. Later `mqtt_topic`
/// and `mqtt_qos` properties overwrite earlier values, even when `v` is absent.
pub fn list_topics(data: &[u8]) -> Result<Vec<MqttTopic>, EditError> {
    Ok(State::load(data)?.mqtt_topics())
}

pub(crate) fn from_root(root: &Element) -> Vec<MqttTopic> {
    let mut topics = Vec::new();
    collect_topics(root, &mut topics);
    topics
}

fn collect_topics(elem: &Element, topics: &mut Vec<MqttTopic>) {
    if elem.name == "C"
        && let Some(t) = elem.attributes.get("Type")
        && (t == "GenTSensor" || t == "GenTActor")
    {
        let title = elem.attributes.get("Title").cloned().unwrap_or_default();
        let direction = if t == "GenTSensor" {
            MqttDirection::Subscribe
        } else {
            MqttDirection::Publish
        };
        let mut topic = String::new();
        let mut qos = String::new();
        for child in &elem.children {
            if let Some(set) = child.as_element()
                && set.name == "SET"
            {
                for prop in &set.children {
                    if let Some(p) = prop.as_element() {
                        if p.name == "mqtt_topic" {
                            topic = p.attributes.get("v").cloned().unwrap_or_default();
                        }
                        if p.name == "mqtt_qos" {
                            qos = p.attributes.get("v").cloned().unwrap_or_default();
                        }
                    }
                }
            }
        }
        topics.push(MqttTopic {
            title,
            direction,
            topic,
            qos,
        });
    }
    for child in &elem.children {
        if let Some(child_elem) = child.as_element() {
            collect_topics(child_elem, topics);
        }
    }
}
