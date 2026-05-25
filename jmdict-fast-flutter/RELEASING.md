# Releasing `jmdict_fast` to pub.dev

This package ships with **precompiled native binaries** (no rustup
required on the consumer side). CI builds + signs the rust crate for
every supported target and uploads to a GitHub Release before publishing
to pub.dev.

## One-time setup

Only needed once (or when rotating the signing key).

### 1. Generate the cargokit signing keypair

```sh
cd jmdict-fast-flutter/flutter_package/cargokit/build_tool
dart pub get
dart bin/build_tool.dart gen-key
```

Output:

```
Private Key: <128 hex chars>
Public Key:  <64 hex chars>
```

Treat the private key like a password. **Back it up to 1Password (or
similar) before doing anything else** — if it's lost you have to rotate
the public key, which invalidates every binary already published under
the old key.

### 2. Register the private key as a GitHub secret

https://github.com/theGlenn/jmdict-fst/settings/secrets/actions →
**New repository secret**:

- Name: `CARGOKIT_PRIVATE_KEY`
- Value: the 128-hex-char private key

### 3. Paste the public key into `cargokit.yaml`

Edit `jmdict-fast-flutter/flutter_package/rust/cargokit.yaml`, replace
`PASTE_PUBLIC_KEY_HERE` with the 64-hex-char public key. Commit.

### 4. Confirm pub.dev "Automated publishing" is on

https://pub.dev/packages/jmdict_fast/admin → **Automated publishing**
section. Repository should be `theGlenn/jmdict-fst`, tag pattern
`flutter-v{{version}}`. This was set up during PR #35.

## Cutting a release

1. Bump versions:
   - `jmdict-fast-flutter/flutter_package/pubspec.yaml`
   - `jmdict-fast-flutter/flutter_package/ios/jmdict_fast.podspec`
   - `jmdict-fast-flutter/flutter_package/macos/jmdict_fast.podspec`
   - `jmdict-fast-flutter/flutter_package/android/build.gradle`
   - `jmdict-fast-flutter/flutter_package/rust/Cargo.toml`
2. Add a CHANGELOG entry, land on `main`.
3. Tag and push:

   ```sh
   git tag flutter-v0.1.7
   git push origin flutter-v0.1.7
   ```

4. The `Publish flutter_package to pub.dev` workflow runs:
   - **precompile-darwin** (~10 min) — iOS + macOS arm64/x86_64
   - **precompile-linux** (~15 min) — Android arm64/armv7/x86_64/i686 + Linux x86_64
   - **precompile-windows** (~8 min) — Windows x86_64
   - **publish** (~2 min) — blocks until all three precompile jobs succeed, then `flutter pub publish`

   Total tag-to-pub.dev: ~15–20 min.

## How consumers consume

When a consumer runs `flutter pub add jmdict_fast`, cargokit:

1. Reads `rust/cargokit.yaml`
2. Hashes the contents of `rust/`
3. Downloads `<target>_libjmdict_fast_flutter.{a,so,dll,dylib}` and the
   matching `.sig` from
   `https://github.com/theGlenn/jmdict-fst/releases/download/precompiled_<hash>/`
4. Verifies the signature against the public key
5. Links the binary — no cargo, no rustup

If the target isn't in the release (e.g. an unsupported arch), cargokit
falls back to source build, which keeps the consumer functional but
requires their machine to have rustup + the relevant toolchain target.

## Re-runs and idempotency

The precompile jobs are idempotent. Each target's artifact is keyed on
both the crate hash and the target triple; if the asset already exists on
the `precompiled_<hash>` release, the job skips the rebuild. So:

- **Re-tagging a version with no rust changes**: precompile jobs find
  every asset, skip every build, finish in seconds.
- **Mid-job crash + re-run via `workflow_dispatch`**: only the targets
  that haven't been uploaded get rebuilt.
- **Bumping the version but not touching rust source**: same hash, same
  release, instant precompile phase.

## Rotating the signing key

If the private key leaks:

1. Generate a new keypair (`gen-key`).
2. Replace the `CARGOKIT_PRIVATE_KEY` secret on GitHub.
3. Update `cargokit.yaml` with the new public key.
4. Bump the package version + tag — the new release ships under the new
   key, old releases keep working with the old signatures (they're
   already cached on consumers).

To force re-verification on the old releases, delete the
`precompiled_<hash>` GitHub Releases for those versions; existing
consumers fall through to source build, new consumers get the new key.
