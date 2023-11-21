use app::{App, CurrentScreen};
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind, KeyModifiers,
};
use crossterm::terminal::{disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::prelude::{Backend, CrosstermBackend};
use ratatui::Terminal;
use std::io;

mod app;
mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture,)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

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

fn run<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    while !app.should_quit {
        terminal.draw(|f| ui::ui(f, app))?;

        match app.current_screen {
            CurrentScreen::Main => {
                if let Event::Key(key) = event::read()? {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }

                    match key.code {
                        event::KeyCode::Char(char) => match char {
                            'q' => app.should_quit = true,
                            'i' => app.current_screen = CurrentScreen::Editing,
                            'j' => {
                                if app.currently_selected < app.days.len() - 1 {
                                    app.currently_selected += 1;
                                }
                            }
                            'k' => {
                                if app.currently_selected > 0 {
                                    app.currently_selected -= 1;
                                }
                            }
                            _ => {}
                        },
                        _ => {}
                    };
                }
            }
            CurrentScreen::Editing => {
                if let Event::Key(key) = event::read()? {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }

                    match key.code {
                        event::KeyCode::Esc => app.current_screen = CurrentScreen::Main,
                        event::KeyCode::Char('w')
                            if key.modifiers.contains(KeyModifiers::CONTROL) =>
                        {
                            while let Some(char) = app.note_buffer.pop() {
                                if char == ' ' {
                                    break;
                                }
                            }
                        }
                        event::KeyCode::Char(char) => app.note_buffer.push(char),
                        event::KeyCode::Backspace => {
                            app.note_buffer.pop();
                        }
                        event::KeyCode::Enter => app.save_note(),
                        _ => {}
                    };
                }
            }
        }
    }

    Ok(())
}
