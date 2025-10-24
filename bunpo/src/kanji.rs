pub fn is_kanji(c: char) -> bool {
    ('\u{4E00}'..='\u{9FFF}').contains(&c)
}

pub fn is_kanji_word(word: &str) -> bool {
    word.chars().all(|c| is_kanji(c))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_kanji() {
        assert_eq!(is_kanji('漢'), true);
        assert_eq!(is_kanji('a'), false);
    }

    #[test]
    fn test_is_kanji_word() {
        assert_eq!(is_kanji_word("漢字"), true);
        assert_eq!(is_kanji_word("あ"), false);
    }
}
