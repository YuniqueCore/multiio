//! In-memory I/O implementations for testing.

use std::io::{self, Cursor, Read, Write};
use std::sync::{Arc, Mutex};

use super::{InputProvider, OutputTarget};

/// In-memory input source for testing.
#[derive(Debug, Clone)]
pub struct InMemorySource {
    id: String,
    data: Arc<Vec<u8>>,
}

impl InMemorySource {
    /// Create a new in-memory source with the given data.
    pub fn new(id: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            id: id.into(),
            data: Arc::new(data),
        }
    }

    /// Create a new in-memory source from a string.
    pub fn from_string(id: impl Into<String>, data: impl Into<String>) -> Self {
        Self::new(id, data.into().into_bytes())
    }
}

impl InputProvider for InMemorySource {
    fn id(&self) -> &str {
        &self.id
    }

    fn open(&self) -> io::Result<Box<dyn Read + Send>> {
        // Use Arc::as_ref to get a reference to the inner Vec, then clone it
        Ok(Box::new(Cursor::new(self.data.as_ref().clone())))
    }
}

/// In-memory output sink for testing.
#[derive(Debug, Clone)]
pub struct InMemorySink {
    id: String,
    buf: Arc<Mutex<Vec<u8>>>,
}

impl InMemorySink {
    /// Create a new empty in-memory sink.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            buf: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get the contents of the sink as bytes.
    pub fn contents(&self) -> Vec<u8> {
        self.buf.lock().unwrap().clone()
    }

    /// Get the contents of the sink as a string.
    pub fn contents_string(&self) -> String {
        String::from_utf8_lossy(&self.contents()).into_owned()
    }

    /// Consume the sink and return its contents.
    pub fn into_inner(self) -> Vec<u8> {
        Arc::try_unwrap(self.buf)
            .map(|m| m.into_inner().unwrap())
            .unwrap_or_else(|arc| arc.lock().unwrap().clone())
    }

    /// Clear the sink contents.
    pub fn clear(&self) {
        self.buf.lock().unwrap().clear();
    }
}

impl OutputTarget for InMemorySink {
    fn id(&self) -> &str {
        &self.id
    }

    fn open_overwrite(&self) -> io::Result<Box<dyn Write + Send>> {
        // Clear existing content for overwrite
        self.buf.lock().unwrap().clear();
        Ok(Box::new(InMemoryWriteHandle {
            buf: self.buf.clone(),
        }))
    }

    fn open_append(&self) -> io::Result<Box<dyn Write + Send>> {
        Ok(Box::new(InMemoryWriteHandle {
            buf: self.buf.clone(),
        }))
    }
}

/// Write handle for in-memory sink.
struct InMemoryWriteHandle {
    buf: Arc<Mutex<Vec<u8>>>,
}

impl std::fmt::Debug for InMemoryWriteHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InMemoryWriteHandle").finish()
    }
}

impl Write for InMemoryWriteHandle {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        let mut guard = self.buf.lock().unwrap();
        guard.extend_from_slice(data);
        Ok(data.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
