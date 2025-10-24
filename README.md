# jmdict-fst

A monorepo containing blazing-fast Japanese dictionary and grammar tools.

## Crates

This repository contains two independently-published crates:

### 📚 [jmdict-fast](./jmdict-fast/)

Blazing-fast Japanese dictionary engine with FST-based indexing.

- Kanji, kana, and romaji lookup
- O(log n) search performance
- Memory-mapped, zero allocations
- JMdict-based dictionary data

### 📖 [bunpo](./bunpo/)

Lightweight Japanese conjugation reversal (deinflection) system.

- Rule-based verb/adjective deinflection
- No external dependencies
- Used by jmdict-fast for conjugation handling

## Installation

```toml
[dependencies]
jmdict-fast = "0.1.1"
bunpo = "0.1.1"  # Optional - only if you need conjugation handling
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

