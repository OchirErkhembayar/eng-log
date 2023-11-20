use chrono::Utc;

pub struct App {
    pub days: Vec<Day>,
    pub editing: usize,
    pub note_buffer: String,
    pub should_quit: bool,
    pub date: chrono::DateTime<Utc>,
}

impl App {
    pub fn new() -> Self {
        App {
            days: Vec::from([Day::new()]),
            editing: 0,
            note_buffer: String::new(),
            should_quit: false,
            date: chrono::Utc::now(),
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
