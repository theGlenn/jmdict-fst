//! Integration tests for the `install` feature.
//!
//! These need `dist/` (or `JMDICT_DATA`) to source raw files, but they do
//! NOT touch the network — the test builds a tarball locally and installs
//! from it. That keeps CI deterministic while still exercising the real
//! extract → write → load pipeline.

#![cfg(feature = "install")]

use flate2::write::GzEncoder;
use flate2::Compression;
use jmdict_fast::install::{InstallOptions, InstallSource};
use jmdict_fast::Dict;
use std::io::Write;
use std::path::{Path, PathBuf};

const REQUIRED_FILES: &[&str] = &[
    "entries.bin",
    "kana.fst",
    "kanji.fst",
    "romaji.fst",
    "id.fst",
    "gloss.fst",
    "gloss_postings.bin",
];

fn dist_dir() -> PathBuf {
    if let Ok(p) = std::env::var("JMDICT_DATA") {
        return PathBuf::from(p);
    }
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../dist")
}

/// Build a `.tar.gz` from `dist/` containing the seven required files.
fn build_tarball() -> Vec<u8> {
    let dist = dist_dir();
    let mut tar_buf = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut tar_buf);
        for name in REQUIRED_FILES {
            let src = dist.join(name);
            let bytes = std::fs::read(&src)
                .unwrap_or_else(|e| panic!("read {} failed: {e}", src.display()));
            let mut header = tar::Header::new_gnu();
            header.set_path(name).unwrap();
            header.set_size(bytes.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append(&header, &bytes[..]).unwrap();
        }
        builder.finish().unwrap();
    }
    let mut gz = GzEncoder::new(Vec::new(), Compression::fast());
    gz.write_all(&tar_buf).unwrap();
    gz.finish().unwrap()
}

#[test]
fn install_from_local_tarball_round_trips() {
    let cache = tempfile::tempdir().unwrap();
    let tarball_path = cache.path().join("data.tar.gz");
    std::fs::write(&tarball_path, build_tarball()).unwrap();

    let dict = Dict::install_with(
        InstallOptions::default()
            .cache_dir(cache.path())
            .source(InstallSource::Tarball(tarball_path)),
    )
    .expect("install failed");

    // The whole point: the installed dict actually answers a real query.
    let results = dict.lookup_exact("猫");
    assert!(!results.is_empty(), "installed dict couldn't find 猫");
}

#[test]
fn install_skips_extract_on_warm_cache() {
    let cache = tempfile::tempdir().unwrap();
    let tarball_path = cache.path().join("data.tar.gz");
    std::fs::write(&tarball_path, build_tarball()).unwrap();

    let opts = || {
        InstallOptions::default()
            .cache_dir(cache.path())
            .source(InstallSource::Tarball(tarball_path.clone()))
    };

    Dict::install_with(opts()).expect("first install failed");

    // Touch the entries.bin so we can tell whether the second install
    // overwrote it. If extract ran again, mtime would advance.
    let installed_dir = cache.path().join("jmdict-fast").join("fmt4").join("3.6.1");
    let entries = installed_dir.join("entries.bin");
    let before = std::fs::metadata(&entries).unwrap().modified().unwrap();

    // Re-run; should be a no-op on the extract side.
    Dict::install_with(opts()).expect("second install failed");
    let after = std::fs::metadata(&entries).unwrap().modified().unwrap();

    assert_eq!(before, after, "warm install re-extracted entries.bin");
}

#[test]
fn install_force_overwrites_corrupted_file() {
    let cache = tempfile::tempdir().unwrap();
    let tarball_path = cache.path().join("data.tar.gz");
    std::fs::write(&tarball_path, build_tarball()).unwrap();

    let opts = |force| {
        InstallOptions::default()
            .cache_dir(cache.path())
            .source(InstallSource::Tarball(tarball_path.clone()))
            .force(force)
    };

    Dict::install_with(opts(false)).expect("first install failed");

    // Corrupt one of the installed files. A warm cache will reuse it
    // (and Dict::load will reject it); force=true must rewrite it.
    let installed_dir = cache.path().join("jmdict-fast").join("fmt4").join("3.6.1");
    let entries = installed_dir.join("entries.bin");
    std::fs::write(&entries, b"not a valid entries.bin").unwrap();

    let dict = Dict::install_with(opts(true)).expect("forced install failed");
    assert!(dict.entry_count() > 0, "forced reinstall should produce a working dict");
}

#[test]
fn install_rejects_tarball_missing_required_files() {
    // A tarball with only some of the seven required files must fail
    // rather than load a half-installed Dict.
    let cache = tempfile::tempdir().unwrap();
    let tarball_path = cache.path().join("partial.tar.gz");

    let mut tar_buf = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut tar_buf);
        let entries = std::fs::read(dist_dir().join("entries.bin")).unwrap();
        let mut h = tar::Header::new_gnu();
        h.set_path("entries.bin").unwrap();
        h.set_size(entries.len() as u64);
        h.set_mode(0o644);
        h.set_cksum();
        builder.append(&h, &entries[..]).unwrap();
        builder.finish().unwrap();
    }
    let mut gz = GzEncoder::new(Vec::new(), Compression::fast());
    gz.write_all(&tar_buf).unwrap();
    std::fs::write(&tarball_path, gz.finish().unwrap()).unwrap();

    let err = Dict::install_with(
        InstallOptions::default()
            .cache_dir(cache.path())
            .source(InstallSource::Tarball(tarball_path)),
    );
    assert!(err.is_err(), "partial tarball should not produce a Dict");
}
