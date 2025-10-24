//! Japanese Conjugation Reversal System
//!
//! This module provides a comprehensive system for reversing Japanese verb and adjective conjugations
//! to find their dictionary forms (lemmas). It's based on the algorithm from the 10ten Japanese Reader
//! project: https://github.com/birchill/10ten-ja-reader/blob/main/src/background/deinflect.ts
//!
//! The original TypeScript implementation by @birchill provides the foundation for this Rust port,
//! which maintains the same sophisticated rule application logic while leveraging Rust's type safety
//! and performance characteristics.

/// Represents the reason for a deinflection rule
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Reason {
    PolitePastNegative,
    PoliteNegative,
    PoliteVolitional,
    Chau,
    Sugiru,
    PolitePast,
    Tara,
    Tari,
    Causative,
    PotentialOrPassive,
    Toku,
    Sou,
    Tai,
    Polite,
    Respectful,
    Humble,
    HumbleOrKansaiDialect,
    Past,
    Negative,
    Passive,
    Ba,
    Volitional,
    Potential,
    EruUru,
    CausativePassive,
    Te,
    Zu,
    Imperative,
    MasuStem,
    Adv,
    Noun,
    ImperativeNegative,
    Continuous,
    Ki,
    SuruNoun,
    ZaruWoEnai,
    NegativeTe,
    Irregular,
}

/// Word types for deinflection rules
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WordType {
    // Final word types
    IchidanVerb = 1 << 0, // i.e. ru-verbs
    GodanVerb = 1 << 1,   // i.e. u-verbs
    IAdj = 1 << 2,
    KuruVerb = 1 << 3,
    SuruVerb = 1 << 4,
    SpecialSuruVerb = 1 << 5,
    NounVS = 1 << 6,
    All = (1 << 0) | (1 << 1) | (1 << 2) | (1 << 3) | (1 << 4) | (1 << 5) | (1 << 6),

    // Intermediate types
    Initial = 1 << 7, // original word before any deinflection
    TaTeStem = 1 << 8,
    DaDeStem = 1 << 9,
    MasuStem = 1 << 10,
    IrrealisStem = 1 << 11,
}

impl WordType {
    pub fn as_u32(&self) -> u32 {
        self.clone() as u32
    }
}

/// Represents a deinflection rule
#[derive(Debug, Clone)]
pub struct DeinflectionRule {
    /// The suffix to match
    pub from: String,
    /// The replacement for the suffix
    pub to: String,
    /// Bit mask representing the type of words this rule can be applied to
    pub from_type: u32,
    /// Bit mask representing the resulting word type
    pub to_type: u32,
    /// Reasons for this deinflection
    pub reasons: Vec<Reason>,
}

/// A candidate word after deinflection
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateWord {
    /// The de-inflected candidate word
    pub word: String,
    /// Sequence of reasons describing how the word was derived
    /// Each array is a sequence of rules applied in turn
    pub reason_chains: Vec<Vec<Reason>>,
    /// Bit field describing the possible types of word this could represent
    pub word_type: u32,
}

/// Group of rules with the same length
#[derive(Debug, Clone)]
struct RuleGroup {
    rules: Vec<DeinflectionRule>,
    from_len: usize,
}

/// Lightweight conjugation reversal system for Japanese
pub struct Deinflector {
    rule_groups: Vec<RuleGroup>,
}

impl Deinflector {
    /// Create a new deinflector with default Japanese conjugation rules
    pub fn new() -> Self {
        let mut deinflector = Self {
            rule_groups: Vec::new(),
        };
        deinflector.add_comprehensive_rules();
        deinflector
    }

    /// Deinflect a conjugated form to find possible lemmas
    pub fn deinflect(&self, word: &str) -> Vec<CandidateWord> {
        let mut result: Vec<CandidateWord> = Vec::new();
        let mut result_index: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        // Start with the original word
        let original = CandidateWord {
            word: word.to_string(),
            word_type: 0xffff
                ^ (WordType::TaTeStem.as_u32()
                    | WordType::DaDeStem.as_u32()
                    | WordType::IrrealisStem.as_u32()),
            reason_chains: Vec::new(),
        };
        result.push(original);
        result_index.insert(word.to_string(), 0);

        let mut i = 0;
        while i < result.len() {
            let this_candidate = result[i].clone();

            // Don't deinflect masu-stem results of Ichidan verbs any further
            if (this_candidate.word_type & WordType::IchidanVerb.as_u32() != 0)
                && this_candidate.reason_chains.len() == 1
                && this_candidate.reason_chains[0].len() == 1
                && this_candidate.reason_chains[0][0] == Reason::MasuStem
            {
                i += 1;
                continue;
            }

            let word = &this_candidate.word;
            let word_type = this_candidate.word_type;

            // Handle ichidan verb stems
            if word_type
                & (WordType::MasuStem.as_u32()
                    | WordType::TaTeStem.as_u32()
                    | WordType::IrrealisStem.as_u32())
                != 0
            {
                let mut reason = Vec::new();

                // Add the "masu" reason only if the word is solely the masu stem
                if word_type & WordType::MasuStem.as_u32() != 0
                    && this_candidate.reason_chains.is_empty()
                {
                    reason.push(vec![Reason::MasuStem]);
                }

                // Check for inapplicable forms
                let inapplicable_form = word_type & WordType::IrrealisStem.as_u32() != 0
                    && this_candidate.reason_chains.len() > 0
                    && this_candidate.reason_chains[0].len() > 0
                    && (this_candidate.reason_chains[0][0] == Reason::Passive
                        || this_candidate.reason_chains[0][0] == Reason::Causative
                        || this_candidate.reason_chains[0][0] == Reason::CausativePassive);

                if !inapplicable_form {
                    let new_word = format!("{}る", word);
                    result.push(CandidateWord {
                        word: new_word,
                        word_type: WordType::IchidanVerb.as_u32() | WordType::KuruVerb.as_u32(),
                        reason_chains: [this_candidate.reason_chains.clone(), reason].concat(),
                    });
                }
            }

            // Apply rules
            for rule_group in &self.rule_groups {
                if rule_group.from_len > word.len() {
                    continue;
                }

                let ending = &word[word.len() - rule_group.from_len..];
                let hiragana_ending = self.kana_to_hiragana(ending);

                for rule in &rule_group.rules {
                    if word_type & rule.from_type == 0 {
                        continue;
                    }

                    if ending != rule.from && hiragana_ending != rule.from {
                        continue;
                    }

                    let new_word = format!("{}{}", &word[..word.len() - rule.from.len()], rule.to);

                    if new_word.is_empty() {
                        continue;
                    }

                    // Check for duplicate reasons in the chain
                    let rule_reasons: std::collections::HashSet<_> = rule.reasons.iter().collect();
                    if this_candidate
                        .reason_chains
                        .iter()
                        .flat_map(|chain| chain.iter())
                        .any(|r| rule_reasons.contains(r))
                    {
                        continue;
                    }

                    // Handle existing candidates
                    if let Some(&existing_index) = result_index.get(&new_word) {
                        if result[existing_index].word_type == rule.to_type {
                            if !rule.reasons.is_empty() {
                                // Start a new reason chain (equivalent to unshift in TypeScript)
                                result[existing_index]
                                    .reason_chains
                                    .insert(0, rule.reasons.clone());
                            }
                            continue;
                        }
                    }

                    result_index.insert(new_word.clone(), result.len());

                    // Create new candidate
                    let mut reason_chains = this_candidate.reason_chains.clone();

                    if !rule.reasons.is_empty() {
                        if !reason_chains.is_empty() {
                            let first_chain = &mut reason_chains[0];

                            // Handle causative + passive combination
                            if rule.reasons[0] == Reason::Causative
                                && !first_chain.is_empty()
                                && first_chain[0] == Reason::PotentialOrPassive
                            {
                                first_chain[0] = Reason::CausativePassive;
                            } else if rule.reasons[0] == Reason::MasuStem && !first_chain.is_empty()
                            {
                                // Do nothing for masu stem when we already have reasons
                            } else {
                                first_chain.splice(0..0, rule.reasons.iter().cloned());
                            }
                        } else {
                            reason_chains.push(rule.reasons.clone());
                        }
                    }

                    result.push(CandidateWord {
                        word: new_word,
                        reason_chains,
                        word_type: rule.to_type,
                    });
                }
            }
            i += 1;
        }

        // Filter out intermediate forms
        result.retain(|r| r.word_type & WordType::All.as_u32() != 0);
        result
    }

    /// Convert katakana to hiragana (simplified version)
    fn kana_to_hiragana(&self, text: &str) -> String {
        text.chars()
            .map(|c| {
                if c >= 'ァ' && c <= 'ン' {
                    char::from_u32(c as u32 - 0x60).unwrap_or(c)
                } else {
                    c
                }
            })
            .collect()
    }

