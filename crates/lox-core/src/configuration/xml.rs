use quick_xml::events::BytesStart;

pub(super) fn attr_value(e: &BytesStart<'_>, name: &[u8]) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == name {
            return String::from_utf8(attr.value.to_vec()).ok();
        }
    }
    None
}
