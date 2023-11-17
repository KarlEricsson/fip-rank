use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    fs::File,
    io::{self, Write},
    path::Path,
};

use anyhow::{anyhow, Context, Result};
use csv::Reader;
use itertools::Itertools;
use serde::Deserialize;
use tabled::Tabled;

pub mod cli;
#[derive(Default, Clone, Debug)]
pub struct DisplayOption<T>(Option<T>);

impl Display for DisplayOption<i16> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.0 {
            Some(val) => match val {
                n if n > &0 => write!(f, "+{val}"),
                n if n < &0 => write!(f, "{}", val),
                _ => write!(f, "{}", val),
            },
            None => write!(f, "-"),
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct RankHistoryDisplay(Vec<RankHistory>);

impl Display for RankHistoryDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for history in &self.0 {
            write!(
                f,
                "Previous: {}, Points: {}, Position: {}",
                history.date, history.points, history.position
            )?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct RankHistory {
    date: String,
    points: i16,
    position: i16,
}

#[derive(Clone, Debug, Deserialize, Tabled)]
pub struct Player {
    #[serde(alias = "Title", rename = "Name")]
    name: String,
    #[serde(rename = "Countries")]
    country: String,
    #[serde(rename = "Points")]
    points: i16,
    #[serde(rename = "Position")]
    position: i16,
    #[serde(skip)]
    // TODO: Not use wrapped?
    #[tabled(skip)]
    rank_history: RankHistoryDisplay,
    #[serde(skip)]
    points_change: DisplayOption<i16>,
    #[serde(skip)]
    position_change: DisplayOption<i16>,
}

impl Player {
    fn add_rank_history_data(&mut self, rank_history_data: RankHistory) {
        self.rank_history.0.push(rank_history_data);
        self.calculate_rank_change();
    }
    fn calculate_rank_change(&mut self) {
        if let Some(old_data) = self.rank_history.0.last() {
            self.points_change.0 = Some(self.points - old_data.points);
            self.position_change.0 = Some(old_data.position - self.position);
        }
    }
}

pub struct PlayerData {}

pub fn poppler_txt_to_csv(text_file: &Path, csv_filename: &Path) -> Result<()> {
    // currently using popplers pdftotext with the following arguments:
    // pdftotext -layout -nopgbrk -enc UTF-8 Ranking-Male-11-09-2023.pdf rank_full.txt
    // ranking is available at padelfip.com
    let file = File::open(text_file)
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
            .split_once("   ")
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
    save_csv(&parsed_result, csv_filename)?;
    Ok(())
}
pub fn save_csv(data: &str, csv_filename: &Path) -> Result<()> {
    let file = Path::new(csv_filename);
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

pub fn add_rank_history(players: &mut [Player], txt_file_path: &Path) -> Result<()> {
    let csv_filename = txt_file_path.with_extension("csv");
    poppler_txt_to_csv(txt_file_path, &csv_filename)?;
    let history_players = read_csv(&csv_filename)?;
    for player in players.iter_mut() {
        if let Some(player_history) = history_players.iter().find(|p| p.name == player.name) {
            let date = String::from(
                txt_file_path
                    .file_stem()
                    .expect("File error")
                    .to_str()
                    .expect("Unicode filename error")
                    .strip_prefix("rank_male_")
                    .expect("File error"),
            );
            let rank_history_data = RankHistory {
                date,
                points: player_history.points,
                position: player_history.position,
            };
            player.add_rank_history_data(rank_history_data);
        }
    }
    Ok(())
}

pub fn get_all_countries(players: &[Player]) -> Vec<&str> {
    let mut all_countries: Vec<&str> = players
        .iter()
        .map(|player| player.country.as_str())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    all_countries.sort_unstable();

    all_countries
}

pub fn get_top_countries(players: &[Player]) -> Vec<&str> {
    let mut country_player_amount: HashMap<&str, i16> = HashMap::new();
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
