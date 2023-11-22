use chrono::{Duration, Utc};

#[derive(PartialEq)]
pub enum CurrentScreen {
    Main,
    Editing,
    ViewingDay,
}

pub struct App {
    pub days: Vec<Day>,
    pub should_quit: bool,
    pub current_screen: CurrentScreen,
    pub date: chrono::DateTime<Utc>,
    pub currently_selected: usize,
}

impl App {
    pub fn new() -> Self {
        let days = Vec::from([
            (
                String::from("Day 2"),
                chrono::Utc::now()
                    .checked_sub_signed(Duration::days(1))
                    .unwrap(),
            ),
            (
                String::from("Day 3"),
                chrono::Utc::now()
                    .checked_sub_signed(Duration::days(2))
                    .unwrap(),
            ),
            (
                String::from("Day 4"),
                chrono::Utc::now()
                    .checked_sub_signed(Duration::days(3))
                    .unwrap(),
            ),
            (
                String::from("Day 5"),
                chrono::Utc::now()
                    .checked_sub_signed(Duration::days(4))
                    .unwrap(),
            ),
        ])
        .into_iter()
        .map(|(note, date)| {
            let mut day = Day::new(date);
            day.notes.push(note);
            day
        })
        .collect();
        App {
            days,
            should_quit: false,
            current_screen: CurrentScreen::Main,
            date: chrono::Utc::now(),
            currently_selected: 0,
        }
    }

    pub fn save_note(&mut self) {
        self.days[self.currently_selected].save_note();
    }
}

pub struct Day {
    pub date: chrono::DateTime<Utc>,
    pub notes: Vec<String>,
    pub currently_selected: usize,
    pub note_buffer: String,
}

impl Day {
    fn new(date: chrono::DateTime<Utc>) -> Self {
        Self {
            date,
            notes: Vec::new(),
            currently_selected: 0,
            note_buffer: String::new(),
        }
    }

    pub fn save_note(&mut self) {
        self.notes.push(self.note_buffer.clone());
        self.note_buffer.clear();
    }
}
