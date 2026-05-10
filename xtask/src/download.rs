use anyhow::Result;
use flate2::read::GzDecoder;
use reqwest::blocking::get;
use std::io::{Cursor, Read};
use tar::Archive;

pub fn download_json_from_tgz(url: &str) -> Result<Vec<u8>> {
    let response = get(url)?.bytes()?;

    // Extract the .json file from the .tgz archive in memory
    let tar = GzDecoder::new(Cursor::new(response));
    let mut archive = Archive::new(tar);

    let mut json_data = None;
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        if path.extension().map_or(false, |ext| ext == "json") {
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf)?;
            json_data = Some(buf);
            break;
        }
    }
    let json_data = json_data.ok_or_else(|| anyhow::anyhow!("No .json file found in archive"))?;

    Ok(json_data)
}
