use chrono::{Duration, TimeZone};
use ratatui::{
    style::Style,
    widgets::{Block, Borders, Padding},
};
use tui_textarea::{CursorMove, Input, TextArea};

#[derive(PartialEq)]
pub enum CurrentScreen {
    Main,
    Editing,
    ViewingDay,
}

pub struct App<'a, T> {
    pub days: Vec<Day<'a>>,
    pub should_quit: bool,
    pub current_screen: CurrentScreen,
    pub timezone: T,
    pub date: chrono::NaiveDate,
    pub currently_selected: usize,
}

impl<'a, T: TimeZone> App<'a, T> {
    pub fn new(timezone: T) -> Self {
        let mut days: Vec<Day> = Vec::from([
            (
                String::from("Day 2"),
                chrono::Utc::now()
                    .checked_sub_signed(Duration::days(4))
                    .unwrap(),
            ),
            (
                String::from("Day 3"),
                chrono::Utc::now()
                    .checked_sub_signed(Duration::days(3))
                    .unwrap(),
            ),
            (
                String::from("Day 4"),
                chrono::Utc::now()
                    .checked_sub_signed(Duration::days(2))
                    .unwrap(),
            ),
            (
                String::from("Day 5"),
                chrono::Utc::now()
                    .checked_sub_signed(Duration::days(1))
                    .unwrap(),
            ),
            (String::from("Day 5"), chrono::Utc::now()),
        ])
        .into_iter()
        .map(|(note, date)| {
            let mut day = Day::new(date.date_naive());
            day.notes.push(note);
            day
        })
        .collect();

        let current_day = chrono::Utc::now();
        let current_day_naive = current_day.date_naive();
        let currently_selected = match days.iter().position(|day| day.date == current_day_naive) {
            Some(index) => index,
            None => {
                days.push(Day::new(current_day.date_naive()));
                days.len() - 1
            }
        };
        App {
            days,
            should_quit: false,
            current_screen: CurrentScreen::Main,
            timezone,
            date: current_day.date_naive(),
            currently_selected,
        }
    }

    pub fn save_note(&mut self) {
        self.days[self.currently_selected].save_note();
    }

    pub fn input_to_current_day(&mut self, input: Input) {
        self.days[self.currently_selected].note_buffer.input(input);
    }

    pub fn increment_selected(&mut self) {
        if self.currently_selected < self.days.len() - 1 {
            self.currently_selected += 1;
        }
    }

    pub fn decrement_selected(&mut self) {
        if self.currently_selected > 0 {
            self.currently_selected -= 1;
        }
    }
}

pub struct Day<'a> {
    pub date: chrono::NaiveDate,
    pub notes: Vec<String>,
    pub currently_selected: usize,
    pub note_buffer: TextArea<'a>,
    pub updating: bool,
}

impl<'a> Day<'a> {
    fn new(date: chrono::NaiveDate) -> Self {
        Self {
            date,
            notes: Vec::new(),
            currently_selected: 0,
            note_buffer: Self::new_text_area(None),
            updating: false,
        }
    }

    pub fn increment_selected(&mut self) {
        if self.currently_selected < self.notes.len() - 1 {
            self.currently_selected += 1;
        }
    }

    pub fn decrement_selected(&mut self) {
        if self.currently_selected > 0 {
            self.currently_selected -= 1;
        }
    }

    pub fn delete_current_note(&mut self) {
        if !self.notes.is_empty() {
            self.notes.remove(self.currently_selected);
            if self.currently_selected >= self.notes.len() && self.currently_selected > 0 {
                self.currently_selected -= 1;
            }
        }
    }

    pub fn edit_currently_selected(&mut self) {
        self.note_buffer = Day::new_text_area(Some(self.notes[self.currently_selected].to_owned()));
        self.note_buffer.move_cursor(CursorMove::End);
        self.updating = true;
    }

    pub fn new_text_area(input: Option<String>) -> TextArea<'a> {
        let mut textarea = match input {
            Some(input) => TextArea::new(vec![input]),
            None => TextArea::default(),
        };
        textarea.set_placeholder_text("Start typing..");
        textarea.set_cursor_line_style(Style::default());
        textarea.set_block(
            Block::default()
                .title("Note")
                .borders(Borders::ALL)
                .padding(Padding::horizontal(1)),
        );
        textarea
    }

    pub fn save_note(&mut self) {
        let trimmed = self.note_buffer.lines().join(" ").trim().to_owned();
        if trimmed.is_empty() {
            return;
        }
        let new_index = if self.notes.is_empty() {
            0
        } else {
            self.currently_selected + 1
        };
        if self.updating {
            self.notes[self.currently_selected] = trimmed;
            self.updating = false;
        } else {
            self.notes.insert(new_index, trimmed);
            self.currently_selected = new_index;
        }
        self.note_buffer = Self::new_text_area(None);
    }
}
