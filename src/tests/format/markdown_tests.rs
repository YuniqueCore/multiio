use crate::format::{FormatKind, deserialize, serialize};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct MdData {
    name: String,
    value: i32,
}

#[test]
fn markdown_roundtrip_json_code_block() {
    let data = MdData {
        name: "md".into(),
        value: 5,
    };

    let bytes = serialize(FormatKind::Markdown, &data).expect("serialize markdown");
    let s = String::from_utf8(bytes.clone()).expect("utf8 markdown");

    // Should contain a json code fence
    assert!(s.contains("```json"));

    let decoded: MdData = deserialize(FormatKind::Markdown, &bytes).expect("deserialize markdown");
    assert_eq!(decoded, data);
}

#[test]
fn markdown_deserialize_from_existing_json_block() {
    let md = r#"
# Title

Some text.

```json
{"name": "x", "value": 42}
```
"#;

    let decoded: MdData =
        deserialize(FormatKind::Markdown, md.as_bytes()).expect("deserialize markdown");
    assert_eq!(decoded.name, "x");
    assert_eq!(decoded.value, 42);
}
