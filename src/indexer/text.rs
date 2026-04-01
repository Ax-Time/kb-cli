use super::{ContentSource, Result};

pub struct TextSource {
    text: String,
}

impl TextSource {
    pub fn new(text: String) -> Self {
        Self { text }
    }
}

impl ContentSource for TextSource {
    fn extract(&self) -> Result<String> {
        Ok(self.text.clone())
    }

    fn source_id(&self) -> String {
        "text".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_source() {
        let source = TextSource::new("hello world".to_string());
        assert_eq!(source.extract().unwrap(), "hello world");
        assert_eq!(source.source_id(), "text");
    }
}
