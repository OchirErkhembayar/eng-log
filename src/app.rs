use chrono::NaiveDate;
use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, Padding},
};
use serde::{Deserialize, Serialize};
use std::{char, fs};
use tui_textarea::{CursorMove, Input, Key, TextArea};

use crate::config::Config;

#[derive(PartialEq)]
pub enum CurrentScreen {
    // determines whether or not we're typing into the filter
    Main(bool),
    ViewingDay,
}

#[derive(PartialEq, Clone)]
pub enum Info {
    About,
}

#[derive(PartialEq, Clone)]
pub enum Popup {
    NewDay,
    ConfDeleteDay,
    Info(Info),
    Config(bool), // bool: whether or not we're editing
}

pub enum PopupBuffer {
    NewDay(NewDayBuffer),
    Config(ConfigBuffer),
}

pub struct ConfigBuffer {
    pub word_limit: String,
}

impl ConfigBuffer {
    pub fn new(word_limit: Option<usize>) -> Self {
        let word_limit = if let Some(limit) = word_limit {
            limit.to_string()
        } else {
            String::new()
        };
        Self { word_limit }
    }

    pub fn push(&mut self, char: char) {
        self.word_limit.push(char);
    }

    pub fn pop(&mut self) {
        self.word_limit.pop();
    }

    pub fn clear(&mut self) {
        self.word_limit.clear();
    }
}

pub struct NewDayBuffer {
    pub day: String,
    pub month: String,
    pub year: String,
    pub currently_selected: u8,
}

impl NewDayBuffer {
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

pub struct App<'a> {
    pub days: Days,
    pub should_quit: bool,
    pub current_screen: CurrentScreen,
    pub date: chrono::NaiveDate,
    pub currently_selected: usize,
    pub text_buffer: TextArea<'a>,
    pub filter_buffer: TextArea<'a>,
    pub popup: Option<Popup>,
    pub popup_buffer: NewDayBuffer,
    pub config_buffer: ConfigBuffer,
    pub file_path: String,
    pub min_index: isize, // kind of a hack. think of a better solution
    pub max_index: isize, // kind of a hack. think of a better solution
    pub saving: bool,
    pub loading: bool,
    pub filter: Option<String>,
    pub config: Config,
}

impl<'a> App<'a> {
    pub fn new(file_path: String, config: Config) -> Self {
        App {
            days: Days::default(),
            should_quit: false,
            current_screen: CurrentScreen::Main(false),
            date: chrono::Local::now().date_naive(),
            currently_selected: 0,
            text_buffer: day_text_area(None),
            filter_buffer: day_text_area(None),
            popup: None,
            popup_buffer: NewDayBuffer::new(),
            config_buffer: ConfigBuffer::new(config.chars_per_line),
            file_path,
            min_index: 0,
            max_index: -1,
            saving: false,
            loading: false,
            filter: None,
            config,
        }
    }

    pub fn switch_to_current_day(&mut self) {
        self.remove_filter();
        let now = self.now();
        self.currently_selected = self.days.iter().position(|d| d.date == now).unwrap();
    }

    fn now(&self) -> NaiveDate {
        chrono::Local::now().date_naive()
    }

    pub fn remove_filter(&mut self) {
        self.filter = None;
        self.current_screen = CurrentScreen::Main(false);
    }

    pub fn load_days(&mut self, switch_screen: bool) {
        let mut days = match fs::read(self.file_path.as_str()) {
            Ok(serialized) => postcard::from_bytes(&serialized).unwrap(),
            Err(_) => {
                let days = Days::default();
                save_inner(&days, self.file_path.as_str());
                days
            }
        };
        let now = self.now();

        if !days.contains_day(now) {
            days.add(Day::new(now));
        }
        let currently_selected = days.iter().position(|d| d.date == now).unwrap();
        self.days = days;
        self.currently_selected = currently_selected;
        if switch_screen {
            self.current_screen = CurrentScreen::ViewingDay;
            self.load_text();
        }
    }

    pub fn load_text(&mut self) {
        let filtered_days: Vec<_> = self.filtered_days().collect();
        if filtered_days.is_empty() {
            self.text_buffer = day_text_area(None);
        } else {
            self.text_buffer =
                day_text_area(Some(filtered_days[self.currently_selected].content_into()));
        }
    }

