use anyhow::Result;
use englog::app::App;
use englog::tui::Tui;
use englog::update::update;
use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;
use std::io;

const DEFAULT_FILE_NAME: &str = "englog.postcard";
#[allow(dead_code)]
const SEED_FILE_NAME: &str = "seed.postcard";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg: englog::config::Config = confy::load("englog", None).unwrap_or_else(|_| {
        let path = confy::get_configuration_file_path("englog", None)
            .expect("Failed to get config file path");
        std::fs::remove_file(path).expect("Failed to remove corrupted config file");
        confy::load("englog", None).expect("Failed to load new config file")
    });

    let mut tui = Tui::new(Terminal::new(CrosstermBackend::new(io::stdout()))?);

    tui.enter()?;

    let mut app = App::new(file_path()?, cfg);
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
fn file_path() -> Result<String> {
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
    let dir = format!("{}/{}", doc_dir.display(), "englog");
    std::fs::create_dir_all(&dir)?;
    Ok(format!("{}/{}", dir, file_name))
}

#[cfg(not(debug_assertions))]
fn file_path() -> Result<String> {
    let doc_dir = dirs_next::document_dir().expect("Failed to find documents directory");
    let dir = format!("{}/{}", doc_dir.display(), "englog");
    std::fs::create_dir_all(&dir)?;
    Ok(format!("{}/{}", dir, DEFAULT_FILE_NAME))
}
