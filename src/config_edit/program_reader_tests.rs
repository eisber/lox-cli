use super::ConfigEditor;
use xmltree::XMLNode;

#[test]
fn program_reader_cached_direct_text_first_field_and_exact_eligibility() {
    let bytes = br#"<Root Date="fixed" DateS="99">
<C Type="SequenceController" U="s" Title="Program" foreign="keep">
<Unknown><Field Name="Configuration">nested</Field></Unknown>
<Field Name="configuration">wrong case</Field>
<Field Name="Configuration">first<Nested>hidden</Nested>second<![CDATA[hidden]]>third</Field>
<Field Name="Configuration">later</Field></C>
<C Type="SequenceController" U="s" Title="Program"><Field Name="Configuration">mirror</Field></C>
<C Type="sequencecontroller" U="wrong" Title="Wrong"/><C U="sparse"/>
<C Type="SequenceController" U="empty" Title="Empty"/></Root>"#;
    let mut editor = ConfigEditor::load(bytes).unwrap();
    let before = editor.to_bytes().unwrap();
    assert_eq!(
        editor.get_program_text("uuid:s").unwrap(),
        "firstsecondthird"
    );
    assert_eq!(editor.get_program_text("Empty").unwrap(), "");
    assert_eq!(editor.to_bytes().unwrap(), before);
    for (selector, expected) in [
        (
            "Wrong",
            "'Wrong' is a sequencecontroller — get-program is only for SequenceController blocks",
        ),
        (
            "uuid:sparse",
            "'' is a  — get-program is only for SequenceController blocks",
        ),
    ] {
        let error = editor.get_program_text(selector).unwrap_err();
        let original = anyhow::anyhow!("{expected}");
        assert_eq!(observation(&error), observation(&original));
    }
    let path = editor.require_one("uuid:s").unwrap();
    let field = editor
        .get_element_mut(&path)
        .children
        .iter_mut()
        .filter_map(XMLNode::as_mut_element)
        .find(|e| {
            e.name == "Field"
                && e.attributes
                    .get("Name")
                    .is_some_and(|n| n == "Configuration")
        })
        .unwrap();
    field.children.push(XMLNode::Text("cached".into()));
    assert_eq!(
        editor.get_program_text("uuid:s").unwrap(),
        "firstsecondthirdcached"
    );
}

fn observation(error: &anyhow::Error) -> (Vec<String>, Vec<bool>) {
    (
        vec![
            format!("{error}"),
            format!("{error:?}"),
            format!("{error:#}"),
            format!("{}", error.root_cause()),
            format!("{:?}", error.root_cause()),
            error
                .chain()
                .map(|cause| format!("{cause}\n{cause:?}"))
                .collect::<Vec<_>>()
                .join("\n---\n"),
        ],
        vec![
            error.downcast_ref::<String>().is_some(),
            error.downcast_ref::<&'static str>().is_some(),
            error.downcast_ref::<xmltree::ParseError>().is_some(),
            error
                .downcast_ref::<Box<dyn std::error::Error + Send + Sync>>()
                .is_some(),
            error.root_cause().is::<xmltree::ParseError>(),
            error.chain().any(|cause| cause.is::<xmltree::ParseError>()),
        ],
    )
}

#[test]
fn program_reader_selection_precedes_type_and_keeps_concrete_head_errors() {
    let editor = ConfigEditor::load(
        br#"<Root><C Type="And" U="a" Title="Program"/><C Type="SequenceController" U="b" Title="Program"/></Root>"#,
    ).unwrap();
    for selector in ["Program", "Progra", "missing"] {
        assert_eq!(
            observation(&editor.get_program_text(selector).unwrap_err()),
            observation(&editor.require_one(selector).unwrap_err()),
        );
    }
    let bytes = b"<C A='1' A='2'/>";
    let public = lox_core::configuration::editor::get_program_text(bytes, "missing").unwrap_err();
    let adapted = super::editor_error(public);
    let cached = ConfigEditor::load(bytes).err().unwrap();
    assert_eq!(observation(&adapted), observation(&cached));
}
