use super::{ConfigEditor, MqttTopic, editor_error};
use xmltree::Element;

// Frozen public-base 153c1bd walker, retained only as a differential oracle.
fn frozen(elem: &Element, topics: &mut Vec<MqttTopic>) {
    if elem.name == "C"
        && let Some(t) = elem.attributes.get("Type")
        && (t == "GenTSensor" || t == "GenTActor")
    {
        let title = elem.attributes.get("Title").cloned().unwrap_or_default();
        let direction = if t == "GenTSensor" {
            "subscribe"
        } else {
            "publish"
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
            direction: direction.to_string(),
            topic,
            qos,
        });
    }
    for child in &elem.children {
        if let Some(child_elem) = child.as_element() {
            frozen(child_elem, topics);
        }
    }
}

#[test]
fn mqtt_catalog_frozen_walker_public_records_and_head_dto_agree() {
    for data in [
        r#"<ControlList><C Type="GenTActor" Title="Ü &amp; 灯"><SET><mqtt_topic v="first"/><mqtt_qos v="9"/></SET><SET><mqtt_topic/><mqtt_qos/><mqtt_qos v=" raw "/></SET><W><C Type="GenTSensor"><SET><mqtt_topic v=" /x/+ "/></SET></C></W></C><C Type="GenTSensor"><SET><mqtt_topic v=" /x/+ "/></SET></C><C Type="gentsensor"/><c Type="GenTActor"/><C Type="OutputRef"/><C Type="GenTActor"><set><mqtt_topic v="ignored"/></set><SET><W><mqtt_topic v="ignored"/></W></SET></C></ControlList>"#,
        r#"<C Type="GenTSensor" Title="quoted 12hTF=x" 12hTF="true"><SET><mqtt_topic v="&lt;&amp;&quot;"/></SET></C>"#,
        r#"<C Type="GenTSensor" Title="same" Title="same"/>"#,
    ] {
        let mut editor = ConfigEditor::load(data.as_bytes()).unwrap();
        let mut old = Vec::new();
        frozen(&editor.root, &mut old);
        let public: Vec<MqttTopic> = lox_core::configuration::mqtt::list_topics(data.as_bytes())
            .unwrap()
            .into_iter()
            .map(Into::into)
            .collect();
        let value = |topics: Vec<MqttTopic>| serde_json::to_value(topics).unwrap();
        assert_eq!(value(old), value(public));
        assert_eq!(
            value(editor.list_mqtt_topics()),
            value(
                lox_core::configuration::mqtt::list_topics(data.as_bytes())
                    .unwrap()
                    .into_iter()
                    .map(Into::into)
                    .collect()
            )
        );
        editor
            .root
            .attributes
            .insert("Type".into(), "GenTActor".into());
        editor.root.name = "C".into();
        let mut changed_old = Vec::new();
        frozen(&editor.root, &mut changed_old);
        assert_eq!(value(editor.list_mqtt_topics()), value(changed_old));
    }
}

#[test]
fn mqtt_catalog_retains_native_error_chain_and_downcast() {
    for data in [b"<C>".as_slice(), b"<C Title='\xff'/>", b"<C A='1' A='2'/>"] {
        let old = ConfigEditor::load(data).err().unwrap();
        let new = editor_error(lox_core::configuration::mqtt::list_topics(data).unwrap_err());
        assert_eq!(format!("{old:#}"), format!("{new:#}"));
        assert_eq!(
            old.downcast_ref::<xmltree::ParseError>().is_some(),
            new.downcast_ref::<xmltree::ParseError>().is_some()
        );
        for error in [&old, &new] {
            let native = error
                .downcast_ref::<Box<dyn std::error::Error + Send + Sync>>()
                .expect("legacy adapter retains its boxed native cause");
            assert!(
                native
                    .as_ref()
                    .downcast_ref::<xmltree::ParseError>()
                    .is_some()
            );
        }
    }
}
