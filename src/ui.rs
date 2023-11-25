use chrono::TimeZone;
use ratatui::{
    prelude::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Padding, Paragraph},
    Frame,
};

use crate::app::{App, CurrentScreen};

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

    if app.popup.is_some() {
        render_popup(f, app);
    }

    render_title(f, app, chunks[0]);

    render_body(f, app, chunks[1]);

    render_footer(f, app, chunks[2]);
}

fn render_popup<T: TimeZone>(f: &mut Frame, app: &mut App<T>) {
    let popup_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(centered_rect(60, 15, f.size()));

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

    let day_text = Paragraph::new(app.popup_buffer.day.clone()).block(day_block);
    f.render_widget(day_text, popup_chunks[0]);
    let month_text = Paragraph::new(app.popup_buffer.month.clone()).block(month_block);
    f.render_widget(month_text, popup_chunks[1]);
    let year_text = Paragraph::new(app.popup_buffer.year.clone()).block(year_block);
    f.render_widget(year_text, popup_chunks[2]);
}

fn render_title<T: TimeZone>(f: &mut Frame, app: &mut App<T>, rect: Rect) {
    let title_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rect);

    let title_block = Block::default()
        .borders(Borders::ALL)
        .padding(Padding::horizontal(2));

    let title = Paragraph::new(Text::styled(
        "Engineering Log",
        Style::default().fg(Color::White).bold(),
    ))
    .block(title_block.clone())
    .alignment(Alignment::Center);

    let content = &match app.current_screen {
        CurrentScreen::Main => "All days".to_string(),
        CurrentScreen::ViewingDay => app.days.days[app.currently_selected]
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
        CurrentScreen::Main => {
            let height = rect.height as usize;
            dbg!(height);
            for (index, day) in app.days.days.iter().enumerate() {
                if index < app.currently_selected - height - 1 {
                    continue;
                }
                if index == app.currently_selected {
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
            if list_items.is_empty() {
                let placeholder_text = Paragraph::new("Press n to add a day").block(
                    Block::default()
                        .borders(Borders::NONE)
                        .padding(Padding::horizontal(1)),
                );
                f.render_widget(placeholder_text, rect);
            } else {
                let list = List::new(list_items)
                    .block(Block::default().padding(Padding::horizontal(1)))
                    .style(Style::default().fg(Color::White));
                f.render_widget(list, rect);
            }
        }
    }
}

pub fn render_footer<T: TimeZone>(f: &mut Frame, app: &mut App<T>, rect: Rect) {
    let footer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(rect);

    let current_keys_hint = {
        let text = if app.popup.is_some() {
            "(esc) cancel | (tab | enter) next/save"
        } else {
            match app.current_screen {
                CurrentScreen::Main => "(q) quit | (enter) view day | (d) delete day | (n) new day",
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
