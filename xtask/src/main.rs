use clap::{Parser, Subcommand};
use std::io::BufReader;

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
    Generate,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate => {
            generate()?;
        }
    }

    Ok(())
}

fn generate() -> anyhow::Result<()> {
    eprintln!("Downloading JMdict data...");
    let cursor = load_jmdict_json()?;

    eprintln!("Parsing JSON...");
    let rdr = BufReader::new(cursor);
    let data: dict::JmdictData = serde_json::from_reader(rdr)?;
    eprintln!("Downloaded {} entries", data.words.len());

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
