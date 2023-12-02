use chrono::TimeZone;
use ratatui::{
    prelude::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Clear, List, ListItem, Padding, Paragraph, Scrollbar, ScrollbarState, Wrap,
    },
    Frame,
};

use crate::app::{App, CurrentScreen, Popup};

pub fn ui<T: TimeZone>(f: &mut Frame, app: &mut App<T>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            // Title
            Constraint::Length(3),
            // Body
            Constraint::Min(1),
            // Footer
            Constraint::Length(3),
        ])
        .split(f.size());

    render_title(f, app, chunks[0]);

    render_body(f, app, chunks[1]);

    render_footer(f, app, chunks[2]);

    if let Some(popup) = &app.popup {
        render_popup(f, app, popup);
    }
}

fn render_popup<T: TimeZone>(f: &mut Frame, app: &App<T>, popup: &Popup) {
    match popup {
        Popup::NewDay => {
            let area = centered_rect(60, 15, f.size());
            let popup_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([
                    Constraint::Ratio(1, 3),
                    Constraint::Ratio(1, 3),
                    Constraint::Ratio(1, 3),
                ])
                .split(area);

            let mut day_block = Block::default().title("Day (1)").borders(Borders::ALL);
            let mut month_block = Block::default().title("Month (1)").borders(Borders::ALL);
            let mut year_block = Block::default().title("Year (1970)").borders(Borders::ALL);

            let active_style = Style::default().bg(Color::Yellow).fg(Color::Black);
            match app.popup_buffer.currently_selected {
                0 => day_block = day_block.style(active_style),
                1 => month_block = month_block.style(active_style),
                2 => year_block = year_block.style(active_style),
                _ => panic!("Time to use an enum buddy"),
            };

            f.render_widget(Clear, area);
            let day_text = Paragraph::new(app.popup_buffer.day.clone()).block(day_block);
            f.render_widget(day_text, popup_chunks[0]);
            let month_text = Paragraph::new(app.popup_buffer.month.clone()).block(month_block);
            f.render_widget(month_text, popup_chunks[1]);
            let year_text = Paragraph::new(app.popup_buffer.year.clone()).block(year_block);
            f.render_widget(year_text, popup_chunks[2]);
        }
        Popup::ConfDeleteDay => {
            let delete_block = Block::default()
                .title("Are you sure?")
                .style(Style::default().bg(Color::Red).fg(Color::White))
                .borders(Borders::ALL);
            let delete_text = Paragraph::new("y for yes\nAny other key to cancel".to_string())
                .wrap(Wrap::default())
                .block(delete_block);
            let area = centered_rect(60, 15, f.size());
            f.render_widget(Clear, area);
            f.render_widget(delete_text, area);
        }
        Popup::Info(_) => {
            let message = "Thanks for trying out the app\n
There are a few known issues which i'm working on:
1. Resizing may cause awkward rendering issues so please just quit and restart the app if this occurs
2. Control scrolling may cause the selected day to go off screen.\n
Any bugs found please just send requests and i'll see what I can do";
            let message_block = Block::default()
                .title("Info")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Green).fg(Color::White));
            let message_text = Paragraph::new(message.to_string())
                .wrap(Wrap::default())
                .block(message_block);
            let area = centered_rect(75, 25, f.size());
            f.render_widget(Clear, area);
            f.render_widget(message_text, area);
        }
        Popup::Config(_) => {
            let timezone = &app.config.timezone;
            let message_block = Block::default()
                .title("Config")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Blue).fg(Color::White));
            let message_text = Paragraph::new(format!("Timezone: {timezone}"))
                .wrap(Wrap::default())
                .block(message_block);
            let area = centered_rect(75, 25, f.size());
            f.render_widget(Clear, area);
            f.render_widget(message_text, area);
        }
    }
}

fn render_title<T: TimeZone>(f: &mut Frame, app: &App<T>, rect: Rect) {
    let title_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rect);

    let title_block = Block::default()
        .borders(Borders::ALL)
        .padding(Padding::horizontal(2));

    let title_message = if app.saving {
        "Saving..."
    } else {
        "Engineering Log"
    };
    let title = Paragraph::new(Text::styled(
        title_message,
        Style::default().fg(Color::White).bold(),
    ))
    .block(title_block.clone())
    .alignment(Alignment::Center);

    let content = &match app.current_screen {
        CurrentScreen::Main(_) => "All days".to_string(),
        CurrentScreen::ViewingDay => app.filtered_days().collect::<Vec<_>>()
            [app.currently_selected]
            .date
            .format("%-d %B, %C%y")
            .to_string(),
    };
    let subtitle = Paragraph::new(Text::styled(
        content,
        Style::default().fg(Color::White).bold(),
    ))
    .block(title_block)
    .alignment(Alignment::Center);

    f.render_widget(subtitle, title_chunks[0]);
    f.render_widget(title, title_chunks[1]);
}

