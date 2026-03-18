use anyhow::Result;
use deunicode::deunicode;
use fst::MapBuilder;
use postcard;
use serde::{Deserialize, Serialize};
use serde_json;
use std::env;
use std::path::{Path, PathBuf};
use std::{fs::File, io::BufReader};

// Dict types inlined from build_utils (now in xtask)
#[derive(Debug, Deserialize, Serialize, Clone)]
struct Entry {
    id: String,
    kanji: Vec<KanjiEntry>,
    kana: Vec<KanaEntry>,
    sense: Vec<SenseEntry>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct KanjiEntry {
    common: bool,
    text: String,
    tags: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct KanaEntry {
    common: bool,
    text: String,
    tags: Vec<String>,
    #[serde(rename = "appliesToKanji")]
    applies_to_kanji: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct SenseEntry {
    #[serde(rename = "partOfSpeech")]
    part_of_speech: Vec<String>,
    #[serde(rename = "appliesToKanji")]
    applies_to_kanji: Vec<String>,
    #[serde(rename = "appliesToKana")]
    applies_to_kana: Vec<String>,
    related: Vec<serde_json::Value>,
    antonym: Vec<serde_json::Value>,
    field: Vec<String>,
    dialect: Vec<String>,
    misc: Vec<String>,
    info: Vec<String>,
    #[serde(rename = "languageSource")]
    language_source: Vec<serde_json::Value>,
    gloss: Vec<GlossEntry>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct GlossEntry {
    lang: String,
    gender: Option<String>,
    #[serde(rename = "type")]
    gloss_type: Option<String>,
    text: String,
}

#[derive(Debug, Deserialize)]
struct JmdictData {
    words: Vec<Entry>,
}

fn main() -> Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir);

    // Check if generated files already exist in OUT_DIR - skip regeneration
    if out_path.join("entries.bin").exists()
        && out_path.join("kana.fst").exists()
        && out_path.join("kanji.fst").exists()
        && out_path.join("romaji.fst").exists()
        && out_path.join("id.fst").exists()
    {
        eprintln!("✅ Generated data files already exist in OUT_DIR, skipping generation");
        return Ok(());
    }

    // Look for cached JSON from xtask in .cache/ directory
    let json_path = find_cached_json()?;

    eprintln!("Parsing JSON from {:?}...", json_path);
    let file = File::open(&json_path)?;
    let rdr = BufReader::new(file);
    let data: JmdictData = serde_json::from_reader(rdr)?;
    let entries = data.words;
    eprintln!("entries: {:?}", entries.len());

    eprintln!("Extracting index keys...");
    let mut kanji_map = Vec::new();
    let mut kana_map = Vec::new();
    let mut romaji_map = Vec::new();

    let mut id_mapping = Vec::new();

    let mut trimmed_entries = Vec::new();
    for (seq_id, entry) in entries.iter().enumerate() {
        let id = seq_id as u64;
        id_mapping.push((entry.id.clone(), id));
        for k in &entry.kanji {
            kanji_map.push((k.text.clone(), id));
            romaji_map.push((deunicode(&k.text).to_lowercase(), id));
        }
        for k in &entry.kana {
            kana_map.push((k.text.clone(), id));
            romaji_map.push((deunicode(&k.text).to_lowercase(), id));
        }

        // Create optimized entry by trimming unused fields
        let mut trimmed = entry.clone();
        for sense in &mut trimmed.sense {
            sense.antonym.clear();
            sense.info.clear();
            sense.field.clear();
            sense.dialect.clear();
            sense.misc.clear();
            sense.language_source.clear();
            sense.related.clear();
        }
        trimmed_entries.push(trimmed);
    }

    if !kana_map.iter().any(|(k, _)| k == "たべる") {
        eprintln!("⚠️ 'たべる' is missing from kana index!");
    }

    eprintln!("Sorting and deduplicating...");
    kanji_map.sort();
    kana_map.sort();
    romaji_map.sort();
    id_mapping.sort();

    kanji_map.dedup_by_key(|(key, _)| key.clone());
    kana_map.dedup_by_key(|(key, _)| key.clone());
    romaji_map.dedup_by_key(|(key, _)| key.clone());
    id_mapping.dedup_by_key(|(key, _)| key.clone());

    eprintln!("Building FSTs...");
    eprintln!("  - kanji.fst: {} entries", kanji_map.len());
    eprintln!("  - kana.fst: {} entries", kana_map.len());
    eprintln!("  - romaji.fst: {} entries", romaji_map.len());
    eprintln!("  - id.fst: {} entries", id_mapping.len());

    write_fst(&out_path.join("kanji.fst"), &kanji_map)?;
    write_fst(&out_path.join("kana.fst"), &kana_map)?;
    write_fst(&out_path.join("romaji.fst"), &romaji_map)?;
    write_fst(&out_path.join("id.fst"), &id_mapping)?;

    eprintln!("Writing binary blob...");
    eprintln!("Writing {} entries to binary blob", trimmed_entries.len());
    write_blob(&out_path.join("entries.bin"), &trimmed_entries)?;

    eprintln!("Done ✅");
    Ok(())
}

/// Find the cached JMdict JSON file from xtask's .cache/ directory
fn find_cached_json() -> Result<PathBuf> {
    // Walk up from CARGO_MANIFEST_DIR to find workspace root with .cache/
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    let mut dir = PathBuf::from(&manifest_dir);

    loop {
        let cache_path = dir.join(".cache").join("jmdict-common.json");
        if cache_path.exists() {
            return Ok(cache_path);
        }
        if !dir.pop() {
            break;
        }
    }

    anyhow::bail!(
        "JMdict JSON not found. Run 'cargo xtask generate' first to download the dictionary data."
    )
}

fn write_fst(path: &Path, entries: &[(String, u64)]) -> Result<()> {
    let wtr = File::create(path)?;
    let mut builder = MapBuilder::new(wtr)?;
    for (k, v) in entries {
        builder.insert(k, *v)?;
    }
    builder.finish()?;
    Ok(())
}

fn write_blob(path: &Path, entries: &[Entry]) -> Result<()> {
    use std::io::{BufWriter, Write};

    let mut out = BufWriter::new(File::create(path)?);

    let entry_count = entries.len() as u32;
    out.write_all(&entry_count.to_le_bytes())?;

    let mut offset_table = Vec::new();
    let mut data_blob = Vec::new();

    for entry in entries {
        let postcard_data = postcard::to_allocvec(entry)?;
        let offset = data_blob.len() as u32;
        let len = postcard_data.len() as u32;
        offset_table.push((offset, len));
        data_blob.extend(postcard_data);
    }

    for (offset, len) in &offset_table {
        out.write_all(&offset.to_le_bytes())?;
        out.write_all(&len.to_le_bytes())?;
    }

    out.write_all(&data_blob)?;
    Ok(())
}
