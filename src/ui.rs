use ratatui::{
    prelude::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Padding, Paragraph},
    Frame,
};

use crate::app::App;

pub fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(f.size());

    let title_block = Block::default()
        .borders(Borders::ALL)
        .padding(Padding::horizontal(2));

    let title = Paragraph::new(Text::styled(
        format!("Some title {}", app.date.format("%d/%m/%Y %H:%M")),
        Style::default().fg(Color::Yellow).bold(),
    ))
    .block(title_block)
    .alignment(Alignment::Left);

    f.render_widget(title, chunks[0]);

    let mut list_items = Vec::<ListItem>::new();

    for note in &app.days[0].notes {
        list_items.push(ListItem::new(Line::from(Span::styled(
            format!("- {note}"),
            Style::default().fg(Color::Yellow),
        ))));
    }

    let list = List::new(list_items);

    f.render_widget(list, chunks[1]);

    let current_keys_hint = Span::styled("(q) to quit", Style::default().fg(Color::Yellow));
    let input_buffer_text = Span::styled(
        format!("> {}", app.note_buffer),
        Style::default().fg(Color::Yellow),
    );

    let key_hints_footer = Paragraph::new(Line::from(current_keys_hint));
    let input_buffer = Paragraph::new(Line::from(input_buffer_text));

    let footer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(chunks[2]);

    f.render_widget(key_hints_footer, footer_chunks[0]);
    f.render_widget(input_buffer, footer_chunks[1]);
}
