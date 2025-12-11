use crate::format::{FormatKind, deserialize, serialize};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TomlData {
    name: String,
    value: i32,
    flag: bool,
}

#[test]
fn toml_roundtrip_simple_struct() {
    let data = TomlData {
        name: "toml".into(),
        value: 7,
        flag: true,
    };

    let bytes = serialize(FormatKind::Toml, &data).expect("serialize toml");
    assert!(!bytes.is_empty());

    let decoded: TomlData = deserialize(FormatKind::Toml, &bytes).expect("deserialize toml");
    assert_eq!(decoded, data);
}
