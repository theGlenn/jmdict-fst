//! Blazing-fast Japanese dictionary engine with FST-based indexing.

mod dict;
mod error;
#[cfg(feature = "install")]
pub mod install;
mod model;
mod query;

pub use dict::{Dict, DictStorage, EntryIter};
pub use error::JmdictError;
pub use model::*;
pub use query::{BatchQueryBuilder, LookupResultIter, MAX_FUZZY_DISTANCE, QueryBuilder};
