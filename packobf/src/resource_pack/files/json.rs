#[derive(Clone, Debug)]
pub struct Json {
    pub path: String,
    pub content: serde_json::Value,
}

impl Json {
    pub fn new(path: impl Into<String>, content: serde_json::Value) -> Self {
        Self {
            path: path.into(),
            content,
        }
    }
}
