use lox_core::configuration::{
    editor::EditError,
    mqtt::{MqttDirection, list_topics},
};
use std::error::Error;

#[test]
fn nested_catalog_retains_raw_values_order_and_last_property_even_without_v() {
    let data = r#"<ControlList>
<C Type="Plugin"><C Type="GenTActor" Title="Küche &amp; 灯">
  <SET><mqtt_topic v="old"/><mqtt_qos v="1"/></SET>
  <SET><mqtt_topic/><mqtt_qos/><mqtt_qos v=" raw "/></SET>
  <Wrapper><C Type="GenTSensor"><SET><mqtt_topic v=" /raw/+ "/></SET></C></Wrapper>
</C></C>
<C Type="gentsensor" Title="ignored"/><c Type="GenTActor"/><C Type="InputRef"/>
<C Type="GenTSensor"><SET><mqtt_topic v=" /raw/+ "/></SET></C>
</ControlList>"#;
    let before = data.as_bytes().to_vec();
    let records = list_topics(&before).unwrap();
    assert_eq!(records.len(), 3);
    assert_eq!(records[0].title, "Küche & 灯");
    assert_eq!(records[0].direction, MqttDirection::Publish);
    assert_eq!(records[0].topic, "");
    assert_eq!(records[0].qos, " raw ");
    assert_eq!(records[1], records[2]);
    assert_eq!(records[1].direction, MqttDirection::Subscribe);
    assert_eq!(records[1].topic, " /raw/+ ");
    assert_eq!(records[1].qos, "");
    assert_eq!(before, data.as_bytes());
}

#[test]
fn legacy_loader_admission_and_known_quoted_digit_name_limit_are_unchanged() {
    let records = list_topics(
        br#"<C Type="GenTSensor" Title="literal 12hTF=x" 12hTF="true"><SET><mqtt_topic v="&lt;x&gt;&amp;&quot;"/></SET></C>"#,
    )
    .unwrap();
    assert_eq!(records[0].title, "literal _12hTF=x"); // lox-n37w, not a new normalization.
    assert_eq!(records[0].topic, "<x>&\"");
    for data in [
        b"<C>".as_slice(),
        b"<C Type='a' Type='b'/>",
        b"<C Title='\xff'/>",
    ] {
        let error = list_topics(data).unwrap_err();
        assert!(matches!(error, EditError::Load(_)));
        assert!(
            error
                .source()
                .unwrap()
                .downcast_ref::<xmltree::ParseError>()
                .is_some()
        );
    }
}
