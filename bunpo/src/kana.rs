use std::collections::HashMap;

pub fn to_hiragana(input: &str) -> String {
    let mut result = String::new();
    let mut is_katakana = false;
    for c in input.chars() {
        if ('\u{30A1}'..='\u{30F6}').contains(&c) {
            let hira = std::char::from_u32(c as u32 - 0x60).unwrap_or(c);
            result.push(hira);
            is_katakana = true;
        } else {
            result.push(c);
        }
    }
    if is_katakana {
        return result;
    }

    romaji_to_hiragana(input)
}

fn romaji_to_hiragana(input: &str) -> String {
    let table = romaji_hiragana_table();
    let mut result = String::new();
    let mut i = 0;
    let chars: Vec<char> = input.to_lowercase().chars().collect();

    while i < chars.len() {
        let mut matched = false;
        for len in (1..=3).rev() {
            if i + len <= chars.len() {
                let slice: String = chars[i..i + len].iter().collect();
                if let Some(hira) = table.get(slice.as_str()) {
                    result.push_str(hira);
                    i += len;
                    matched = true;
                    break;
                }
            }
        }
        if !matched {
            if i + 1 < chars.len()
                && chars[i] == chars[i + 1]
                && is_consonant(chars[i])
                && chars[i] != 'n'
            {
                result.push('っ');
                i += 1;
            } else if chars[i] == 'n' {
                if i + 1 == chars.len() || !is_vowel(chars[i + 1]) {
                    result.push('ん');
                    i += 1;
                } else {
                    i += 1;
                }
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }
    }
    result
}

fn is_consonant(c: char) -> bool {
    matches!(
        c,
        'b' | 'c'
            | 'd'
            | 'f'
            | 'g'
            | 'h'
            | 'j'
            | 'k'
            | 'l'
            | 'm'
            | 'p'
            | 'q'
            | 'r'
            | 's'
            | 't'
            | 'v'
            | 'w'
            | 'x'
            | 'z'
    )
}

fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'i' | 'u' | 'e' | 'o')
}

