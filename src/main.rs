use app::{App, CurrentScreen, Day};
use chrono::{TimeZone, Utc};
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::prelude::{Backend, CrosstermBackend};
use ratatui::Terminal;
use std::io;
use tui_textarea::{CursorMove, Input, Key};

mod app;
mod ui;

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

        match app.current_screen {
            CurrentScreen::Main => {
                if let Event::Key(key) = event::read()? {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }

                    match key.code {
                        event::KeyCode::Enter => app.current_screen = CurrentScreen::ViewingDay,
                        event::KeyCode::Char(char) => match char {
                            'q' => app.should_quit = true,
                            'i' => app.current_screen = CurrentScreen::Editing,
                            'j' => {
                                if app.currently_selected < app.days.len() - 1 {
                                    app.currently_selected += 1;
                                }
                            }
                            'l' => app.current_screen = CurrentScreen::ViewingDay,
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

                    match key.into() {
                        Input {
                            key: Key::Enter, ..
                        } => {
                            app.save_note();
                            app.current_screen = CurrentScreen::ViewingDay;
                        }
                        Input { key: Key::Esc, .. } => {
                            app.current_screen = CurrentScreen::ViewingDay;
                            let day = &mut app.days[app.currently_selected];
                            if day.updating {
                                day.updating = false;
                            }
                        }
                        input => {
                            app.days[app.currently_selected].note_buffer.input(input);
                        }
                    };
                }
            }
            CurrentScreen::ViewingDay => {
                if let Event::Key(key) = event::read()? {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    let day = &mut app.days[app.currently_selected];

                    match key.code {
                        event::KeyCode::Esc => app.current_screen = CurrentScreen::Main,
                        event::KeyCode::Char(char) => {
                            match char {
                                'd' => {
                                    if !day.notes.is_empty() {
                                        day.notes.remove(day.currently_selected);
                                        if day.currently_selected > 0 {
                                            day.currently_selected -= 1;
                                        }
                                    }
                                }
                                'e' => {
                                    let day = &mut app.days[app.currently_selected];
                                    if !day.notes.is_empty() {
                                        day.note_buffer = Day::new_text_area(Some(
                                            day.notes[day.currently_selected].to_owned(),
                                        ));
                                        day.note_buffer.move_cursor(CursorMove::End);
                                        day.updating = true;
                                        app.current_screen = CurrentScreen::Editing;
                                    }
                                }
                                'q' => app.should_quit = true,
                                'i' => app.current_screen = CurrentScreen::Editing,
                                'j' => {
                                    if day.currently_selected < day.notes.len() - 1 {
                                        day.currently_selected += 1;
                                    }
                                }
                                'h' => app.current_screen = CurrentScreen::Main,
                                'k' => {
                                    if day.currently_selected > 0 {
                                        day.currently_selected -= 1;
                                    }
                                }
                                _ => {}
                            };
                        }
                        _ => {}
                    };
                }
            }
        }
    }

    Ok(())
}

pub fn initialize_panic_handler() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen).unwrap();
        crossterm::terminal::disable_raw_mode().unwrap();
        original_hook(panic_info);
    }));
}
