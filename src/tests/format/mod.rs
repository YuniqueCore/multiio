//! Format module tests.

#[cfg(feature = "json")]
mod custom_stream_tests;
#[cfg(feature = "json")]
mod custom_tests;
#[cfg(feature = "json")]
mod registry_tests;

#[cfg(feature = "csv")]
mod csv_tests;
#[cfg(feature = "ini")]
mod ini_tests;
#[cfg(feature = "plaintext")]
mod plaintext_stream_tests;
#[cfg(feature = "toml")]
mod toml_tests;
#[cfg(feature = "xml")]
mod xml_tests;
#[cfg(feature = "yaml")]
mod yaml_tests;