fn romaji_hiragana_table() -> HashMap<&'static str, &'static str> {
    // non-exhaustive table, but covers most common syllables and digraphs
    let mut m = HashMap::new();
    // Vowels
    m.insert("a", "あ");
    m.insert("i", "い");
    m.insert("u", "う");
    m.insert("e", "え");
    m.insert("o", "お");
    // K
    m.insert("ka", "か");
    m.insert("ki", "き");
    m.insert("ku", "く");
    m.insert("ke", "け");
    m.insert("ko", "こ");
    m.insert("kya", "きゃ");
    m.insert("kyu", "きゅ");
    m.insert("kyo", "きょ");
    // S
    m.insert("sa", "さ");
    m.insert("shi", "し");
    m.insert("su", "す");
    m.insert("se", "せ");
    m.insert("so", "そ");
    m.insert("sha", "しゃ");
    m.insert("shu", "しゅ");
    m.insert("sho", "しょ");
    // T
    m.insert("ta", "た");
    m.insert("chi", "ち");
    m.insert("tsu", "つ");
    m.insert("te", "て");
    m.insert("to", "と");
    m.insert("cha", "ちゃ");
    m.insert("chu", "ちゅ");
    m.insert("cho", "ちょ");
    // N
    m.insert("na", "な");
    m.insert("ni", "に");
    m.insert("nu", "ぬ");
    m.insert("ne", "ね");
    m.insert("no", "の");
    m.insert("nya", "にゃ");
    m.insert("nyu", "にゅ");
    m.insert("nyo", "にょ");
    // H
    m.insert("ha", "は");
    m.insert("hi", "ひ");
    m.insert("fu", "ふ");
    m.insert("he", "へ");
    m.insert("ho", "ほ");
    m.insert("hya", "ひゃ");
    m.insert("hyu", "ひゅ");
    m.insert("hyo", "ひょ");
    // M
    m.insert("ma", "ま");
    m.insert("mi", "み");
    m.insert("mu", "む");
    m.insert("me", "め");
    m.insert("mo", "も");
    m.insert("mya", "みゃ");
    m.insert("myu", "みゅ");
    m.insert("myo", "みょ");
    // Y
    m.insert("ya", "や");
    m.insert("yu", "ゆ");
    m.insert("yo", "よ");
    // R
    m.insert("ra", "ら");
    m.insert("ri", "り");
    m.insert("ru", "る");
    m.insert("re", "れ");
    m.insert("ro", "ろ");
    m.insert("rya", "りゃ");
    m.insert("ryu", "りゅ");
    m.insert("ryo", "りょ");
    // W
    m.insert("wa", "わ");
    m.insert("wo", "を");
    // G
    m.insert("ga", "が");
    m.insert("gi", "ぎ");
    m.insert("gu", "ぐ");
    m.insert("ge", "げ");
    m.insert("go", "ご");
    m.insert("gya", "ぎゃ");
    m.insert("gyu", "ぎゅ");
    m.insert("gyo", "ぎょ");
    // Z
    m.insert("za", "ざ");
    m.insert("ji", "じ");
    m.insert("zu", "ず");
    m.insert("ze", "ぜ");
    m.insert("zo", "ぞ");
    m.insert("ja", "じゃ");
    m.insert("ju", "じゅ");
    m.insert("jo", "じょ");
    // D
    m.insert("da", "だ");
    m.insert("di", "ぢ");
    m.insert("du", "づ");
    m.insert("de", "で");
    m.insert("do", "ど");
    // B
    m.insert("ba", "ば");
    m.insert("bi", "び");
    m.insert("bu", "ぶ");
    m.insert("be", "べ");
    m.insert("bo", "ぼ");
    m.insert("bya", "びゃ");
    m.insert("byu", "びゅ");
    m.insert("byo", "びょ");
    // P
    m.insert("pa", "ぱ");
    m.insert("pi", "ぴ");
    m.insert("pu", "ぷ");
    m.insert("pe", "ぺ");
    m.insert("po", "ぽ");
    m.insert("pya", "ぴゃ");
    m.insert("pyu", "ぴゅ");
    m.insert("pyo", "ぴょ");
    // Small tsu
    m.insert("xtsu", "っ");
    m.insert("ltsu", "っ");
    // N
    m.insert("n", "ん");
    // Special
    m.insert("vu", "ゔ");
    m
}

/// Returns true if the character is a hiragana character.
pub fn is_hiragana(c: char) -> bool {
    ('\u{3040}'..='\u{309F}').contains(&c)
}

/// Returns true if the character is a katakana character.
///
/// Covers the standard Katakana block. Half-width katakana (U+FF66..=U+FF9F)
/// is intentionally excluded — callers that need it should normalize first.
pub fn is_katakana(c: char) -> bool {
    ('\u{30A0}'..='\u{30FF}').contains(&c)
}

/// Returns true if every character in `word` is hiragana.
pub fn is_kana_word(word: &str) -> bool {
    word.chars().all(is_hiragana)
}

/// Returns true if every character in `word` is katakana.
pub fn is_katakana_word(word: &str) -> bool {
    !word.is_empty() && word.chars().all(is_katakana)
}

/// Convert hiragana characters in `input` to katakana. Romaji and other
/// scripts are first run through [`to_hiragana`], so `to_katakana("neko")`
/// returns `"ネコ"`.
pub fn to_katakana(input: &str) -> String {
    let hira = to_hiragana(input);
    let mut out = String::with_capacity(hira.len());
    for c in hira.chars() {
        if is_hiragana(c) {
            // Hiragana → katakana is a fixed +0x60 shift.
            let k = std::char::from_u32(c as u32 + 0x60).unwrap_or(c);
            out.push(k);
        } else {
            out.push(c);
        }
    }
    out
}

