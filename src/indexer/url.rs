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
    let re_nav =
        regex::Regex::new(r"(?si)<script[^>]*>.*?</script>|<style[^>]*>.*?</style>").unwrap();
    let html = re_nav.replace_all(html, "");

    let re_ltx = regex::Regex::new(
        r#"<(p|div|h[1-6])[^>]*class="ltx_(p|section|title|abstract)[^"]*"[^>]*>([^<]*)"#,
    )
    .unwrap();
    let re_tags = regex::Regex::new(r"<[^>]+>").unwrap();

    let mut content = String::new();

    for cap in re_ltx.captures_iter(&html) {
        if let Some(text) = cap.get(3) {
            let text = text.as_str().trim();
            if !text.is_empty() && text.len() > 20 {
                content.push_str(text);
                content.push('\n');
            }
        }
    }

    if content.len() < 100 {
        let re_p = regex::Regex::new(r"<p[^>]*>([^<]+)</p>").unwrap();
        for cap in re_p.captures_iter(&html) {
            if let Some(text) = cap.get(1) {
                let text = text.as_str().trim();
                if !text.is_empty() && text.len() > 30 {
                    content.push_str(text);
                    content.push('\n');
                }
            }
        }
    }

    if content.len() < 100 {
        content = re_tags.replace_all(&html, " ").to_string();
        content = content
            .replace("&nbsp;", " ")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'");
        let re_spaces = regex::Regex::new(r"\s+").unwrap();
        content = re_spaces.replace_all(&content, " ").trim().to_string();
    }

    let re_cite = regex::Regex::new(r"\[[0-9,]+\]").unwrap();
    let lines: Vec<&str> = content
        .lines()
        .filter(|line| {
            let line = line.trim();
            !line.is_empty() && line.len() > 30
        })
        .collect();

    let text = lines.join("\n");
    let text = re_cite.replace_all(&text, "").to_string();

    text.trim().to_string()
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
