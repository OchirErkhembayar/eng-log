use std::io::{self, Stdout};

use anyhow::Result;
use chrono::TimeZone;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;

use crate::{app::App, event::EventHandler};

type CrosstermTerminal = ratatui::Terminal<CrosstermBackend<Stdout>>;

pub struct Tui {
    terminal: CrosstermTerminal,
    pub event_handler: EventHandler,
}

impl Tui {
    pub fn new(terminal: CrosstermTerminal, events: EventHandler) -> Self {
        Self {
            terminal,
            event_handler: events,
        }
    }

    pub fn enter(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stderr(), EnterAlternateScreen, EnableMouseCapture)?;

        // Define a custom panic hook to reset the terminal properties.
        // This way, you won't have your terminal messed up if an unexpected error happens.
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen)
                .unwrap();
            crossterm::terminal::disable_raw_mode().unwrap();
            original_hook(panic_info);
        }));

        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }

    fn reset() -> Result<()> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture)?;
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        Self::reset()?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}

impl Tui {
    pub fn draw<T: TimeZone>(&mut self, app: &mut App<T>) -> Result<()> {
        self.terminal.draw(|f| crate::ui::ui(f, app))?;
        Ok(())
    }
}
