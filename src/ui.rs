use ratatui::{
    prelude::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Padding, Paragraph},
    Frame,
};

use crate::app::{App, CurrentScreen};

pub fn ui(f: &mut Frame, app: &mut App) {
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

    // Title
    {
        let title_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[0]);

        let title_block = Block::default()
            .borders(Borders::ALL)
            .padding(Padding::horizontal(2));

        let title = Paragraph::new(Text::styled(
            "Engineering Log",
            Style::default().fg(Color::Yellow).bold(),
        ))
        .block(title_block.clone())
        .alignment(Alignment::Center);

        let date = Paragraph::new(Text::styled(
            format!("Date: {}", app.date.format("%d/%m/%Y %H:%M")),
            Style::default().fg(Color::Yellow).bold(),
        ))
        .block(title_block)
        .alignment(Alignment::Center);

        f.render_widget(title, title_chunks[0]);
        f.render_widget(date, title_chunks[1]);
    }

    // Body
    {
        let mut list_items = Vec::<ListItem>::new();
        let mut render_day = || {
            let day = &app.days[app.currently_selected];
            for (index, note) in day.notes.iter().enumerate() {
                let list_item = ListItem::new(Line::from(Span::styled(
                    format!("- {note}"),
                    Style::default().fg(Color::Yellow),
                )));
                if index == day.currently_selected {
                    list_items.push(list_item.bold());
                } else {
                    list_items.push(list_item);
                }
            }
        };
        match app.current_screen {
            CurrentScreen::ViewingDay => render_day(),
            CurrentScreen::Editing => {
                render_day();
                let popup_block = Block::default()
                    .title("Enter a note")
                    .borders(Borders::NONE)
                    .style(Style::default().bg(Color::DarkGray));
                let area = centered_rect(60, 25, f.size());
                f.render_widget(popup_block, area);

                let input_block = Block::default().title("Input").borders(Borders::ALL);
                let input_text = Paragraph::new(app.note_buffer.clone()).block(input_block);

                f.render_widget(input_text, area);
            }
            CurrentScreen::Main => {
                for (index, day) in app.days.iter().enumerate() {
                    let list_item = ListItem::new(Line::from(Span::styled(
                        format!("- {}", day.date.format("%d-%m-%Y")),
                        Style::default().fg(Color::Yellow),
                    )));
                    if index == app.currently_selected {
                        list_items.push(list_item.bold());
                    } else {
                        list_items.push(list_item);
                    }
                }
            }
        }
        let list = List::new(list_items)
            .block(Block::default().title(format!(
                "Day: {}",
                app.days[app.currently_selected].date.format("%d-%m-%Y"),
            )))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol(">>");
        f.render_widget(list, chunks[1]);
    }

    // Footer
    {
        let footer_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(chunks[2]);

        let current_keys_hint = {
            let text = match app.current_screen {
                CurrentScreen::Main => "(q) quit | (enter) view day",
                CurrentScreen::Editing => "(esc) back | (enter) save",
                CurrentScreen::ViewingDay => {
                    "(q) quit | (esc) back | (i) add note | (d) delete note"
                }
            };
            Span::styled(text, Style::default().fg(Color::Yellow))
        };
        let key_hints_footer = Paragraph::new(Line::from(current_keys_hint))
            .block(Block::default().padding(Padding::horizontal(1)));

        f.render_widget(key_hints_footer, footer_chunks[1]);
    }
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
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
