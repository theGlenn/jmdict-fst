//! flutter_rust_bridge bindings for jmdict-fast.
//!
//! The public Dart-facing API lives in [`api`]. `flutter_rust_bridge_codegen`
//! scans that module to emit Rust glue (`src/frb_generated.rs`) and Dart
//! bindings under `flutter_package/lib/src/`.
//!
//! `src/frb_generated.rs` is only compiled into the build when the
//! `frb-generated` feature is on — the published `flutter_package`
//! enables it via cargokit. Downstream Rust callers that re-use the
//! types in [`api`] but emit their own FRB code (the in-repo demo at
//! `example/rust/` does exactly that) must keep this feature off,
//! otherwise both crates define the same `frb_pde_ffi_dispatcher_*`
//! symbols and the linker fails.

pub mod api;

#[cfg(feature = "frb-generated")]
#[allow(clippy::all)]
mod frb_generated;
