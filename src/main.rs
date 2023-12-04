use anyhow::Result;
use app::{App, Day};
use arg::Cli;
use chrono::{Days, Utc};
use clap::Parser;
use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::process::exit;
use tui::Tui;
use update::update;

mod app;
mod arg;
mod config;
mod tui;
mod ui;
mod update;

const DEV_FILE_PATH: &str = "dev.postcard";
const SEEDED_FILE_PATH: &str = "seed.postcard";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let cfg: config::Config = confy::load("eng-log", None)?;
    let timezone = match cfg.timezone.as_str() {
        "UTC" => Utc,
        _ => Utc,
    };

    let doc_dir = dirs_next::document_dir().expect("Failed to find documents directory");

    let environment = cli.environment.unwrap_or("dev".to_string());

    let file_path = match environment.as_str() {
        "dev" => {
            format!("{}/{}", doc_dir.display(), DEV_FILE_PATH)
        }
        "seed" => {
            let file_path = format!("{}/{}", doc_dir.display(), SEEDED_FILE_PATH);
            let mut app = App::new(Utc, file_path.clone(), cfg.clone());
            for day in 0..500 {
                let date = chrono::Utc::now()
                    .checked_sub_days(Days::new(day))
                    .unwrap()
                    .date_naive();
                let mut day = Day::new(date);
                day.content.push("A note!".to_string());
                app.days.add(day);
                app.save();
            }
            file_path
        }
        _ => {
            println!("Wrong environment. Expected \"dev\" or \"seed\"");
            exit(1);
        }
    };

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    let mut app = App::new(timezone, file_path.to_string(), cfg);

    let mut tui = Tui::new(terminal);

    tui.enter()?;

    while !app.should_quit {
        tui.draw(&mut app)?;
        if let Some(event) = tui.next().await {
            update(event, &mut app, &tui);
        }
    }

    tui.exit()?;
    Ok(())
}
