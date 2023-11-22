use chrono::{Duration, TimeZone};

#[derive(PartialEq)]
pub enum CurrentScreen {
    Main,
    Editing,
    ViewingDay,
}

pub struct App<T> {
    pub days: Vec<Day>,
    pub should_quit: bool,
    pub current_screen: CurrentScreen,
    pub timezone: T,
    pub date: chrono::NaiveDate,
    pub currently_selected: usize,
}

impl<T: TimeZone> App<T> {
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
}

pub struct Day {
    pub date: chrono::NaiveDate,
    pub notes: Vec<String>,
    pub currently_selected: usize,
    pub note_buffer: String,
    pub updating: bool,
}

impl Day {
    fn new(date: chrono::NaiveDate) -> Self {
        Self {
            date,
            notes: Vec::new(),
            currently_selected: 0,
            note_buffer: String::new(),
            updating: false,
        }
    }

    pub fn save_note(&mut self) {
        let trimmed = self.note_buffer.trim().to_owned();
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
        self.note_buffer.clear();
    }
}
