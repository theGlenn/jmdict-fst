//! Demo API: a tiny shim around `jmdict-fast-flutter` that the example app
//! calls into. Each `pub fn` here becomes a Dart function in the generated
//! bindings.
//!
//! Real Flutter apps that depend on `jmdict-fast-flutter` follow this same
//! pattern: their own Rust crate has whatever shim / orchestration they
//! want, and pulls in `jmdict-fast-flutter` for the heavy lifting.

use std::sync::{Arc, Mutex};

use jmdict_fast_flutter::api::Dict;

/// Process-wide singleton dictionary handle. Set by `init_dictionary`,
/// cloned (cheap — internal `Arc`) for every lookup so the mutex isn't
/// held across an `await`. A real app would propagate the handle through
/// its state layer instead of a static.
static DICT: Mutex<Option<Arc<Dict>>> = Mutex::new(None);

/// Compact lookup row designed for the demo screen: just enough to render
/// a list entry. The full `LookupResult` is reachable through the
/// `jmdict-fast-flutter` crate directly for apps that need it.
pub struct DemoHit {
    pub kanji: Option<String>,
    pub kana: Option<String>,
    pub gloss: Option<String>,
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}

/// Load the dictionary from `data_dir`. Must be called before any lookup.
/// `async` so FRB runs the mmap on a worker thread — the first call
/// touches every FST file.
pub async fn init_dictionary(data_dir: String) -> Result<u64, String> {
    let dict = Dict::load(data_dir).await.map_err(|e| e.to_string())?;
    let count = dict.entry_count();
    *DICT.lock().unwrap() = Some(Arc::new(dict));
    Ok(count)
}

/// Whether `init_dictionary` has been called and succeeded.
#[flutter_rust_bridge::frb(sync)]
pub fn is_dictionary_ready() -> bool {
    DICT.lock().unwrap().is_some()
}

/// Exact match across kana/kanji/romaji. Sync because an FST hit is
/// microsecond-scale — no point making the Dart side `await` it.
#[flutter_rust_bridge::frb(sync)]
pub fn lookup_exact(term: String) -> Vec<DemoHit> {
    handle().lookup_exact(term).into_iter().map(into_demo_hit).collect()
}

/// Prefix search ("starts with").
#[flutter_rust_bridge::frb(sync)]
pub fn lookup_partial(prefix: String) -> Vec<DemoHit> {
    handle()
        .lookup_partial(prefix)
        .into_iter()
        .map(into_demo_hit)
        .collect()
}

/// Reverse lookup by English gloss. `async` because posting-list
/// intersection over common tokens can run thousands of entries.
pub async fn lookup_gloss(query: String) -> Vec<DemoHit> {
    let dict = handle();
    dict.lookup_gloss(query)
        .await
        .into_iter()
        .map(into_demo_hit)
        .collect()
}

fn handle() -> Arc<Dict> {
    DICT.lock()
        .unwrap()
        .as_ref()
        .cloned()
        .expect("init_dictionary must be called before lookups")
}

fn into_demo_hit(r: jmdict_fast_flutter::api::LookupResult) -> DemoHit {
    DemoHit {
        kanji: r.entry.kanji.first().map(|k| k.text.clone()),
        kana: r.entry.kana.first().map(|k| k.text.clone()),
        gloss: r
            .entry
            .sense
            .first()
            .and_then(|s| s.gloss.first())
            .map(|g| g.text.clone()),
    }
}
