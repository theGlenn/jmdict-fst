//! Download + extract helpers used by `install_with`.

use crate::error::JmdictError;
use crate::install::REQUIRED_FILES;
use flate2::read::GzDecoder;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use tar::Archive;

/// Maximum tarball size accepted by `download` (50 MB). The current fmt4
/// blob is ~24 MB; this caps a runaway redirect or replaced asset from
/// ballooning memory before we even look at the content.
const MAX_DOWNLOAD_BYTES: usize = 50 * 1024 * 1024;

/// Maximum declared size of any single tar entry. Matches
/// `MAX_DOWNLOAD_BYTES`; a tarball can't legitimately contain a file
/// larger than itself, and oversize headers are rejected *before* the
/// copy so we never produce a half-written, silently-truncated file.
const MAX_ENTRY_BYTES: u64 = MAX_DOWNLOAD_BYTES as u64;

/// Blocking HTTPS GET → bytes. Uses `ureq` so we don't drag tokio into the
/// crate; install is a one-shot operation and threading is the caller's
/// problem (FFI consumers already run it on a worker thread).
pub(crate) fn download(url: &str) -> Result<Vec<u8>, JmdictError> {
    let resp = ureq::get(url)
        .call()
        .map_err(|e| JmdictError::NetworkError(e.to_string()))?;

    let mut buf = Vec::new();
    resp.into_reader()
        .take((MAX_DOWNLOAD_BYTES + 1) as u64)
        .read_to_end(&mut buf)
        .map_err(JmdictError::IoError)?;

    if buf.len() > MAX_DOWNLOAD_BYTES {
        return Err(JmdictError::NetworkError(format!(
            "tarball exceeds {MAX_DOWNLOAD_BYTES} byte limit"
        )));
    }
    Ok(buf)
}

/// Extract a `.tar.gz` byte buffer into `target`. Only the seven runtime
/// files are written; anything else in the archive is silently skipped so
/// a tarball with an extra README or .DS_Store still installs cleanly.
pub(crate) fn extract(bytes: &[u8], target: &Path) -> Result<(), JmdictError> {
    let gz = GzDecoder::new(Cursor::new(bytes));
    extract_archive(Archive::new(gz), target)
}

/// Same as [`extract`] but reads the tarball from disk — used by
/// [`crate::Dict::install_from_tarball`] so the caller doesn't have to
/// slurp the file themselves.
pub(crate) fn extract_from_path(path: &Path, target: &Path) -> Result<(), JmdictError> {
    let file = std::fs::File::open(path).map_err(JmdictError::from)?;
    let gz = GzDecoder::new(file);
    extract_archive(Archive::new(gz), target)
}

fn extract_archive<R: Read>(mut archive: Archive<R>, target: &Path) -> Result<(), JmdictError> {
    // `tar` exposes io::Result everywhere, so `?` lets `From<io::Error>`
    // map NotFound → DataNotFound and the rest to IoError — accurate for
    // both the network and the local-tarball entry points, instead of
    // misattributing a local read failure as NetworkError.
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.into_owned();

        // Strip to basename. The workflow ships files at the archive
        // root, but historical or mirror tarballs sometimes wrap them
        // in a top-level dir; accepting the basename makes both layouts
        // work. As a side-effect this neutralizes any `../` traversal a
        // hostile archive could try.
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !REQUIRED_FILES.contains(&name) {
            continue;
        }

        // Check the *declared* size from the tar header before copying.
        // Using `Read::take(limit)` instead would silently truncate an
        // oversize entry to `limit` bytes and produce a corrupted file
        // that only fails much later at `Dict::load`. Better to refuse
        // the install up front.
        if entry.size() > MAX_ENTRY_BYTES {
            return Err(JmdictError::DataCorrupted);
        }

        let dest: PathBuf = target.join(name);
        let mut out = std::fs::File::create(&dest)?;
        std::io::copy(&mut entry, &mut out)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    /// Build a tar.gz in memory from `(name, bytes)` pairs. Used by the
    /// tests below to avoid network access and to exercise odd layouts
    /// (extra files, nested directories, symlinks).
    fn make_targz(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut tar_buf = Vec::new();
        {
            let mut builder = tar::Builder::new(&mut tar_buf);
            for (name, bytes) in entries {
                let mut header = tar::Header::new_gnu();
                header.set_path(name).unwrap();
                header.set_size(bytes.len() as u64);
                header.set_mode(0o644);
                header.set_cksum();
                builder.append(&header, *bytes).unwrap();
            }
            builder.finish().unwrap();
        }
        let mut gz = GzEncoder::new(Vec::new(), Compression::fast());
        gz.write_all(&tar_buf).unwrap();
        gz.finish().unwrap()
    }

    #[test]
    fn extract_writes_only_required_files() {
        let tmp = tempfile::tempdir().unwrap();
        let tarball = make_targz(&[
            ("entries.bin", b"e"),
            ("README.md", b"ignore me"),
            ("kana.fst", b"k"),
        ]);
        extract(&tarball, tmp.path()).unwrap();

        assert!(tmp.path().join("entries.bin").exists());
        assert!(tmp.path().join("kana.fst").exists());
        assert!(!tmp.path().join("README.md").exists());
    }

    #[test]
    fn extract_tolerates_nested_directory_layout() {
        // Some mirrors wrap files in a top-level dir like `dist/`. The
        // extractor strips to file_name() so both layouts work.
        let tmp = tempfile::tempdir().unwrap();
        let tarball = make_targz(&[
            ("dist/entries.bin", b"e"),
            ("dist/kana.fst", b"k"),
        ]);
        extract(&tarball, tmp.path()).unwrap();

        assert!(tmp.path().join("entries.bin").exists());
        assert!(tmp.path().join("kana.fst").exists());
    }
}
