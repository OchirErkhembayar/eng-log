use chrono::TimeZone;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use tui_textarea::{Input, Key};

use crate::app::{App, CurrentScreen};

pub fn update<T>(key_event: KeyEvent, app: &mut App<T>)
where
    T: TimeZone,
{
    if key_event.kind != KeyEventKind::Press {
        return;
    }
    let screen = &mut app.current_screen;
    match screen {
        CurrentScreen::Main => match key_event.code {
            KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
                *screen = CurrentScreen::ViewingDay
            }
            KeyCode::Up | KeyCode::Char('k') => app.decrement_selected(),
            KeyCode::Down | KeyCode::Char('j') => app.increment_selected(),
            KeyCode::Char(char) => match char {
                'q' => app.should_quit = true,
                'i' => *screen = CurrentScreen::Editing,
                _ => {}
            },
            _ => {}
        },
        CurrentScreen::Editing => {
            match key_event.into() {
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
                input => app.input_to_current_day(input),
            };
        }
        CurrentScreen::ViewingDay => {
            match key_event.code {
                KeyCode::Esc | KeyCode::Left | KeyCode::Char('h') => *screen = CurrentScreen::Main,
                KeyCode::Down | KeyCode::Char('j') => {
                    app.days[app.currently_selected].increment_selected()
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    app.days[app.currently_selected].decrement_selected()
                }
                KeyCode::Char(char) => {
                    let day = &mut app.days[app.currently_selected];
                    match char {
                        'd' => day.delete_current_note(),
                        'e' => {
                            if !day.notes.is_empty() {
                                day.edit_currently_selected();
                                *screen = CurrentScreen::Editing;
                            }
                        }
                        'q' => app.should_quit = true,
                        'i' => *screen = CurrentScreen::Editing,
                        _ => {}
                    };
                }
                _ => {}
            };
        }
    }
}