/// Convert kana characters in `input` to lowercase ASCII romaji (Hepburn).
///
/// This is the inverse of [`to_hiragana`] for the syllables in the lookup
/// table. Characters that are not kana are passed through unchanged, which
/// makes the function safe to call on mixed-script strings like `"カフェ au lait"`.
pub fn to_romaji(input: &str) -> String {
    let table = hiragana_romaji_table();
    // Normalize to hiragana first so katakana and romaji-in both work.
    let hira = to_hiragana(input);
    let chars: Vec<char> = hira.chars().collect();
    let mut out = String::with_capacity(hira.len());
    let mut i = 0;
    while i < chars.len() {
        // Sokuon (small tsu) doubles the next consonant.
        if chars[i] == 'っ' && i + 1 < chars.len() {
            let next_slice: String = chars[i + 1..].iter().take(2).collect();
            let (rom, consumed) = lookup_kana(&table, &next_slice);
            if let Some(rom) = rom {
                if let Some(first) = rom.chars().next() {
                    out.push(first);
                }
                out.push_str(rom);
                // skip the small tsu plus the consumed kana
                i += 1 + consumed;
                continue;
            }
        }

        let slice: String = chars[i..].iter().take(2).collect();
        let (rom, consumed) = lookup_kana(&table, &slice);
        match rom {
            Some(rom) => {
                out.push_str(rom);
                i += consumed;
            }
            None => {
                out.push(chars[i]);
                i += 1;
            }
        }
    }
    out
}

/// Try a 2-char digraph first, then fall back to single-char. Returns the
/// matched romaji string plus the number of source chars consumed.
fn lookup_kana<'a>(
    table: &'a HashMap<&'static str, &'static str>,
    slice: &str,
) -> (Option<&'a &'static str>, usize) {
    if slice.chars().count() >= 2 {
        let two: String = slice.chars().take(2).collect();
        if let Some(rom) = table.get(two.as_str()) {
            return (Some(rom), 2);
        }
    }
    let one: String = slice.chars().take(1).collect();
    if let Some(rom) = table.get(one.as_str()) {
        return (Some(rom), 1);
    }
    (None, 1)
}

