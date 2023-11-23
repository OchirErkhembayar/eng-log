use app::App;
use chrono::{TimeZone, Utc};
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event};
use crossterm::terminal::{disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::prelude::{Backend, CrosstermBackend};
use ratatui::Terminal;
use std::io;
use update::update;

mod app;
mod ui;
mod update;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    initialize_panic_handler();
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture,)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(Utc);

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
