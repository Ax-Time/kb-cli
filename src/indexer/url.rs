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

fn extract_text_from_html(html: &str) -> String {
    let re = regex::Regex::new(r"(?si)<script[^>]*>.*?</script>|<style[^>]*>.*?</style>").unwrap();
    let text = re.replace_all(html, " ");
    let re_tags = regex::Regex::new(r"<[^>]+>").unwrap();
    let text = re_tags.replace_all(&text, " ");
    let text = text
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'");
    let re_spaces = regex::Regex::new(r"\s+").unwrap();
    re_spaces.replace_all(&text, " ").trim().to_string()
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

        let text = extract_text_from_html(&html);

        if text.trim().is_empty() {
            return Err(KbError::IndexingFailed(
                self.url.clone(),
                "no text content extracted".to_string(),
            ));
        }

        Ok(text)
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
