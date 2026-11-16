//! Owned MQTT observations; Core retains loading, matching and ordering policy.

use lox_core::configuration::{editor::EditError, mqtt as core};
use std::error::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum MqttDirection {
    Subscribe,
    Publish,
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct MqttTopic {
    pub title: String,
    pub direction: MqttDirection,
    pub topic: String,
    pub qos: String,
}

impl From<core::MqttTopic> for MqttTopic {
    fn from(value: core::MqttTopic) -> Self {
        Self {
            title: value.title,
            direction: match value.direction {
                core::MqttDirection::Subscribe => MqttDirection::Subscribe,
                core::MqttDirection::Publish => MqttDirection::Publish,
            },
            topic: value.topic,
            qos: value.qos,
        }
    }
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum MqttCatalogError {
    #[error("Configuration load failed: {source_message}")]
    Load { source_message: String },
    #[error("{message}")]
    Other {
        message: String,
        source_message: Option<String>,
    },
}

impl From<EditError> for MqttCatalogError {
    fn from(error: EditError) -> Self {
        match error {
            EditError::Load(cause) => Self::Load {
                source_message: cause.to_string(),
            },
            other => Self::Other {
                message: other.to_string(),
                source_message: other.source().map(ToString::to_string),
            },
        }
    }
}

/// Catalog supplied configuration bytes without I/O, mutation or MQTT validation.
/// The legacy shared loader limitation lox-n37w remains unchanged.
#[uniffi::export]
pub fn list_mqtt_topics(data: Vec<u8>) -> Result<Vec<MqttTopic>, MqttCatalogError> {
    Ok(core::list_topics(&data)?
        .into_iter()
        .map(Into::into)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_catalog_preserves_core_projection_and_input_independence() {
        let input = include_bytes!("../tests/fixtures/mqtt_catalog.xml").to_vec();
        let expected: Vec<MqttTopic> = core::list_topics(&input)
            .unwrap()
            .into_iter()
            .map(Into::into)
            .collect();
        let mut actual = list_mqtt_topics(input.clone()).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(actual.len(), 3);
        assert_eq!(actual[0].direction, MqttDirection::Publish);
        assert_eq!(actual[0].qos, " raw ");
        assert_eq!(actual[1], actual[2]);
        actual[0].title.clear();
        assert_eq!(list_mqtt_topics(input).unwrap(), expected);
    }

    #[test]
    fn existing_loader_refusals_and_known_quoted_digit_limit_survive() {
        for input in [
            b"<C>".as_slice(),
            b"<C Type='a' Type='b'/>",
            b"<C Title='\xff'/>",
        ] {
            let core_error = core::list_topics(input).unwrap_err();
            let source_message = core_error.source().unwrap().to_string();
            match list_mqtt_topics(input.to_vec()).unwrap_err() {
                MqttCatalogError::Load {
                    source_message: actual,
                } => assert_eq!(actual, source_message),
                error => panic!("wrong variant: {error:?}"),
            }
        }
        let rows = list_mqtt_topics(
            br#"<C Type="GenTSensor" Title="literal 12hTF=x" 12hTF="true"/>"#.to_vec(),
        )
        .unwrap();
        assert_eq!(rows[0].title, "literal _12hTF=x");
        assert!(list_mqtt_topics(b"<Root/>".to_vec()).unwrap().is_empty());
    }
}
