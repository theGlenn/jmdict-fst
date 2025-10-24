# bunpo

Lightweight Japanese conjugation reversal (deinflection) system.

> **Note:** This crate is part of the [jmdict-fst](https://github.com/theGlenn/jmdict-fst) monorepo but can be used independently.

## Features

- Fast rule-based deinflection
- Supports verbs, adjectives, and copula
- No external dependencies
- Pure Rust implementation

## Example

```rust
use bunpo::deinflector::Deinflector;

let deinflector = Deinflector::new();
let results = deinflector.deinflect("食べます");

for candidate in results {
    println!("{}: {}", candidate.word, candidate.reason);
}
```

## Installation

```toml
[dependencies]
bunpo = "0.1.1"
```

## Documentation

See [DEINFLECTOR.md](./DEINFLECTOR.md) for detailed usage and rules.

## License

MIT

