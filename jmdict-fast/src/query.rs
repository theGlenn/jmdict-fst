use crate::dict::{Dict, MatchCandidate};
use crate::error::JmdictError;
use crate::model::{LookupResult, MatchMode};
use std::vec;

/// Upper bound on the Levenshtein edit distance accepted by the fuzzy
/// search builders. The `fst` Levenshtein automaton's DFA grows rapidly
/// with distance; values above 4 are rarely useful and risk large
/// allocations.
pub const MAX_FUZZY_DISTANCE: u32 = 4;

/// Returns true when `filter` is empty, or any haystack value contains any
/// filter substring. Matches the existing `pos` filter semantics: filters
/// are case-sensitive substrings of the JMdict codes (`"v"` catches every
/// verb POS, `"v1"` only ichidan).
fn filter_passes(filter: &[String], haystack: &[String]) -> bool {
    filter.is_empty()
        || haystack
            .iter()
            .any(|h| filter.iter().any(|f| h.contains(f.as_str())))
}

/// An iterator that lazily deserializes dictionary entries from pre-sorted match candidates.
pub struct LookupResultIter<'d> {
    dict: &'d Dict,
    candidates: vec::IntoIter<MatchCandidate>,
    common_only: bool,
    pos_filter: Vec<String>,
    misc_filter: Vec<String>,
    field_filter: Vec<String>,
    dialect_filter: Vec<String>,
    limit: Option<usize>,
    yielded: usize,
}

impl<'d> Iterator for LookupResultIter<'d> {
    type Item = LookupResult;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(limit) = self.limit {
            if self.yielded >= limit {
                return None;
            }
        }

        let any_sense_filter = !self.pos_filter.is_empty()
            || !self.misc_filter.is_empty()
            || !self.field_filter.is_empty()
            || !self.dialect_filter.is_empty();

        loop {
            let mc = self.candidates.next()?;
            let entry = match self.dict.load_entry(mc.id) {
                Some(e) => e,
                None => continue,
            };

            if self.common_only && !entry.is_common() {
                continue;
            }

            // Sense-level filters are conjunctive *within* a sense: a single
            // sense must satisfy every active filter for the entry to match.
            // This mirrors JMdict's structure where pos/misc/field/dialect
            // are recorded per sense, so a verb-sense + noun-sense entry
            // won't match `pos=v` + `misc=abbr` unless one sense is both.
            if any_sense_filter {
                let any_match = entry.sense.iter().any(|s| {
                    filter_passes(&self.pos_filter, &s.part_of_speech)
                        && filter_passes(&self.misc_filter, &s.misc)
                        && filter_passes(&self.field_filter, &s.field)
                        && filter_passes(&self.dialect_filter, &s.dialect)
                });
                if !any_match {
                    continue;
                }
            }

            self.yielded += 1;
            return Some(LookupResult {
                entry,
                match_type: mc.match_type,
                match_key: mc.key,
                score: mc.score,
                deinflection: mc.deinflection,
            });
        }
    }
}

/// A builder for configuring and executing dictionary lookups.
pub struct QueryBuilder<'d> {
    dict: &'d Dict,
    term: String,
    mode: MatchMode,
    common_only: bool,
    pos_filter: Vec<String>,
    misc_filter: Vec<String>,
    field_filter: Vec<String>,
    dialect_filter: Vec<String>,
    limit: Option<usize>,
    max_distance: u32,
}

impl<'d> QueryBuilder<'d> {
    pub(crate) fn new(dict: &'d Dict, term: impl Into<String>) -> Self {
        Self {
            dict,
            term: term.into(),
            mode: MatchMode::Exact,
            common_only: false,
            pos_filter: Vec::new(),
            misc_filter: Vec::new(),
            field_filter: Vec::new(),
            dialect_filter: Vec::new(),
            limit: None,
            max_distance: 2,
        }
    }

    /// Set the match mode for this query.
    pub fn mode(mut self, mode: MatchMode) -> Self {
        self.mode = mode;
        self
    }

    /// Filter to entries where any KanjiEntry or KanaEntry has `common: true`.
    pub fn common_only(mut self, common: bool) -> Self {
        self.common_only = common;
        self
    }

