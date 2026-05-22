//! flutter_rust_bridge bindings for jmdict-fast.
//!
//! The public Dart-facing API lives in [`api`]. `flutter_rust_bridge_codegen`
//! scans that module to emit Rust glue (`src/frb_generated.rs` here) and
//! Dart bindings one directory up at `../lib/src/`. The generated Rust
//! file is committed (regenerated only on intentional API changes via
//! `flutter_rust_bridge_codegen generate`) so CI can build the crate from
//! a fresh checkout without running the tool — cargokit, which drives
//! cargo from inside the Flutter build, has no first-class hook for
//! "run codegen before compile."
//!
//! `mod frb_generated;` is gated on the `frb-generated` feature (default
//! ON). The published `flutter_package` inherits the default. Downstream
//! Rust callers that re-use the types in [`api`] but emit their own FRB
//! code (the in-repo demo at `example/rust/` does exactly that) must
//! depend with `default-features = false`, otherwise both crates emit
//! the same `frb_pde_ffi_dispatcher_*` symbols and the linker fails.

pub mod api;

#[cfg(feature = "frb-generated")]
#[allow(clippy::all)]
mod frb_generated;
