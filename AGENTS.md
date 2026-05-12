# AGENTS.md

Guidance for AI coding agents working in this repository. Humans should read [README.md](./README.md) first.

## Repository layout

Cargo workspace with three members:

- `jmdict-fast/` — FST-backed Japanese dictionary engine (published crate)
- `bunpo/` — Deinflection engine for Japanese verbs/adjectives (published crate)
- `xtask/` — Build tooling that generates the dictionary data files (not published; `publish = false`)

Generated artifacts land in `dist/` (gitignored). The data tarball published with each release is named `jmdict-data-jmdict<JMDICT_VERSION>-fmt<FORMAT_VERSION>.tar.gz`.

## Commands

| Task | Command |
|---|---|
| Generate dictionary data | `cargo xtask generate` |
| Build everything | `cargo build --workspace` |
| Run all tests | `cargo test --workspace` |
| Check `jmdict-fast` without default features | `cargo check -p jmdict-fast --no-default-features` |
| Check the `embedded` feature builds | `cargo check -p jmdict-fast --features embedded` (requires `dist/` to exist) |
| Lint | `cargo clippy --workspace --all-targets` |
| Format | `cargo fmt --all` |

Tests, the no-default-features check, and `cargo xtask generate` all run in CI (`.github/workflows/ci.yml`). Match those before declaring work done.

## Feature flags worth knowing

- `embedded` on `jmdict-fast` bakes `dist/` data into the binary via `include_bytes!`. It is **off by default**; runtime loading is the default path. Don't reintroduce unconditional `include_bytes!`.
- `MAGIC` and `FORMAT_VERSION` constants are owned by `jmdict-fast` and re-used by `xtask`. Keep them in one place (see commit `bd7919c`).

## Conventions

- **Conventional Commits** are required — `release-plz` parses them to generate `CHANGELOG.md` and bump versions. Use `feat:`, `fix:`, `chore:`, `refactor:`, `ci:`, `docs:`, etc. Breaking changes use `!` or a `BREAKING CHANGE:` footer.
- **Releases are automated** by `release-plz`. Do not hand-edit `CHANGELOG.md` or bump versions in `Cargo.toml` manually — open a normal PR with a conventional-commit title and let the Release PR handle the bump.
- **Workspace version** lives in the root `Cargo.toml` under `[workspace.package]`. Member crates inherit via `version.workspace = true`.
- **Edition**: 2021. **MSRV**: whatever the latest stable supports unless CI says otherwise.
- **`xtask` stays unpublished.** It is excluded from `release-plz.toml`. Don't add it to crates.io.

## Data files

- The FST indexes and `entries.bin` blob in `dist/` are generated, not committed. If a test needs them, generate first.
- The format version (`FORMAT_VERSION`) must be bumped when the on-disk layout changes. The released tarball name encodes both the JMdict source version and the format version so consumers can detect mismatches.

## Things to avoid

- Don't commit anything under `dist/`.
- Don't bypass `release-plz` by tagging or publishing manually unless explicitly asked.
- Don't add backwards-compatibility shims for the old "always-embedded" API — that migration is done (see README "Migration from embedded-only API"). Embedded mode is opt-in now.
- Don't pull in heavy runtime dependencies in `bunpo` — it's intentionally zero-dep.

## When opening a PR

- Base branch: `main`.
- Title: conventional commit (`feat: …`, `fix: …`, etc.).
- Make sure `cargo test --workspace`, `cargo check -p jmdict-fast --no-default-features`, and `cargo xtask generate` all pass — these are the gates CI runs.
