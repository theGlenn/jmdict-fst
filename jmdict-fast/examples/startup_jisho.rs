use std::hint::black_box;
use std::time::Instant;

fn main() {
    let word = "猫";

    // jisho uses lazy_static — the dictionary is loaded inside the first lookup() call.
    // Report the combined time as FIRST_LOOKUP_US; LOAD_MS is reported as 0 since there is
    // no separate load step you can call ahead of time.
    let t0 = Instant::now();
    let results = black_box(jisho::lookup(black_box(word)));
    let first_lookup = t0.elapsed();
    let n = results.len();

    let rss_mb = memory_stats::memory_stats()
        .map(|s| s.physical_mem as f64 / 1024.0 / 1024.0)
        .unwrap_or(f64::NAN);

    println!(
        "ENGINE=jisho LOAD_MS=0.000 FIRST_LOOKUP_US={:.3} RSS_MB={:.2} RESULTS={}",
        first_lookup.as_secs_f64() * 1_000_000.0,
        rss_mb,
        n,
    );
}
