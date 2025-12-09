//! Tests for in-memory IO implementations.

use crate::{InMemorySink, InMemorySource, InputProvider, OutputTarget};
use std::io::{Read, Write};

#[test]
fn in_memory_source_reads_data() {
    let src = InMemorySource::from_string("id", "hello");

    let mut reader = src.open().expect("open in-memory source");
    let mut buf = String::new();
    reader.read_to_string(&mut buf).unwrap();

    assert_eq!(buf, "hello");
}

#[test]
fn in_memory_sink_writes_and_reads_back() {
    let sink = InMemorySink::new("out");

    // overwrite
    {
        let mut w = sink.open_overwrite().unwrap();
        w.write_all(b"abc").unwrap();
    }
    assert_eq!(sink.contents(), b"abc".to_vec());

    // append
    {
        let mut w = sink.open_append().unwrap();
        w.write_all(b"def").unwrap();
    }
    assert_eq!(sink.contents(), b"abcdef".to_vec());
}
