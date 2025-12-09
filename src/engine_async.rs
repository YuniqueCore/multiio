//! Asynchronous I/O engine for orchestrating async read and write operations.

use futures::stream::{self, BoxStream, StreamExt};
use serde::{Serialize, de::DeserializeOwned};
use tokio::io::AsyncReadExt;

use crate::config::{AsyncInputSpec, AsyncOutputSpec, FileExistsPolicy};
use crate::error::{AggregateError, ErrorPolicy, SingleIoError, Stage};
use crate::format::{self, AsyncFormatRegistry};

/// Asynchronous I/O engine for orchestrating multi-input/multi-output operations.
pub struct AsyncIoEngine {
    registry: AsyncFormatRegistry,
    error_policy: ErrorPolicy,
    inputs: Vec<AsyncInputSpec>,
    outputs: Vec<AsyncOutputSpec>,
}

impl AsyncIoEngine {
    /// Create a new async I/O engine.
    pub fn new(
        registry: AsyncFormatRegistry,
        error_policy: ErrorPolicy,
        inputs: Vec<AsyncInputSpec>,
        outputs: Vec<AsyncOutputSpec>,
    ) -> Self {
        Self {
            registry,
            error_policy,
            inputs,
            outputs,
        }
    }

    /// Get the format registry.
    pub fn registry(&self) -> &AsyncFormatRegistry {
        &self.registry
    }

    /// Get the error policy.
    pub fn error_policy(&self) -> ErrorPolicy {
        self.error_policy
    }

    /// Get the input specifications.
    pub fn inputs(&self) -> &[AsyncInputSpec] {
        &self.inputs
    }

    /// Get the output specifications.
    pub fn outputs(&self) -> &[AsyncOutputSpec] {
        &self.outputs
    }

    /// Read all inputs and deserialize each into type T.
    pub async fn read_all<T>(&self) -> Result<Vec<T>, AggregateError>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let mut results = Vec::new();
        let mut errors = Vec::new();

        for spec in &self.inputs {
            match self.read_one::<T>(spec).await {
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

    /// Read a single input asynchronously.
    async fn read_one<T>(&self, spec: &AsyncInputSpec) -> Result<T, SingleIoError>
    where
        T: DeserializeOwned + Send + 'static,
    {
        // Open the input stream
        let mut reader = spec.provider.open().await.map_err(|e| SingleIoError {
            stage: Stage::Open,
            target: spec.raw.clone(),
            error: Box::new(e),
        })?;

        // Read all bytes
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .await
            .map_err(|e| SingleIoError {
                stage: Stage::Open,
                target: spec.raw.clone(),
                error: Box::new(e),
            })?;

        // Resolve the format
        let kind = self
            .registry
            .resolve(spec.explicit_format.as_ref(), &spec.format_candidates)
            .map_err(|e| SingleIoError {
                stage: Stage::ResolveInput,
                target: spec.raw.clone(),
                error: Box::new(e),
            })?;

        // Deserialize
        format::deserialize_async::<T>(kind, &bytes)
            .await
            .map_err(|e| SingleIoError {
                stage: Stage::Parse,
                target: spec.raw.clone(),
                error: Box::new(e),
            })
    }

    /// Write values to all outputs asynchronously.
    pub async fn write_all<T>(&self, values: &[T]) -> Result<(), AggregateError>
    where
        T: Serialize + Sync,
    {
        let mut errors = Vec::new();

        for spec in &self.outputs {
            if let Err(e) = self.write_one(spec, values).await {
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
    pub async fn write_one_value<T>(&self, value: &T) -> Result<(), AggregateError>
    where
        T: Serialize + Sync,
    {
        let mut errors = Vec::new();

        for spec in &self.outputs {
            if let Err(e) = self.write_single(spec, value).await {
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
    async fn write_one<T>(&self, spec: &AsyncOutputSpec, values: &[T]) -> Result<(), SingleIoError>
    where
        T: Serialize + Sync,
    {
        // Resolve the format
        let kind = self
            .registry
            .resolve(spec.explicit_format.as_ref(), &spec.format_candidates)
            .map_err(|e| SingleIoError {
                stage: Stage::ResolveOutput,
                target: spec.raw.clone(),
                error: Box::new(e),
            })?;

        // Serialize to bytes
        let bytes = format::serialize_async(kind, &values)
            .await
            .map_err(|e| SingleIoError {
                stage: Stage::Serialize,
                target: spec.raw.clone(),
                error: Box::new(e),
            })?;

        // Open the output stream
        let mut writer = self.open_output(spec).await?;

        // Write bytes
        tokio::io::AsyncWriteExt::write_all(&mut *writer, &bytes)
            .await
            .map_err(|e| SingleIoError {
                stage: Stage::Serialize,
                target: spec.raw.clone(),
                error: Box::new(e),
            })
    }

    /// Write a single value to a specific output.
    async fn write_single<T>(&self, spec: &AsyncOutputSpec, value: &T) -> Result<(), SingleIoError>
    where
        T: Serialize + Sync,
    {
        // Resolve the format
        let kind = self
            .registry
            .resolve(spec.explicit_format.as_ref(), &spec.format_candidates)
            .map_err(|e| SingleIoError {
                stage: Stage::ResolveOutput,
                target: spec.raw.clone(),
                error: Box::new(e),
            })?;

        // Serialize to bytes
        let bytes = format::serialize_async(kind, value)
            .await
            .map_err(|e| SingleIoError {
                stage: Stage::Serialize,
                target: spec.raw.clone(),
                error: Box::new(e),
            })?;

        // Open the output stream
        let mut writer = self.open_output(spec).await?;

        // Write bytes
        tokio::io::AsyncWriteExt::write_all(&mut *writer, &bytes)
            .await
            .map_err(|e| SingleIoError {
                stage: Stage::Serialize,
                target: spec.raw.clone(),
                error: Box::new(e),
            })
    }

    /// Open an output based on the file exists policy.
    async fn open_output(
        &self,
        spec: &AsyncOutputSpec,
    ) -> Result<Box<dyn tokio::io::AsyncWrite + Unpin + Send>, SingleIoError> {
        let result = match spec.file_exists_policy {
            FileExistsPolicy::Overwrite => spec.target.open_overwrite().await,
            FileExistsPolicy::Append => spec.target.open_append().await,
            FileExistsPolicy::Error => spec.target.open_overwrite().await,
        };

        result.map_err(|e| SingleIoError {
            stage: Stage::Open,
            target: spec.raw.clone(),
            error: Box::new(e),
        })
    }

    /// Create a stream that reads inputs with bounded concurrency.
    ///
    /// Uses `buffer_unordered` to process multiple inputs concurrently.
    pub fn read_stream_async<T>(
        &self,
        concurrency: usize,
    ) -> BoxStream<'_, Result<T, SingleIoError>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let futs = self.inputs.iter().map(|spec| self.read_one::<T>(spec));
        stream::iter(futs).buffer_unordered(concurrency).boxed()
    }
}
