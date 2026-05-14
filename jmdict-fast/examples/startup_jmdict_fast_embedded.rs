use std::hint::black_box;
use std::time::Instant;

use jmdict_fast::Dict;

fn main() {
    let word = "猫";

    let t0 = Instant::now();
    let dict = Dict::load_embedded().expect("Failed to load embedded dictionary");
    let load = t0.elapsed();

    let t1 = Instant::now();
    let results = black_box(dict.lookup_exact(black_box(word)));
    let first_lookup = t1.elapsed();
    let n = results.len();

    let rss_mb = memory_stats::memory_stats()
        .map(|s| s.physical_mem as f64 / 1024.0 / 1024.0)
        .unwrap_or(f64::NAN);

    println!(
        "ENGINE=jmdict-fast-embedded LOAD_MS={:.3} FIRST_LOOKUP_US={:.3} RSS_MB={:.2} RESULTS={}",
        load.as_secs_f64() * 1000.0,
        first_lookup.as_secs_f64() * 1_000_000.0,
        rss_mb,
        n,
    );
}
