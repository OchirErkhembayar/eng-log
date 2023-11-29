use anyhow::Result;
use app::{App, Day};
use chrono::{Days, TimeZone, Utc};
use event::EventHandler;
use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;
use std::process::exit;
use std::{env, io};
use tui::Tui;
use update::update;

mod app;
mod event;
mod tui;
mod ui;
mod update;

const DEV_FILE_PATH: &str = "./dev.postcard";
const SEEDED_FILE_PATH: &str = "./seed.postcard";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().collect();

    let default_environment = "dev".to_string();
    let environment = args.get(1).unwrap_or_else(|| {
        println!("Defaulting to dev environment");
        &default_environment
    });

    let file_path = match environment.as_str() {
        "dev" => DEV_FILE_PATH,
        "seed" => {
            let mut app = App::new(Utc, SEEDED_FILE_PATH.to_string());
            for day in 0..1000 {
                let date = chrono::Utc::now()
                    .checked_sub_days(Days::new(day))
                    .unwrap()
                    .date_naive();
                let mut day = Day::new(date);
                day.content.push("A note!".to_string());
                app.days.add(day);
                app.save();
            }
            SEEDED_FILE_PATH
        }
        _ => {
            println!("Wrong environment. Expected \"dev\" or \"seed\"");
            exit(1);
        }
    };

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    let mut app = App::new(Utc, file_path.to_string());

    let eventhandler = EventHandler::new(250);
    let mut tui = Tui::new(terminal, eventhandler);

    tui.enter()?;
    run(&mut tui, &mut app)?;

    tui.exit()?;
    Ok(())
}

fn run<T>(tui: &mut Tui, app: &mut App<T>) -> Result<()>
where
    T: TimeZone,
{
    while !app.should_quit {
        tui.draw(app)?;
        let event = tui.events.next()?;
        update(event, app, tui);
    }

    Ok(())
}
