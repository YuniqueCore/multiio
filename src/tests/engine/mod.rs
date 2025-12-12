//! Engine module tests.

#[cfg(feature = "async")]
mod async_tests;
mod csv_row_stream_tests;
mod json_row_stream_tests;
#[cfg(any(feature = "json", feature = "async"))]
mod stream_tests;
mod sync_tests;
