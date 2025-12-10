use std::io::Cursor;

use crate::format::{
    FormatKind, default_registry, deserialize, deserialize_yaml_stream, serialize,
};
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

#[test]
fn yaml_stream_reads_multiple_documents() {
    let multi_doc = b"---\nname: a\nvalue: 1\nflag: true\n---\nname: b\nvalue: 2\nflag: false\n";

    let iter = deserialize_yaml_stream::<YamlData, _>(Cursor::new(&multi_doc[..]));
    let docs: Vec<YamlData> = iter
        .collect::<Result<_, _>>()
        .expect("yaml docs should parse");

    assert_eq!(docs.len(), 2);
    assert_eq!(docs[0].name, "a");
    assert_eq!(docs[1].name, "b");
}

#[test]
fn yaml_stream_via_registry_reads_multiple_documents() {
    let multi_doc = b"---\nname: a\nvalue: 1\nflag: true\n---\nname: b\nvalue: 2\nflag: false\n";
    let registry = default_registry();

    let iter = registry
        .stream_deserialize_into::<YamlData>(
            Some(&FormatKind::Yaml),
            &[],
            Box::new(Cursor::new(&multi_doc[..])),
        )
        .expect("yaml streaming should be supported");

    let docs: Vec<YamlData> = iter
        .collect::<Result<_, _>>()
        .expect("yaml docs should parse");

    assert_eq!(docs.len(), 2);
    assert_eq!(docs[0].name, "a");
    assert_eq!(docs[1].name, "b");
}