pub fn render_body<T: TimeZone>(f: &mut Frame, app: &mut App<T>, rect: Rect) {
    let mut list_items = Vec::<ListItem>::new();
    match app.current_screen {
        CurrentScreen::ViewingDay => {
            f.render_widget(app.text_buffer.widget(), rect);
        }
        CurrentScreen::Main(_) => {
            let current = app.currently_selected as isize;
            if app.max_index == -1 {
                app.max_index = rect.height as isize;
            } else if current >= app.max_index - 2 {
                app.max_index += 1;
                app.min_index += 1;
            } else if current < app.min_index {
                app.min_index -= 1;
                app.max_index -= 1;
            }
            for (index, day) in app.filtered_days().enumerate() {
                let index = index as isize;
                if index < app.min_index || index > app.max_index {
                    continue;
                }

                if index == current {
                    let list_item = ListItem::new(Line::from(Span::styled(
                        day.date_pretty(),
                        Style::default().fg(Color::White).bg(Color::Blue),
                    )));
                    list_items.push(list_item.bold());
                } else {
                    let list_item = ListItem::new(Line::from(Span::styled(
                        day.date_pretty(),
                        Style::default().fg(Color::White),
                    )));
                    list_items.push(list_item);
                }
            }
            let scrollbar = Scrollbar::new(ratatui::widgets::ScrollbarOrientation::VerticalRight);
            let mut scrollbar_state =
                ScrollbarState::new(app.days.days.len()).position(app.currently_selected);
            let layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Min(10),
                    Constraint::Percentage(70),
                    Constraint::Length(2),
                ])
                .split(rect);
            f.render_stateful_widget(scrollbar, layout[2], &mut scrollbar_state);
            if !app.days.days.is_empty() {
                app.load_text();
                app.text_buffer.set_cursor_style(Style::default());
                f.render_widget(app.text_buffer.widget(), layout[1]);
            }
            if list_items.is_empty() {
                let placeholder_text = Paragraph::new("Press n to add a day").block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Days")
                        .padding(Padding::horizontal(1)),
                );
                f.render_widget(placeholder_text, layout[0]);
            } else {
                let list = List::new(list_items)
                    .block(
                        Block::default()
                            .padding(Padding::horizontal(1))
                            .title("Days")
                            .borders(Borders::ALL),
                    )
                    .style(Style::default().fg(Color::White));
                f.render_widget(list, layout[0]);
            }
        }
    }
}

pub fn render_footer<T: TimeZone>(f: &mut Frame, app: &App<T>, rect: Rect) {
    let footer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(rect);

    let current_keys_hint = {
        let text = if let Some(popup) = &app.popup {
            match popup {
                Popup::NewDay => "(esc) cancel | (tab | enter) next/save",
                Popup::ConfDeleteDay => "(esc) cancel | (enter) save | \"y\" save",
                Popup::Info(_) => "(esc) close",
                Popup::Config(_) => "(esc) close",
            }
        } else {
            match app.current_screen {
                CurrentScreen::Main(true) => "(esc) cancel | (enter) done",
                CurrentScreen::Main(false) => {
                    //TODO these are getting pretty long.
                    // Add some code for "extended menu" or maybe split it into two with a toggle
                    if app.filter.is_some() {
                        "(q) quit | (enter) edit day | (esc) clear filter | (d) delete day | (n) new day | (i) info | (:) filter | vim motions if you're cool"
                    } else {
                        "(q) quit | (enter) edit day | (d) delete day | (n) new day | (i) info | (:) filter | vim motions if you're cool"
                    }
                }
                CurrentScreen::ViewingDay => "(esc) back",
            }
        };
        Span::styled(
            text,
            Style::default().bold().fg(Color::White).bg(Color::Blue),
        )
    };
    let key_hints_footer = Paragraph::new(Line::from(current_keys_hint))
        .block(Block::default().padding(Padding::horizontal(1)));

    if let CurrentScreen::Main(true) = app.current_screen {
        f.render_widget(app.filter_buffer.widget(), footer_chunks[0]);
    }
    f.render_widget(key_hints_footer, footer_chunks[1]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
