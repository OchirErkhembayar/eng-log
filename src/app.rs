use chrono::{Datelike, NaiveDate, TimeZone, Utc};
use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, Padding},
};
use serde::{Deserialize, Serialize};
use std::{char, fs};
use tui_textarea::{CursorMove, Input, TextArea};

#[derive(PartialEq)]
pub enum CurrentScreen {
    Main,
    ViewingDay,
}

#[derive(PartialEq, Clone)]
pub enum Popup {
    NewDay,
}

pub struct PopupBuffer {
    pub day: String,
    pub month: String,
    pub year: String,
    pub currently_selected: u8,
}

impl PopupBuffer {
    fn new() -> Self {
        Self {
            day: String::new(),
            month: String::new(),
            year: String::new(),
            currently_selected: 0,
        }
    }

    pub fn push(&mut self, char: char) {
        match self.currently_selected {
            0 => self.day.push(char),
            1 => self.month.push(char),
            2 => self.year.push(char),
            _ => panic!("Use an enum!"),
        };
    }

    pub fn pop(&mut self) {
        match self.currently_selected {
            0 => self.day.pop(),
            1 => self.month.pop(),
            2 => self.year.pop(),
            _ => panic!("Use an enum!"),
        };
    }

    pub fn clear(&mut self) {
        self.day.clear();
        self.month.clear();
        self.year.clear();
        self.currently_selected = 0;
    }
}

pub struct App<'a, T> {
    pub days: Days,
    pub should_quit: bool,
    pub current_screen: CurrentScreen,
    pub timezone: T,
    pub date: chrono::NaiveDate,
    pub currently_selected: usize,
    pub text_buffer: TextArea<'a>,
    pub popup: Option<Popup>,
    pub popup_buffer: PopupBuffer,
    file_path: String,
    pub min_index: isize, // kind of a hack. think of a better solution
    pub max_index: isize, // kind of a hack. think of a better solution
}

impl<'a, T: TimeZone> App<'a, T> {
    pub fn new(timezone: T, file_path: String) -> Self {
        let days = Self::load_days(file_path.as_str());
        let now = Utc::now().date_naive();
        Self::create_app(days, now, timezone, file_path)
    }

    fn create_app(mut days: Days, now: NaiveDate, timezone: T, file_path: String) -> Self {
        if !days.contains_day(now) {
            days.add(Day::new(now));
        }
        let currently_selected = days.days.iter().position(|d| d.date == now).unwrap();
        let mut app = App {
            days,
            should_quit: false,
            current_screen: CurrentScreen::ViewingDay,
            timezone,
            date: chrono::Utc::now().date_naive(),
            currently_selected,
            text_buffer: App::<T>::new_text_area(None),
            popup: None,
            popup_buffer: PopupBuffer::new(),
            file_path,
            min_index: 0,
            max_index: -1,
        };
        app.load_text();
        app
    }

    fn load_days(file_path: &str) -> Days {
        match fs::read(file_path) {
            Ok(serialized) => postcard::from_bytes(&serialized).unwrap(),
            Err(_) => {
                let days = Days::default();
                App::<T>::save_inner(&days, file_path);
                days
            }
        }
    }

    pub fn load_text(&mut self) {
        if self.days.days.is_empty() {
            self.text_buffer = Self::new_text_area(None);
        } else {
            self.text_buffer =
                Self::new_text_area(Some(self.days.days[self.currently_selected].content_into()));
        }
    }

    pub fn input_to_current_day(&mut self, input: Input) {
        self.text_buffer.input(input);
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

    pub fn save(&mut self) {
        App::<T>::save_inner(&self.days, &self.file_path);
    }

    pub fn finish_editing(&mut self) {
        if !self.days.days.is_empty() {
            self.days.days[self.currently_selected].content = Vec::from(self.text_buffer.lines());
        }
        App::<T>::save_inner(&self.days, &self.file_path);
    }

    fn save_inner(days: &Days, file_path: &str) {
        let serialized: Vec<u8> = postcard::to_allocvec(days).unwrap();
        fs::write(file_path, serialized).expect("Failed to write to file");
    }

    pub fn new_text_area(input: Option<Vec<String>>) -> TextArea<'a> {
        let mut textarea = match input {
            Some(input) => TextArea::new(input),
            None => TextArea::default(),
        };
        textarea.set_style(Style::default().fg(Color::White));
        textarea.set_placeholder_text("Start typing..");
        textarea.set_cursor_line_style(Style::default());
        textarea.set_block(
            Block::default()
                .title("Note")
                .style(Style::default().fg(Color::White))
                .borders(Borders::ALL)
                .padding(Padding::horizontal(1)),
        );
        textarea.move_cursor(CursorMove::Bottom);
        textarea.move_cursor(CursorMove::End);
        textarea
    }

    pub fn remove_day(&mut self) {
        if self.days.len() > 0 {
            self.days.days.remove(self.currently_selected);
            if self.currently_selected >= self.days.len() && self.currently_selected > 0 {
                self.currently_selected -= 1;
            }
            self.save();
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Days {
    pub days: Vec<Day>,
}

impl Days {
    fn default() -> Self {
        Self {
            days: Vec::from([Day::new(chrono::Utc::now().date_naive())]),
        }
    }

    fn len(&self) -> usize {
        self.days.len()
    }

    pub fn add(&mut self, day: Day) -> usize {
        match self.days.binary_search(&day) {
            Ok(pos) => pos,
            Err(pos) => {
                self.days.insert(pos, day);
                pos
            }
        }
    }

    fn contains_day(&self, date: NaiveDate) -> bool {
        self.days.iter().any(|d| d.date == date)
    }
}

#[derive(Serialize, Deserialize, Debug, Eq)]
pub struct Day {
    pub date: NaiveDate,
    pub content: Vec<String>,
}

impl PartialEq for Day {
    fn eq(&self, other: &Self) -> bool {
        self.date == other.date
    }
}

impl PartialOrd for Day {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Day {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.date.cmp(&other.date).reverse()
    }
}

impl Day {
    pub fn new(date: NaiveDate) -> Self {
        Self {
            date,
            content: Vec::new(),
        }
    }

    pub fn date_pretty(&self) -> String {
        format!(
            "{}/{}/{}",
            self.date.day(),
            self.date.month(),
            self.date.year()
        )
    }

    pub fn content_into(&self) -> Vec<String> {
        self.content.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Days;

    const TEST_FILE_PATH: &str = "./test/test.postcard";

    #[test]
    fn save_500_days() {
        let mut app = App::new(Utc, TEST_FILE_PATH.to_string());
        for day in 1..1000 {
            let date = chrono::Utc::now()
                .checked_sub_days(Days::new(day))
                .unwrap()
                .date_naive();
            let day = Day::new(date);
            app.days.add(day);
            app.save();
        }
    }
}