    /// Add comprehensive rules based on the TypeScript implementation
    ///
    /// These rules are derived from the 10ten Japanese Reader project's deinflect.ts file:
    /// https://github.com/birchill/10ten-ja-reader/blob/main/src/background/deinflect.ts
    fn add_comprehensive_rules(&mut self) {
        // Rule data: (from, to, from_type, to_type, reasons)
        // Based on the comprehensive rule set from 10ten Japanese Reader
        let rule_data = vec![
            // Length 7 rules
            (
                "ていらっしゃい",
                "",
                WordType::Initial.as_u32(),
                WordType::TaTeStem.as_u32(),
                vec![Reason::Respectful, Reason::Continuous, Reason::Imperative],
            ),
            (
                "ていらっしゃる",
                "",
                WordType::GodanVerb.as_u32(),
                WordType::TaTeStem.as_u32(),
                vec![Reason::Respectful, Reason::Continuous],
            ),
            (
                "でいらっしゃい",
                "",
                WordType::Initial.as_u32(),
                WordType::DaDeStem.as_u32(),
                vec![Reason::Respectful, Reason::Continuous, Reason::Imperative],
            ),
            (
                "でいらっしゃる",
                "",
                WordType::GodanVerb.as_u32(),
                WordType::DaDeStem.as_u32(),
                vec![Reason::Respectful, Reason::Continuous],
            ),
            // Length 6 rules
            (
                "いらっしゃい",
                "いらっしゃる",
                WordType::MasuStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::MasuStem],
            ),
            (
                "いらっしゃい",
                "いらっしゃる",
                WordType::Initial.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Imperative],
            ),
            (
                "くありません",
                "い",
                WordType::Initial.as_u32(),
                WordType::IAdj.as_u32(),
                vec![Reason::PoliteNegative],
            ),
            (
                "ざるをえない",
                "",
                WordType::IAdj.as_u32(),
                WordType::IrrealisStem.as_u32(),
                vec![Reason::ZaruWoEnai],
            ),
            (
                "ざるを得ない",
                "",
                WordType::IAdj.as_u32(),
                WordType::IrrealisStem.as_u32(),
                vec![Reason::ZaruWoEnai],
            ),
            (
                "ませんでした",
                "",
                WordType::Initial.as_u32(),
                WordType::MasuStem.as_u32(),
                vec![Reason::PolitePastNegative],
            ),
            (
                "てらっしゃい",
                "",
                WordType::Initial.as_u32(),
                WordType::TaTeStem.as_u32(),
                vec![Reason::Respectful, Reason::Continuous, Reason::Imperative],
            ),
            (
                "てらっしゃい",
                "てらっしゃる",
                WordType::MasuStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::MasuStem],
            ),
            (
                "てらっしゃる",
                "",
                WordType::GodanVerb.as_u32(),
                WordType::TaTeStem.as_u32(),
                vec![Reason::Respectful, Reason::Continuous],
            ),
            (
                "でらっしゃい",
                "",
                WordType::Initial.as_u32(),
                WordType::DaDeStem.as_u32(),
                vec![Reason::Respectful, Reason::Continuous, Reason::Imperative],
            ),
            (
                "でらっしゃい",
                "でらっしゃる",
                WordType::MasuStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::MasuStem],
            ),
            (
                "でらっしゃる",
                "",
                WordType::GodanVerb.as_u32(),
                WordType::DaDeStem.as_u32(),
                vec![Reason::Respectful, Reason::Continuous],
            ),
            // Length 5 rules
            (
                "おっしゃい",
                "おっしゃる",
                WordType::MasuStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::MasuStem],
            ),
            (
                "おっしゃい",
                "おっしゃる",
                WordType::Initial.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Imperative],
            ),
            (
                "ざるえない",
                "",
                WordType::IAdj.as_u32(),
                WordType::IrrealisStem.as_u32(),
                vec![Reason::ZaruWoEnai],
            ),
            (
                "ざる得ない",
                "",
                WordType::IAdj.as_u32(),
                WordType::IrrealisStem.as_u32(),
                vec![Reason::ZaruWoEnai],
            ),
            (
                "ざるをえぬ",
                "",
                WordType::IAdj.as_u32(),
                WordType::IrrealisStem.as_u32(),
                vec![Reason::ZaruWoEnai],
            ),
            (
                "ざるを得ぬ",
                "",
                WordType::IAdj.as_u32(),
                WordType::IrrealisStem.as_u32(),
                vec![Reason::ZaruWoEnai],
            ),
            (
                "ざるえぬ",
                "",
                WordType::IAdj.as_u32(),
                WordType::IrrealisStem.as_u32(),
                vec![Reason::ZaruWoEnai],
            ),
            (
                "ざる得ぬ",
                "",
                WordType::IAdj.as_u32(),
                WordType::IrrealisStem.as_u32(),
                vec![Reason::ZaruWoEnai],
            ),
            // Length 4 rules
            (
                "かったら",
                "い",
                WordType::Initial.as_u32(),
                WordType::IAdj.as_u32(),
                vec![Reason::Tara],
            ),
            (
                "かったり",
                "い",
                WordType::Initial.as_u32(),
                WordType::IAdj.as_u32(),
                vec![Reason::Tari],
            ),
            (
                "ください",
                "くださる",
                WordType::MasuStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::MasuStem],
            ),
            (
                "ください",
                "くださる",
                WordType::Initial.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Imperative],
            ),
            (
                "こさせる",
                "くる",
                WordType::IchidanVerb.as_u32(),
                WordType::KuruVerb.as_u32(),
                vec![Reason::Causative],
            ),
            (
                "こられる",
                "くる",
                WordType::IchidanVerb.as_u32(),
                WordType::KuruVerb.as_u32(),
                vec![Reason::PotentialOrPassive],
            ),
            (
                "さないで",
                "する",
                WordType::Initial.as_u32(),
                WordType::SpecialSuruVerb.as_u32(),
                vec![Reason::Irregular, Reason::NegativeTe],
            ),
            (
                "しないで",
                "する",
                WordType::Initial.as_u32(),
                WordType::SuruVerb.as_u32(),
                vec![Reason::NegativeTe],
            ),
            (
                "しさせる",
                "する",
                WordType::IchidanVerb.as_u32(),
                WordType::SpecialSuruVerb.as_u32(),
                vec![Reason::Irregular, Reason::Causative],
            ),
            (
                "しられる",
                "する",
                WordType::IchidanVerb.as_u32(),
                WordType::SpecialSuruVerb.as_u32(),
                vec![Reason::Irregular, Reason::PotentialOrPassive],
            ),
            (
                "せさせる",
                "する",
                WordType::IchidanVerb.as_u32(),
                WordType::SpecialSuruVerb.as_u32(),
                vec![Reason::Irregular, Reason::Causative],
            ),
            (
                "せられる",
                "する",
                WordType::IchidanVerb.as_u32(),
                WordType::SpecialSuruVerb.as_u32(),
                vec![Reason::Irregular, Reason::PotentialOrPassive],
            ),
            (
                "ぜさせる",
                "ずる",
                WordType::IchidanVerb.as_u32(),
                WordType::SpecialSuruVerb.as_u32(),
                vec![Reason::Irregular, Reason::Causative],
            ),
            (
                "ぜられる",
                "ずる",
                WordType::IchidanVerb.as_u32(),
                WordType::SpecialSuruVerb.as_u32(),
                vec![Reason::Irregular, Reason::PotentialOrPassive],
            ),
            (
                "たゆたう",
                "たゆたう",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "たゆとう",
                "たゆとう",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "のたまう",
                "のたまう",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "のたもう",
                "のたもう",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "ましたら",
                "",
                WordType::Initial.as_u32(),
                WordType::MasuStem.as_u32(),
                vec![Reason::Polite, Reason::Tara],
            ),
            (
                "ましたり",
                "",
                WordType::Initial.as_u32(),
                WordType::MasuStem.as_u32(),
                vec![Reason::Polite, Reason::Tari],
            ),
            (
                "ましょう",
                "",
                WordType::Initial.as_u32(),
                WordType::MasuStem.as_u32(),
                vec![Reason::PoliteVolitional],
            ),
            // Length 3 rules
            (
                "いたす",
                "",
                WordType::GodanVerb.as_u32(),
                WordType::MasuStem.as_u32(),
                vec![Reason::Humble],
            ),
            (
                "いたす",
                "",
                WordType::GodanVerb.as_u32(),
                WordType::NounVS.as_u32(),
                vec![Reason::SuruNoun, Reason::Humble],
            ),
            (
                "かった",
                "い",
                WordType::Initial.as_u32(),
                WordType::IAdj.as_u32(),
                vec![Reason::Past],
            ),
            (
                "下さい",
                "下さる",
                WordType::MasuStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::MasuStem],
            ),
            (
                "下さい",
                "下さる",
                WordType::Initial.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Imperative],
            ),
            (
                "くない",
                "い",
                WordType::IAdj.as_u32(),
                WordType::IAdj.as_u32(),
                vec![Reason::Negative],
            ),
            (
                "ければ",
                "い",
                WordType::Initial.as_u32(),
                WordType::IAdj.as_u32(),
                vec![Reason::Ba],
            ),
            (
                "こよう",
                "くる",
                WordType::Initial.as_u32(),
                WordType::KuruVerb.as_u32(),
                vec![Reason::Volitional],
            ),
            (
                "これる",
                "くる",
                WordType::IchidanVerb.as_u32(),
                WordType::KuruVerb.as_u32(),
                vec![Reason::Potential],
            ),
            (
                "来れる",
                "来る",
                WordType::IchidanVerb.as_u32(),
                WordType::KuruVerb.as_u32(),
                vec![Reason::Potential],
            ),
            (
                "來れる",
                "來る",
                WordType::IchidanVerb.as_u32(),
                WordType::KuruVerb.as_u32(),
                vec![Reason::Potential],
            ),
            (
                "ござい",
                "ござる",
                WordType::MasuStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::MasuStem],
            ),
            (
                "ご座い",
                "ご座る",
                WordType::MasuStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::MasuStem],
            ),
            (
                "御座い",
                "御座る",
                WordType::MasuStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::MasuStem],
            ),
            (
                "させる",
                "る",
                WordType::IchidanVerb.as_u32(),
                WordType::IchidanVerb.as_u32() | WordType::KuruVerb.as_u32(),
                vec![Reason::Causative],
            ),
            (
                "させる",
                "する",
                WordType::IchidanVerb.as_u32(),
                WordType::SuruVerb.as_u32(),
                vec![Reason::Causative],
            ),
            (
                "さない",
                "する",
                WordType::IAdj.as_u32(),
                WordType::SpecialSuruVerb.as_u32(),
                vec![Reason::Irregular, Reason::Negative],
            ),
            (
                "される",
                "",
                WordType::IchidanVerb.as_u32(),
                WordType::IrrealisStem.as_u32(),
                vec![Reason::CausativePassive],
            ),
            (
                "される",
                "する",
                WordType::IchidanVerb.as_u32(),
                WordType::SuruVerb.as_u32(),
                vec![Reason::Passive],
            ),
            (
                "しうる",
                "する",
                WordType::Initial.as_u32(),
                WordType::SuruVerb.as_u32(),
                vec![Reason::EruUru],
            ),
            (
                "しえる",
                "する",
                WordType::IchidanVerb.as_u32(),
                WordType::SuruVerb.as_u32(),
                vec![Reason::EruUru],
            ),
            (
                "しない",
                "する",
                WordType::IAdj.as_u32(),
                WordType::SuruVerb.as_u32(),
                vec![Reason::Negative],
            ),
            (
                "しよう",
                "する",
                WordType::Initial.as_u32(),
                WordType::SuruVerb.as_u32(),
                vec![Reason::Volitional],
            ),
            (
                "じゃう",
                "",
                WordType::GodanVerb.as_u32(),
                WordType::DaDeStem.as_u32(),
                vec![Reason::Chau],
            ),
            (
                "すぎる",
                "い",
                WordType::IchidanVerb.as_u32(),
                WordType::IAdj.as_u32(),
                vec![Reason::Sugiru],
            ),
            (
                "すぎる",
                "",
                WordType::IchidanVerb.as_u32(),
                WordType::MasuStem.as_u32(),
                vec![Reason::Sugiru],
            ),
            (
                "過ぎる",
                "い",
                WordType::IchidanVerb.as_u32(),
                WordType::IAdj.as_u32(),
                vec![Reason::Sugiru],
            ),
            (
                "過ぎる",
                "",
                WordType::IchidanVerb.as_u32(),
                WordType::MasuStem.as_u32(),
                vec![Reason::Sugiru],
            ),
            (
                "ずれば",
                "ずる",
                WordType::Initial.as_u32(),
                WordType::SpecialSuruVerb.as_u32(),
                vec![Reason::Irregular, Reason::Ba],
            ),
            (
                "たまう",
                "たまう",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "たもう",
                "たもう",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "揺蕩う",
                "揺蕩う",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "ちゃう",
                "",
                WordType::GodanVerb.as_u32(),
                WordType::TaTeStem.as_u32(),
                vec![Reason::Chau],
            ),
            (
                "ている",
                "",
                WordType::IchidanVerb.as_u32(),
                WordType::TaTeStem.as_u32(),
                vec![Reason::Continuous],
            ),
            (
                "ておる",
                "",
                WordType::GodanVerb.as_u32(),
                WordType::TaTeStem.as_u32(),
                vec![Reason::HumbleOrKansaiDialect, Reason::Continuous],
            ),
            (
                "でいる",
                "",
                WordType::IchidanVerb.as_u32(),
                WordType::DaDeStem.as_u32(),
                vec![Reason::Continuous],
            ),
            (
                "でおる",
                "",
                WordType::GodanVerb.as_u32(),
                WordType::DaDeStem.as_u32(),
                vec![Reason::HumbleOrKansaiDialect, Reason::Continuous],
            ),
            (
                "できる",
                "する",
                WordType::IchidanVerb.as_u32(),
                WordType::SuruVerb.as_u32(),
                vec![Reason::Potential],
            ),
            (
                "ないで",
                "",
                WordType::Initial.as_u32(),
                WordType::IrrealisStem.as_u32(),
                vec![Reason::NegativeTe],
            ),
            (
                "なさい",
                "",
                WordType::Initial.as_u32(),
                WordType::MasuStem.as_u32(),
                vec![Reason::Respectful, Reason::Imperative],
            ),
            (
                "なさい",
                "なさる",
                WordType::MasuStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::MasuStem],
            ),
            (
                "なさい",
                "なさる",
                WordType::Initial.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Imperative],
            ),
            (
                "なさる",
                "",
                WordType::GodanVerb.as_u32(),
                WordType::MasuStem.as_u32(),
                vec![Reason::Respectful],
            ),
            (
                "なさる",
                "",
                WordType::GodanVerb.as_u32(),
                WordType::NounVS.as_u32(),
                vec![Reason::SuruNoun, Reason::Respectful],
            ),
            (
                "になる",
                "",
                WordType::GodanVerb.as_u32(),
                WordType::MasuStem.as_u32(),
                vec![Reason::Respectful],
            ),
            (
                "になる",
                "",
                WordType::GodanVerb.as_u32(),
                WordType::NounVS.as_u32(),
                vec![Reason::SuruNoun, Reason::Respectful],
            ),
            (
                "ました",
                "",
                WordType::Initial.as_u32(),
                WordType::MasuStem.as_u32(),
                vec![Reason::PolitePast],
            ),
            (
                "まして",
                "",
                WordType::Initial.as_u32(),
                WordType::MasuStem.as_u32(),
                vec![Reason::Polite, Reason::Te],
            ),
            (
                "ません",
                "",
                WordType::Initial.as_u32(),
                WordType::MasuStem.as_u32(),
                vec![Reason::PoliteNegative],
            ),
            (
                "られる",
                "る",
                WordType::IchidanVerb.as_u32(),
                WordType::IchidanVerb.as_u32() | WordType::KuruVerb.as_u32(),
                vec![Reason::PotentialOrPassive],
            ),
            // Length 2 rules
            (
                "致す",
                "",
                WordType::GodanVerb.as_u32(),
                WordType::MasuStem.as_u32(),
                vec![Reason::Humble],
            ),
            (
                "致す",
                "",
                WordType::GodanVerb.as_u32(),
                WordType::NounVS.as_u32(),
                vec![Reason::SuruNoun, Reason::Humble],
            ),
            (
                "えば",
                "う",
                WordType::Initial.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Ba],
            ),
            (
                "える",
                "う",
                WordType::IchidanVerb.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Potential],
            ),
            (
                "得る",
                "",
                WordType::IchidanVerb.as_u32(),
                WordType::MasuStem.as_u32(),
                vec![Reason::EruUru],
            ),
            (
                "おう",
                "う",
                WordType::Initial.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Volitional],
            ),
            (
                "仰い",
                "仰る",
                WordType::MasuStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::MasuStem],
            ),
            (
                "仰い",
                "仰る",
                WordType::Initial.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Imperative],
            ),
            (
                "くて",
                "い",
                WordType::Initial.as_u32(),
                WordType::IAdj.as_u32(),
                vec![Reason::Te],
            ),
            (
                "けば",
                "く",
                WordType::Initial.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Ba],
            ),
            (
                "げば",
                "ぐ",
                WordType::Initial.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Ba],
            ),
            (
                "ける",
                "く",
                WordType::IchidanVerb.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Potential],
            ),
            (
                "げる",
                "ぐ",
                WordType::IchidanVerb.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Potential],
            ),
            (
                "こい",
                "くる",
                WordType::Initial.as_u32(),
                WordType::KuruVerb.as_u32(),
                vec![Reason::Imperative],
            ),
            (
                "こう",
                "く",
                WordType::Initial.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Volitional],
            ),
            (
                "ごう",
                "ぐ",
                WordType::Initial.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Volitional],
            ),
            (
                "しろ",
                "する",
                WordType::Initial.as_u32(),
                WordType::SuruVerb.as_u32(),
                vec![Reason::Imperative],
            ),
            (
                "さず",
                "する",
                WordType::Initial.as_u32(),
                WordType::SpecialSuruVerb.as_u32(),
                vec![Reason::Irregular, Reason::Zu],
            ),
            (
                "すぎ",
                "い",
                WordType::Initial.as_u32(),
                WordType::IAdj.as_u32(),
                vec![Reason::Sugiru],
            ),
            (
                "すぎ",
                "",
                WordType::Initial.as_u32(),
                WordType::MasuStem.as_u32(),
                vec![Reason::Sugiru],
            ),
            (
                "過ぎ",
                "い",
                WordType::Initial.as_u32(),
                WordType::IAdj.as_u32(),
                vec![Reason::Sugiru],
            ),
            (
                "過ぎ",
                "",
                WordType::Initial.as_u32(),
                WordType::MasuStem.as_u32(),
                vec![Reason::Sugiru],
            ),
            (
                "する",
                "",
                WordType::SuruVerb.as_u32(),
                WordType::NounVS.as_u32(),
                vec![Reason::SuruNoun],
            ),
            (
                "せず",
                "する",
                WordType::Initial.as_u32(),
                WordType::SuruVerb.as_u32(),
                vec![Reason::Zu],
            ),
            (
                "せぬ",
                "する",
                WordType::Initial.as_u32(),
                WordType::SuruVerb.as_u32(),
                vec![Reason::Negative],
            ),
            (
                "せん",
                "する",
                WordType::Initial.as_u32(),
                WordType::SuruVerb.as_u32(),
                vec![Reason::Negative],
            ),
            (
                "せば",
                "す",
                WordType::Initial.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Ba],
            ),
            (
                "せば",
                "する",
                WordType::Initial.as_u32(),
                WordType::SpecialSuruVerb.as_u32(),
                vec![Reason::Irregular, Reason::Ba],
            ),
            (
                "せよ",
                "する",
                WordType::Initial.as_u32(),
                WordType::SuruVerb.as_u32(),
                vec![Reason::Imperative],
            ),
            (
                "せる",
                "す",
                WordType::IchidanVerb.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Potential],
            ),
            (
                "せる",
                "",
                WordType::IchidanVerb.as_u32(),
                WordType::IrrealisStem.as_u32(),
                vec![Reason::Causative],
            ),
            (
                "ぜず",
                "ずる",
                WordType::Initial.as_u32(),
                WordType::SpecialSuruVerb.as_u32(),
                vec![Reason::Irregular, Reason::Zu],
            ),
            (
                "ぜぬ",
                "ずる",
                WordType::Initial.as_u32(),
                WordType::SpecialSuruVerb.as_u32(),
                vec![Reason::Irregular, Reason::Negative],
            ),
            (
                "ぜよ",
                "ずる",
                WordType::Initial.as_u32(),
                WordType::SpecialSuruVerb.as_u32(),
                vec![Reason::Irregular, Reason::Imperative],
            ),
            (
                "そう",
                "",
                WordType::Initial.as_u32(),
                WordType::MasuStem.as_u32(),
                vec![Reason::Sou],
            ),
            (
                "そう",
                "い",
                WordType::Initial.as_u32(),
                WordType::IAdj.as_u32(),
                vec![Reason::Sou],
            ),
            (
                "そう",
                "す",
                WordType::Initial.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Volitional],
            ),
            (
                "そう",
                "する",
                WordType::Initial.as_u32(),
                WordType::SpecialSuruVerb.as_u32(),
                vec![Reason::Irregular, Reason::Volitional],
            ),
            (
                "たい",
                "",
                WordType::IAdj.as_u32(),
                WordType::MasuStem.as_u32(),
                vec![Reason::Tai],
            ),
            (
                "たら",
                "",
                WordType::Initial.as_u32(),
                WordType::TaTeStem.as_u32(),
                vec![Reason::Tara],
            ),
            (
                "だら",
                "",
                WordType::Initial.as_u32(),
                WordType::DaDeStem.as_u32(),
                vec![Reason::Tara],
            ),
            (
                "たり",
                "",
                WordType::Initial.as_u32(),
                WordType::TaTeStem.as_u32(),
                vec![Reason::Tari],
            ),
            (
                "だり",
                "",
                WordType::Initial.as_u32(),
                WordType::DaDeStem.as_u32(),
                vec![Reason::Tari],
            ),
            (
                "てば",
                "つ",
                WordType::Initial.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Ba],
            ),
            (
                "てる",
                "つ",
                WordType::IchidanVerb.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Potential],
            ),
            (
                "てる",
                "",
                WordType::IchidanVerb.as_u32(),
                WordType::TaTeStem.as_u32(),
                vec![Reason::Continuous],
            ),
            (
                "でる",
                "",
                WordType::IchidanVerb.as_u32(),
                WordType::DaDeStem.as_u32(),
                vec![Reason::Continuous],
            ),
            (
                "とう",
                "つ",
                WordType::Initial.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Volitional],
            ),
            (
                "とく",
                "",
                WordType::GodanVerb.as_u32(),
                WordType::TaTeStem.as_u32(),
                vec![Reason::Toku],
            ),
            (
                "とる",
                "",
                WordType::GodanVerb.as_u32(),
                WordType::TaTeStem.as_u32(),
                vec![Reason::HumbleOrKansaiDialect, Reason::Continuous],
            ),
            (
                "どく",
                "",
                WordType::GodanVerb.as_u32(),
                WordType::DaDeStem.as_u32(),
                vec![Reason::Toku],
            ),
            (
                "どる",
                "",
                WordType::GodanVerb.as_u32(),
                WordType::DaDeStem.as_u32(),
                vec![Reason::HumbleOrKansaiDialect, Reason::Continuous],
            ),
            (
                "ない",
                "",
                WordType::IAdj.as_u32(),
                WordType::IrrealisStem.as_u32(),
                vec![Reason::Negative],
            ),
            (
                "ねば",
                "ぬ",
                WordType::Initial.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Ba],
            ),
            (
                "ねる",
                "ぬ",
                WordType::IchidanVerb.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Potential],
            ),
            (
                "のう",
                "ぬ",
                WordType::Initial.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Volitional],
            ),
            (
                "べば",
                "ぶ",
                WordType::Initial.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Ba],
            ),
            (
                "べる",
                "ぶ",
                WordType::IchidanVerb.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Potential],
            ),
            (
                "ぼう",
                "ぶ",
                WordType::Initial.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Volitional],
            ),
            (
                "ます",
                "",
                WordType::Initial.as_u32(),
                WordType::MasuStem.as_u32(),
                vec![Reason::Polite],
            ),
            (
                "ませ",
                "",
                WordType::Initial.as_u32(),
                WordType::MasuStem.as_u32(),
                vec![Reason::Polite, Reason::Imperative],
            ),
            (
                "めば",
                "む",
                WordType::Initial.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Ba],
            ),
            (
                "める",
                "む",
                WordType::IchidanVerb.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Potential],
            ),
            (
                "もう",
                "む",
                WordType::Initial.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Volitional],
            ),
            (
                "よう",
                "る",
                WordType::Initial.as_u32(),
                WordType::IchidanVerb.as_u32() | WordType::KuruVerb.as_u32(),
                vec![Reason::Volitional],
            ),
            (
                "れば",
                "る",
                WordType::Initial.as_u32(),
                WordType::IchidanVerb.as_u32()
                    | WordType::GodanVerb.as_u32()
                    | WordType::KuruVerb.as_u32()
                    | WordType::SuruVerb.as_u32(),
                vec![Reason::Ba],
            ),
            (
                "れる",
                "る",
                WordType::IchidanVerb.as_u32(),
                WordType::IchidanVerb.as_u32() | WordType::GodanVerb.as_u32(),
                vec![Reason::Potential],
            ),
            (
                "れる",
                "",
                WordType::IchidanVerb.as_u32(),
                WordType::IrrealisStem.as_u32(),
                vec![Reason::Passive],
            ),
            (
                "ろう",
                "る",
                WordType::Initial.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Volitional],
            ),
            // Irregular て-form stems
            (
                "いっ",
                "いく",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "おう",
                "おう",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "こう",
                "こう",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "そう",
                "そう",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "とう",
                "とう",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "行っ",
                "行く",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "逝っ",
                "逝く",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "往っ",
                "往く",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "請う",
                "請う",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "乞う",
                "乞う",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "恋う",
                "恋う",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "問う",
                "問う",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "負う",
                "負う",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "沿う",
                "沿う",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "添う",
                "添う",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "副う",
                "副う",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "厭う",
                "厭う",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "給う",
                "給う",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "賜う",
                "賜う",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "宣う",
                "宣う",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "曰う",
                "曰う",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            // Length 1 rules
            (
                "い",
                "う",
                WordType::MasuStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::MasuStem],
            ),
            (
                "い",
                "く",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "い",
                "ぐ",
                WordType::DaDeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "い",
                "る",
                WordType::Initial.as_u32(),
                WordType::KuruVerb.as_u32(),
                vec![Reason::Imperative],
            ),
            (
                "え",
                "う",
                WordType::Initial.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Imperative],
            ),
            (
                "か",
                "く",
                WordType::IrrealisStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "が",
                "ぐ",
                WordType::IrrealisStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "き",
                "い",
                WordType::Initial.as_u32(),
                WordType::IAdj.as_u32(),
                vec![Reason::Ki],
            ),
            (
                "き",
                "く",
                WordType::MasuStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::MasuStem],
            ),
            (
                "き",
                "くる",
                WordType::TaTeStem.as_u32(),
                WordType::KuruVerb.as_u32(),
                vec![],
            ),
            (
                "き",
                "くる",
                WordType::MasuStem.as_u32(),
                WordType::KuruVerb.as_u32(),
                vec![Reason::MasuStem],
            ),
            (
                "ぎ",
                "ぐ",
                WordType::MasuStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::MasuStem],
            ),
            (
                "く",
                "い",
                WordType::Initial.as_u32(),
                WordType::IAdj.as_u32(),
                vec![Reason::Adv],
            ),
            (
                "け",
                "く",
                WordType::Initial.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Imperative],
            ),
            (
                "げ",
                "ぐ",
                WordType::Initial.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Imperative],
            ),
            (
                "こ",
                "くる",
                WordType::IrrealisStem.as_u32(),
                WordType::KuruVerb.as_u32(),
                vec![],
            ),
            (
                "さ",
                "い",
                WordType::Initial.as_u32(),
                WordType::IAdj.as_u32(),
                vec![Reason::Noun],
            ),
            (
                "さ",
                "す",
                WordType::IrrealisStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "し",
                "す",
                WordType::MasuStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::MasuStem],
            ),
            (
                "し",
                "する",
                WordType::MasuStem.as_u32(),
                WordType::SuruVerb.as_u32(),
                vec![Reason::MasuStem],
            ),
            (
                "し",
                "す",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "し",
                "する",
                WordType::TaTeStem.as_u32(),
                WordType::SuruVerb.as_u32(),
                vec![],
            ),
            (
                "ず",
                "",
                WordType::Initial.as_u32(),
                WordType::IrrealisStem.as_u32(),
                vec![Reason::Zu],
            ),
            (
                "せ",
                "す",
                WordType::Initial.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Imperative],
            ),
            (
                "せ",
                "する",
                WordType::Initial.as_u32(),
                WordType::SpecialSuruVerb.as_u32(),
                vec![Reason::Irregular, Reason::Imperative],
            ),
            (
                "た",
                "つ",
                WordType::IrrealisStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "た",
                "",
                WordType::Initial.as_u32(),
                WordType::TaTeStem.as_u32(),
                vec![Reason::Past],
            ),
            (
                "だ",
                "",
                WordType::Initial.as_u32(),
                WordType::DaDeStem.as_u32(),
                vec![Reason::Past],
            ),
            (
                "ち",
                "つ",
                WordType::MasuStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::MasuStem],
            ),
            (
                "っ",
                "う",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "っ",
                "つ",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "っ",
                "る",
                WordType::TaTeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "て",
                "",
                WordType::Initial.as_u32(),
                WordType::TaTeStem.as_u32(),
                vec![Reason::Te],
            ),
            (
                "て",
                "つ",
                WordType::Initial.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Imperative],
            ),
            (
                "で",
                "",
                WordType::Initial.as_u32(),
                WordType::DaDeStem.as_u32(),
                vec![Reason::Te],
            ),
            (
                "な",
                "ぬ",
                WordType::IrrealisStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "な",
                "",
                WordType::Initial.as_u32(),
                WordType::IchidanVerb.as_u32()
                    | WordType::GodanVerb.as_u32()
                    | WordType::KuruVerb.as_u32()
                    | WordType::SuruVerb.as_u32(),
                vec![Reason::ImperativeNegative],
            ),
            (
                "に",
                "ぬ",
                WordType::MasuStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::MasuStem],
            ),
            (
                "ぬ",
                "",
                WordType::Initial.as_u32(),
                WordType::IrrealisStem.as_u32(),
                vec![Reason::Negative],
            ),
            (
                "ね",
                "ぬ",
                WordType::Initial.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Imperative],
            ),
            (
                "ば",
                "ぶ",
                WordType::IrrealisStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "び",
                "ぶ",
                WordType::MasuStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::MasuStem],
            ),
            (
                "べ",
                "ぶ",
                WordType::Initial.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Imperative],
            ),
            (
                "ま",
                "む",
                WordType::IrrealisStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "み",
                "む",
                WordType::MasuStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::MasuStem],
            ),
            (
                "め",
                "む",
                WordType::Initial.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Imperative],
            ),
            (
                "よ",
                "る",
                WordType::Initial.as_u32(),
                WordType::IchidanVerb.as_u32(),
                vec![Reason::Imperative],
            ),
            (
                "ら",
                "る",
                WordType::IrrealisStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "り",
                "る",
                WordType::MasuStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::MasuStem],
            ),
            (
                "れ",
                "る",
                WordType::Initial.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![Reason::Imperative],
            ),
            (
                "ろ",
                "る",
                WordType::Initial.as_u32(),
                WordType::IchidanVerb.as_u32(),
                vec![Reason::Imperative],
            ),
            (
                "わ",
                "う",
                WordType::IrrealisStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "ん",
                "ぬ",
                WordType::DaDeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "ん",
                "ぶ",
                WordType::DaDeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "ん",
                "む",
                WordType::DaDeStem.as_u32(),
                WordType::GodanVerb.as_u32(),
                vec![],
            ),
            (
                "ん",
                "",
                WordType::Initial.as_u32(),
                WordType::IrrealisStem.as_u32(),
                vec![Reason::Negative],
            ),
        ];

        // Group rules by length
        let mut rule_groups: std::collections::HashMap<usize, Vec<DeinflectionRule>> =
            std::collections::HashMap::new();

        for (from, to, from_type, to_type, reasons) in rule_data {
            let rule = DeinflectionRule {
                from: from.to_string(),
                to: to.to_string(),
                from_type,
                to_type,
                reasons,
            };

            rule_groups
                .entry(from.len())
                .or_insert_with(Vec::new)
                .push(rule);
        }

        // Convert to RuleGroup format and sort by length (longest first)
        let mut groups: Vec<RuleGroup> = rule_groups
            .into_iter()
            .map(|(from_len, rules)| RuleGroup { rules, from_len })
            .collect();

        groups.sort_by(|a, b| b.from_len.cmp(&a.from_len));
        self.rule_groups = groups;
    }
}