    /// Filter to entries with matching part_of_speech values in any SenseEntry.
    pub fn pos(mut self, pos: &[&str]) -> Self {
        self.pos_filter = pos.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Filter to entries with any of the given JMdict `misc` codes
    /// (e.g. `"uk"` for "usually written in kana", `"abbr"` for abbreviation).
    pub fn misc(mut self, misc: &[&str]) -> Self {
        self.misc_filter = misc.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Filter to entries with any of the given JMdict `field` codes
    /// (e.g. `"med"` for medicine, `"comp"` for computing).
    pub fn field(mut self, field: &[&str]) -> Self {
        self.field_filter = field.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Filter to entries with any of the given JMdict `dialect` codes
    /// (e.g. `"ksb"` for Kansai-ben, `"ktb"` for Kantou-ben).
    pub fn dialect(mut self, dialect: &[&str]) -> Self {
        self.dialect_filter = dialect.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Set the maximum edit distance for fuzzy search (default: 2).
    ///
    /// Clamped to a maximum of [`MAX_FUZZY_DISTANCE`] to keep the Levenshtein DFA
    /// from blowing up — the automaton's state space grows quickly with distance.
    pub fn max_distance(mut self, n: u32) -> Self {
        self.max_distance = n.min(MAX_FUZZY_DISTANCE);
        self
    }

    /// Cap results after filtering and sorting.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Execute the query and return all results collected into a Vec.
    pub fn execute(self) -> Result<Vec<LookupResult>, JmdictError> {
        Ok(self.execute_iter()?.collect())
    }

    /// Execute the query and return a lazy iterator that deserializes entries on demand.
    ///
    /// This is more memory-efficient than `execute()` for large result sets (e.g., prefix
    /// or fuzzy queries with many matches), as entries are only deserialized as consumed.
    pub fn execute_iter(self) -> Result<LookupResultIter<'d>, JmdictError> {
        let candidates = match self.mode {
            MatchMode::Exact => self.dict.exact_candidates(&self.term),
            MatchMode::Prefix => self.dict.prefix_candidates(&self.term),
            MatchMode::Deinflect => self.dict.deinflect_candidates(&self.term),
            MatchMode::Fuzzy => self.dict.fuzzy_candidates(&self.term, self.max_distance)?,
        };

        Ok(LookupResultIter {
            dict: self.dict,
            candidates: candidates.into_iter(),
            common_only: self.common_only,
            pos_filter: self.pos_filter,
            misc_filter: self.misc_filter,
            field_filter: self.field_filter,
            dialect_filter: self.dialect_filter,
            limit: self.limit,
            yielded: 0,
        })
    }
}

/// A builder for configuring and executing batch dictionary lookups.
pub struct BatchQueryBuilder<'d> {
    dict: &'d Dict,
    terms: Vec<String>,
    mode: MatchMode,
    common_only: bool,
    pos_filter: Vec<String>,
    misc_filter: Vec<String>,
    field_filter: Vec<String>,
    dialect_filter: Vec<String>,
    limit: Option<usize>,
    max_distance: u32,
}

impl<'d> BatchQueryBuilder<'d> {
    pub(crate) fn new(dict: &'d Dict, terms: Vec<String>) -> Self {
        Self {
            dict,
            terms,
            mode: MatchMode::Exact,
            common_only: false,
            pos_filter: Vec::new(),
            misc_filter: Vec::new(),
            field_filter: Vec::new(),
            dialect_filter: Vec::new(),
            limit: None,
            max_distance: 2,
        }
    }

    /// Set the match mode for this batch query.
    pub fn mode(mut self, mode: MatchMode) -> Self {
        self.mode = mode;
        self
    }

    /// Filter to entries where any KanjiEntry or KanaEntry has `common: true`.
    pub fn common_only(mut self, common: bool) -> Self {
        self.common_only = common;
        self
    }

    /// Filter to entries with matching part_of_speech values in any SenseEntry.
    pub fn pos(mut self, pos: &[&str]) -> Self {
        self.pos_filter = pos.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Filter to entries with any of the given JMdict `misc` codes.
    pub fn misc(mut self, misc: &[&str]) -> Self {
        self.misc_filter = misc.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Filter to entries with any of the given JMdict `field` codes.
    pub fn field(mut self, field: &[&str]) -> Self {
        self.field_filter = field.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Filter to entries with any of the given JMdict `dialect` codes.
    pub fn dialect(mut self, dialect: &[&str]) -> Self {
        self.dialect_filter = dialect.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Cap results per term after filtering and sorting.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the maximum edit distance for fuzzy search (default: 2).
    ///
    /// Clamped to a maximum of [`MAX_FUZZY_DISTANCE`].
    pub fn max_distance(mut self, n: u32) -> Self {
        self.max_distance = n.min(MAX_FUZZY_DISTANCE);
        self
    }

    /// Execute the batch query and return results paired with each input term.
    pub fn execute(self) -> Result<Vec<(String, Vec<LookupResult>)>, JmdictError> {
        let pos_refs: Vec<&str> = self.pos_filter.iter().map(|s| s.as_str()).collect();
        let misc_refs: Vec<&str> = self.misc_filter.iter().map(|s| s.as_str()).collect();
        let field_refs: Vec<&str> = self.field_filter.iter().map(|s| s.as_str()).collect();
        let dialect_refs: Vec<&str> = self.dialect_filter.iter().map(|s| s.as_str()).collect();
        let mut batch_results = Vec::with_capacity(self.terms.len());
        for term in &self.terms {
            let mut builder = self
                .dict
                .lookup(term)
                .mode(self.mode.clone())
                .common_only(self.common_only)
                .pos(&pos_refs)
                .misc(&misc_refs)
                .field(&field_refs)
                .dialect(&dialect_refs)
                .max_distance(self.max_distance);
            if let Some(limit) = self.limit {
                builder = builder.limit(limit);
            }
            batch_results.push((term.clone(), builder.execute()?));
        }
        Ok(batch_results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn filter_passes_empty_filter_always_matches() {
        assert!(filter_passes(&[], &s(&[])));
        assert!(filter_passes(&[], &s(&["v1"])));
    }

    #[test]
    fn filter_passes_substring_match() {
        // "v" catches every verb POS code that contains "v"
        assert!(filter_passes(&s(&["v"]), &s(&["v1"])));
        assert!(filter_passes(&s(&["v"]), &s(&["v5k", "vt"])));
        // "v1" is more selective
        assert!(filter_passes(&s(&["v1"]), &s(&["v1", "vt"])));
        assert!(!filter_passes(&s(&["v1"]), &s(&["v5k"])));
    }

    #[test]
    fn filter_passes_misses_when_no_haystack_value_matches() {
        assert!(!filter_passes(&s(&["v"]), &s(&["n"])));
        assert!(!filter_passes(&s(&["v"]), &s(&[])));
    }
}
