//! Format module tests.

mod custom_stream_tests;
mod custom_tests;
mod registry_tests;

#[cfg(feature = "csv")]
mod csv_tests;
#[cfg(feature = "markdown")]
mod markdown_tests;
#[cfg(feature = "plaintext")]
mod plaintext_stream_tests;
#[cfg(feature = "xml")]
mod xml_tests;
#[cfg(feature = "yaml")]
mod yaml_tests;
