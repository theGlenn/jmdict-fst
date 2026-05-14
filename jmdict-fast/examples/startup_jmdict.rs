use std::hint::black_box;
use std::time::Instant;

use jmdict::entries;

fn main() {
    let word = "猫";

    // The jmdict crate has no separate load step; data is embedded as `entries()`.
    // Linear scan during the lookup is the cost. Report it as FIRST_LOOKUP_US.
    let t0 = Instant::now();
    let results: Vec<_> = entries()
        .filter(|entry| {
            entry.kanji_elements().any(|k| k.text == word)
                || entry.reading_elements().any(|r| r.text == word)
        })
        .collect();
    let first_lookup = t0.elapsed();
    let n = black_box(results).len();

    let rss_mb = memory_stats::memory_stats()
        .map(|s| s.physical_mem as f64 / 1024.0 / 1024.0)
        .unwrap_or(f64::NAN);

    println!(
        "ENGINE=jmdict LOAD_MS=0.000 FIRST_LOOKUP_US={:.3} RSS_MB={:.2} RESULTS={}",
        first_lookup.as_secs_f64() * 1_000_000.0,
        rss_mb,
        n,
    );
}
