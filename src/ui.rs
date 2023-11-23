use ratatui::{
    prelude::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Padding, Paragraph},
    Frame,
};

use crate::app::{App, CurrentScreen};

pub fn ui<T>(f: &mut Frame, app: &mut App<T>) {
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
            Style::default().fg(Color::White).bold(),
        ))
        .block(title_block.clone())
        .alignment(Alignment::Center);

        let mut content = if app.current_screen == CurrentScreen::Editing {
            if app.days[app.currently_selected].updating {
                "Editing: ".to_string()
            } else {
                "Inserting: ".to_string()
            }
        } else {
            "Viewing: ".to_string()
        };
        content.push_str(&match app.current_screen {
            CurrentScreen::Main => "Days".to_string(),
            CurrentScreen::Editing | CurrentScreen::ViewingDay => {
                format!("{}", app.date.format("%d/%m/%Y"))
            }
        });
        let subtitle = Paragraph::new(Text::styled(
            content,
            Style::default().fg(Color::White).bold(),
        ))
        .block(title_block)
        .alignment(Alignment::Center);

        f.render_widget(subtitle, title_chunks[0]);
        f.render_widget(title, title_chunks[1]);
    }

    // Body
    {
        let mut list_items = Vec::<ListItem>::new();
        let mut render_day = || {
            let day = &app.days[app.currently_selected];
            for (index, note) in day.notes.iter().enumerate() {
                if index == day.currently_selected {
                    let list_item = ListItem::new(Line::from(Span::styled(
                        note.clone(),
                        Style::default().fg(Color::White).bg(Color::Blue),
                    )));
                    list_items.push(list_item.bold());
                } else {
                    let list_item = ListItem::new(Line::from(Span::styled(
                        note.clone(),
                        Style::default().fg(Color::White),
                    )));
                    list_items.push(list_item);
                }
            }
        };
        match app.current_screen {
            CurrentScreen::ViewingDay => render_day(),
            CurrentScreen::Editing => {
                render_day();
                let popup_block = Block::default()
                    .borders(Borders::NONE)
                    .style(Style::default().bg(Color::DarkGray));
                let area = centered_rect(60, 15, f.size());
                f.render_widget(popup_block, area);

                let text_area = &mut app.days[app.currently_selected].note_buffer;
                text_area.style().not_underlined();

                f.render_widget(text_area.widget(), area);
            }
            CurrentScreen::Main => {
                for (index, day) in app.days.iter().enumerate() {
                    if index == app.currently_selected {
                        let list_item = ListItem::new(Line::from(Span::styled(
                            format!("{}", day.date.format("%d-%m-%Y")),
                            Style::default().fg(Color::White).bg(Color::Blue),
                        )));
                        list_items.push(list_item.bold());
                    } else {
                        let list_item = ListItem::new(Line::from(Span::styled(
                            format!("{}", day.date.format("%d-%m-%Y")),
                            Style::default().fg(Color::White),
                        )));
                        list_items.push(list_item);
                    }
                }
            }
        }
        let list = List::new(list_items)
            .block(Block::default().padding(Padding::horizontal(1)))
            .style(Style::default().fg(Color::White));
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
                CurrentScreen::Main => "(q) quit | (enter) view day | (i) add note",
                CurrentScreen::Editing => "(esc) back | (enter) save",
                CurrentScreen::ViewingDay => {
                    "(q) quit | (esc) back | (i) add note | (e) edit note | (d) delete note"
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
