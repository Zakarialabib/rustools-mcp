use anyhow::{anyhow, Result};
use reqwest::Client;
use scraper::{Html, Selector};

const ERROR_INDEX_BASE_URL: &str = "https://doc.rust-lang.org/error_codes/";

#[derive(Clone)]
pub struct ErrorIndexClient {
    client: Client,
}

impl ErrorIndexClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Fetches the explanation for a given error code (e.g., "E0382").
    pub async fn get_error_explanation(&self, code: &str) -> Result<String> {
        let code = code.trim().to_uppercase();
        if !code.starts_with('E') || code.len() < 3 {
            return Err(anyhow!(
                "Invalid error code format. Expected format like 'E0382'"
            ));
        }

        let url = format!("{}{}.html", ERROR_INDEX_BASE_URL, code);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Error code '{}' not found (Status: {})",
                code,
                response.status()
            ));
        }

        let html_content = response.text().await?;
        let document = Html::parse_document(&html_content);

        // The error explanation is usually in a specific container.
        // Looking at the page source of https://doc.rust-lang.org/error_codes/E0382.html
        // It seems to be just the body or main content.
        // Typically rustdoc generated pages have a <main> tag.

        let main_selector =
            Selector::parse("main").map_err(|e| anyhow!("Selector error: {:?}", e))?; // fallback to body if needed?

        if let Some(main) = document.select(&main_selector).next() {
            let html = main.html();
            let md = html2md::parse_html(&html);
            Ok(format!("# Error Code {}\n\n{}", code, md))
        } else {
            // Fallback: convert the whole body? Or just return error.
            // Usually all rustdoc pages have main.
            Err(anyhow!(
                "Could not find <main> content for error code {}",
                code
            ))
        }
    }
}

impl Default for ErrorIndexClient {
    fn default() -> Self {
        Self::new()
    }
}
