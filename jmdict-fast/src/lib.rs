//! Blazing-fast Japanese dictionary engine with FST-based indexing.

mod dict;
mod error;
mod model;
mod query;

pub use dict::Dict;
pub use error::JmdictError;
pub use model::*;
pub use query::{BatchQueryBuilder, LookupResultIter, QueryBuilder};

#[cfg(test)]
mod tests;
