use std::path::Path;

use anyhow::Result;
use fip_rank::{cli::main_screen, poppler_txt_to_csv, read_csv};

fn main() -> Result<()> {
    let csvfile = Path::new("rank.csv");
    if !csvfile.exists() {
        poppler_txt_to_csv()?;
    }
    let players = read_csv(csvfile)?;

    loop {
        main_screen(&players)?;
    }
}
