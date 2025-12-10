//! Asynchronous I/O engine for orchestrating async read and write operations.

use futures::stream::{self, BoxStream, StreamExt};
use serde::{Serialize, de::DeserializeOwned};
use tokio::io::AsyncReadExt;

use crate::config::{AsyncInputSpec, AsyncOutputSpec, FileExistsPolicy};
use crate::error::{AggregateError, ErrorPolicy, SingleIoError, Stage};
use crate::format::{self, AsyncFormatRegistry, FormatKind, FormatRegistry};

/// Asynchronous I/O engine for orchestrating multi-input/multi-output operations.
pub struct AsyncIoEngine {
    registry: AsyncFormatRegistry,
    sync_registry: Option<FormatRegistry>,
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
            sync_registry: None,
            error_policy,
            inputs,
            outputs,
        }
    }

    /// Create a new async I/O engine with an associated sync FormatRegistry.
    ///
    /// The sync registry can be used to resolve and stream custom formats (and
    /// also built-in formats) while the async registry continues to be used
    /// for feature gating and extension inference.
    pub fn new_with_sync_registry(
        registry: AsyncFormatRegistry,
        sync_registry: FormatRegistry,
        error_policy: ErrorPolicy,
        inputs: Vec<AsyncInputSpec>,
        outputs: Vec<AsyncOutputSpec>,
    ) -> Self {
        Self {
            registry,
            sync_registry: Some(sync_registry),
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
        let mut results = Vec::with_capacity(self.inputs.len());
        let mut errors = Vec::new();
        let mut buffer = Vec::new();

        for spec in &self.inputs {
            match self.read_one_with_buffer::<T>(spec, &mut buffer).await {
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

    /// Stream records from all inputs using their resolved formats.
    ///
    /// This mirrors the synchronous `IoEngine::read_records` API but for async
    /// inputs. Each input is read asynchronously into memory and then
    /// deserialized using format-specific streaming implementations when
    /// available (e.g. JSON NDJSON, CSV rows, YAML multi-doc, plaintext
    /// line-based). For formats without streaming support this falls back to a
    /// single-item deserialization.
    pub fn read_records_async<T>(
        &self,
        concurrency: usize,
    ) -> BoxStream<'_, Result<T, SingleIoError>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let futs = self
            .inputs
            .iter()
            .map(|spec| self.records_stream_for_spec_async::<T>(spec));

        stream::iter(futs)
            .buffer_unordered(concurrency)
            .flat_map(|s| s)
            .boxed()
    }

    /// Read a single input asynchronously.
    async fn read_one<T>(&self, spec: &AsyncInputSpec) -> Result<T, SingleIoError>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let mut buffer = Vec::new();
        self.read_one_with_buffer::<T>(spec, &mut buffer).await
    }

    async fn read_one_with_buffer<T>(
        &self,
        spec: &AsyncInputSpec,
        buffer: &mut Vec<u8>,
    ) -> Result<T, SingleIoError>
    where
        T: DeserializeOwned + Send + 'static,
    {
        // Open the input stream
        let mut reader = spec.provider.open().await.map_err(|e| SingleIoError {
            stage: Stage::Open,
            target: spec.raw.clone(),
            error: Box::new(e),
        })?;

        // Read all bytes into the reusable buffer
        buffer.clear();
        reader
            .read_to_end(buffer)
            .await
            .map_err(|e| SingleIoError {
                stage: Stage::Open,
                target: spec.raw.clone(),
                error: Box::new(e),
            })?;

        // If a sync registry is available, delegate resolution and
        // deserialization to it so that custom formats participate fully in
        // decoding. Otherwise, fall back to the async-format helpers.
        if let Some(sync_registry) = &self.sync_registry {
            match sync_registry.deserialize_value::<T>(
                spec.explicit_format.as_ref(),
                &spec.format_candidates,
                buffer,
            ) {
                Ok(value) => Ok(value),
                Err(e) => {
                    let stage = match e {
                        format::FormatError::UnknownFormat(_)
                        | format::FormatError::NoFormatMatched
                        | format::FormatError::NotEnabled(_) => Stage::ResolveInput,
                        _ => Stage::Parse,
                    };

                    Err(SingleIoError {
                        stage,
                        target: spec.raw.clone(),
                        error: Box::new(e),
                    })
                }
            }
        } else {
            // Resolve the format using the async registry and fall back to the
            // existing async-format helpers.
            let kind = self
                .registry
                .resolve(spec.explicit_format.as_ref(), &spec.format_candidates)
                .map_err(|e| SingleIoError {
                    stage: Stage::ResolveInput,
                    target: spec.raw.clone(),
                    error: Box::new(e),
                })?;

            // Deserialize
            format::deserialize_async::<T>(kind, buffer)
                .await
                .map_err(|e| SingleIoError {
                    stage: Stage::Parse,
                    target: spec.raw.clone(),
                    error: Box::new(e),
                })
        }
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
        // If a sync registry is available, delegate resolution and
        // serialization to it so that custom formats participate fully in
        // encoding. Otherwise, fall back to the async-format helpers.
        let bytes = if let Some(sync_registry) = &self.sync_registry {
            match sync_registry.serialize_value(
                spec.explicit_format.as_ref(),
                &spec.format_candidates,
                &values,
            ) {
                Ok(bytes) => bytes,
                Err(e) => {
                    let stage = match e {
                        format::FormatError::UnknownFormat(_)
                        | format::FormatError::NoFormatMatched
                        | format::FormatError::NotEnabled(_) => Stage::ResolveOutput,
                        _ => Stage::Serialize,
                    };

                    return Err(SingleIoError {
                        stage,
                        target: spec.raw.clone(),
                        error: Box::new(e),
                    });
                }
            }
        } else {
            let kind = self.resolve_output_kind(spec)?;

            format::serialize_async(kind, &values)
                .await
                .map_err(|e| SingleIoError {
                    stage: Stage::Serialize,
                    target: spec.raw.clone(),
                    error: Box::new(e),
                })?
        };

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
        // If a sync registry is available, delegate resolution and
        // serialization to it so that custom formats participate fully in
        // encoding. Otherwise, fall back to the async-format helpers.
        let bytes = if let Some(sync_registry) = &self.sync_registry {
            match sync_registry.serialize_value(
                spec.explicit_format.as_ref(),
                &spec.format_candidates,
                value,
            ) {
                Ok(bytes) => bytes,
                Err(e) => {
                    let stage = match e {
                        format::FormatError::UnknownFormat(_)
                        | format::FormatError::NoFormatMatched
                        | format::FormatError::NotEnabled(_) => Stage::ResolveOutput,
                        _ => Stage::Serialize,
                    };

                    return Err(SingleIoError {
                        stage,
                        target: spec.raw.clone(),
                        error: Box::new(e),
                    });
                }
            }
        } else {
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
            format::serialize_async(kind, value)
                .await
                .map_err(|e| SingleIoError {
                    stage: Stage::Serialize,
                    target: spec.raw.clone(),
                    error: Box::new(e),
                })?
        };

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

    fn resolve_output_kind(&self, spec: &AsyncOutputSpec) -> Result<FormatKind, SingleIoError> {
        self.registry
            .resolve(spec.explicit_format.as_ref(), &spec.format_candidates)
            .map_err(|e| SingleIoError {
                stage: Stage::ResolveOutput,
                target: spec.raw.clone(),
                error: Box::new(e),
            })
    }

    /// Create a per-input stream of records for the given spec.
    ///
    /// This helper reads the entire async input into memory and then uses the
    /// same format-specific streaming helpers as the sync engine where
    /// possible. It is primarily a convenience wrapper around the synchronous
    /// streaming deserializers for use in async contexts.
    async fn records_stream_for_spec_async<'a, T>(
        &'a self,
        spec: &'a AsyncInputSpec,
    ) -> BoxStream<'a, Result<T, SingleIoError>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        // Open the input stream
        let mut reader = match spec.provider.open().await {
            Ok(r) => r,
            Err(e) => {
                let err = SingleIoError {
                    stage: Stage::Open,
                    target: spec.raw.clone(),
                    error: Box::new(e),
                };
                return stream::iter(std::iter::once(Err(err))).boxed();
            }
        };

        // Read all bytes into an internal buffer
        let mut buffer = Vec::new();
        if let Err(e) = reader.read_to_end(&mut buffer).await {
            let err = SingleIoError {
                stage: Stage::Open,
                target: spec.raw.clone(),
                error: Box::new(e),
            };
            return stream::iter(std::iter::once(Err(err))).boxed();
        }

        // Resolve the format. If a sync registry is available we use it so
        // that custom formats (and their streaming handlers) participate in
        // resolution; otherwise we fall back to the async registry. Resolution
        // failures are treated as parse-stage errors, mirroring the sync
        // `read_records` behavior where resolution happens inside the format
        // registry.
        let kind = if let Some(sync_registry) = &self.sync_registry {
            match sync_registry.resolve(spec.explicit_format.as_ref(), &spec.format_candidates) {
                Ok(k) => k,
                Err(e) => {
                    let err = SingleIoError {
                        stage: Stage::Parse,
                        target: spec.raw.clone(),
                        error: Box::new(e),
                    };
                    return stream::iter(std::iter::once(Err(err))).boxed();
                }
            }
        } else {
            match self
                .registry
                .resolve(spec.explicit_format.as_ref(), &spec.format_candidates)
            {
                Ok(k) => k,
                Err(e) => {
                    let err = SingleIoError {
                        stage: Stage::Parse,
                        target: spec.raw.clone(),
                        error: Box::new(e),
                    };
                    return stream::iter(std::iter::once(Err(err))).boxed();
                }
            }
        };

        let target = spec.raw.clone();

        // Use format-specific streaming helpers where available.
        if let FormatKind::Json = kind {
            #[cfg(feature = "json")]
            {
                let reader = std::io::Cursor::new(buffer);
                let iter = crate::format::deserialize_json_stream::<T, _>(reader);
                return Self::iter_to_stream(iter, target);
            }
            #[cfg(not(feature = "json"))]
            {
                let err = SingleIoError {
                    stage: Stage::Parse,
                    target,
                    error: Box::new(crate::format::FormatError::NotEnabled(kind)),
                };
                return stream::iter(std::iter::once(Err(err))).boxed();
            }
        }

        // If we have a sync registry and the resolved kind is a custom format,
        // bridge to the sync FormatRegistry's streaming implementation. This
        // supports custom streaming handlers and falls back to non-streaming
        // single-item deserialization when no streaming handler is registered.
        if let (Some(sync_registry), FormatKind::Custom(_)) = (&self.sync_registry, kind) {
            use std::io::Cursor;

            let reader: Box<dyn std::io::Read> = Box::new(Cursor::new(buffer));
            let iter_result = sync_registry.stream_deserialize_into::<T>(Some(&kind), &[], reader);

            let target_for_stream = target.clone();

            let iter = match iter_result {
                Ok(iter) => iter,
                Err(e) => {
                    let err = SingleIoError {
                        stage: Stage::Parse,
                        target,
                        error: Box::new(e),
                    };
                    return stream::iter(std::iter::once(Err(err))).boxed();
                }
            };

            // The returned iterator may not be Send; collect into a Vec before
            // turning it into a stream so that the resulting iterator is Send.
            let collected: Vec<Result<T, format::FormatError>> = iter.collect();
            return Self::iter_to_stream(collected.into_iter(), target_for_stream);
        }

        if let FormatKind::Csv = kind {
            #[cfg(feature = "csv")]
            {
                let reader = std::io::Cursor::new(buffer);
                let iter = crate::format::deserialize_csv_stream::<T, _>(reader);
                return Self::iter_to_stream(iter, target);
            }
            #[cfg(not(feature = "csv"))]
            {
                let err = SingleIoError {
                    stage: Stage::Parse,
                    target,
                    error: Box::new(crate::format::FormatError::NotEnabled(kind)),
                };
                return stream::iter(std::iter::once(Err(err))).boxed();
            }
        }

        if let FormatKind::Yaml = kind {
            #[cfg(feature = "yaml")]
            {
                let reader = std::io::Cursor::new(buffer);
                let iter = crate::format::deserialize_yaml_stream::<T, _>(reader);
                let collected: Vec<_> = iter.collect();
                return Self::iter_to_stream(collected.into_iter(), target);
            }
            #[cfg(not(feature = "yaml"))]
            {
                let err = SingleIoError {
                    stage: Stage::Parse,
                    target,
                    error: Box::new(crate::format::FormatError::NotEnabled(kind)),
                };
                return stream::iter(std::iter::once(Err(err))).boxed();
            }
        }

        if let FormatKind::Plaintext = kind {
            #[cfg(feature = "plaintext")]
            {
                let reader = std::io::Cursor::new(buffer);
                let iter = crate::format::deserialize_plaintext_stream::<T, _>(reader);
                return Self::iter_to_stream(iter, target);
            }
            #[cfg(not(feature = "plaintext"))]
            {
                let err = SingleIoError {
                    stage: Stage::Parse,
                    target,
                    error: Box::new(crate::format::FormatError::NotEnabled(kind)),
                };
                return stream::iter(std::iter::once(Err(err))).boxed();
            }
        }

        // Other formats (including unsupported/custom): fall back to
        // non-streaming single-item deserialization.
        let value = format::deserialize_async::<T>(kind, &buffer).await;
        let result = value.map_err(|e| SingleIoError {
            stage: Stage::Parse,
            target,
            error: Box::new(e),
        });

        stream::iter(std::iter::once(result)).boxed()
    }

    fn iter_to_stream<T, I>(iter: I, target: String) -> BoxStream<'static, Result<T, SingleIoError>>
    where
        T: DeserializeOwned + Send + 'static,
        I: Iterator<Item = Result<T, format::FormatError>> + Send + 'static,
    {
        let mapped = iter.map(move |res| {
            res.map_err(|e| SingleIoError {
                stage: Stage::Parse,
                target: target.clone(),
                error: Box::new(e),
            })
        });

        stream::iter(mapped).boxed()
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
