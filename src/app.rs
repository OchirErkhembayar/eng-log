use chrono::Utc;

#[derive(PartialEq)]
pub enum CurrentScreen {
    Main,
    Editing,
}

pub struct App {
    pub days: Vec<Day>,
    pub editing: usize,
    pub note_buffer: String,
    pub should_quit: bool,
    pub current_screen: CurrentScreen,
    pub date: chrono::DateTime<Utc>,
    pub currently_selected: usize,
}

impl App {
    pub fn new() -> Self {
        let days = Vec::from([
            String::from("Random"),
            String::from("Day 3"),
            String::from("Day 4"),
            String::from("Day 5"),
        ])
        .into_iter()
        .map(|note| {
            let mut day = Day::new();
            day.notes.push(note);
            day
        })
        .collect();
        App {
            days,
            editing: 0,
            note_buffer: String::new(),
            should_quit: false,
            current_screen: CurrentScreen::Main,
            date: chrono::Utc::now(),
            currently_selected: 0,
        }
    }

    pub fn save_note(&mut self) {
        self.days[self.editing].notes.push(self.note_buffer.clone());
        self.note_buffer.clear();
    }
}

pub struct Day {
    pub notes: Vec<String>,
}

impl Day {
    fn new() -> Self {
        Self { notes: Vec::new() }
    }
}
