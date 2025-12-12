//! Synchronous I/O engine for orchestrating read and write operations.

use std::io::Read;

use serde::{Serialize, de::DeserializeOwned};

use crate::config::{FileExistsPolicy, InputSpec, OutputSpec};
use crate::error::{AggregateError, ErrorPolicy, SingleIoError, Stage};
use crate::format::FormatRegistry;

/// Synchronous I/O engine for orchestrating multi-input/multi-output operations.
pub struct IoEngine {
    registry: FormatRegistry,
    error_policy: ErrorPolicy,
    inputs: Vec<InputSpec>,
    outputs: Vec<OutputSpec>,
}

impl IoEngine {
    /// Create a new I/O engine.
    pub fn new(
        registry: FormatRegistry,
        error_policy: ErrorPolicy,
        inputs: Vec<InputSpec>,
        outputs: Vec<OutputSpec>,
    ) -> Self {
        Self {
            registry,
            error_policy,
            inputs,
            outputs,
        }
    }

    /// Get the format registry.
    pub fn registry(&self) -> &FormatRegistry {
        &self.registry
    }

    /// Get the error policy.
    pub fn error_policy(&self) -> ErrorPolicy {
        self.error_policy
    }

    /// Get the input specifications.
    pub fn inputs(&self) -> &[InputSpec] {
        &self.inputs
    }

    /// Get the output specifications.
    pub fn outputs(&self) -> &[OutputSpec] {
        &self.outputs
    }

