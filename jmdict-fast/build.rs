use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    // Without the 'embedded' feature, build.rs is a no-op
    if env::var("CARGO_FEATURE_EMBEDDED").is_err() {
        return;
    }

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir);

    let required_files = ["entries.bin", "kana.fst", "kanji.fst", "romaji.fst", "id.fst"];

    if required_files.iter().all(|f| out_path.join(f).exists()) {
        return;
    }

    let dist_dir = find_dist_dir().unwrap_or_else(|| {
        panic!(
            "jmdict-fast: 'embedded' feature is enabled but no `dist/` directory was found. \
             Run `cargo xtask generate` to produce data files, or download a release artifact."
        )
    });

    let missing: Vec<&&str> = required_files
        .iter()
        .filter(|f| !dist_dir.join(f).exists())
        .collect();
    if !missing.is_empty() {
        panic!(
            "jmdict-fast: 'embedded' feature is enabled but {} is missing required files: {:?}. \
             Run `cargo xtask generate` to regenerate.",
            dist_dir.display(),
            missing
        );
    }

    println!("cargo:rerun-if-changed={}", dist_dir.display());
    for file in &required_files {
        fs::copy(dist_dir.join(file), out_path.join(file)).unwrap_or_else(|e| {
            panic!(
                "Failed to copy {} from {} to OUT_DIR: {}",
                file,
                dist_dir.display(),
                e
            );
        });
    }
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
