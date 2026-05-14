use jmdict_fast_ffi as facade;

/// Errors surfaced to Dart. flutter_rust_bridge maps `Result<T, Error>` into
/// a thrown Dart exception with the variant + payload preserved.
#[derive(Debug, Clone)]
pub enum Error {
    DataNotFound,
    DataVersionMismatch { expected: u32, found: u32 },
    DataCorrupted,
    InvalidQuery,
    Io { message: String },
    Deserialization,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Defer to the facade's user-facing strings so every FFI consumer
        // (bolt, frb, future uniffi) prints the same diagnostics.
        let facade_err: facade::Error = self.clone().into();
        std::fmt::Display::fmt(&facade_err, f)
    }
}

impl std::error::Error for Error {}

impl From<facade::Error> for Error {
    fn from(e: facade::Error) -> Self {
        match e {
            facade::Error::DataNotFound => Error::DataNotFound,
            facade::Error::DataVersionMismatch { expected, found } => {
                Error::DataVersionMismatch { expected, found }
            }
            facade::Error::DataCorrupted => Error::DataCorrupted,
            facade::Error::InvalidQuery => Error::InvalidQuery,
            facade::Error::Io { message } => Error::Io { message },
            facade::Error::Deserialization => Error::Deserialization,
        }
    }
}

impl From<Error> for facade::Error {
    fn from(e: Error) -> Self {
        match e {
            Error::DataNotFound => facade::Error::DataNotFound,
            Error::DataVersionMismatch { expected, found } => {
                facade::Error::DataVersionMismatch { expected, found }
            }
            Error::DataCorrupted => facade::Error::DataCorrupted,
            Error::InvalidQuery => facade::Error::InvalidQuery,
            Error::Io { message } => facade::Error::Io { message },
            Error::Deserialization => facade::Error::Deserialization,
        }
    }
}
