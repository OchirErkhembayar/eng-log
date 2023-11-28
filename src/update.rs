use chrono::{NaiveDate, TimeZone};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use tui_textarea::{Input, Key};

use crate::app::{App, CurrentScreen, Day, Info, Popup};

pub fn update<T>(key_event: KeyEvent, app: &mut App<T>)
where
    T: TimeZone,
{
    if key_event.kind != KeyEventKind::Press {
        return;
    }
    if let Some(popup) = &app.popup {
        update_popup(app, key_event, popup.clone());
    } else {
        update_screen(app, key_event);
    }
}

fn update_popup<T: TimeZone>(app: &mut App<T>, key_event: KeyEvent, popup: Popup) {
    match popup {
        Popup::NewDay => match key_event.code {
            KeyCode::Esc => {
                app.popup = None;
                app.popup_buffer.clear();
            }
            KeyCode::Enter | KeyCode::Tab => {
                let popup_buffer = &mut app.popup_buffer;
                let currently_selected = &mut popup_buffer.currently_selected;
                match currently_selected {
                    0 | 1 => *currently_selected += 1,
                    2 => {
                        let year = popup_buffer.year.parse().unwrap_or(20000);
                        let month = popup_buffer.month.parse().unwrap_or(20000);
                        let day = popup_buffer.day.parse().unwrap_or(20000);
                        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                            let day = Day::new(date);
                            app.currently_selected = app.days.add(day);
                            app.current_screen = CurrentScreen::ViewingDay;
                            app.save();
                            app.load_text();
                        }
                        app.popup_buffer.clear();
                        app.popup = None;
                    }
                    _ => panic!("Ok just use an enum bro."),
                }
            }
            KeyCode::Char(char) => {
                app.popup_buffer.push(char);
            }
            KeyCode::Backspace => {
                app.popup_buffer.pop();
            }
            KeyCode::BackTab => {
                if app.popup_buffer.currently_selected > 0 {
                    app.popup_buffer.currently_selected -= 1;
                }
            }
            _ => {}
        },
        Popup::ConfDeleteDay => {
            match key_event.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    app.remove_day();
                }
                _ => {}
            };
            app.popup = None;
        }
        Popup::Info(_) => app.popup = None,
    }
}

fn update_screen<T: TimeZone>(app: &mut App<T>, key_event: KeyEvent) {
    match app.current_screen {
        CurrentScreen::Main => match key_event.code {
            KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
                app.load_text();
                app.current_screen = CurrentScreen::ViewingDay;
            }
            KeyCode::Up | KeyCode::Char('k') => app.decrement_selected(),
            KeyCode::Down | KeyCode::Char('j') => app.increment_selected(),
            KeyCode::Char('u') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                if app.currently_selected < 10 {
                    app.currently_selected = 0;
                } else {
                    app.currently_selected -= 10;
                }
            }
            KeyCode::Char('d') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                let max_index = app.days.days.len() - 1;
                if max_index - app.currently_selected < 10 {
                    app.currently_selected = max_index;
                } else {
                    app.currently_selected += 10;
                }
            }
            KeyCode::Char('d') => app.popup = Some(Popup::ConfDeleteDay),
            KeyCode::Char('i') => app.popup = Some(Popup::Info(Info::About)),
            KeyCode::Char('n') => app.popup = Some(Popup::NewDay),
            KeyCode::Char('q') => app.should_quit = true,
            _ => {}
        },
        CurrentScreen::ViewingDay => {
            match key_event.into() {
                Input { key: Key::Esc, .. } => {
                    app.finish_editing()
                }
                input => app.input_to_current_day(input),
            };
        }
    }
}
