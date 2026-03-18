use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    // Without the 'embedded' feature, build.rs is a no-op
    if env::var("CARGO_FEATURE_EMBEDDED").is_err() {
        return;
    }

    // Re-run build script when dist/ contents change
    println!("cargo:rerun-if-changed=../dist");

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir);

    let required_files = ["entries.bin", "kana.fst", "kanji.fst", "romaji.fst", "id.fst"];

    // Check if generated files already exist in OUT_DIR - skip copy
    if required_files.iter().all(|f| out_path.join(f).exists()) {
        return;
    }

    // Look for pre-generated files in dist/ directory (from cargo xtask generate)
    if let Some(dist_dir) = find_dist_dir() {
        if required_files.iter().all(|f| dist_dir.join(f).exists()) {
            for file in &required_files {
                fs::copy(dist_dir.join(file), out_path.join(file)).unwrap_or_else(|e| {
                    panic!("Failed to copy {} from dist/ to OUT_DIR: {}", file, e);
                });
            }
            return;
        }
    }

    // Data files not found - emit a warning
    println!("cargo:warning=Data files not found. Run 'cargo xtask generate' first to generate dictionary data.");
}

/// Find the dist/ directory by walking up from CARGO_MANIFEST_DIR to the workspace root
fn find_dist_dir() -> Option<PathBuf> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").ok()?;
    let mut dir = PathBuf::from(&manifest_dir);

    loop {
        let dist_path = dir.join("dist");
        if dist_path.is_dir() {
            return Some(dist_path);
        }
        if !dir.pop() {
            break;
        }
    }

    None
}
