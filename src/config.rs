use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub timezone: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            timezone: "UTC".to_string(),
        }
    }
}
