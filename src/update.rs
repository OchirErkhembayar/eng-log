use chrono::NaiveDate;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc::UnboundedSender;
use tui_textarea::{Input, Key};

use crate::{
    app::{save_inner, App, CurrentScreen, Day, Info, Popup},
    tui::{Event, Loading, Tui},
};

pub fn update(event: Event, app: &mut App<'_>, tui: &Tui) {
    match event {
        Event::Key(key_event) => {
            if let Some(popup) = &app.popup {
                update_popup(app, key_event, popup.clone());
            } else {
                update_screen(app, key_event, &tui.event_tx);
            }
        }
        Event::Loading(Loading::Saving(state)) => app.saving = state,
        Event::Loading(Loading::Loading(state)) => app.loading = state,
        Event::Tick => {}
        Event::LoadDays(switch_screen) => {
            tui.event_tx
                .send(Event::Loading(Loading::Loading(true)))
                .expect("Failed to send loading message");
            app.load_days(switch_screen);
            tui.event_tx
                .send(Event::Loading(Loading::Loading(false)))
                .expect("Failed to send loading message");
        }
    }
}

fn update_popup(app: &mut App, key_event: KeyEvent, popup: Popup) {
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
        Popup::Config(editing) => {
            if editing {
                match key_event.code {
                    KeyCode::Esc => app.popup = None,
                    KeyCode::Backspace => app.config_buffer.pop(),
                    KeyCode::Char(c) => app.config_buffer.push(c),
                    KeyCode::Enter => {
                        let new_limit = &app.config_buffer.word_limit;
                        if new_limit.trim().is_empty() {
                            app.config.chars_per_line = None;
                        } else {
                            match new_limit.parse::<usize>() {
                                Ok(limit) => app.config.chars_per_line = Some(limit),
                                Err(_) => app.config_buffer.clear(),
                            }
                        }
                        app.config_buffer.word_limit =
                            if let Some(limit) = app.config.chars_per_line {
                                limit.to_string()
                            } else {
                                String::new()
                            };
                        confy::store("englog", None, &app.config).expect("Failed to save config");
                        app.popup = None;
                    }
                    _ => {}
                }
            } else {
                match key_event.code {
                    KeyCode::Char('e') => {
                        app.popup = Some(Popup::Config(true));
                    }
                    _ => app.popup = None,
                }
            }
        }
        Popup::Info(_) => app.popup = None,
    }
}

fn update_screen(app: &mut App, key_event: KeyEvent, rx: &UnboundedSender<Event>) {
    match app.current_screen {
        CurrentScreen::Main(true) => match key_event.into() {
            Input {
                key: Key::Enter, ..
            } => {
                app.current_screen = CurrentScreen::Main(false);
                let count = app.filtered_days().count();
                if app.currently_selected >= count && count > 0 {
                    app.currently_selected = count - 1;
                }
            }
            Input { key: Key::Esc, .. } => {
                app.filter = None;
                app.current_screen = CurrentScreen::Main(false);
            }
            input => app.input_to_filter_buffer(input),
        },
        CurrentScreen::Main(false) => match key_event.code {
            KeyCode::Char('r') => app.switch_to_current_day(),
            KeyCode::Esc => app.filter = None,
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
                let max_index = app.filtered_days().count() - 1;
                if max_index - app.currently_selected < 10 {
                    app.currently_selected = max_index;
                } else {
                    app.currently_selected += 10;
                }
            }
            KeyCode::Char('d') => app.popup = Some(Popup::ConfDeleteDay),
            KeyCode::Char('i') => app.popup = Some(Popup::Info(Info::About)),
            KeyCode::Char('c') => app.popup = Some(Popup::Config(false)),
            KeyCode::Char('n') => app.popup = Some(Popup::NewDay),
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Char('b') => app.currently_selected = app.days.days.len() - 1,
            KeyCode::Char(':') => {
                app.current_screen = CurrentScreen::Main(true);
                app.init_filter_text();
            }
            _ => {}
        },
        CurrentScreen::ViewingDay => {
            match key_event.into() {
                Input { key: Key::Esc, .. } => {
                    app.update_day_from_buffer();
                    app.current_screen = CurrentScreen::Main(false);
                    //TODO remove this useless testing stuff and use Tokio
                    let sender = rx.clone();
                    sender.send(Event::Loading(Loading::Saving(true))).unwrap();
                    save_inner(&app.days, &app.file_path);
                    sender.send(Event::Loading(Loading::Saving(false))).unwrap();
                }
                input => app.input_to_current_day(input),
            };
        }
    }
}
