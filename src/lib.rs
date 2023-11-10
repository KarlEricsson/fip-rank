use anyhow::{anyhow, Context, Result};
use csv::Reader;
use itertools::Itertools;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use tabled::Tabled;

pub mod cli;

#[derive(Clone, Debug, Deserialize, Tabled)]
pub struct Player {
    #[serde(alias = "Title", rename = "Name")]
    name: String,
    #[serde(rename = "Countries")]
    country: String,
    #[serde(rename = "Points")]
    points: u16,
    #[serde(rename = "Position")]
    position: u16,
}

pub fn poppler_txt_to_csv() -> Result<()> {
    // currently using popplers pdftotext with the following arguments:
    // pdftotext -layout -nopgbrk -enc UTF-8 Ranking-Male-11-09-2023.pdf rank_full.txt
    // ranking is available at padelfip.com
    let file = File::open("rank_full-UTF-8.txt")
        .context("Please provide a parsed textfile called rank_full-UTF-8.txt")?;
    let reader = io::read_to_string(file)?;
    let mut reader_lines = reader.lines();

    // Extract header
    let mut parsed_result: String = reader_lines
        .next()
        .ok_or_else(|| anyhow!("Iterator should not be empty. Empty or damaged file?"))?
        .split_whitespace()
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join(",");

    for line in reader_lines {
        let data = line
            .split_once("  ")
            .ok_or_else(|| anyhow!("Line should not be empty"))?;

        let mut name = data.0.to_string();

        // ugly hack to fix encoding error in original pdf
        let replacements = [
            ("Ã±", "ñ"),
            ("Ã¤", "ä"),
            ("Ã²", "ò"),
            ("Ã§", "ç"),
            ("Ã£", "ã"),
            ("Ã¶", "ö"),
            ("Ã©", "é"),
            ("Ã¡", "á"),
            ("Ã³", "ó"),
            ("Å„", "ń"),
            ("Ã¥", "å"),
            ("Ã¯", "ï"),
            ("Å ", "Š"),
            ("Å«", "ū"),
            ("Å¾", "ž"),
            ("Ã¬", "ì"),
            ("Å½", "Ž"),
            ("Ã", "Á"),
            ("Â", ""),
            ("Å†", "ņ"),
            ("Å¡", "š"),
            ("Ä±", "ı"),
        ];
        if !name.is_ascii() {
            for (from, to) in &replacements {
                name = name.replace(from, to);
            }
        }

        //player_data = all data after name
        let mut player_data = data.1.split_whitespace().collect::<Vec<_>>();

        //some records are missing country code, adding empty string.
        if player_data.len() == 2 {
            player_data.insert(0, "");
        };
        parsed_result.push_str(format!("\r\n{},{}", name, player_data.join(",")).as_str());
    }
    save_csv(&parsed_result)?;
    Ok(())
}
pub fn save_csv(data: &str) -> Result<()> {
    let file = Path::new("rank.csv");
    if file.exists() {
        println!(
            "{} already exists. Skipping creating.",
            file.to_string_lossy()
        );
    } else {
        println!("Writing to {}", file.to_string_lossy());
        let mut writer = File::create(file)?;
        writer.write_all(data.as_bytes())?;
    }
    Ok(())
}

pub fn read_csv(file: &Path) -> Result<Vec<Player>> {
    let mut rdr = Reader::from_path(file)?;
    let players: Result<Vec<Player>, _> = rdr.deserialize().collect();

    Ok(players?)
}

fn get_all_countries(players: &[Player]) -> Vec<&str> {
    let mut all_countries: Vec<&str> = players
        .iter()
        .map(|player| player.country.as_str())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    all_countries.sort_unstable();

    all_countries
}

fn get_top_countries(players: &[Player]) -> Vec<&str> {
    let mut country_player_amount: HashMap<&str, u16> = HashMap::new();
    for player in players {
        country_player_amount
            .entry(&player.country)
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }
    let result: Vec<_> = country_player_amount
        .iter()
        .sorted_by_key(|x| x.1)
        .map(|(country, _)| *country)
        .rev()
        .take(10)
        .collect();

    result
}
