use super::{ContentSource, Result};
use crate::error::KbError;

pub struct UrlSource {
    url: String,
}

impl UrlSource {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
        }
    }
}

impl ContentSource for UrlSource {
    fn extract(&self) -> Result<String> {
        let client = reqwest::blocking::Client::builder()
            .build()
            .map_err(|e| KbError::HttpError(e.to_string()))?;

        let response = client
            .get(&self.url)
            .send()
            .map_err(|e| KbError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(KbError::HttpError(format!(
                "HTTP {}: {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown")
            )));
        }

        let html = response
            .text()
            .map_err(|e| KbError::HttpError(e.to_string()))?;

        let mut readability =
            dom_smoothie::Readability::new(html, Some(&self.url), None).map_err(|e| {
                KbError::IndexingFailed(
                    self.url.clone(),
                    format!("readability init error: {:?}", e),
                )
            })?;

        let article = readability.parse().map_err(|e| {
            KbError::IndexingFailed(self.url.clone(), format!("extraction error: {:?}", e))
        })?;

        Ok(article.text_content.to_string())
    }

    fn source_id(&self) -> String {
        self.url.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_source_id() {
        let source = UrlSource::new("https://example.com");
        assert_eq!(source.source_id(), "https://example.com");
    }
}
