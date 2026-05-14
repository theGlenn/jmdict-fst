//! The Dart-facing surface scanned by `flutter_rust_bridge_codegen`.
//!
//! Layout follows FRB v2 conventions:
//! - Plain Rust structs without macros become **Dart classes** with `final`
//!   fields. We mirror the facade's records here because FRB scans this
//!   crate; cross-crate scanning is fragile and the mirroring is mechanical.
//! - `Dict` holds `Arc<facade::Dict>` and lives behind FRB's opaque-handle
//!   machinery — Dart receives an opaque object pointer and invokes methods
//!   through the generated glue.
//! - `Result<T, Error>` translates to a Dart exception.
//!
//! Everything substantive (FST lookups, deinflection, scoring) lives in
//! `jmdict-fast` via the `jmdict-fast-ffi` facade; this module is a thin
//! shape adapter.

pub mod dictionary;
pub mod error;
pub mod install;
pub mod model;

pub use dictionary::*;
pub use error::*;
pub use install::*;
pub use model::*;
