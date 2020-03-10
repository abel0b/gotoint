use serde::{Serialize, Deserialize};

// TODO: add update date
#[derive(Serialize, Deserialize)]
pub struct Page {
    _id: String,
    pub extract: String,
}

impl Page {
    pub fn new(url: String, extract: String) -> Self {
        Self {
            _id: url,
            extract,
        }
    }
}
