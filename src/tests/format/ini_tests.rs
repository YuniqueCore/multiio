use crate::format::{FormatKind, deserialize, serialize};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct IniData {
    name: String,
    value: i32,
}

#[test]
fn ini_roundtrip_simple_struct() {
    let data = IniData {
        name: "ini".into(),
        value: 10,
    };

    let bytes = serialize(FormatKind::Ini, &data).expect("serialize ini");
    assert!(!bytes.is_empty());

    let decoded: IniData = deserialize(FormatKind::Ini, &bytes).expect("deserialize ini");
    assert_eq!(decoded, data);
}
