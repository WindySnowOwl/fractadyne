//! The app-level error type. Unifies the crate errors (`GpuError`, `ExportError`) behind one
//! `?`-friendly enum, so the headless render/export paths return a typed failure instead of a
//! stringly `Result<_, String>` and the worker's cancel check is a pattern match rather than an
//! `e == "canceled"` string compare. `Display` is transparent, so status/log text is unchanged.
//!
//! Covers the render/export paths (`Gpu`/`Export`) and the `.kfr` location import (`Io`/`Parse`/
//! `Message`). `Display` is transparent for the `#[from]` sources, so status/log text is unchanged.

/// A failure in an app-level render / export / location-import operation.
#[derive(Debug, thiserror::Error)]
pub(crate) enum AppError {
    #[error(transparent)]
    Gpu(#[from] fractadyne_gpu::GpuError),
    #[error(transparent)]
    Export(#[from] fractadyne_export::ExportError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// A file didn't parse as the expected format (e.g. not a valid `.kfr` location).
    #[error("{0}")]
    Parse(String),
    /// A validation / precondition failure carrying a ready-to-show message.
    #[error("{0}")]
    Message(String),
}
