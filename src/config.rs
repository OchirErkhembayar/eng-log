use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub chars_per_line: Option<usize>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            chars_per_line: Some(80),
        }
    }
}
