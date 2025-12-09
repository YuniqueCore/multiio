//! Synchronous I/O engine for orchestrating read and write operations.

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
        let mut results = Vec::new();
        let mut errors = Vec::new();

        for spec in &self.inputs {
            match self.read_one::<T>(spec) {
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
        // Open the input stream
        let mut reader = spec.provider.open().map_err(|e| SingleIoError {
            stage: Stage::Open,
            target: spec.raw.clone(),
            error: Box::new(e),
        })?;

        // Resolve the format
        let fmt = self
            .registry
            .resolve(spec.explicit_format.as_ref(), &spec.format_candidates)
            .map_err(|e| SingleIoError {
                stage: Stage::ResolveInput,
                target: spec.raw.clone(),
                error: Box::new(e),
            })?;

        // Deserialize
        fmt.deserialize::<T>(&mut *reader)
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
        // Resolve the format
        let fmt = self
            .registry
            .resolve(spec.explicit_format.as_ref(), &spec.format_candidates)
            .map_err(|e| SingleIoError {
                stage: Stage::ResolveOutput,
                target: spec.raw.clone(),
                error: Box::new(e),
            })?;

        // Open the output stream based on policy
        let mut writer = self.open_output(spec)?;

        // Serialize
        fmt.serialize(&values, &mut *writer)
            .map_err(|e| SingleIoError {
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
        // Resolve the format
        let fmt = self
            .registry
            .resolve(spec.explicit_format.as_ref(), &spec.format_candidates)
            .map_err(|e| SingleIoError {
                stage: Stage::ResolveOutput,
                target: spec.raw.clone(),
                error: Box::new(e),
            })?;

        // Open the output stream based on policy
        let mut writer = self.open_output(spec)?;

        // Serialize
        fmt.serialize(value, &mut *writer)
            .map_err(|e| SingleIoError {
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
}
