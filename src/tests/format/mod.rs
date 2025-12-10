//! Format module tests.

mod custom_tests;
mod json_tests;
mod registry_tests;

#[cfg(feature = "csv")]
mod csv_tests;
#[cfg(feature = "markdown")]
mod markdown_tests;
#[cfg(feature = "xml")]
mod xml_tests;
#[cfg(feature = "yaml")]
mod yaml_tests;
