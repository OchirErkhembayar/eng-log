extern crate englog;

use anyhow::Result;
use chrono::Days;
use englog::app::{App, Day};
use englog::config::Config;

const SEEDED_FILE_PATH: &str = "seed.postcard";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg: Config = confy::load("englog", None)?;
    let doc_dir = dirs_next::document_dir().expect("Failed to find documents directory");
    let dir_path = format!("{}/{}", doc_dir.display(), "englog");
    std::fs::create_dir_all(&dir_path)?;
    let file_path = format!("{}/{}", dir_path, SEEDED_FILE_PATH);
    let mut app = App::new(file_path.clone(), cfg.clone());
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