impl Default for Deinflector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performs_deinflection() {
        let deinflector = Deinflector::new();
        let result = deinflector.deinflect("走ります");
        let match_candidate = result.iter().find(|candidate| candidate.word == "走る");
        assert_eq!(
            match_candidate,
            Some(&CandidateWord {
                word: "走る".to_string(),
                reason_chains: vec![vec![Reason::Polite]],
                word_type: WordType::GodanVerb.as_u32(),
            })
        );
    }

    #[test]
    fn test_performs_deinflection_recursively() {
        let deinflector = Deinflector::new();
        let result = deinflector.deinflect("踊りたくなかった");
        let match_candidate = result.iter().find(|candidate| candidate.word == "踊る");
        assert_eq!(
            match_candidate,
            Some(&CandidateWord {
                word: "踊る".to_string(),
                reason_chains: vec![vec![Reason::Tai, Reason::Negative, Reason::Past]],
                word_type: WordType::GodanVerb.as_u32(),
            })
        );
    }

    #[test]
    fn test_does_not_allow_duplicates_in_reason_chain() {
        let deinflector = Deinflector::new();
        let cases = vec![
            "見させさせる",   // causative < causative
            "見させてさせる", // causative < continuous < causative
            "見ていている",   // continuous < continuous
            "見てさせている", // continuous < causative < continuous
            "見とけとく",     // -te oku < potential < -te oku
        ];

        for inflected in cases {
            let result = deinflector.deinflect(inflected);
            let match_candidate = result.iter().find(|candidate| {
                candidate.word == "見る"
                    && (candidate.word_type & WordType::IchidanVerb.as_u32()) != 0
            });
            assert!(
                match_candidate.is_none(),
                "Should not find '見る' for '{}'",
                inflected
            );
        }
    }

    #[test]
    fn test_deinflects_kana_variations() {
        let deinflector = Deinflector::new();
        let cases = vec![
            (
                "走ります",
                "走る",
                vec![vec![Reason::Polite]],
                WordType::GodanVerb.as_u32(),
            ),
            (
                "走りまス",
                "走る",
                vec![vec![Reason::Polite]],
                WordType::GodanVerb.as_u32(),
            ),
            (
                "走りマス",
                "走る",
                vec![vec![Reason::Polite]],
                WordType::GodanVerb.as_u32(),
            ),
            (
                "走リマス",
                "走る",
                vec![vec![Reason::Polite]],
                WordType::GodanVerb.as_u32(),
            ),
            (
                "走リマす",
                "走る",
                vec![vec![Reason::Polite]],
                WordType::GodanVerb.as_u32(),
            ),
            (
                "走った",
                "走る",
                vec![vec![Reason::Past]],
                WordType::GodanVerb.as_u32(),
            ),
            (
                "走っタ",
                "走る",
                vec![vec![Reason::Past]],
                WordType::GodanVerb.as_u32(),
            ),
            (
                "走ッタ",
                "走る",
                vec![vec![Reason::Past]],
                WordType::GodanVerb.as_u32(),
            ),
            (
                "走ッた",
                "走る",
                vec![vec![Reason::Past]],
                WordType::GodanVerb.as_u32(),
            ),
        ];

        for (inflected, plain, reason_chains, word_type) in cases {
            let result = deinflector.deinflect(inflected);
            let match_candidate = result.iter().find(|candidate| candidate.word == plain);
            assert!(
                match_candidate.is_some(),
                "Failed to find '{}' in '{}'",
                plain,
                inflected
            );
            let candidate = match_candidate.unwrap();
            assert_eq!(candidate.reason_chains, reason_chains);
            assert_eq!(candidate.word_type, word_type);
            assert_eq!(candidate.word, plain);
        }
    }

    #[test]
    fn test_deinflects_masu_stem_forms() {
        let deinflector = Deinflector::new();
        let result = deinflector.deinflect("食べ");
        let match_candidate = result.iter().find(|candidate| candidate.word == "食べる");
        assert_eq!(
            match_candidate,
            Some(&CandidateWord {
                word: "食べる".to_string(),
                reason_chains: vec![vec![Reason::MasuStem]],
                word_type: WordType::IchidanVerb.as_u32() | WordType::KuruVerb.as_u32(),
            })
        );
    }

    #[test]
    fn test_deinflects_nu() {
        let deinflector = Deinflector::new();
        let cases = vec![
            ("思わぬ", "思う", WordType::GodanVerb.as_u32()),
            ("行かぬ", "行く", WordType::GodanVerb.as_u32()),
            ("話さぬ", "話す", WordType::GodanVerb.as_u32()),
            ("経たぬ", "経つ", WordType::GodanVerb.as_u32()),
            ("死なぬ", "死ぬ", WordType::GodanVerb.as_u32()),
            ("遊ばぬ", "遊ぶ", WordType::GodanVerb.as_u32()),
            ("止まぬ", "止む", WordType::GodanVerb.as_u32()),
            ("切らぬ", "切る", WordType::GodanVerb.as_u32()),
            (
                "見ぬ",
                "見る",
                WordType::IchidanVerb.as_u32() | WordType::KuruVerb.as_u32(),
            ),
            ("こぬ", "くる", WordType::KuruVerb.as_u32()),
            ("せぬ", "する", WordType::SuruVerb.as_u32()),
        ];

        for (inflected, plain, word_type) in cases {
            let result = deinflector.deinflect(inflected);
            let match_candidate = result.iter().find(|candidate| candidate.word == plain);
            assert!(
                match_candidate.is_some(),
                "Failed to find '{}' in '{}'",
                plain,
                inflected
            );
            let candidate = match_candidate.unwrap();
            assert_eq!(candidate.reason_chains, vec![vec![Reason::Negative]]);
            assert_eq!(candidate.word_type, word_type);
            assert_eq!(candidate.word, plain);
        }
    }

    #[test]
    fn test_recursively_deinflects_nu() {
        let deinflector = Deinflector::new();
        let result = deinflector.deinflect("食べられぬ");
        let match_candidate = result.iter().find(|candidate| candidate.word == "食べる");
        assert_eq!(
            match_candidate,
            Some(&CandidateWord {
                word: "食べる".to_string(),
                reason_chains: vec![vec![Reason::PotentialOrPassive, Reason::Negative]],
                word_type: WordType::IchidanVerb.as_u32() | WordType::KuruVerb.as_u32(),
            })
        );
    }

    #[test]
    fn test_deinflects_ki_to_kuru() {
        let deinflector = Deinflector::new();
        let result = deinflector.deinflect("き");
        let match_candidate = result.iter().find(|candidate| candidate.word == "くる");
        assert_eq!(
            match_candidate,
            Some(&CandidateWord {
                word: "くる".to_string(),
                reason_chains: vec![vec![Reason::MasuStem]],
                word_type: WordType::KuruVerb.as_u32(),
            })
        );
    }

    #[test]
    fn test_deinflects_ki_ending_for_i_adj() {
        let deinflector = Deinflector::new();
        let result = deinflector.deinflect("美しき");
        let match_candidate = result.iter().find(|candidate| candidate.word == "美しい");
        assert_eq!(
            match_candidate,
            Some(&CandidateWord {
                word: "美しい".to_string(),
                reason_chains: vec![vec![Reason::Ki]],
                word_type: WordType::IAdj.as_u32(),
            })
        );
    }

    #[test]
    fn test_deinflects_all_forms_of_suru() {
        let deinflector = Deinflector::new();
        let cases = vec![
            ("した", vec![Reason::Past]),
            ("しよう", vec![Reason::Volitional]),
            ("しない", vec![Reason::Negative]),
            ("せぬ", vec![Reason::Negative]),
            ("せん", vec![Reason::Negative]),
            ("せず", vec![Reason::Zu]),
            ("される", vec![Reason::Passive]),
            ("させる", vec![Reason::Causative]),
            ("しろ", vec![Reason::Imperative]),
            ("せよ", vec![Reason::Imperative]),
            ("すれば", vec![Reason::Ba]),
            ("できる", vec![Reason::Potential]),
        ];

        for (inflected, reasons) in cases {
            let result = deinflector.deinflect(inflected);
            let match_candidate = result.iter().find(|candidate| {
                candidate.word == "する" && (candidate.word_type & WordType::SuruVerb.as_u32()) != 0
            });
            assert!(
                match_candidate.is_some(),
                "Failed to find 'する' for '{}'",
                inflected
            );
            let candidate = match_candidate.unwrap();
            assert_eq!(candidate.reason_chains, vec![reasons]);
        }
    }

    #[test]
    fn test_deinflects_additional_forms_of_special_class_suru_verbs() {
        let deinflector = Deinflector::new();
        let cases = vec![
            (
                "発せさせる",
                "発する",
                vec![Reason::Irregular, Reason::Causative],
            ),
            (
                "発せられる",
                "発する",
                vec![Reason::Irregular, Reason::PotentialOrPassive],
            ),
            (
                "発しさせる",
                "発する",
                vec![Reason::Irregular, Reason::Causative],
            ),
            (
                "発しられる",
                "発する",
                vec![Reason::Irregular, Reason::PotentialOrPassive],
            ),
            // 五段化
            (
                "発さない",
                "発する",
                vec![Reason::Irregular, Reason::Negative],
            ),
            (
                "発さないで",
                "発する",
                vec![Reason::Irregular, Reason::NegativeTe],
            ),
            ("発さず", "発する", vec![Reason::Irregular, Reason::Zu]),
            (
                "発そう",
                "発する",
                vec![Reason::Irregular, Reason::Volitional],
            ),
            ("愛せば", "愛する", vec![Reason::Irregular, Reason::Ba]),
            (
                "愛せ",
                "愛する",
                vec![Reason::Irregular, Reason::Imperative],
            ),
            // ずる / vz class verbs
            (
                "信ぜぬ",
                "信ずる",
                vec![Reason::Irregular, Reason::Negative],
            ),
            ("信ぜず", "信ずる", vec![Reason::Irregular, Reason::Zu]),
            (
                "信ぜさせる",
                "信ずる",
                vec![Reason::Irregular, Reason::Causative],
            ),
            (
                "信ぜられる",
                "信ずる",
                vec![Reason::Irregular, Reason::PotentialOrPassive],
            ),
            ("信ずれば", "信ずる", vec![Reason::Irregular, Reason::Ba]),
            (
                "信ぜよ",
                "信ずる",
                vec![Reason::Irregular, Reason::Imperative],
            ),
        ];

        for (inflected, plain, reasons) in cases {
            let result = deinflector.deinflect(inflected);
            let match_candidate = result.iter().find(|candidate| candidate.word == plain);
            assert!(
                match_candidate.is_some(),
                "Failed to find '{}' for '{}'",
                plain,
                inflected
            );
            let candidate = match_candidate.unwrap();
            assert_eq!(candidate.word_type, WordType::SpecialSuruVerb.as_u32());
            assert_eq!(candidate.reason_chains, vec![reasons]);
        }
    }

    #[test]
    fn test_deinflects_irregular_forms_of_iku() {
        let deinflector = Deinflector::new();
        let cases = vec![
            ("行った", "行く", Reason::Past, WordType::GodanVerb.as_u32()),
            ("行って", "行く", Reason::Te, WordType::GodanVerb.as_u32()),
            (
                "行ったり",
                "行く",
                Reason::Tari,
                WordType::GodanVerb.as_u32(),
            ),
            (
                "行ったら",
                "行く",
                Reason::Tara,
                WordType::GodanVerb.as_u32(),
            ),
            ("いった", "いく", Reason::Past, WordType::GodanVerb.as_u32()),
            ("いって", "いく", Reason::Te, WordType::GodanVerb.as_u32()),
            (
                "いったり",
                "いく",
                Reason::Tari,
                WordType::GodanVerb.as_u32(),
            ),
            (
                "いったら",
                "いく",
                Reason::Tara,
                WordType::GodanVerb.as_u32(),
            ),
            ("逝った", "逝く", Reason::Past, WordType::GodanVerb.as_u32()),
            ("往った", "往く", Reason::Past, WordType::GodanVerb.as_u32()),
        ];

        for (inflected, plain, reason, word_type) in cases {
            let result = deinflector.deinflect(inflected);
            let match_candidate = result.iter().find(|candidate| candidate.word == plain);
            assert!(
                match_candidate.is_some(),
                "Failed to find '{}' for '{}'",
                plain,
                inflected
            );
            let candidate = match_candidate.unwrap();
            assert_eq!(candidate.reason_chains, vec![vec![reason]]);
            assert_eq!(candidate.word_type, word_type);
            assert_eq!(candidate.word, plain);
        }
    }

    #[test]
    fn test_does_not_deinflect_other_verbs_ending_in_ku_like_iku() {
        let deinflector = Deinflector::new();
        let result = deinflector.deinflect("もどって");
        let match_candidate = result.iter().find(|candidate| candidate.word == "もどく");
        assert!(match_candidate.is_none(), "Should not find 'もどく'");
    }

    #[test]
    fn test_deinflects_other_irregular_verbs() {
        let deinflector = Deinflector::new();
        let cases = vec![
            "請うた",
            "請う",
            "乞うた",
            "乞う",
            "恋うた",
            "恋う",
            "こうた",
            "こう",
            "問うた",
            "問う",
            "とうた",
            "とう",
            "負うた",
            "負う",
            "おうた",
            "おう",
            "沿うた",
            "沿う",
            "添うた",
            "添う",
            "副うた",
            "副う",
            "そうた",
            "そう",
            "厭うた",
            "厭う",
            "いとうた",
            "いとう",
            "のたまうた",
            "のたまう",
            "のたもうた",
            "のたもう",
            "宣うた",
            "宣う",
            "曰うた",
            "曰う",
            "たまうた",
            "たまう",
            "たもうた",
            "たもう",
            "給うた",
            "給う",
            "賜うた",
            "賜う",
            "たゆたうた",
            "たゆたう",
            "たゆとうた",
            "たゆとう",
            "揺蕩うた",
            "揺蕩う",
        ];

        for i in (0..cases.len()).step_by(2) {
            let inflected = cases[i];
            let plain = cases[i + 1];
            let result = deinflector.deinflect(inflected);
            let match_candidate = result.iter().find(|candidate| candidate.word == plain);
            assert!(
                match_candidate.is_some(),
                "Failed to find '{}' for '{}'",
                plain,
                inflected
            );
            let candidate = match_candidate.unwrap();
            assert_eq!(candidate.reason_chains, vec![vec![Reason::Past]]);
            assert_eq!(candidate.word_type, WordType::GodanVerb.as_u32());
            assert_eq!(candidate.word, plain);
        }
    }

    #[test]
    fn test_deinflects_continuous_forms_of_other_irregular_verbs() {
        let deinflector = Deinflector::new();
        let cases = vec![
            ("請うている", "請う", vec![Reason::Continuous]),
            ("乞うている", "乞う", vec![Reason::Continuous]),
            ("恋うている", "恋う", vec![Reason::Continuous]),
            ("こうてる", "こう", vec![Reason::Continuous]),
            ("問うてる", "問う", vec![Reason::Continuous]),
            ("とうてる", "とう", vec![Reason::Continuous]),
            ("負うていた", "負う", vec![Reason::Continuous, Reason::Past]),
            ("おうていた", "おう", vec![Reason::Continuous, Reason::Past]),
            ("沿うていた", "沿う", vec![Reason::Continuous, Reason::Past]),
            ("添うてた", "添う", vec![Reason::Continuous, Reason::Past]),
            ("副うてた", "副う", vec![Reason::Continuous, Reason::Past]),
            ("そうてた", "そう", vec![Reason::Continuous, Reason::Past]),
            ("厭うていて", "厭う", vec![Reason::Continuous, Reason::Te]),
            (
                "いとうていて",
                "いとう",
                vec![Reason::Continuous, Reason::Te],
            ),
            ("のたまうている", "のたまう", vec![Reason::Continuous]),
            (
                "のたもうていた",
                "のたもう",
                vec![Reason::Continuous, Reason::Past],
            ),
            ("宣うてた", "宣う", vec![Reason::Continuous, Reason::Past]),
            ("曰うてて", "曰う", vec![Reason::Continuous, Reason::Te]),
            ("たまうている", "たまう", vec![Reason::Continuous]),
            (
                "たもうていた",
                "たもう",
                vec![Reason::Continuous, Reason::Past],
            ),
            ("給うてた", "給う", vec![Reason::Continuous, Reason::Past]),
            ("賜うてて", "賜う", vec![Reason::Continuous, Reason::Te]),
            ("たゆたうている", "たゆたう", vec![Reason::Continuous]),
            (
                "たゆとうていた",
                "たゆとう",
                vec![Reason::Continuous, Reason::Past],
            ),
            ("揺蕩うてて", "揺蕩う", vec![Reason::Continuous, Reason::Te]),
        ];

        for (inflected, plain, reasons) in cases {
            let result = deinflector.deinflect(inflected);
            let match_candidate = result.iter().find(|candidate| candidate.word == plain);
            assert!(
                match_candidate.is_some(),
                "Failed to find '{}' for '{}'",
                plain,
                inflected
            );
            let candidate = match_candidate.unwrap();
            assert_eq!(candidate.reason_chains, vec![reasons]);
            assert_eq!(candidate.word_type, WordType::GodanVerb.as_u32());
            assert_eq!(candidate.word, plain);
        }
    }

    #[test]
    fn test_deinflects_gozaru() {
        let deinflector = Deinflector::new();
        let cases = vec![
            ("ございます", "ござる", Reason::Polite),
            ("ご座います", "ご座る", Reason::Polite),
            ("御座います", "御座る", Reason::Polite),
            ("ございません", "ござる", Reason::PoliteNegative),
            ("ご座いません", "ご座る", Reason::PoliteNegative),
            ("御座いません", "御座る", Reason::PoliteNegative),
            ("ございませんでした", "ござる", Reason::PolitePastNegative),
            ("ご座いませんでした", "ご座る", Reason::PolitePastNegative),
            ("御座いませんでした", "御座る", Reason::PolitePastNegative),
        ];

        for (inflected, plain, reason) in cases {
            let result = deinflector.deinflect(inflected);
            let match_candidate = result.iter().find(|candidate| candidate.word == plain);
            assert!(
                match_candidate.is_some(),
                "Failed to find '{}' for '{}'",
                plain,
                inflected
            );
            let candidate = match_candidate.unwrap();
            assert_eq!(candidate.reason_chains, vec![vec![reason]]);
            assert_eq!(candidate.word, plain);
        }
    }

    #[test]
    fn test_deinflects_kudasaru() {
        let deinflector = Deinflector::new();
        let cases = vec![
            ("くださいます", "くださる", Reason::Polite),
            ("下さいます", "下さる", Reason::Polite),
            ("くださいません", "くださる", Reason::PoliteNegative),
            ("下さいません", "下さる", Reason::PoliteNegative),
            (
                "くださいませんでした",
                "くださる",
                Reason::PolitePastNegative,
            ),
            ("下さいませんでした", "下さる", Reason::PolitePastNegative),
        ];

        for (inflected, plain, reason) in cases {
            let result = deinflector.deinflect(inflected);
            let match_candidate = result.iter().find(|candidate| candidate.word == plain);
            assert!(
                match_candidate.is_some(),
                "Failed to find '{}' for '{}'",
                plain,
                inflected
            );
            let candidate = match_candidate.unwrap();
            assert_eq!(candidate.reason_chains, vec![vec![reason]]);
            assert_eq!(candidate.word, plain);
        }
    }

    #[test]
    fn test_deinflects_irassharu() {
        let deinflector = Deinflector::new();
        let cases = vec![
            (
                "いらっしゃいます",
                "いらっしゃる",
                vec![vec![Reason::Polite]],
            ),
            (
                "いらっしゃい",
                "いらっしゃる",
                vec![vec![Reason::Imperative], vec![Reason::MasuStem]],
            ),
            ("いらっしゃって", "いらっしゃる", vec![vec![Reason::Te]]),
        ];

        for (inflected, plain, reason_chains) in cases {
            let result = deinflector.deinflect(inflected);
            let match_candidate = result.iter().find(|candidate| candidate.word == plain);
            assert!(
                match_candidate.is_some(),
                "Failed to find '{}' for '{}'",
                plain,
                inflected
            );
            let candidate = match_candidate.unwrap();
            assert_eq!(candidate.reason_chains, reason_chains);
            assert_eq!(candidate.word, plain);
        }
    }

    #[test]
    fn test_deinflects_the_continuous_form() {
        let deinflector = Deinflector::new();
        let cases = vec![
            // U-verbs
            ("戻っている", "戻る", WordType::GodanVerb.as_u32(), None),
            ("戻ってる", "戻る", WordType::GodanVerb.as_u32(), None),
            ("歩いている", "歩く", WordType::GodanVerb.as_u32(), None),
            ("歩いてる", "歩く", WordType::GodanVerb.as_u32(), None),
            ("泳いでいる", "泳ぐ", WordType::GodanVerb.as_u32(), None),
            ("泳いでる", "泳ぐ", WordType::GodanVerb.as_u32(), None),
            ("話している", "話す", WordType::GodanVerb.as_u32(), None),
            ("話してる", "話す", WordType::GodanVerb.as_u32(), None),
            ("死んでいる", "死ぬ", WordType::GodanVerb.as_u32(), None),
            ("死んでる", "死ぬ", WordType::GodanVerb.as_u32(), None),
            ("飼っている", "飼う", WordType::GodanVerb.as_u32(), None),
            ("飼ってる", "飼う", WordType::GodanVerb.as_u32(), None),
            ("放っている", "放つ", WordType::GodanVerb.as_u32(), None),
            ("放ってる", "放つ", WordType::GodanVerb.as_u32(), None),
            ("遊んでいる", "遊ぶ", WordType::GodanVerb.as_u32(), None),
            ("遊んでる", "遊ぶ", WordType::GodanVerb.as_u32(), None),
            ("歩んでいる", "歩む", WordType::GodanVerb.as_u32(), None),
            ("歩んでる", "歩む", WordType::GodanVerb.as_u32(), None),
            // Ru-verbs
            (
                "食べている",
                "食べる",
                WordType::IchidanVerb.as_u32() | WordType::KuruVerb.as_u32(),
                None,
            ),
            (
                "食べてる",
                "食べる",
                WordType::IchidanVerb.as_u32() | WordType::KuruVerb.as_u32(),
                None,
            ),
            // Special verbs
            ("している", "する", WordType::SuruVerb.as_u32(), None),
            ("してる", "する", WordType::SuruVerb.as_u32(), None),
            (
                "来ている",
                "来る",
                WordType::IchidanVerb.as_u32() | WordType::KuruVerb.as_u32(),
                None,
            ),
            (
                "来てる",
                "来る",
                WordType::IchidanVerb.as_u32() | WordType::KuruVerb.as_u32(),
                None,
            ),
            ("きている", "くる", WordType::KuruVerb.as_u32(), None),
            ("きてる", "くる", WordType::KuruVerb.as_u32(), None),
            // Combinations
            (
                "戻っています",
                "戻る",
                WordType::GodanVerb.as_u32(),
                Some(vec![Reason::Continuous, Reason::Polite]),
            ),
            (
                "戻ってます",
                "戻る",
                WordType::GodanVerb.as_u32(),
                Some(vec![Reason::Continuous, Reason::Polite]),
            ),
            (
                "戻っていない",
                "戻る",
                WordType::GodanVerb.as_u32(),
                Some(vec![Reason::Continuous, Reason::Negative]),
            ),
            (
                "戻ってない",
                "戻る",
                WordType::GodanVerb.as_u32(),
                Some(vec![Reason::Continuous, Reason::Negative]),
            ),
            (
                "戻っていた",
                "戻る",
                WordType::GodanVerb.as_u32(),
                Some(vec![Reason::Continuous, Reason::Past]),
            ),
            (
                "戻ってた",
                "戻る",
                WordType::GodanVerb.as_u32(),
                Some(vec![Reason::Continuous, Reason::Past]),
            ),
        ];

        for (inflected, plain, word_type, reasons) in cases {
            let result = deinflector.deinflect(inflected);
            let match_candidate = result.iter().find(|candidate| candidate.word == plain);
            assert!(
                match_candidate.is_some(),
                "Failed to find '{}' for '{}'",
                plain,
                inflected
            );
            let candidate = match_candidate.unwrap();
            let expected_reasons = reasons.unwrap_or(vec![Reason::Continuous]);
            assert_eq!(candidate.reason_chains, vec![expected_reasons]);
            assert_eq!(candidate.word_type, word_type);
            assert_eq!(candidate.word, plain);
        }

        // Check we don't get false positives
        let result = deinflector.deinflect("食べて");
        let match_candidate = result.iter().find(|candidate| candidate.word == "食べる");
        assert!(match_candidate.is_some());
        let candidate = match_candidate.unwrap();
        assert!(!candidate
            .reason_chains
            .iter()
            .any(|chain| chain.contains(&Reason::Continuous)));
    }

    #[test]
    fn test_deinflects_respectful_continuous_forms() {
        let deinflector = Deinflector::new();
        let cases = vec![
            (
                "分かっていらっしゃる",
                "分かる",
                vec![Reason::Respectful, Reason::Continuous],
            ),
            (
                "分かっていらっしゃい",
                "分かる",
                vec![Reason::Respectful, Reason::Continuous, Reason::Imperative],
            ),
            (
                "分かってらっしゃる",
                "分かる",
                vec![Reason::Respectful, Reason::Continuous],
            ),
            (
                "分かってらっしゃい",
                "分かる",
                vec![Reason::Respectful, Reason::Continuous, Reason::Imperative],
            ),
            (
                "読んでいらっしゃる",
                "読む",
                vec![Reason::Respectful, Reason::Continuous],
            ),
            (
                "読んでいらっしゃい",
                "読む",
                vec![Reason::Respectful, Reason::Continuous, Reason::Imperative],
            ),
            (
                "読んでらっしゃる",
                "読む",
                vec![Reason::Respectful, Reason::Continuous],
            ),
            (
                "読んでらっしゃい",
                "読む",
                vec![Reason::Respectful, Reason::Continuous, Reason::Imperative],
            ),
            (
                "起きていらっしゃる",
                "起きる",
                vec![Reason::Respectful, Reason::Continuous],
            ),
            (
                "起きていらっしゃい",
                "起きる",
                vec![Reason::Respectful, Reason::Continuous, Reason::Imperative],
            ),
            (
                "起きてらっしゃる",
                "起きる",
                vec![Reason::Respectful, Reason::Continuous],
            ),
            (
                "起きてらっしゃい",
                "起きる",
                vec![Reason::Respectful, Reason::Continuous, Reason::Imperative],
            ),
            (
                "分かっていらっしゃいます",
                "分かる",
                vec![Reason::Respectful, Reason::Continuous, Reason::Polite],
            ),
            (
                "分かってらっしゃいます",
                "分かる",
                vec![Reason::Respectful, Reason::Continuous, Reason::Polite],
            ),
            (
                "分かっていらっしゃって",
                "分かる",
                vec![Reason::Respectful, Reason::Continuous, Reason::Te],
            ),
            (
                "分かってらっしゃって",
                "分かる",
                vec![Reason::Respectful, Reason::Continuous, Reason::Te],
            ),
        ];

        for (inflected, plain, reasons) in cases {
            let result = deinflector.deinflect(inflected);
            let match_candidate = result.iter().find(|candidate| candidate.word == plain);
            assert!(
                match_candidate.is_some(),
                "Failed to find '{}' for '{}'",
                plain,
                inflected
            );
            let candidate = match_candidate.unwrap();
            assert_eq!(candidate.reason_chains, vec![reasons]);
        }
    }

    #[test]
    fn test_deinflects_nasaru_as_respectful_speech_for_suru() {
        let deinflector = Deinflector::new();
        let cases = vec![
            (
                "なさい",
                "なさる",
                vec![vec![Reason::Imperative], vec![Reason::MasuStem]],
            ),
            (
                "食べなさい",
                "食べる",
                vec![vec![Reason::Respectful, Reason::Imperative]],
            ),
            (
                "帰りなさいませ",
                "帰る",
                vec![vec![Reason::Respectful, Reason::Polite, Reason::Imperative]],
            ),
            (
                "仕事なさる",
                "仕事",
                vec![vec![Reason::SuruNoun, Reason::Respectful]],
            ),
            (
                "エンジョイなさって",
                "エンジョイ",
                vec![vec![Reason::SuruNoun, Reason::Respectful, Reason::Te]],
            ),
            (
                "喜びなさった",
                "喜ぶ",
                vec![vec![Reason::Respectful, Reason::Past]],
            ),
        ];

        for (inflected, plain, reason_chains) in cases {
            let result = deinflector.deinflect(inflected);
            let match_candidate = result.iter().find(|candidate| candidate.word == plain);
            assert!(
                match_candidate.is_some(),
                "Failed to find '{}' for '{}'",
                plain,
                inflected
            );
            let candidate = match_candidate.unwrap();
            assert_eq!(candidate.reason_chains, reason_chains);
        }
    }

    #[test]
    fn test_deinflects_ni_naru_as_respectful_speech() {
        let deinflector = Deinflector::new();
        let cases = vec![
            (
                "到着になります",
                "到着",
                vec![Reason::SuruNoun, Reason::Respectful, Reason::Polite],
            ),
            (
                "読みになります",
                "読む",
                vec![Reason::Respectful, Reason::Polite],
            ),
            (
                "見えになります",
                "見える",
                vec![Reason::Respectful, Reason::Polite],
            ),
        ];

        for (inflected, plain, reasons) in cases {
            let result = deinflector.deinflect(inflected);
            let match_candidate = result.iter().find(|candidate| candidate.word == plain);
            assert!(
                match_candidate.is_some(),
                "Failed to find '{}' for '{}'",
                plain,
                inflected
            );
            let candidate = match_candidate.unwrap();
            assert_eq!(candidate.reason_chains, vec![reasons]);
        }
    }

    #[test]
    fn test_deinflects_humble_or_kansai_dialect_continuous_forms() {
        let deinflector = Deinflector::new();
        let cases = vec![
            (
                "行っておる",
                "行く",
                vec![Reason::HumbleOrKansaiDialect, Reason::Continuous],
            ),
            (
                "行っており",
                "行く",
                vec![
                    Reason::HumbleOrKansaiDialect,
                    Reason::Continuous,
                    Reason::MasuStem,
                ],
            ),
            (
                "行っとる",
                "行く",
                vec![Reason::HumbleOrKansaiDialect, Reason::Continuous],
            ),
            (
                "行っとり",
                "行く",
                vec![
                    Reason::HumbleOrKansaiDialect,
                    Reason::Continuous,
                    Reason::MasuStem,
                ],
            ),
            (
                "読んでおる",
                "読む",
                vec![Reason::HumbleOrKansaiDialect, Reason::Continuous],
            ),
            (
                "読んでおり",
                "読む",
                vec![
                    Reason::HumbleOrKansaiDialect,
                    Reason::Continuous,
                    Reason::MasuStem,
                ],
            ),
            (
                "読んどる",
                "読む",
                vec![Reason::HumbleOrKansaiDialect, Reason::Continuous],
            ),
            (
                "読んどり",
                "読む",
                vec![
                    Reason::HumbleOrKansaiDialect,
                    Reason::Continuous,
                    Reason::MasuStem,
                ],
            ),
            (
                "起きておる",
                "起きる",
                vec![Reason::HumbleOrKansaiDialect, Reason::Continuous],
            ),
            (
                "起きており",
                "起きる",
                vec![
                    Reason::HumbleOrKansaiDialect,
                    Reason::Continuous,
                    Reason::MasuStem,
                ],
            ),
            (
                "起きとる",
                "起きる",
                vec![Reason::HumbleOrKansaiDialect, Reason::Continuous],
            ),
            (
                "起きとり",
                "起きる",
                vec![
                    Reason::HumbleOrKansaiDialect,
                    Reason::Continuous,
                    Reason::MasuStem,
                ],
            ),
        ];

        for (inflected, plain, reasons) in cases {
            let result = deinflector.deinflect(inflected);
            let match_candidate = result.iter().find(|candidate| candidate.word == plain);
            assert!(
                match_candidate.is_some(),
                "Failed to find '{}' for '{}'",
                plain,
                inflected
            );
            let candidate = match_candidate.unwrap();
            assert_eq!(candidate.reason_chains, vec![reasons]);
        }
    }

    #[test]
    fn test_deinflects_itasu_as_humble_speech_for_suru() {
        let deinflector = Deinflector::new();
        let cases = vec![
            (
                "お願いいたします",
                "お願い",
                vec![Reason::SuruNoun, Reason::Humble, Reason::Polite],
            ),
            (
                "お願い致します",
                "お願い",
                vec![Reason::SuruNoun, Reason::Humble, Reason::Polite],
            ),
            (
                "待ちいたします",
                "待つ",
                vec![Reason::Humble, Reason::Polite],
            ),
            ("待ち致します", "待つ", vec![Reason::Humble, Reason::Polite]),
            (
                "食べいたします",
                "食べる",
                vec![Reason::Humble, Reason::Polite],
            ),
            (
                "食べ致します",
                "食べる",
                vec![Reason::Humble, Reason::Polite],
            ),
        ];

        for (inflected, plain, reasons) in cases {
            let result = deinflector.deinflect(inflected);
            let match_candidate = result.iter().find(|candidate| candidate.word == plain);
            assert!(
                match_candidate.is_some(),
                "Failed to find '{}' for '{}'",
                plain,
                inflected
            );
            let candidate = match_candidate.unwrap();
            assert_eq!(candidate.reason_chains, vec![reasons]);
        }
    }

    #[test]
    fn test_deinflects_zaru_wo_enai() {
        let deinflector = Deinflector::new();
        let cases = vec![
            "闘わざるを得なかった",
            "闘わざるをえなかった",
            "やらざるを得ぬ",
            "やらざるをえぬ",
            "闘わざる得なかった",
            "闘わざるえなかった",
            "やらざる得ぬ",
            "やらざるえぬ",
        ];
        for inflected in cases {
            let result = deinflector.deinflect(inflected);
            let match_candidate = result
                .iter()
                .find(|candidate| candidate.word == "闘う" || candidate.word == "やる");
            assert!(
                match_candidate.is_some(),
                "Failed to find result for '{}'",
                inflected
            );
            let candidate = match_candidate.unwrap();
            // The ざるを得ない reason should be the first one in the list
            assert_eq!(candidate.reason_chains[0][0], Reason::ZaruWoEnai);
        }
    }

    #[test]
    fn test_deinflects_naide() {
        let deinflector = Deinflector::new();
        let cases = vec![
            "遊ばないで",
            "やらないで",
            "食べないで",
            "しないで",
            "こないで",
            "来ないで",
        ];
        for inflected in cases {
            let result = deinflector.deinflect(inflected);
            let match_candidate = result.iter().find(|candidate| {
                candidate.word == "遊ぶ"
                    || candidate.word == "やる"
                    || candidate.word == "食べる"
                    || candidate.word == "する"
                    || candidate.word == "くる"
                    || candidate.word == "来る"
            });
            assert!(
                match_candidate.is_some(),
                "Failed to find result for '{}'",
                inflected
            );
            let candidate = match_candidate.unwrap();
            assert_eq!(candidate.reason_chains[0][0], Reason::NegativeTe);
        }
    }

    #[test]
    fn test_deinflects_eru_uru() {
        let deinflector = Deinflector::new();
        let cases = vec![
            "し得る",
            "しえる",
            "しうる",
            "来得る",
            "あり得る",
            "考え得る",
        ];
        for inflected in cases {
            let result = deinflector.deinflect(inflected);
            let match_candidate = result.iter().find(|candidate| {
                candidate.word == "する"
                    || candidate.word == "来る"
                    || candidate.word == "ある"
                    || candidate.word == "考える"
            });
            assert!(
                match_candidate.is_some(),
                "Failed to find result for '{}'",
                inflected
            );
            let candidate = match_candidate.unwrap();
            assert_eq!(candidate.reason_chains, vec![vec![Reason::EruUru]]);
        }
    }

    #[test]
    fn test_kana_conversion() {
        let deinflector = Deinflector::new();

        // Test katakana to hiragana conversion
        assert_eq!(deinflector.kana_to_hiragana("カタカナ"), "かたかな");
        assert_eq!(deinflector.kana_to_hiragana("ひらがな"), "ひらがな"); // Already hiragana
    }
}
