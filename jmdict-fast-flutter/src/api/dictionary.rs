use std::sync::Arc;

use jmdict_fast_ffi as facade;

use super::error::Error;
use super::model::{
    BatchResult, DataVersion, Entry, LookupResult, QueryOptions, Xref,
};

/// FFI-exported dictionary handle.
///
/// flutter_rust_bridge detects that `Dict` carries a non-`Clone`, non-record
/// field (`Arc<facade::Dict>`) and treats it as an *opaque* type: Dart
/// receives an opaque pointer and invokes methods via the generated glue.
/// The handle is shareable across isolates because `Arc<facade::Dict>` is
/// `Send + Sync`.
pub struct Dict {
    inner: Arc<facade::Dict>,
}

impl Dict {
    pub fn load(path: String) -> Result<Self, Error> {
        let inner = facade::Dict::load(path)?;
        Ok(Self { inner })
    }

    pub fn load_default() -> Result<Self, Error> {
        let inner = facade::Dict::load_default()?;
        Ok(Self { inner })
    }

    pub fn entry_count(&self) -> u64 {
        self.inner.entry_count()
    }

    pub fn version(&self) -> DataVersion {
        self.inner.version().into()
    }

    pub fn lookup_exact(&self, term: String) -> Vec<LookupResult> {
        self.inner
            .lookup_exact(term)
            .into_iter()
            .map(Into::into)
            .collect()
    }

    pub fn lookup_partial(&self, prefix: String) -> Vec<LookupResult> {
        self.inner
            .lookup_partial(prefix)
            .into_iter()
            .map(Into::into)
            .collect()
    }

    pub fn lookup_exact_with_deinflection(&self, term: String) -> Vec<LookupResult> {
        self.inner
            .lookup_exact_with_deinflection(term)
            .into_iter()
            .map(Into::into)
            .collect()
    }

    pub fn lookup_gloss(&self, query: String) -> Vec<LookupResult> {
        self.inner
            .lookup_gloss(query)
            .into_iter()
            .map(Into::into)
            .collect()
    }

    pub fn lookup_by_id(&self, jmdict_id: String) -> Option<LookupResult> {
        self.inner.lookup_by_id(jmdict_id).map(Into::into)
    }

    pub fn lookup_with_options(
        &self,
        term: String,
        options: QueryOptions,
    ) -> Result<Vec<LookupResult>, Error> {
        let results = self.inner.lookup_with_options(term, options.into())?;
        Ok(results.into_iter().map(Into::into).collect())
    }

    pub fn lookup_batch(
        &self,
        terms: Vec<String>,
        options: QueryOptions,
    ) -> Result<Vec<BatchResult>, Error> {
        let results = self.inner.lookup_batch(terms, options.into())?;
        Ok(results.into_iter().map(Into::into).collect())
    }

    pub fn resolve_xref(&self, xref: Xref) -> Vec<LookupResult> {
        self.inner
            .resolve_xref(xref.into())
            .into_iter()
            .map(Into::into)
            .collect()
    }

    pub fn get(&self, seq_id: u64) -> Option<Entry> {
        self.inner.get(seq_id).map(Into::into)
    }

    pub fn iter_entries(&self, start: u64, count: u64) -> Vec<Entry> {
        self.inner
            .iter_entries(start, count)
            .into_iter()
            .map(Into::into)
            .collect()
    }
}
