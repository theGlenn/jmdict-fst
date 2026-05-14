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

fn bench_lookup_word_jisho(c: &mut Criterion) {
    let word = "猫";
    let warmup = jisho::lookup(word);
    eprintln!("jisho::lookup(\"猫\") returned {} entries", warmup.len());

    c.bench_function("lookup 猫 (jisho)", |b| {
        b.iter(|| {
            let _ = jisho::lookup(black_box(word));
        });
    });
}

criterion_group!(
    benches,
    bench_lookup_word_fast,
    bench_lookup_word_jmdict,
    bench_lookup_word_jisho
);
criterion_main!(benches);
