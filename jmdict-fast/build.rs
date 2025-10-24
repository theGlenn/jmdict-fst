use anyhow::Result;
use deunicode::deunicode;
use fst::MapBuilder;
use postcard;
use serde_json;
use std::env;
use std::io::Cursor;
use std::path::Path;
use std::{fs::File, io::BufReader};

use build_utils::dict::JmdictData;

fn main() -> Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir);

    eprintln!("Parsing JSON...");
    // let file = File::open("build_data/jmdict-eng-3.6.1.json")?;
    // let rdr = BufReader::new(file);
    let rdr = BufReader::new(load_jmdict_json()?);
    let data: JmdictData = serde_json::from_reader(rdr)?;
    let entries = data.words;
    eprintln!("entries: {:?}", entries.len());

    eprintln!("Extracting index keys...");
    let mut kanji_map = Vec::new();
    let mut kana_map = Vec::new();
    let mut romaji_map = Vec::new();

    // Create a mapping from original ID to sequential ID
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
            // Keep only essential fields: part_of_speech, applies_to_kanji, applies_to_kana, gloss
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

    // Deduplicate by keeping only the first occurrence of each key
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

mod build_utils;
use build_utils::cache::{load_cached_data, CacheConfig};

use crate::build_utils::download::download_json_from_tgz;

// Download the JSON file from the GitHub release and cache it
fn load_jmdict_json() -> Result<Cursor<Vec<u8>>> {
    let config = CacheConfig::new("jmdict-common.json", "jmdict-version.txt", "3.6.1");

    load_cached_data(config, download_jmdict_json)
}

const ARTIFACT_URL: &str = "https://github.com/scriptin/jmdict-simplified/releases/download/3.6.1%2B20250714122633/jmdict-eng-3.6.1+20250714122633.json.tgz";
fn download_jmdict_json() -> Result<Vec<u8>> {
    let url = ARTIFACT_URL;
    eprintln!("Downloading dictionary from {url}...");
    let json_data = download_json_from_tgz(url)?;
    Ok(json_data)
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

fn write_blob(path: &Path, entries: &[build_utils::dict::Entry]) -> Result<()> {
    use std::io::{BufWriter, Write};

    let mut out = BufWriter::new(File::create(path)?);

    // Write entry count first
    let entry_count = entries.len() as u32;
    out.write_all(&entry_count.to_le_bytes())?;

    // Calculate offsets and data blob
    let mut offset_table = Vec::new();
    let mut data_blob = Vec::new();

    for entry in entries {
        let postcard_data = postcard::to_allocvec(entry)?;
        let offset = data_blob.len() as u32;
        let len = postcard_data.len() as u32;
        offset_table.push((offset, len));
        data_blob.extend(postcard_data);
    }

    // Write offset table
    for (offset, len) in &offset_table {
        out.write_all(&offset.to_le_bytes())?;
        out.write_all(&len.to_le_bytes())?;
    }

    // Write data blob
    out.write_all(&data_blob)?;
    Ok(())
}
