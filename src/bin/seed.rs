extern crate englog;

use anyhow::Result;
use chrono::{Days, Utc};
use englog::app::{App, Day};
use englog::config::Config;

const SEEDED_FILE_PATH: &str = "seed.postcard";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg: Config = confy::load("eng-log", None)?;
    let doc_dir = dirs_next::document_dir().expect("Failed to find documents directory");
    let file_path = format!("{}/{}", doc_dir.display(), SEEDED_FILE_PATH);
    let mut app = App::new(Utc, file_path.clone(), cfg.clone());
    for day in 1000..2000 {
        let date = chrono::Utc::now()
            .checked_sub_days(Days::new(day))
            .unwrap()
            .date_naive();
        let mut day = Day::new(date);
        day.content.push("A note!".to_string());
        app.days.add(day);
    }
    app.save();

    Ok(())
}
