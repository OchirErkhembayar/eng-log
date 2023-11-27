use app::{App, Day};
use chrono::{Days, TimeZone, Utc};
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event};
use crossterm::terminal::{disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::prelude::{Backend, CrosstermBackend};
use ratatui::Terminal;
use std::process::exit;
use std::{env, io};
use update::update;

mod app;
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

    initialize_panic_handler();
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture,)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(Utc, file_path.to_string());

    run(&mut terminal, &mut app)?;

    disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn run<B, T>(terminal: &mut Terminal<B>, app: &mut App<T>) -> io::Result<()>
where
    B: Backend,
    T: TimeZone,
{
    while !app.should_quit {
        terminal.draw(|f| ui::ui(f, app))?;
        if let Event::Key(key) = event::read()? {
            update(key, app);
        }
    }

    Ok(())
}

fn initialize_panic_handler() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen).unwrap();
        crossterm::terminal::disable_raw_mode().unwrap();
        original_hook(panic_info);
    }));
}
