use std::fmt;

/// Typed error enum for jmdict-fast, suitable for FFI consumers.
#[derive(Debug)]
pub enum JmdictError {
    /// Data files not found at the expected path.
    DataNotFound,
    /// Binary format version mismatch between data and library.
    DataVersionMismatch { expected: u32, found: u32 },
    /// Data files are corrupted or have an invalid format.
    DataCorrupted,
    /// The query was invalid (e.g., empty string).
    InvalidQuery,
    /// An I/O error occurred while reading data files.
    IoError(std::io::Error),
    /// Failed to deserialize entry data.
    DeserializationError,
    /// `Dict::install*` requires an explicit cache directory on this
    /// platform (typically iOS / Android / WASM, where the OS doesn't
    /// expose a sensible writable default to native code). The host must
    /// call `init_sdk_cache_dir(...)` or pass `InstallOptions::cache_dir(...)`.
    #[cfg(feature = "install")]
    CacheDirRequired { platform: &'static str },
    /// A network request inside `Dict::install*` failed.
    #[cfg(feature = "install")]
    NetworkError(String),
    /// [`crate::install::init_sdk_cache_dir`] was called more than once.
    /// The cache root is process-global and first-set-wins, so a second
    /// call is rejected rather than silently leaving previously-loaded
    /// `Dict`s pointing at a different root than future installs.
    #[cfg(feature = "install")]
    CacheDirAlreadySet,
}

impl JmdictError {
    /// Returns a stable numeric code for each error variant, useful for FFI.
    pub fn code(&self) -> u32 {
        match self {
            JmdictError::DataNotFound => 1,
            JmdictError::DataVersionMismatch { .. } => 2,
            JmdictError::DataCorrupted => 3,
            JmdictError::InvalidQuery => 4,
            JmdictError::IoError(_) => 5,
            JmdictError::DeserializationError => 6,
            #[cfg(feature = "install")]
            JmdictError::CacheDirRequired { .. } => 7,
            #[cfg(feature = "install")]
            JmdictError::NetworkError(_) => 8,
            #[cfg(feature = "install")]
            JmdictError::CacheDirAlreadySet => 9,
        }
    }
}

impl fmt::Display for JmdictError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JmdictError::DataNotFound => {
                write!(f, "Dictionary data files not found. Run `cargo xtask generate` or set JMDICT_DATA env var.")
            }
            JmdictError::DataVersionMismatch { expected, found } => {
                write!(
                    f,
                    "Data format version {found}, library expects {expected}. Regenerate with cargo xtask generate."
                )
            }
            JmdictError::DataCorrupted => {
                write!(f, "Dictionary data is corrupted or has an invalid format.")
            }
            JmdictError::InvalidQuery => {
                write!(f, "The search query is invalid.")
            }
            JmdictError::IoError(err) => {
                write!(f, "I/O error: {err}")
            }
            JmdictError::DeserializationError => {
                write!(f, "Failed to deserialize dictionary entry data.")
            }
            #[cfg(feature = "install")]
            JmdictError::CacheDirRequired { platform } => {
                write!(
                    f,
                    "Cache directory required on {platform}: this platform has no portable default. \
                     Call `jmdict_fast::install::init_sdk_cache_dir(path)` from the host (e.g. via \
                     path_provider on Flutter, Application.getCacheDir on Android, \
                     FileManager URLs on iOS), or pass `InstallOptions::cache_dir(...)` per call."
                )
            }
            #[cfg(feature = "install")]
            JmdictError::NetworkError(msg) => write!(f, "Network error during install: {msg}"),
            #[cfg(feature = "install")]
            JmdictError::CacheDirAlreadySet => write!(
                f,
                "init_sdk_cache_dir was already called for this process; the cache root is one-shot."
            ),
        }
    }
}

impl std::error::Error for JmdictError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            JmdictError::IoError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for JmdictError {
    fn from(err: std::io::Error) -> Self {
        // Map NotFound to DataNotFound for better semantics
        if err.kind() == std::io::ErrorKind::NotFound {
            JmdictError::DataNotFound
        } else {
            JmdictError::IoError(err)
        }
    }
}

impl From<fst::Error> for JmdictError {
    fn from(_: fst::Error) -> Self {
        JmdictError::DataCorrupted
    }
}
