use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jmdict::entries;
use jmdict_fast::Dict;

fn bench_lookup_word_fast(c: &mut Criterion) {
    let dict = black_box(Dict::load_embedded().expect("Failed to load embedded dictionary"));
    let word = "猫";

    c.bench_function("lookup_exact 猫 (jmdict-fast)", |b| {
        b.iter(|| {
            let _ = dict.lookup_exact(black_box(word));
        });
    });
}

fn bench_lookup_word_jmdict(c: &mut Criterion) {
    let word = "猫";

    c.bench_function("lookup_word 猫 (jmdict)", |b| {
        b.iter(|| {
            let _ = entries()
                .filter(|entry| {
                    entry.kanji_elements().any(|k| k.text == word)
                        || entry.reading_elements().any(|r| r.text == word)
                })
                .collect::<Vec<_>>();
        });
    });
}

criterion_group!(benches, bench_lookup_word_fast, bench_lookup_word_jmdict);
criterion_main!(benches);
