use crate::format::{FormatKind, deserialize, serialize};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct XmlInner {
    id: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct XmlData {
    name: String,
    inner: XmlInner,
}

#[test]
fn xml_roundtrip_nested_struct() {
    let data = XmlData {
        name: "xml".into(),
        inner: XmlInner { id: 7 },
    };

    let bytes = serialize(FormatKind::Xml, &data).expect("serialize xml");
    assert!(!bytes.is_empty());

    let decoded: XmlData = deserialize(FormatKind::Xml, &bytes).expect("deserialize xml");
    assert_eq!(decoded, data);
}
