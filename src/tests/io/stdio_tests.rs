//! Tests for standard IO providers.

use crate::{FileInput, FileOutput, InputProvider, OutputTarget};
use std::fs;
use std::io::Read;

#[test]
fn file_input_reads_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("input.txt");
    fs::write(&path, b"hello world").unwrap();

    let inp = FileInput::new(path.clone());
    let mut reader = inp.open().unwrap();
    let mut buf = String::new();
    reader.read_to_string(&mut buf).unwrap();

    assert_eq!(buf, "hello world");
}

#[test]
fn file_output_writes_and_appends() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("out.txt");

    let out = FileOutput::new(path.clone());

    {
        let mut w = out.open_overwrite().unwrap();
        std::io::Write::write_all(&mut w, b"abc").unwrap();
    }
    assert_eq!(fs::read(&path).unwrap(), b"abc".to_vec());

    {
        let mut w = out.open_append().unwrap();
        std::io::Write::write_all(&mut w, b"def").unwrap();
    }
    assert_eq!(fs::read(&path).unwrap(), b"abcdef".to_vec());
}