fn hiragana_romaji_table() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    // Built by inverting the romaji → hiragana table. We keep only the
    // canonical Hepburn spelling per kana — alternates like "si" / "shi" go
    // one way (romaji → hiragana) but not the other.
    let pairs: &[(&str, &str)] = &[
        ("あ", "a"), ("い", "i"), ("う", "u"), ("え", "e"), ("お", "o"),
        ("か", "ka"), ("き", "ki"), ("く", "ku"), ("け", "ke"), ("こ", "ko"),
        ("きゃ", "kya"), ("きゅ", "kyu"), ("きょ", "kyo"),
        ("さ", "sa"), ("し", "shi"), ("す", "su"), ("せ", "se"), ("そ", "so"),
        ("しゃ", "sha"), ("しゅ", "shu"), ("しょ", "sho"),
        ("た", "ta"), ("ち", "chi"), ("つ", "tsu"), ("て", "te"), ("と", "to"),
        ("ちゃ", "cha"), ("ちゅ", "chu"), ("ちょ", "cho"),
        ("な", "na"), ("に", "ni"), ("ぬ", "nu"), ("ね", "ne"), ("の", "no"),
        ("にゃ", "nya"), ("にゅ", "nyu"), ("にょ", "nyo"),
        ("は", "ha"), ("ひ", "hi"), ("ふ", "fu"), ("へ", "he"), ("ほ", "ho"),
        ("ひゃ", "hya"), ("ひゅ", "hyu"), ("ひょ", "hyo"),
        ("ま", "ma"), ("み", "mi"), ("む", "mu"), ("め", "me"), ("も", "mo"),
        ("みゃ", "mya"), ("みゅ", "myu"), ("みょ", "myo"),
        ("や", "ya"), ("ゆ", "yu"), ("よ", "yo"),
        ("ら", "ra"), ("り", "ri"), ("る", "ru"), ("れ", "re"), ("ろ", "ro"),
        ("りゃ", "rya"), ("りゅ", "ryu"), ("りょ", "ryo"),
        ("わ", "wa"), ("を", "wo"), ("ん", "n"),
        ("が", "ga"), ("ぎ", "gi"), ("ぐ", "gu"), ("げ", "ge"), ("ご", "go"),
        ("ぎゃ", "gya"), ("ぎゅ", "gyu"), ("ぎょ", "gyo"),
        ("ざ", "za"), ("じ", "ji"), ("ず", "zu"), ("ぜ", "ze"), ("ぞ", "zo"),
        ("じゃ", "ja"), ("じゅ", "ju"), ("じょ", "jo"),
        ("だ", "da"), ("ぢ", "ji"), ("づ", "zu"), ("で", "de"), ("ど", "do"),
        ("ば", "ba"), ("び", "bi"), ("ぶ", "bu"), ("べ", "be"), ("ぼ", "bo"),
        ("びゃ", "bya"), ("びゅ", "byu"), ("びょ", "byo"),
        ("ぱ", "pa"), ("ぴ", "pi"), ("ぷ", "pu"), ("ぺ", "pe"), ("ぽ", "po"),
        ("ぴゃ", "pya"), ("ぴゅ", "pyu"), ("ぴょ", "pyo"),
        ("ゔ", "vu"),
    ];
    for (k, v) in pairs {
        m.insert(*k, *v);
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_hiragana() {
        assert_eq!(is_hiragana('あ'), true);
        assert_eq!(is_hiragana('a'), false);
    }

    #[test]
    fn test_is_kana_word() {
        assert_eq!(is_kana_word("あ"), true);
        assert_eq!(is_kana_word("漢字"), false);
    }

    #[test]
    fn test_romaji_to_hiragana_basic() {
        assert_eq!(to_hiragana("ike"), "いけ");
        assert_eq!(to_hiragana("イケ"), "いけ");
        assert_eq!(to_hiragana("いけ"), "いけ");
        assert_eq!(to_hiragana("zubon"), "ずぼん");
        assert_eq!(to_hiragana("durai"), "づらい");
    }

    #[test]
    fn test_romaji_to_hiragana_small_tsu() {
        assert_eq!(to_hiragana("itt"), "いっt");
        assert_eq!(to_hiragana("itte"), "いって");
        assert_eq!(to_hiragana("ittt"), "いっっt");
        assert_eq!(to_hiragana("ittte"), "いっって");
        assert_eq!(to_hiragana("itttte"), "いっっって");
    }

    #[test]
    fn test_is_katakana() {
        assert!(is_katakana('ア'));
        assert!(is_katakana('ン'));
        assert!(!is_katakana('あ'));
        assert!(!is_katakana('a'));
    }

    #[test]
    fn test_is_katakana_word() {
        assert!(is_katakana_word("ネコ"));
        assert!(!is_katakana_word("ねこ"));
        assert!(!is_katakana_word(""));
        assert!(!is_katakana_word("ネコa"));
    }

    #[test]
    fn test_to_katakana() {
        assert_eq!(to_katakana("ねこ"), "ネコ");
        assert_eq!(to_katakana("ネコ"), "ネコ");
        assert_eq!(to_katakana("neko"), "ネコ");
    }

    #[test]
    fn test_to_romaji_basic() {
        assert_eq!(to_romaji("ねこ"), "neko");
        assert_eq!(to_romaji("ネコ"), "neko");
        assert_eq!(to_romaji("たべる"), "taberu");
        assert_eq!(to_romaji("にほん"), "nihon");
    }

    #[test]
    fn test_to_romaji_digraphs() {
        assert_eq!(to_romaji("きょう"), "kyou");
        assert_eq!(to_romaji("しゅみ"), "shumi");
    }

    #[test]
    fn test_to_romaji_sokuon() {
        assert_eq!(to_romaji("いって"), "itte");
        assert_eq!(to_romaji("がっこう"), "gakkou");
    }
}