    /// Read all inputs and deserialize each into type T.
    ///
    /// Returns a vector of deserialized values, one for each input.
    /// If error_policy is FastFail, stops at the first error.
    /// If error_policy is Accumulate, collects all errors.
    pub fn read_all<T>(&self) -> Result<Vec<T>, AggregateError>
    where
        T: DeserializeOwned,
    {
        let mut results = Vec::with_capacity(self.inputs.len());
        let mut errors = Vec::new();
        let mut buffer = Vec::new();

        for spec in &self.inputs {
            match self.read_one_with_buffer::<T>(spec, &mut buffer) {
                Ok(value) => results.push(value),
                Err(e) => {
                    errors.push(e);
                    if matches!(self.error_policy, ErrorPolicy::FastFail) {
                        return Err(AggregateError { errors });
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(results)
        } else {
            Err(AggregateError { errors })
        }
    }

    /// Read a single input and deserialize into type T.
    fn read_one<T>(&self, spec: &InputSpec) -> Result<T, SingleIoError>
    where
        T: DeserializeOwned,
    {
        let mut buffer = Vec::new();
        self.read_one_with_buffer::<T>(spec, &mut buffer)
    }

    fn read_one_with_buffer<T>(
        &self,
        spec: &InputSpec,
        buffer: &mut Vec<u8>,
    ) -> Result<T, SingleIoError>
    where
        T: DeserializeOwned,
    {
        // Open the input stream
        let mut reader = spec.provider.open().map_err(|e| SingleIoError {
            stage: Stage::Open,
            target: spec.raw.clone(),
            error: Box::new(e),
        })?;

        // Read all bytes into the reusable buffer
        buffer.clear();
        reader.read_to_end(buffer).map_err(|e| SingleIoError {
            stage: Stage::Open,
            target: spec.raw.clone(),
            error: Box::new(e),
        })?;

        // Deserialize (handles both built-in and custom formats)
        self.registry
            .deserialize_value::<T>(
                spec.explicit_format.as_ref(),
                &spec.format_candidates,
                buffer,
            )
            .map_err(|e| SingleIoError {
                stage: Stage::Parse,
                target: spec.raw.clone(),
                error: Box::new(e),
            })
    }

    /// Write values to all outputs.
    ///
    /// Each output receives the same serialized values.
    pub fn write_all<T>(&self, values: &[T]) -> Result<(), AggregateError>
    where
        T: Serialize,
    {
        let mut errors = Vec::new();

        for spec in &self.outputs {
            if let Err(e) = self.write_one(spec, values) {
                errors.push(e);
                if matches!(self.error_policy, ErrorPolicy::FastFail) {
                    return Err(AggregateError { errors });
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(AggregateError { errors })
        }
    }

    /// Write a single value to all outputs.
    pub fn write_one_value<T>(&self, value: &T) -> Result<(), AggregateError>
    where
        T: Serialize,
    {
        let mut errors = Vec::new();

        for spec in &self.outputs {
            if let Err(e) = self.write_single(spec, value) {
                errors.push(e);
                if matches!(self.error_policy, ErrorPolicy::FastFail) {
                    return Err(AggregateError { errors });
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(AggregateError { errors })
        }
    }

    /// Write values to a single output.
    fn write_one<T>(&self, spec: &OutputSpec, values: &[T]) -> Result<(), SingleIoError>
    where
        T: Serialize,
    {
        // Serialize to bytes (handles both built-in and custom formats)
        let bytes = self
            .registry
            .serialize_value::<&[T]>(
                spec.explicit_format.as_ref(),
                &spec.format_candidates,
                &values,
            )
            .map_err(|e| SingleIoError {
                stage: Stage::Serialize,
                target: spec.raw.clone(),
                error: Box::new(e),
            })?;

        // Open the output stream based on policy
        let mut writer = self.open_output(spec)?;

        // Write bytes
        std::io::Write::write_all(&mut *writer, &bytes).map_err(|e| SingleIoError {
            stage: Stage::Serialize,
            target: spec.raw.clone(),
            error: Box::new(e),
        })
    }

    /// Write a single value to a specific output.
    fn write_single<T>(&self, spec: &OutputSpec, value: &T) -> Result<(), SingleIoError>
    where
        T: Serialize,
    {
        // Serialize to bytes (handles both built-in and custom formats)
        let bytes = self
            .registry
            .serialize_value(
                spec.explicit_format.as_ref(),
                &spec.format_candidates,
                value,
            )
            .map_err(|e| SingleIoError {
                stage: Stage::Serialize,
                target: spec.raw.clone(),
                error: Box::new(e),
            })?;

        // Open the output stream based on policy
        let mut writer = self.open_output(spec)?;

        // Write bytes
        std::io::Write::write_all(&mut *writer, &bytes).map_err(|e| SingleIoError {
            stage: Stage::Serialize,
            target: spec.raw.clone(),
            error: Box::new(e),
        })
    }

    /// Open an output based on the file exists policy.
    fn open_output(
        &self,
        spec: &OutputSpec,
    ) -> Result<Box<dyn std::io::Write + Send>, SingleIoError> {
        let result = match spec.file_exists_policy {
            FileExistsPolicy::Overwrite => spec.target.open_overwrite(),
            FileExistsPolicy::Append => spec.target.open_append(),
            FileExistsPolicy::Error => {
                // For file outputs, check if file exists
                // For now, just use overwrite (can be enhanced)
                spec.target.open_overwrite()
            }
        };

        result.map_err(|e| SingleIoError {
            stage: Stage::Open,
            target: spec.raw.clone(),
            error: Box::new(e),
        })
    }

    /// Create an iterator that reads each input lazily.
    ///
    /// This allows processing inputs one at a time without loading all into memory.
    pub fn read_stream<T>(&self) -> impl Iterator<Item = Result<T, SingleIoError>> + '_
    where
        T: DeserializeOwned,
    {
        self.inputs.iter().map(|spec| self.read_one::<T>(spec))
    }

    /// Stream records from all inputs using their resolved formats.
    ///
    /// This uses format-specific streaming implementations when available
    /// (e.g. JSON NDJSON, CSV rows, custom streaming handlers), and falls
    /// back to non-streaming deserialization as a single item for formats
    /// that do not support multi-record streaming.
    pub fn read_records<T>(&self) -> impl Iterator<Item = Result<T, SingleIoError>> + '_
    where
        T: DeserializeOwned + 'static,
    {
        self.inputs
            .iter()
            .flat_map(move |spec| self.records_stream_for_spec::<T>(spec))
    }

    /// Stream JSON records from all inputs whose resolved format is JSON.
    ///
    /// Each top-level JSON value is deserialized into `T`. Errors are reported per-record
    /// as `SingleIoError` with appropriate stage and target information.
    #[cfg(feature = "json")]
    pub fn read_json_records<T>(&self) -> impl Iterator<Item = Result<T, SingleIoError>> + '_
    where
        T: DeserializeOwned + 'static,
    {
        self.inputs
            .iter()
            .flat_map(move |spec| self.json_stream_for_spec::<T>(spec))
    }

    /// Stream CSV records from all inputs whose resolved format is CSV.
    ///
    /// Each CSV record is deserialized into `T`. Errors are reported per-record
    /// as `SingleIoError` with appropriate stage and target information.
    #[cfg(feature = "csv")]
    pub fn read_csv_records<T>(&self) -> impl Iterator<Item = Result<T, SingleIoError>> + '_
    where
        T: DeserializeOwned + 'static,
    {
        self.inputs
            .iter()
            .flat_map(move |spec| self.csv_stream_for_spec::<T>(spec))
    }

    #[cfg(feature = "csv")]
    fn csv_stream_for_spec<T>(
        &self,
        spec: &InputSpec,
    ) -> Box<dyn Iterator<Item = Result<T, SingleIoError>> + '_>
    where
        T: DeserializeOwned + 'static,
    {
        // Resolve format first
        let kind = match self
            .registry
            .resolve(spec.explicit_format.as_ref(), &spec.format_candidates)
        {
            Ok(k) => k,
            Err(e) => {
                let err = SingleIoError {
                    stage: Stage::ResolveInput,
                    target: spec.raw.clone(),
                    error: Box::new(e),
                };
                return Box::new(std::iter::once(Err(err)));
            }
        };

        if kind != crate::format::FormatKind::Csv {
            let err = SingleIoError {
                stage: Stage::ResolveInput,
                target: spec.raw.clone(),
                error: Box::new(crate::format::FormatError::UnknownFormat(kind)),
            };
            return Box::new(std::iter::once(Err(err)));
        }

        // Open the input
        let reader = match spec.provider.open() {
            Ok(r) => r,
            Err(e) => {
                let err = SingleIoError {
                    stage: Stage::Open,
                    target: spec.raw.clone(),
                    error: Box::new(e),
                };
                return Box::new(std::iter::once(Err(err)));
            }
        };

        let target = spec.raw.clone();
        let iter = crate::format::deserialize_csv_stream::<T, _>(reader).map(move |res| {
            res.map_err(|e| SingleIoError {
                stage: Stage::Parse,
                target: target.clone(),
                error: Box::new(e),
            })
        });

        Box::new(iter)
    }

    fn records_stream_for_spec<T>(
        &self,
        spec: &InputSpec,
    ) -> Box<dyn Iterator<Item = Result<T, SingleIoError>> + '_>
    where
        T: DeserializeOwned + 'static,
    {
        // Open the input
        let reader = match spec.provider.open() {
            Ok(r) => r,
            Err(e) => {
                let err = SingleIoError {
                    stage: Stage::Open,
                    target: spec.raw.clone(),
                    error: Box::new(e),
                };
                return Box::new(std::iter::once(Err(err)));
            }
        };

        let target = spec.raw.clone();
        let iter_result = self.registry.stream_deserialize_into::<T>(
            spec.explicit_format.as_ref(),
            &spec.format_candidates,
            Box::new(reader),
        );

        let iter = match iter_result {
            Ok(iter) => iter,
            Err(e) => {
                let err = SingleIoError {
                    stage: Stage::Parse,
                    target: target.clone(),
                    error: Box::new(e),
                };
                return Box::new(std::iter::once(Err(err)));
            }
        };

        let mapped = iter.map(move |res| {
            res.map_err(|e| SingleIoError {
                stage: Stage::Parse,
                target: target.clone(),
                error: Box::new(e),
            })
        });

        Box::new(mapped)
    }

    #[cfg(feature = "json")]
    fn json_stream_for_spec<T>(
        &self,
        spec: &InputSpec,
    ) -> Box<dyn Iterator<Item = Result<T, SingleIoError>> + '_>
    where
        T: DeserializeOwned + 'static,
    {
        // Resolve format first
        let kind = match self
            .registry
            .resolve(spec.explicit_format.as_ref(), &spec.format_candidates)
        {
            Ok(k) => k,
            Err(e) => {
                let err = SingleIoError {
                    stage: Stage::ResolveInput,
                    target: spec.raw.clone(),
                    error: Box::new(e),
                };
                return Box::new(std::iter::once(Err(err)));
            }
        };

        if kind != crate::format::FormatKind::Json {
            let err = SingleIoError {
                stage: Stage::ResolveInput,
                target: spec.raw.clone(),
                error: Box::new(crate::format::FormatError::UnknownFormat(kind)),
            };
            return Box::new(std::iter::once(Err(err)));
        }

        // Open the input
        let reader = match spec.provider.open() {
            Ok(r) => r,
            Err(e) => {
                let err = SingleIoError {
                    stage: Stage::Open,
                    target: spec.raw.clone(),
                    error: Box::new(e),
                };
                return Box::new(std::iter::once(Err(err)));
            }
        };

        let target = spec.raw.clone();
        let iter = crate::format::deserialize_json_stream::<T, _>(reader).map(move |res| {
            res.map_err(|e| SingleIoError {
                stage: Stage::Parse,
                target: target.clone(),
                error: Box::new(e),
            })
        });

        Box::new(iter)
    }
}
