use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Page {
    _id: String,
    body: String,
}

impl Page {
    pub fn new(url: String, body: String) -> Self {
        Self {
            _id: url,
            body,
        }
    }
}
