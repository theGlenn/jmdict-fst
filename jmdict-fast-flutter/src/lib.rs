//! flutter_rust_bridge bindings for jmdict-fast.
//!
//! The public Dart-facing API lives in [`api`]. `flutter_rust_bridge_codegen`
//! scans that module to emit Rust glue (`src/frb_generated.rs`) and Dart
//! bindings (`dart/lib/`). After the first codegen run, add:
//!
//! ```ignore
//! mod frb_generated;
//! ```
//!
//! below this comment. We omit it from the committed scaffold so the crate
//! compiles before anyone runs the codegen tool.

pub mod api;
