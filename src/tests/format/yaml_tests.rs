use crate::format::{FormatKind, deserialize, serialize};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct YamlData {
    name: String,
    value: i32,
    flag: bool,
}

#[test]
fn yaml_roundtrip_simple_struct() {
    let data = YamlData {
        name: "test".into(),
        value: 42,
        flag: true,
    };

    let bytes = serialize(FormatKind::Yaml, &data).expect("serialize yaml");
    assert!(!bytes.is_empty());

    let decoded: YamlData = deserialize(FormatKind::Yaml, &bytes).expect("deserialize yaml");
    assert_eq!(decoded, data);
}

#[test]
fn yaml_handles_multiple_documents_error() {
    // Our implementation uses serde_yaml::from_slice, which expects a single document.
    let multi_doc = b"---\nname: a\nvalue: 1\n---\nname: b\nvalue: 2\n";
    let result: Result<YamlData, _> = deserialize(FormatKind::Yaml, multi_doc);
    assert!(result.is_err());
}
