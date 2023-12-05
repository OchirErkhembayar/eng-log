use anyhow::Result;
use chrono::Utc;
use englog::app::App;
use englog::tui::Tui;
use englog::update::update;
use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;
use std::io;

const DEFAULT_FILE_NAME: &str = "dev.postcard";
#[allow(dead_code)]
const SEED_FILE_NAME: &str = "seed.postcard";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg: englog::config::Config = confy::load("eng-log", None)?;
    let timezone = match cfg.timezone.as_str() {
        "UTC" => Utc,
        _ => Utc,
    };

    let mut tui = Tui::new(Terminal::new(CrosstermBackend::new(io::stdout()))?);

    tui.enter()?;

    let mut app = App::new(timezone, file_path(), cfg);
    while !app.should_quit {
        tui.draw(&mut app)?;
        if let Some(event) = tui.next().await {
            update(event, &mut app, &tui);
        }
    }

    tui.exit()?;
    Ok(())
}

#[cfg(debug_assertions)]
fn file_path() -> String {
    let doc_dir = dirs_next::document_dir().expect("Failed to find documents directory");
    let file_name = {
        let seed = std::env::args()
            .collect::<Vec<_>>()
            .get(1)
            .is_some_and(|s| s.eq("seed"));
        if seed {
            SEED_FILE_NAME
        } else {
            DEFAULT_FILE_NAME
        }
    };
    format!("{}/{}", doc_dir.display(), file_name)
}

#[cfg(not(debug_assertions))]
fn file_path() -> String {
    let doc_dir = dirs_next::document_dir().expect("Failed to find documents directory");
    format!("{}/{}", doc_dir.display(), DEFAULT_FILE_NAME)
}
