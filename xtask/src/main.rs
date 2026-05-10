use clap::{Parser, Subcommand};
use deunicode::deunicode;
use fst::MapBuilder;
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};

mod cache;
mod dict;
mod download;

use cache::{load_cached_data, CacheConfig};
use download::download_json_from_tgz;

const ARTIFACT_URL: &str = "https://github.com/scriptin/jmdict-simplified/releases/download/3.6.1%2B20250714122633/jmdict-eng-3.6.1+20250714122633.json.tgz";
const JMDICT_VERSION: &str = "3.6.1";

#[derive(Parser)]
#[command(name = "xtask", about = "Development tasks for jmdict-fast")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate FST and entries.bin data files from JMdict
    Generate {
        /// Output directory for generated files
        #[arg(short, long, default_value = "dist")]
        output: PathBuf,
    },
    /// Print JMdict source version and binary format version
    Version,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { output } => {
            generate(&output)?;
        }
        Commands::Version => {
            println!("jmdict_version={JMDICT_VERSION}");
            println!("format_version={FORMAT_VERSION}");
        }
    }

    Ok(())
}

fn generate(output_dir: &Path) -> anyhow::Result<()> {
    // Ensure output directory exists
    fs::create_dir_all(output_dir)?;

    eprintln!("Downloading JMdict data...");
    let cursor = load_jmdict_json()?;

    eprintln!("Parsing JSON...");
    let rdr = BufReader::new(cursor);
    let data: dict::JmdictData = serde_json::from_reader(rdr)?;
    let entries: Vec<dict::Entry> = data.words.into_iter().map(|w| w.into_entry()).collect();
    eprintln!("Parsed {} entries", entries.len());

    eprintln!("Extracting index keys...");
    let mut kanji_map = Vec::new();
    let mut kana_map = Vec::new();
    let mut romaji_map = Vec::new();
    let mut id_mapping = Vec::new();

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

    write_fst(&output_dir.join("kanji.fst"), &kanji_map)?;
    write_fst(&output_dir.join("kana.fst"), &kana_map)?;
    write_fst(&output_dir.join("romaji.fst"), &romaji_map)?;
    write_fst(&output_dir.join("id.fst"), &id_mapping)?;

    eprintln!("Writing binary blob...");
    eprintln!("Writing {} entries to binary blob", entries.len());
    let generated_at = chrono::Utc::now().to_rfc3339();
    write_blob(
        &output_dir.join("entries.bin"),
        &entries,
        JMDICT_VERSION,
        &generated_at,
    )?;

    eprintln!("Done ✅ Output written to {}", output_dir.display());
    Ok(())
}

fn write_fst(path: &Path, entries: &[(String, u64)]) -> anyhow::Result<()> {
    let wtr = fs::File::create(path)?;
    let mut builder = MapBuilder::new(wtr)?;
    for (k, v) in entries {
        builder.insert(k, *v)?;
    }
    builder.finish()?;
    Ok(())
}

/// Magic bytes at the start of entries.bin (must match jmdict-fast lib)
const MAGIC: &[u8; 4] = b"JMDF";
/// Binary format version (must match jmdict-fast lib FORMAT_VERSION)
const FORMAT_VERSION: u32 = 3;

fn write_blob(
    path: &Path,
    entries: &[dict::Entry],
    jmdict_version: &str,
    generated_at: &str,
) -> anyhow::Result<()> {
    use std::io::{BufWriter, Write};

    let mut out = BufWriter::new(fs::File::create(path)?);

    // Write header: magic bytes + format version
    out.write_all(MAGIC)?;
    out.write_all(&FORMAT_VERSION.to_le_bytes())?;

    // Write jmdict_version (u16 len + bytes)
    out.write_all(&(jmdict_version.len() as u16).to_le_bytes())?;
    out.write_all(jmdict_version.as_bytes())?;

    // Write generated_at (u16 len + bytes)
    out.write_all(&(generated_at.len() as u16).to_le_bytes())?;
    out.write_all(generated_at.as_bytes())?;

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

fn load_jmdict_json() -> anyhow::Result<std::io::Cursor<Vec<u8>>> {
    let config = CacheConfig::new("jmdict-common.json", "jmdict-version.txt", JMDICT_VERSION);
    load_cached_data(config, download_jmdict_json)
}

fn download_jmdict_json() -> anyhow::Result<Vec<u8>> {
    eprintln!("Downloading dictionary from {ARTIFACT_URL}...");
    let json_data = download_json_from_tgz(ARTIFACT_URL)?;
    Ok(json_data)
}
