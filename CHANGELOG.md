# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.3](https://github.com/YuniqueCore/multiio/compare/v0.2.2...v0.2.3) - 2025-12-16

### Fixed

- sync version snippets and features table on release

## [0.2.2](https://github.com/YuniqueCore/multiio/compare/v0.2.1...v0.2.2) - 2025-12-16

### Added

- *(cli)* add new `multiio_sarge` binary with sarge-based CLI
- *(cli)* remove sarge from default features and reorganize tests
- *(io)* add support for inline content and explicit file paths in CLI
- *(cli)* add support for `sarge` feature and argument parsing

## [0.2.0](https://github.com/YuniqueCore/multiio/releases/tag/v0.2.0) - 2025-12-12

### Added

- *(format)* remove markdown format support
- *(e2e)* add comprehensive e2e tests for TOML/INI pipelines
- *(format)* add support for TOML and INI formats
- *(e2e)* update baseline JSON files and improve comparison logic
- *(e2e)* add end-to-end testing setup with Python 3.13
- *(async-engine)* add support for sync format registry in async IO engine
- *(engine)* add async record streaming with format-specific support
- *(engine)* add streaming record reading support for multiple formats
- *(engine)* add support for streaming JSON records from inputs
- *(io)* introduce buffered reading and CSV streaming
- *(benchmark)* add read stream benchmark for engine
- *(io)* add buffering to standard I/O streams
- *(async)* add support for configurable async I/O pipeline inputs and outputs
- *(format)* add support for custom formats with dynamic registration
- *(io)* refactor sync and async engines to use byte-based serialization
- *(io)* implement synchronous and asynchronous IO builders
- *(io)* implement unified I/O orchestration library

### Other

- *(github)* add release-plz workflows for automated releases
- *(readme)* update signature image alt text
- *(license)* update copyright year and add author information
- *(deps)* update rust toolchain and github actions versions
- *(ci)* add dependabot and CI/CD workflows
- *(tests)* improve feature matrix compilation tests
- *(format)* introduce `custom` feature flag and refactor custom format handling
- *(format)* adjust default features and binary requirements
- *(cli)* restructure default features and add new binaries and examples
- *(rust)* update rust toolchain to version 1.92.0
- *(format)* replace repetitive format registration loops
- *(format)* refactor deserialization and serialization with macros
- *(format)* centralize format definitions and default order
- *(cli)* remove redundant comments and simplify documentation
- *(test)* add input and output data for manual multi-source stdin and file test case
- *(rust)* update toml and related dependencies in Cargo.lock
- *(readme)* update supported formats and version references
- *(readme)* update features table and add cli binaries section
- *(cli)* add multiio_records_demo binary with JSON/CSV streaming support
- *(e2e)* add error path and large dataset end-to-end tests
- *(e2e)* add manual CLI end-to-end tests and input/output data
- *(e2e)* add async pipeline support and expand test matrix
- *(e2e)* add sync pipeline topology tests and baseline data
- *(e2e)* add end-to-end tests and test infrastructure
- *(builder)* add `with_custom_format` method and implement `Default` trait
- *(async)* add support for custom format registration in async builder
- *(engine)* add support for sync FormatRegistry in AsyncIoEngine
- *(lib)* add documentation for streaming usage and semantics
- *(benchmark)* add benchmark for engine read/write operations
- *(tests)* add end-to-end test for async pipeline with mixed formats
- *(builder)* pre-allocate vectors with known capacity
- *(readme)* update README with custom formats and pipeline config examples
- *(builder)* add e2e test for multi-input multi-output json pipeline
- *(builder_async)* extract format inference logic into helper method
- *(builder)* extract format inference logic into helper method
- *(csv)* improve numeric field inference and error handling
- *(format)* implement modular format handlers for CSV, JSON, Markdown, plaintext, XML, and YAML
- *(engine)* add comprehensive async and sync engine tests
- *(readme)* improve formatting and readability of documentation
- *(tests)* restructure test modules and simplify imports
- *(tests)* move test files into src/tests directory
- *(config)* add tests for PipelineConfig and Input/OutputSpec parsing
- *(cli)* move cli parsing tests to dedicated module
- add comprehensive README with examples and architecture overview
- *(deps)* update Cargo.lock with new dependencies and versions
- *(config)* add autocorrect and typos configuration files
- *(rust)* initialize multiio project with basic addition function
