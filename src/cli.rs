use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Select};

use crate::{get_all_countries, get_top_countries, print_table, Player};

pub fn main_screen(players: &[Player]) -> Result<()> {
    //let options =

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose your action")
        .default(0)
        .item("Print all players")
        .item("Print top 50 ranking")
        .item("Print country ranking")
        // .item("Print error names")
        .interact_opt()?;
    if let Some(index) = selection {
        match index {
            0 => print_table(players)?,
            1 => print_table(&players[0..49])?,
            2 => select_country_rank_screen(players)?,
            //3 => print_error_names(players),
            _ => (),
        }
    } else {
        std::process::exit(0)
    };

    Ok(())
}
pub fn select_country_rank_screen(players: &[Player]) -> Result<()> {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose your action")
        .default(0)
        .item("Top 10 countries")
        .item("All countries")
        .interact_opt()?;

    if let Some(index) = selection {
        match index {
            0 => country_rank_screen(players, get_top_countries(players))?,
            1 => country_rank_screen(players, get_all_countries(players))?,
            _ => (),
        }
    }
    Ok(())
}

pub fn country_rank_screen(players: &[Player], countries: Vec<&str>) -> Result<()> {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose your action")
        .default(0)
        .max_length(10)
        .items(&countries)
        .interact_opt()?;

    if let Some(i) = selection {
        let players_from_country: Vec<Player> = players
            .iter()
            .filter(|&player| player.country == countries[i])
            .cloned()
            .collect();
        print_table(&players_from_country)?;
    }
    Ok(())
}
