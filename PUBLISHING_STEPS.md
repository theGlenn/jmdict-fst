# Publishing Steps to crates.io

## Current Status

✅ **bunpo** - Ready to publish (packaged successfully)  
⏳ **jmdict-fast** - Waiting for bunpo publication

---

## Step 1: Publish bunpo

```bash
cd bunpo
cargo publish
```

**Note:** If this is your first time publishing to crates.io, you'll need to:
1. Get a token from https://crates.io/settings/tokens
2. Run `cargo login <your-token>`

### After bunpo is published successfully

You should see it at: https://crates.io/crates/bunpo

---

## Step 2: Update jmdict-fast dependencies

Once `bunpo` is published, update `jmdict-fast/Cargo.toml`:

**Current (line 15):**
```toml
bunpo = { path = "../bunpo" }
```

**Change to:**
```toml
bunpo = "0.1.1"
```

**Also update build-dependencies (line 27):**
```toml
bunpo = "0.1.1"
```

---

## Step 3: Test jmdict-fast packaging

```bash
cd jmdict-fast
cargo package --allow-dirty
```

This will verify that everything works with the published `bunpo` dependency.

⚠️ **Important:** The build will download JMdict JSON and generate ~17MB of FST/bin files, which will be embedded in the crate.

---

## Step 4: Publish jmdict-fast

```bash
cd jmdict-fast
cargo publish
```

---

## Expected Results

- **bunpo**: Small crate (~14KB compressed)
- **jmdict-fast**: Large crate (~17MB+ due to embedded dictionary data)

### URLs after publication:
- bunpo: https://crates.io/crates/bunpo
- jmdict-fast: https://crates.io/crates/jmdict-fast

---

## Important Notes

### Binary File Size Issue

As discussed in `CRATES.IO_PREPARATION.md`, `jmdict-fast` will be large (~17MB) because it embeds the dictionary data. This is:
- ✅ Acceptable for v0.1.1 initial release
- ⚠️ Users will have longer build times
- ⚠️ Users need network access during `cargo build`
- 💡 Consider implementing runtime download in v0.2.0+

### First-Time Publishing

If you're publishing for the first time:
1. Visit https://crates.io
2. Sign in with GitHub
3. Go to https://crates.io/settings/tokens
4. Create an API token
5. Run `cargo login <token>`

### After Publishing

1. Wait a few minutes for docs.rs to build documentation
2. Check https://docs.rs/bunpo
3. Check https://docs.rs/jmdict-fast
4. Update README badges with crates.io links

---

## Need Help?

- Cargo publish docs: https://doc.rust-lang.org/cargo/commands/cargo-publish.html
- crates.io package policies: https://crates.io/policies

