use ratatui::{
    prelude::{Alignment, Constraint, Direction, Layout},
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
            "Some title",
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
        for (index, day) in app.days.iter().enumerate() {
            for note in day.notes.iter() {
                let list_item = ListItem::new(Line::from(Span::styled(
                    format!("- {note}"),
                    Style::default().fg(Color::Yellow),
                )));
                if index == app.currently_selected {
                    list_items.push(list_item.bold());
                } else {
                    list_items.push(list_item);
                }
            }
        }
        let list = List::new(list_items)
            .block(Block::default().title("Day TODO add date"))
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

        if app.current_screen == CurrentScreen::Editing {
            let input_buffer_text = Span::styled(
                format!("> {}{}", app.note_buffer, "█"),
                Style::default().fg(Color::Yellow),
            );
            let input_buffer = Paragraph::new(Line::from(input_buffer_text));
            f.render_widget(input_buffer, footer_chunks[0]);
        }

        let current_keys_hint = {
            let text = match app.current_screen {
                CurrentScreen::Main => "(q) quit | (i) edit",
                CurrentScreen::Editing => "(esc) back | (enter) save",
            };
            Span::styled(text, Style::default().fg(Color::Yellow))
        };
        let key_hints_footer = Paragraph::new(Line::from(current_keys_hint))
            .block(Block::default().padding(Padding::horizontal(1)));

        f.render_widget(key_hints_footer, footer_chunks[1]);
    }
}
