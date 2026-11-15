#[derive(Clone, Debug)]
pub struct UnknownFile {
    pub path: String,
    pub bytes: Vec<u8>,
}

impl UnknownFile {
    pub fn new(path: impl Into<String>, bytes: Vec<u8>) -> Self {
        Self {
            path: path.into(),
            bytes,
        }
    }

    pub fn from_text(path: impl Into<String>, text: impl Into<String>) -> Self {
        Self::new(path, text.into().into_bytes())
    }
}