    pub fn input_to_current_day(&mut self, input: Input) {
        if let Key::Char(_) = input.key {
            if let Some(limit) = self.config.chars_per_line {
                if !input.ctrl {
                    let (y, _) = self.text_buffer.cursor();
                    let current_line = &self.text_buffer.lines()[y];
                    if current_line.len() + 1 > limit {
                        if current_line.split_whitespace().count() > 1 {
                            self.text_buffer.move_cursor(CursorMove::WordBack);
                            self.text_buffer.delete_next_word();
                            self.text_buffer.insert_newline();
                            self.text_buffer.insert_str(self.text_buffer.yank_text());
                        } else {
                            self.text_buffer.insert_newline();
                        }
                    }
                }
            }
        }
        self.text_buffer.input(input);
    }

    pub fn input_to_filter_buffer(&mut self, input: Input) {
        self.filter_buffer.input(input);
        self.filter = Some(self.filter_buffer.lines().join(""));
        let count = self.filtered_days().count();
        if self.currently_selected >= count && count > 0 {
            self.currently_selected = count - 1;
        }
    }

    // This count is getting relied on quite heavily
    // Inefficient but it seems quite fast ATM
    pub fn filtered_days(&self) -> impl Iterator<Item = &Day> {
        self.days.iter_filtered(self.filter.as_deref())
    }

    pub fn filtered_days_mut(&mut self) -> impl Iterator<Item = &mut Day> {
        self.days.iter_mut_filtered(self.filter.as_deref())
    }

    pub fn increment_selected(&mut self) {
        if self.currently_selected < self.filtered_days().count() - 1 {
            self.currently_selected += 1;
        }
    }

    pub fn decrement_selected(&mut self) {
        if self.currently_selected > 0 {
            self.currently_selected -= 1;
        }
    }

    pub fn save(&mut self) {
        save_inner(&self.days, &self.file_path);
    }

    pub fn update_day_from_buffer(&mut self) {
        let selected = self.currently_selected;
        let content = Vec::from(self.text_buffer.lines());
        let mut filtered_days: Vec<_> = self.filtered_days_mut().collect();
        if !filtered_days.is_empty() {
            filtered_days[selected].content = content;
        }
    }

    pub fn init_filter_text(&mut self) {
        self.filter_buffer = new_text_area(
            None,
            Block::default()
                .style(Style::default().fg(Color::White))
                .borders(Borders::NONE)
                .padding(Padding::horizontal(1)),
        );
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

pub fn save_inner(days: &Days, file_path: &str) {
    let serialized: Vec<u8> = postcard::to_allocvec(days).unwrap();
    let _ = fs::File::create(file_path);
    fs::write(file_path, serialized).expect("Failed to write to file");
}

fn new_text_area(input: Option<Vec<String>>, block: Block<'_>) -> TextArea<'_> {
    let mut textarea = match input {
        Some(input) => TextArea::new(input),
        None => TextArea::default(),
    };
    textarea.set_block(block);
    textarea.set_style(Style::default().fg(Color::White));
    textarea.set_cursor_line_style(Style::default());
    textarea.move_cursor(CursorMove::Bottom);
    textarea.move_cursor(CursorMove::End);
    textarea
}

pub fn day_text_area<'a>(input: Option<Vec<String>>) -> TextArea<'a> {
    let mut textarea = new_text_area(
        input,
        Block::default()
            .title("Note")
            .style(Style::default().fg(Color::White))
            .borders(Borders::ALL)
            .padding(Padding::horizontal(1)),
    );
    textarea.set_placeholder_text("Start typing..");
    textarea
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Days {
    pub days: Vec<Day>,
}

impl Days {
    fn default() -> Self {
        Self { days: Vec::new() }
    }

    pub fn iter_mut_filtered<'a>(
        &'a mut self,
        string: Option<&'a str>,
    ) -> impl Iterator<Item = &mut Day> + 'a {
        self.days.iter_mut().filter(move |d| {
            let date_str = d.date.format("%d/%m/%Y").to_string();
            if let Some(string) = string {
                date_str.contains(string)
            } else {
                true
            }
        })
    }

    pub fn iter_filtered<'a>(&'a self, string: Option<&'a str>) -> impl Iterator<Item = &Day> + 'a {
        self.days.iter().filter(move |d| {
            let date_str = d.date.format("%d/%m/%Y").to_string();
            if let Some(string) = string {
                date_str.contains(string)
            } else {
                true
            }
        })
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

    pub fn iter(&self) -> impl Iterator<Item = &Day> {
        self.days.iter()
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, Clone)]
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
        self.date.format("%d/%m/%Y").to_string()
    }

    pub fn content_into(&self) -> Vec<String> {
        self.content.clone()
    }
}
