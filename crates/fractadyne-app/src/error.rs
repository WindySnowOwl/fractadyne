//! The app-level error type. Unifies the crate errors (`GpuError`, `ExportError`) behind one
//! `?`-friendly enum, so the headless render/export paths return a typed failure instead of a
//! stringly `Result<_, String>` and the worker's cancel check is a pattern match rather than an
//! `e == "canceled"` string compare. `Display` is transparent, so status/log text is unchanged.
//!
//! Only the variants the export path actually constructs live here; later slices that adopt
//! `AppError` in the `.kfr`/settings paths will add `Io` / `Parse` / `Message`.

/// A failure in an app-level render/export operation.
#[derive(Debug, thiserror::Error)]
pub(crate) enum AppError {
    #[error(transparent)]
    Gpu(#[from] fractadyne_gpu::GpuError),
    #[error(transparent)]
    Export(#[from] fractadyne_export::ExportError),
}
