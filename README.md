# jmdict-fst

A monorepo for **high-performance Japanese dictionary and grammar tools**.

[![jmdict-fast](https://img.shields.io/crates/v/jmdict-fast.svg)](https://crates.io/crates/jmdict-fast)
[![bunpo](https://img.shields.io/crates/v/bunpo.svg)](https://crates.io/crates/bunpo)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

## Overview

This repository includes two Rust crates, published independently:

### 📚 [jmdict-fast](./jmdict-fast/)

A **blazing-fast Japanese dictionary engine** powered by FST (finite state transducer) indexing.

- Supports kanji, kana, and romaji lookups  
- Achieves **O(log n)** search performance  
- Uses memory-mapped data with zero allocations  
- Built from the official **JMdict** dictionary dataset

### 📖 [bunpo](./bunpo/)

A **lightweight deinflection engine** for Japanese verbs and adjectives.

- Rule-based conjugation reversal  
- Zero external dependencies  
- Integrated with `jmdict-fast` for conjugation-aware lookups

## Installation

Add one or both crates to your `Cargo.toml`:

```toml
[dependencies]
jmdict-fast = "0.1.1"
bunpo = "0.1.1"  # Optional – only needed for conjugation handling
```

## Quick Start

```rust
use jmdict_fast::Dict;

fn main() -> anyhow::Result<()> {
    let dict = Dict::load_default()?;
    
    // Lookup exact term
    let results = dict.lookup_exact("猫");
    for entry in &results {
        println!("{}: {}", entry.kanji[0].text, entry.sense[0].gloss[0].text);
    }
    
    // Prefix search
    let results = dict.lookup_partial("こんに");
    
    // With deinflection
    let results = dict.lookup_exact_with_deinflection("食べます");
    
    Ok(())
}
```

## License

MIT License - see [LICENSE](./LICENSE)

## Acknowledgments

- **JMdict** - The source dictionary data
- **FST crate** - Fast finite state transducer implementation

