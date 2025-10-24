use bunpo::deinflector::Deinflector;
use jmdict_fast::Dict;

fn main() -> anyhow::Result<()> {
    println!("=== Japanese Lightweight Conjugation Reversal Demo ===\n");

    // Initialize the deinflector with default rules
    let deinflector = Deinflector::new();

    // Try to load the dictionary (optional - for validation)
    let dict = Dict::load_default().ok();

    // Example conjugated forms to test
    let test_words = vec![
        "食べます",   // Polite form of 食べる
        "食べた",     // Past tense of 食べる
        "食べて",     // Te-form of 食べる
        "美しかった", // Past tense of 美しい
        "美しくない", // Negative of 美しい
        "学生です",   // Polite copula
        "行きます",   // Polite form of 行く
        "見られる",   // Potential form of 見る
        "書かせる",   // Causative form of 書く
        "猫",         // Not conjugated
    ];

    for word in test_words {
        println!("Input: {}", word);

        // Deinflect to find possible lemmas
        let candidates = deinflector.deinflect(word);
        println!("  Possible lemmas:");
        for candidate in &candidates {
            println!(
                "    - {} (type: {:?}, reasons: {:?})",
                candidate.word, candidate.word_type, candidate.reason_chains
            );

            if let Some(ref dict) = dict {
                let mut valid_entries = Vec::new();
                for candidate in &candidates {
                    valid_entries.extend(dict.lookup_exact(&candidate.word));
                }

                if !valid_entries.is_empty() {
                    println!("  Valid dictionary entries found:");
                    for entry in valid_entries.iter().take(3) {
                        // Limit to first 3
                        if let Some(kanji) = entry.kanji.first() {
                            println!(
                                "    - {} ({})",
                                kanji.text,
                                entry
                                    .kana
                                    .first()
                                    .map(|k| &k.text)
                                    .unwrap_or(&"".to_string())
                            );
                        }
                    }
                    if valid_entries.len() > 3 {
                        println!("    ... and {} more entries", valid_entries.len() - 3);
                    }
                } else {
                    println!("  No valid dictionary entries found");
                }
            }
        }

        println!();
    }

    println!("\n=== Performance Note ===");
    println!("This lightweight system uses:");
    println!("- Precompiled suffix matching");
    println!("- Rule-based deinflection");
    println!("- No morphological analysis required");
    println!("- Fast O(n*m) complexity where n=word length, m=number of rules");
    println!("- Memory efficient: ~10KB for all rules");

    Ok(())
}
