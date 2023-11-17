use std::path::Path;

use anyhow::Result;
use fip_rank::{cli::main_screen, poppler_txt_to_csv, read_csv};

fn main() -> Result<()> {
    let csvfile = Path::new("rank.csv");
    if !csvfile.exists() {
        poppler_txt_to_csv(Path::new("rank_full-UTF-8.txt"), Path::new("rank.csv"))?;
    }
    let mut players = read_csv(csvfile)?;
    loop {
        main_screen(&mut players)?;
    }
}
