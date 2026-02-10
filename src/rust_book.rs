use anyhow::{anyhow, Result};
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use strsim::jaro_winkler;
use tokio::sync::Mutex;

const BOOK_URL: &str = "https://doc.rust-lang.org/book/";
const TOC_URL: &str = "https://doc.rust-lang.org/book/toc.html";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookChapter {
    pub title: String,
    pub url: String,
    pub content: Option<String>,
}

#[derive(Clone)]
pub struct RustBookClient {
    client: Client,
    chapters: Arc<Mutex<Vec<BookChapter>>>,
}

impl RustBookClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            chapters: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Ensures the Table of Contents is loaded.
    async fn ensure_toc(&self) -> Result<()> {
        let mut chapters = self.chapters.lock().await;
        if !chapters.is_empty() {
            return Ok(());
        }

        let response = self.client.get(TOC_URL).send().await?.text().await?;
        let document = Html::parse_document(&response);
        let selector =
            Selector::parse(".chapter-item a").map_err(|e| anyhow!("Selector error: {:?}", e))?;

        for element in document.select(&selector) {
            if let Some(href) = element.value().attr("href") {
                let text = element.text().collect::<Vec<_>>().join(" ");
                let clean_text = text.trim().to_string();

                // Skip internal anchor links or empty links
                if !href.starts_with("#") && !clean_text.is_empty() {
                    let full_url = if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!("{}{}", BOOK_URL, href)
                    };

                    chapters.push(BookChapter {
                        title: clean_text,
                        url: full_url,
                        content: None,
                    });
                }
            }
        }

        if chapters.is_empty() {
            return Err(anyhow!("Failed to parse Table of Contents from Rust Book"));
        }

        Ok(())
    }

    /// Searches for a concept in the book chapters.
    pub async fn search_concept(&self, query: &str) -> Result<Option<BookChapter>> {
        self.ensure_toc().await?;
        let chapters = self.chapters.lock().await;

        // 1. Exact match (case insensitive)
        if let Some(chapter) = chapters
            .iter()
            .find(|c| c.title.to_lowercase() == query.to_lowercase())
        {
            return Ok(Some(chapter.clone()));
        }

        // 2. Contains match
        if let Some(chapter) = chapters
            .iter()
            .find(|c| c.title.to_lowercase().contains(&query.to_lowercase()))
        {
            return Ok(Some(chapter.clone()));
        }

        // 3. Fuzzy match
        let mut best_match: Option<&BookChapter> = None;
        let mut best_score = 0.0;

        for chapter in chapters.iter() {
            let score = jaro_winkler(&chapter.title.to_lowercase(), &query.to_lowercase());
            if score > best_score {
                best_score = score;
                best_match = Some(chapter);
            }
        }

        if best_score > 0.8 {
            // Threshold for fuzzy match
            return Ok(best_match.cloned());
        }

        Ok(None)
    }

    /// Fetches the content of a chapter.
    pub async fn get_chapter_content(&self, url: &str) -> Result<String> {
        let response = self.client.get(url).send().await?.text().await?;
        let document = Html::parse_document(&response);
        let main_selector =
            Selector::parse("main").map_err(|e| anyhow!("Selector error: {:?}", e))?;

        if let Some(main) = document.select(&main_selector).next() {
            // Convert to Markdown using html2md for better LLM readability
            let html = main.html();
            let md = html2md::parse_html(&html);
            Ok(md)
        } else {
            Err(anyhow!("Could not find <main> content in chapter page"))
        }
    }
}

impl Default for RustBookClient {
    fn default() -> Self {
        Self::new()
    }
}
