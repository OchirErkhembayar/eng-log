use anyhow::Result;
use chrono::Utc;
use englog::app::App;
use englog::tui::Tui;
use englog::update::update;
use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;
use std::io;

const DEFAULT_FILE_NAME: &str = "dev.postcard";
const SEED_FILE_NAME: &str = "seed.postcard";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg: englog::config::Config = confy::load("eng-log", None)?;
    let timezone = match cfg.timezone.as_str() {
        "UTC" => Utc,
        _ => Utc,
    };

    let doc_dir = dirs_next::document_dir().expect("Failed to find documents directory");

    let seed = std::env::args()
        .collect::<Vec<_>>()
        .get(1)
        .is_some_and(|s| s.eq("seed"));
    let file_name = if seed {
        SEED_FILE_NAME
    } else {
        DEFAULT_FILE_NAME
    };
    let file_path = format!("{}/{}", doc_dir.display(), file_name);

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
