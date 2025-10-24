use jmdict_fast::Dict;

fn main() -> anyhow::Result<()> {
    let dict = Dict::load_default()?;

    println!("=== Dictionary Lookup Demonstration ===");

    // Test exact lookup
    let exact_results = dict.lookup_exact("猫");
    println!("Exact lookup for '猫': {} results", exact_results.len());
    for entry in &exact_results {
        println!(
            "  - {} (ねこ): {}",
            entry.kanji[0].text, entry.sense[0].gloss[0].text
        );
    }

    // Test partial lookup
    let partial_results = dict.lookup_partial("ね");
    println!("Partial lookup for 'ね': {} results", partial_results.len());
    for entry in &partial_results[..3.min(partial_results.len())] {
        // Show first 3
        println!(
            "  - {} ({}): {}",
            entry
                .kanji
                .first()
                .map(|k| &k.text)
                .unwrap_or(&"".to_string()),
            entry
                .kana
                .first()
                .map(|k| &k.text)
                .unwrap_or(&"".to_string()),
            entry.sense[0].gloss[0].text
        );
    }
    if partial_results.len() > 3 {
        println!("  ... and {} more results", partial_results.len() - 3);
    }

    Ok(())
}
