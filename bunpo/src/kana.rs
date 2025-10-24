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

pub fn is_kana_word(word: &str) -> bool {
    word.chars().all(|c| is_hiragana(c))
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
}
