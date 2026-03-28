use crate::dict::{Dict, MatchCandidate};
use crate::error::JmdictError;
use crate::model::{LookupResult, MatchMode};
use std::vec;

/// An iterator that lazily deserializes dictionary entries from pre-sorted match candidates.
pub struct LookupResultIter<'d, 'a> {
    dict: &'d Dict<'a>,
    candidates: vec::IntoIter<MatchCandidate>,
    common_only: bool,
    pos_filter: Vec<String>,
    limit: Option<usize>,
    yielded: usize,
}

impl<'d, 'a> Iterator for LookupResultIter<'d, 'a> {
    type Item = LookupResult;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(limit) = self.limit {
            if self.yielded >= limit {
                return None;
            }
        }

        loop {
            let mc = self.candidates.next()?;
            let entry = match self.dict.load_entry(mc.id) {
                Some(e) => e,
                None => continue,
            };

            if self.common_only {
                let is_common = entry.kanji.iter().any(|k| k.common)
                    || entry.kana.iter().any(|k| k.common);
                if !is_common {
                    continue;
                }
            }

            if !self.pos_filter.is_empty() {
                let matches_pos = entry.sense.iter().any(|s| {
                    s.part_of_speech
                        .iter()
                        .any(|p| self.pos_filter.iter().any(|f| p.contains(f.as_str())))
                });
                if !matches_pos {
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
pub struct QueryBuilder<'d, 'a> {
    dict: &'d Dict<'a>,
    term: String,
    mode: MatchMode,
    common_only: bool,
    pos_filter: Vec<String>,
    limit: Option<usize>,
    max_distance: u32,
}

impl<'d, 'a> QueryBuilder<'d, 'a> {
    pub(crate) fn new(dict: &'d Dict<'a>, term: impl Into<String>) -> Self {
        Self {
            dict,
            term: term.into(),
            mode: MatchMode::Exact,
            common_only: false,
            pos_filter: Vec::new(),
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

    /// Set the maximum edit distance for fuzzy search (default: 2).
    pub fn max_distance(mut self, n: u32) -> Self {
        self.max_distance = n;
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
    pub fn execute_iter(self) -> Result<LookupResultIter<'d, 'a>, JmdictError> {
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
            limit: self.limit,
            yielded: 0,
        })
    }
}

/// A builder for configuring and executing batch dictionary lookups.
pub struct BatchQueryBuilder<'d, 'a> {
    dict: &'d Dict<'a>,
    terms: Vec<String>,
    mode: MatchMode,
    common_only: bool,
    pos_filter: Vec<String>,
    limit: Option<usize>,
    max_distance: u32,
}

impl<'d, 'a> BatchQueryBuilder<'d, 'a> {
    pub(crate) fn new(dict: &'d Dict<'a>, terms: Vec<String>) -> Self {
        Self {
            dict,
            terms,
            mode: MatchMode::Exact,
            common_only: false,
            pos_filter: Vec::new(),
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

    /// Cap results per term after filtering and sorting.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the maximum edit distance for fuzzy search (default: 2).
    pub fn max_distance(mut self, n: u32) -> Self {
        self.max_distance = n;
        self
    }

    /// Execute the batch query and return results paired with each input term.
    pub fn execute(self) -> Result<Vec<(String, Vec<LookupResult>)>, JmdictError> {
        let pos_refs: Vec<&str> = self.pos_filter.iter().map(|s| s.as_str()).collect();
        let mut batch_results = Vec::with_capacity(self.terms.len());
        for term in &self.terms {
            let mut builder = self
                .dict
                .lookup(term)
                .mode(self.mode.clone())
                .common_only(self.common_only)
                .pos(&pos_refs)
                .max_distance(self.max_distance);
            if let Some(limit) = self.limit {
                builder = builder.limit(limit);
            }
            batch_results.push((term.clone(), builder.execute()?));
        }
        Ok(batch_results)
    }
}
