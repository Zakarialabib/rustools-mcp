//! Documentation fetching and parsing functionality for docs.rs.

use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use rmcp::schemars;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur when fetching and parsing documentation.
#[derive(Debug, Error, Serialize)]
pub enum DocsFetchError {
    /// Error occurred during HTTP request
    #[error("Request error: {0}")]
    RequestError(String), // We store string representation

    /// Error parsing or constructing URLs
    #[error("Invalid URL: {0}")]
    UrlError(String),

    /// Documentation was not found at the specified location
    #[error("Failed to find documentation")]
    DocsNotFound,

    /// Error occurred while parsing documentation content
    #[error("Failed to parse documentation: {0}")]
    ParseError(String),

    #[error("Crate not found: {0}")]
    CrateNotFound(String),

    #[error("Item not found: {0}")]
    ItemNotFound(String),

    #[error("Documentation content not found in page")]
    ContentNotFound,

    #[error("Version not found: {0} for crate {1}")]
    VersionNotFound(String, String),

    #[error("Command failed: {0}")]
    CommandError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

impl From<reqwest::Error> for DocsFetchError {
    fn from(e: reqwest::Error) -> Self {
        DocsFetchError::RequestError(e.to_string())
    }
}

impl From<reqwest_middleware::Error> for DocsFetchError {
    fn from(e: reqwest_middleware::Error) -> Self {
        DocsFetchError::RequestError(e.to_string())
    }
}

impl From<url::ParseError> for DocsFetchError {
    fn from(e: url::ParseError) -> Self {
        DocsFetchError::UrlError(e.to_string())
    }
}

/// Parameters for specifying which documentation to fetch from docs.rs.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, Eq, PartialEq, Hash)]
pub struct DocsRsParams {
    /// Name of the crate to fetch documentation for
    #[schemars(description = "name of crate")]
    pub crate_name: String,

    /// Version of the crate (e.g., "1.0.0")
    /// If not specified, the latest version will be used.
    #[schemars(
        description = "version of crate, e.g. 1.0.0. If not specified, the latest version will be used."
    )]
    pub version: String,

    /// Path to the specific documentation page
    #[schemars(description = "path of the module, struct, function, trait, etc.")]
    pub path: String,
}

/// Documentation content fetched from docs.rs.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct DocContent {
    /// The extracted documentation content as plain text
    pub content: String,
}

#[derive(Deserialize, Debug)]
struct CrateResponse {
    #[serde(rename = "crate")]
    krate: CrateInfo,
}

#[derive(Deserialize, Debug)]
struct CrateInfo {
    max_version: String,
    #[allow(dead_code)]
    description: Option<String>,
    repository: Option<String>,
}

#[derive(Deserialize, Debug)]
struct CrateVersionResponse {
    version: VersionInfo,
}

#[derive(Deserialize, Debug)]
struct VersionInfo {
    features: HashMap<String, Vec<String>>,
}

#[derive(Deserialize, Debug)]
struct CratesIoSearchResponse {
    crates: Vec<CratesIoCrate>,
}

#[derive(Deserialize, Debug)]
struct CratesIoCrate {
    name: String,
    max_version: String,
    description: Option<String>,
}

/// Cache key for documentation requests
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum CacheKey {
    FindCrates {
        query: String,
        limit: Option<u32>,
        fuzzy: Option<bool>,
    },
    GetCrateOverview {
        crate_name: String,
        version: Option<String>,
    },
    GetCrateModules {
        crate_name: String,
        version: Option<String>,
        limit: Option<usize>,
    },
    GetSymbolDocs {
        crate_name: String,
        symbol_path: String,
        version: Option<String>,
    },
    FetchRawDoc(DocsRsParams),
    ReadSourceFile {
        crate_name: String,
        path: String,
        version: Option<String>,
        start_line: Option<usize>,
        end_line: Option<usize>,
    },
    GetCrateDependencies {
        crate_name: String,
        version: String,
        kind: Option<String>,
    },
    GetCrateExamples {
        crate_name: String,
        version: Option<String>,
        limit: Option<usize>,
    },
    AnalyzeFeatureFlags {
        crate_name: String,
        version: String,
    },
    FindBySignature {
        crate_name: String,
        signature_pattern: String,
        version: Option<String>,
    },
    FindTraitImplementors {
        crate_name: String,
        trait_path: String,
        version: Option<String>,
        limit: Option<usize>,
    },
    GetContextBundle {
        crate_name: String,
        version: Option<String>,
    },
}

/// Client for fetching documentation from docs.rs.
#[derive(Clone)]
pub struct DocsRsClient {
    /// HTTP client for making requests
    client: ClientWithMiddleware,
    /// Base URL for the docs.rs service
    base_url: String,
}

impl Default for DocsRsClient {
    fn default() -> Self {
        Self::new()
    }
}

impl DocsRsClient {
    /// Creates a new client instance with the default docs.rs base URL.
    pub fn new() -> Self {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
        let user_agent = format!("rustools-mcp/{}", env!("CARGO_PKG_VERSION"));
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .user_agent(user_agent)
            .build()
            .unwrap_or_default();

        let client = ClientBuilder::new(client)
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        Self {
            client,
            base_url: "https://docs.rs".to_string(),
        }
    }

    /// Helper to get the latest version of a crate from crates.io
    async fn get_latest_version(&self, crate_name: &str) -> Result<String, DocsFetchError> {
        self.get_crate_metadata(crate_name)
            .await
            .map(|info| info.max_version)
    }

    async fn get_crate_metadata(&self, crate_name: &str) -> Result<CrateInfo, DocsFetchError> {
        let url = format!("https://crates.io/api/v1/crates/{}", crate_name);
        let response = self
            .client
            .get(&url)
            .header(
                "User-Agent",
                format!("rdoc-mcp/{}", env!("CARGO_PKG_VERSION")),
            )
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(DocsFetchError::CrateNotFound(crate_name.to_string()));
        }

        let crate_data: CrateResponse = response
            .json()
            .await
            .map_err(|e| DocsFetchError::ParseError(e.to_string()))?;

        Ok(crate_data.krate)
    }

    /// Helper to resolve version "latest" or None to the actual latest version
    async fn resolve_version(
        &self,
        crate_name: &str,
        version: Option<String>,
    ) -> Result<String, DocsFetchError> {
        if self.is_std_crate(crate_name) {
            return Ok(version.unwrap_or_else(|| "latest".to_string()));
        }
        match version {
            Some(v) if v != "latest" => Ok(v),
            _ => self.get_latest_version(crate_name).await,
        }
    }

    fn is_std_crate(&self, name: &str) -> bool {
        matches!(name, "std" | "core" | "alloc" | "proc_macro" | "test")
    }

    fn get_base_url(&self, crate_name: &str, version: &str) -> String {
        if self.is_std_crate(crate_name) {
            if version == "latest" {
                "https://doc.rust-lang.org".to_string()
            } else {
                format!("https://doc.rust-lang.org/{}", version)
            }
        } else {
            self.base_url.clone()
        }
    }

    pub async fn search_crates(
        &self,
        query: String,
        limit: Option<u32>,
        fuzzy: bool,
    ) -> Result<String, DocsFetchError> {
        let requested_limit = limit.unwrap_or(10);
        let fetch_limit = if fuzzy { std::cmp::max(50, requested_limit * 2) } else { requested_limit };
        
        let url = format!(
            "https://crates.io/api/v1/crates?q={}&per_page={}",
            query, fetch_limit
        );

        let response = self.client.get(&url).send().await?;

        let data: CratesIoSearchResponse = response
            .json()
            .await
            .map_err(|e| DocsFetchError::ParseError(e.to_string()))?;

        let mut crates = data.crates;

        if fuzzy {
            crates.sort_by(|a, b| {
                let score_a = strsim::jaro_winkler(&a.name, &query);
                let score_b = strsim::jaro_winkler(&b.name, &query);
                score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
            });
            if crates.len() > requested_limit as usize {
                crates.truncate(requested_limit as usize);
            }
        } else if crates.len() > requested_limit as usize {
            crates.truncate(requested_limit as usize);
        }

        let mut output = format!("# Search Results for '{}' (fuzzy: {})\n\n", query, fuzzy);
        output.push_str(
            "| Crate | Latest Version | Description |\n|-------|----------------|-------------|\n",
        );

        for krate in crates {
            output.push_str(&format!(
                "| {} | {} | {} |\n",
                krate.name,
                krate.max_version,
                krate.description.unwrap_or_default().replace("\n", " ")
            ));
        }

        Ok(output)
    }

    pub async fn lookup_crate(
        &self,
        crate_name: String,
        version: Option<String>,
    ) -> Result<DocContent, DocsFetchError> {
        let ver = self.resolve_version(&crate_name, version).await?;
        let base = self.get_base_url(&crate_name, &ver);

        let url = if self.is_std_crate(&crate_name) {
            format!("{}/{}/index.html", base, crate_name)
        } else {
            format!("{}/{}/{}/", base, crate_name, ver)
        };

        // Fetch metadata for non-std crates to add context
        let metadata_header = if !self.is_std_crate(&crate_name) {
            match self.get_crate_metadata(&crate_name).await {
                Ok(info) => {
                    let mut header = format!("# Crate: {} {}\n\n", crate_name, ver);
                    if let Some(desc) = info.description {
                        header.push_str(&format!("> {}\n\n", desc));
                    }
                    if let Some(repo) = info.repository {
                        header.push_str(&format!("- **Repository**: {}\n", repo));
                    }
                    header.push_str(&format!("- **Docs**: {}\n\n---\n\n", url));
                    Some(header)
                }
                Err(_) => None,
            }
        } else {
            None
        };

        let response = self.client.get(&url).send().await?;
        if !response.status().is_success() {
            return Err(DocsFetchError::DocsNotFound);
        }

        let html = response.text().await?;
        let mut content = DocsRsClient::extract_content(&html, &url)?;

        if let Some(header) = metadata_header {
            content = format!("{}{}", header, content);
        }

        Ok(DocContent { content })
    }

    pub async fn extract_modules(
        &self,
        crate_name: String,
        version: Option<String>,
        _limit: Option<usize>,
    ) -> Result<DocContent, DocsFetchError> {
        // Just fetch the root page and parse modules
        self.lookup_crate(crate_name, version).await
    }

    pub async fn lookup_item(
        &self,
        crate_name: String,
        symbol_path: String,
        version: Option<String>,
    ) -> Result<DocContent, DocsFetchError> {
        // Handle fully qualified paths (e.g. "std::vec::Vec")
        let clean_path = if symbol_path.starts_with(&format!("{}::", crate_name)) {
            symbol_path
                .trim_start_matches(&format!("{}::", crate_name))
                .to_string()
        } else if symbol_path.starts_with("::") {
            symbol_path.trim_start_matches("::").to_string()
        } else {
            symbol_path
        };

        let parts: Vec<&str> = clean_path.split("/").collect(); // Using / as separator internally from mcp.rs normalization
                                                                // Fallback to :: if no / found
        let parts: Vec<&str> = if parts.len() == 1 {
            clean_path.split("::").collect()
        } else {
            parts
        };

        // Resolve version once here
        let resolved_version = self.resolve_version(&crate_name, version).await?;
        match self
            .lookup_item_internal(&crate_name, &parts, resolved_version)
            .await
        {
            Ok(content) => Ok(DocContent { content }),
            Err(e) => {
                // Special handling for 'windows' crate which hosts docs externally
                if crate_name == "windows" {
                    if let Ok(content) = self.lookup_item_external_windows(&parts).await {
                        return Ok(DocContent { content });
                    }
                }
                Err(e)
            }
        }
    }

    async fn lookup_item_external_windows(&self, parts: &[&str]) -> Result<String, DocsFetchError> {
        let item_name = parts.last().unwrap();
        let module_path = parts[..parts.len() - 1].join("/");
        let base = "https://microsoft.github.io/windows-docs-rs/doc";

        let item_types = [
            "struct", "enum", "trait", "fn", "macro", "type", "constant", "mod",
        ];

        for item_type in item_types.iter() {
            let url = if *item_type == "mod" {
                if module_path.is_empty() {
                    format!("{}/windows/{}/index.html", base, item_name)
                } else {
                    format!("{}/windows/{}/{}/index.html", base, module_path, item_name)
                }
            } else if module_path.is_empty() {
                format!("{}/windows/{}.{}.html", base, item_type, item_name)
            } else {
                format!(
                    "{}/windows/{}/{}.{}.html",
                    base, module_path, item_type, item_name
                )
            };

            tracing::debug!("Trying External URL: {}", url);

            if let Ok(response) = self.client.get(&url).send().await {
                if response.status().is_success() {
                    if let Ok(html) = response.text().await {
                        return DocsRsClient::extract_content(&html, &url);
                    }
                }
            }
        }

        Err(DocsFetchError::ItemNotFound(
            "External windows docs not found".to_string(),
        ))
    }

    async fn lookup_item_internal(
        &self,
        crate_name: &str,
        parts: &[&str],
        version: String,
    ) -> Result<String, DocsFetchError> {
        let item_name = parts.last().unwrap();
        let module_path = parts[..parts.len() - 1].join("/");
        let base = self.get_base_url(crate_name, &version);

        let item_types = [
            "struct", "enum", "trait", "fn", "macro", "type", "constant", "mod",
        ];
        let mut last_error = None;

        for item_type in item_types.iter() {
            let url = if self.is_std_crate(crate_name) {
                // std structure: {base}/{crate}/{module_path}/{item_type}.{item_name}.html
                // or mod: {base}/{crate}/{module_path}/{item_name}/index.html
                if *item_type == "mod" {
                    if module_path.is_empty() {
                        format!("{}/{}/{}/index.html", base, crate_name, item_name)
                    } else {
                        format!(
                            "{}/{}/{}/{}/index.html",
                            base, crate_name, module_path, item_name
                        )
                    }
                } else if module_path.is_empty() {
                    format!("{}/{}/{}.{}.html", base, crate_name, item_type, item_name)
                } else {
                    format!(
                        "{}/{}/{}/{}.{}.html",
                        base, crate_name, module_path, item_type, item_name
                    )
                }
            } else {
                // docs.rs structure: {base}/{crate}/{version}/{crate}/{module_path}/{item_type}.{item_name}.html
                if *item_type == "mod" {
                    if module_path.is_empty() {
                        format!(
                            "{}/{}/{}/{}/{}/index.html",
                            base, crate_name, version, crate_name, item_name
                        )
                    } else {
                        format!(
                            "{}/{}/{}/{}/{}/{}/index.html",
                            base, crate_name, version, crate_name, module_path, item_name
                        )
                    }
                } else if module_path.is_empty() {
                    format!(
                        "{}/{}/{}/{}/{}.{}.html",
                        base, crate_name, version, crate_name, item_type, item_name
                    )
                } else {
                    format!(
                        "{}/{}/{}/{}/{}/{}.{}.html",
                        base, crate_name, version, crate_name, module_path, item_type, item_name
                    )
                }
            };

            tracing::debug!("Trying URL: {}", url);

            let response = match self.client.get(&url).send().await {
                Ok(r) => r,
                Err(e) => {
                    last_error = Some(DocsFetchError::RequestError(e.to_string()));
                    continue;
                }
            };

            if response.status().is_success() {
                let html = response
                    .text()
                    .await
                    .map_err(|e| DocsFetchError::RequestError(e.to_string()))?;
                return Self::extract_content(&html, &url);
            }
        }

        Err(last_error
            .unwrap_or_else(|| DocsFetchError::ItemNotFound("Status: 404 Not Found".to_string())))
    }

    pub async fn fetch_docs(&self, params: DocsRsParams) -> Result<DocContent, DocsFetchError> {
        let ver = self
            .resolve_version(&params.crate_name, Some(params.version))
            .await?;
        let url = format!(
            "{}/{}/{}/{}",
            self.base_url, params.crate_name, ver, params.path
        );
        let response = self.client.get(&url).send().await?;
        if !response.status().is_success() {
            return Err(DocsFetchError::DocsNotFound);
        }
        let html = response.text().await?;
        let content = Self::extract_content(&html, &url)?;
        Ok(DocContent { content })
    }

    /// Fetches a raw URL and returns content (converted to markdown if HTML).
    pub async fn fetch_url(&self, url: &str) -> Result<String, DocsFetchError> {
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(DocsFetchError::RequestError(format!(
                "Status: {}",
                response.status()
            )));
        }
        let html = response.text().await?;
        Self::extract_content(&html, url)
    }

    pub fn extract_content(html: &str, url: &str) -> Result<String, DocsFetchError> {
        let document = Html::parse_document(html);
        let selector = Selector::parse("#main-content").unwrap();

        if let Some(element) = document.select(&selector).next() {
            let html_fragment = element.html();
            let markdown = html2md::parse_html(&html_fragment);

            // Cleanup Markdown
            // 1. Reduce excessive newlines (3+ -> 2)
            let mut clean_md = markdown.replace("\n\n\n", "\n\n");
            while clean_md.contains("\n\n\n") {
                clean_md = clean_md.replace("\n\n\n", "\n\n");
            }

            let output = format!("Source: {}\n\n{}", url, clean_md);
            return Ok(output);
        }
        Err(DocsFetchError::ContentNotFound)
    }

    pub async fn analyze_feature_flags(
        &self,
        crate_name: String,
        version: String,
    ) -> Result<String, DocsFetchError> {
        let ver = self.resolve_version(&crate_name, Some(version)).await?;

        let url = format!("https://crates.io/api/v1/crates/{}/{}", crate_name, ver);
        let response = self
            .client
            .get(&url)
            .header(
                "User-Agent",
                format!(
                    "rdoc-mcp/{} (contact@example.com)",
                    env!("CARGO_PKG_VERSION")
                ),
            )
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(DocsFetchError::VersionNotFound(ver, crate_name));
        }

        let api_response: CrateVersionResponse = response
            .json()
            .await
            .map_err(|e| DocsFetchError::ParseError(e.to_string()))?;

        let mut output = format!("# Feature Flags for {} {}\n\n", crate_name, ver);
        if api_response.version.features.is_empty() {
            output.push_str("No feature flags found.\n");
            return Ok(output);
        }
        output.push_str("| Feature | Enabled By Default | Dependencies |\n|---------|--------------------|--------------|\n");
        let mut features: Vec<_> = api_response.version.features.iter().collect();
        features.sort_by(|a, b| a.0.cmp(b.0));
        for (feature_name, enables) in features {
            let is_default = feature_name == "default";
            let enables_str = if enables.is_empty() {
                "-".to_string()
            } else {
                enables.join(", ")
            };
            output.push_str(&format!(
                "| **{}** | {} | {} |\n",
                feature_name,
                if is_default { "Yes" } else { "No" },
                enables_str
            ));
        }
        Ok(output)
    }

    pub async fn read_source_file(
        &self,
        crate_name: String,
        path: String,
        version: Option<String>,
        _start_line: Option<usize>,
        _end_line: Option<usize>,
    ) -> Result<DocContent, DocsFetchError> {
        // Construct source URL: https://docs.rs/{crate}/{version}/src/{crate_safe}/{path}.html
        // where {crate_safe} has dashes replaced by underscores.
        let ver = self.resolve_version(&crate_name, version).await?;

        let crate_safe = crate_name.replace("-", "_");
        let clean_path = path.trim_start_matches("src/").trim_start_matches("/");

        let url = format!(
            "{}/{}/{}/src/{}/{}.html",
            self.base_url, crate_name, ver, crate_safe, clean_path
        );

        tracing::debug!("Fetching source from URL: {}", url);

        let response = self.client.get(&url).send().await?;
        if !response.status().is_success() {
            // Try fallback: maybe it's at the root src level?
            // Some crates might have weird structures.
            return Err(DocsFetchError::DocsNotFound);
        }
        let html = response.text().await?;

        // Extract code from html
        // The code is usually inside <pre class="rust"> or similar.
        // We reuse extract_content but we might want to be more specific for source files to avoid getting navigation bars.
        // For now, extract_content pulls #main-content which includes the code.
        let content = DocsRsClient::extract_content(&html, &url)?;
        Ok(DocContent { content })
    }

    pub async fn get_crate_modules(
        &self,
        crate_name: String,
        version: Option<String>,
        limit: Option<usize>,
    ) -> Result<String, DocsFetchError> {
        let ver = self.resolve_version(&crate_name, version).await?;
        let base = self.get_base_url(&crate_name, &ver);

        let url = if self.is_std_crate(&crate_name) {
            format!("{}/{}/index.html", base, crate_name)
        } else {
            format!("{}/{}/{}/index.html", base, crate_name, ver)
        };

        let response = self.client.get(&url).send().await?;
        if !response.status().is_success() {
            return Err(DocsFetchError::ItemNotFound(format!(
                "Crate root not found at {}",
                url
            )));
        }

        let html = response
            .text()
            .await
            .map_err(|e| DocsFetchError::RequestError(e.to_string()))?;
        let document = Html::parse_document(&html);

        // Robust selector for rows in item tables (handles various rustdoc versions)
        let row_selector = Selector::parse(".item-table .item-row, .item-table li, table.item-table tr").unwrap();
        // Selector for item name links
        let name_selector = Selector::parse(".item-name > a, .item-left > a, td > a.mod, td > a.struct, td > a.enum, td > a.trait, td > a.fn, td > a.macro, td > a.type, td > a.constant").unwrap();
        // Selector for descriptions
        let desc_selector = Selector::parse(".desc, .item-right, .docblock-short").unwrap();

        let mut items_by_type: HashMap<String, Vec<(String, String)>> = HashMap::new();
        let mut total_found = 0;

        for row in document.select(&row_selector) {
            if let Some(link) = row.select(&name_selector).next() {
                let name = link.text().collect::<Vec<_>>().join("");
                let name = name.trim().to_string();
                if name.is_empty() { continue; }

                // Determine type from class
                let classes: Vec<&str> = link.value().classes().collect();
                let item_type = if classes.contains(&"mod") { "Modules" }
                else if classes.contains(&"struct") { "Structs" }
                else if classes.contains(&"enum") { "Enums" }
                else if classes.contains(&"trait") { "Traits" }
                else if classes.contains(&"fn") { "Functions" }
                else if classes.contains(&"macro") { "Macros" }
                else if classes.contains(&"type") { "Types" }
                else if classes.contains(&"constant") { "Constants" }
                else { "Other" };

                // Get description
                let desc = row.select(&desc_selector).next()
                    .map(|d| d.text().collect::<Vec<_>>().join(" "))
                    .unwrap_or_default();
                let desc = desc.trim().replace("\n", " ");
                // Truncate desc if too long
                let desc = if desc.len() > 100 {
                    format!("{}...", &desc[..97])
                } else {
                    desc
                };

                items_by_type.entry(item_type.to_string())
                    .or_default()
                    .push((name, desc));
                total_found += 1;
            }
        }

        if total_found == 0 {
            // Fallback for very old or weirdly structured docs: just list all links with known classes
            let fallback_selector = Selector::parse("a.mod, a.struct, a.enum, a.trait, a.fn, a.macro").unwrap();
            for link in document.select(&fallback_selector) {
                let name = link.text().collect::<Vec<_>>().join(" ").trim().to_string();
                if name.is_empty() { continue; }
                 let classes: Vec<&str> = link.value().classes().collect();
                let item_type = if classes.contains(&"mod") { "Modules" }
                else if classes.contains(&"struct") { "Structs" }
                else if classes.contains(&"enum") { "Enums" }
                else if classes.contains(&"trait") { "Traits" }
                else if classes.contains(&"fn") { "Functions" }
                else if classes.contains(&"macro") { "Macros" }
                else { "Other" };

                items_by_type.entry(item_type.to_string())
                    .or_default()
                    .push((name, String::new()));
                total_found += 1;
            }
        }

        if total_found == 0 {
             return Ok(format!("No modules or items found at {}. The documentation might use a non-standard layout.", url));
        }

        // Format output
        let limit = limit.unwrap_or(50);
        let mut output = String::new();
        output.push_str(&format!("# API Overview for {} {}\n\n", crate_name, ver));
        output.push_str(&format!("*Found {} items. Showing top {}.*\n\n", total_found, limit.min(total_found)));

        let order = ["Modules", "Structs", "Enums", "Traits", "Functions", "Macros", "Types", "Constants", "Other"];
        let mut printed_count = 0;

        for category in order {
            if let Some(items) = items_by_type.get(category) {
                if items.is_empty() { continue; }
                
                output.push_str(&format!("## {}\n", category));
                
                let remaining = limit.saturating_sub(printed_count);
                if remaining == 0 {
                    output.push_str("*... remaining items hidden ...*\n");
                    break;
                }
                
                // Heuristic: Don't let one category consume everything if there are others, 
                // UNLESS it's "Modules" (most important).
                let category_limit = if category == "Modules" { remaining } else { remaining.min(10) };
                
                let count_to_show = items.len().min(category_limit);
                
                for (name, desc) in items.iter().take(count_to_show) {
                    if desc.is_empty() {
                         output.push_str(&format!("- **{}**\n", name));
                    } else {
                         output.push_str(&format!("- **{}**: {}\n", name, desc));
                    }
                }
                
                if items.len() > count_to_show {
                    output.push_str(&format!("*... and {} more ...*\n", items.len() - count_to_show));
                }
                
                printed_count += count_to_show;
                output.push_str("\n");
            }
        }
        
        Ok(output)
    }

    pub async fn get_crate_dependencies(
        &self,
        crate_name: String,
        version: String,
        _kind: Option<String>,
    ) -> Result<String, DocsFetchError> {
        // Use crates.io dependencies API: https://crates.io/api/v1/crates/{crate_name}/{version}/dependencies
        let ver = self.resolve_version(&crate_name, Some(version)).await?;
        let url = format!(
            "https://crates.io/api/v1/crates/{}/{}/dependencies",
            crate_name, ver
        );
        let response = self
            .client
            .get(&url)
            .header(
                "User-Agent",
                format!(
                    "rdoc-mcp/{} (contact@example.com)",
                    env!("CARGO_PKG_VERSION")
                ),
            )
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(DocsFetchError::ParseError(
                "Failed to fetch dependencies".to_string(),
            ));
        }
        // Parse JSON and format table (omitted for brevity, returning raw json)
        let text = response
            .text()
            .await
            .map_err(|e| DocsFetchError::RequestError(e.to_string()))?;
        Ok(text)
    }

    pub async fn get_crate_examples(
        &self,
        crate_name: String,
        version: Option<String>,
        _limit: Option<usize>,
    ) -> Result<String, DocsFetchError> {
        let ver = self.resolve_version(&crate_name, version).await?;
        let crate_safe = crate_name.replace("-", "_");
        let url = format!(
            "{}/{}/{}/src/{}/examples/",
            self.base_url, crate_name, ver, crate_safe
        );

        tracing::debug!("Checking for examples at: {}", url);

        let response = self.client.get(&url).send().await?;
        if response.status().is_success() {
            Ok(format!(
                "Examples directory found at source root.\n\nPlease browse the examples here:\n{}",
                url
            ))
        } else {
            // Fallback: Fetch metadata to get repo URL
            let repo_msg = match self.get_crate_metadata(&crate_name).await {
                Ok(info) => match info.repository {
                    Some(repo) => format!(
                        "Please check the crate's repository for examples:\n{}",
                        repo
                    ),
                    None => "No repository link found in crate metadata.".to_string(),
                },
                Err(_) => "Could not fetch crate metadata.".to_string(),
            };

            Ok(format!("No 'examples' directory found in published source at {}.\n\nNote: Many crates exclude examples from the published package to reduce size.\n\n{}", url, repo_msg))
        }
    }

    pub async fn find_by_signature(
        &self,
        crate_name: String,
        pattern: String,
        version: Option<String>,
    ) -> Result<String, DocsFetchError> {
        let ver = self.resolve_version(&crate_name, version).await?;
        let base = self.get_base_url(&crate_name, &ver);

        let url = if self.is_std_crate(&crate_name) {
            format!(
                "{}/{}/?search={}",
                base,
                crate_name,
                urlencoding::encode(&pattern)
            )
        } else {
            format!(
                "{}/{}/{}/{}/?search={}",
                base,
                crate_name,
                ver,
                crate_name,
                urlencoding::encode(&pattern)
            )
        };

        Ok(format!(
            "Search by signature requires client-side JavaScript and cannot be performed via this tool.\n\nPlease visit the search results page directly:\n{}", 
            url
        ))
    }

    pub async fn find_trait_implementors(
        &self,
        crate_name: String,
        trait_path: String,
        version: Option<String>,
        _limit: Option<usize>,
    ) -> Result<String, DocsFetchError> {
        // We reuse lookup_item to fetch the trait page and extract its content.
        // The markdown conversion should include the "Implementors" section if present.
        match self.lookup_item(crate_name, trait_path, version).await {
            Ok(doc) => {
                // Try to find the implementors section in the markdown
                if let Some(idx) = doc.content.find("# Implementors") {
                    Ok(doc.content[idx..].to_string())
                } else if let Some(idx) = doc.content.find("## Implementors") {
                    Ok(doc.content[idx..].to_string())
                } else {
                    // Return the whole doc if we can't pinpoint the section,
                    // or just a message.
                    Ok(format!(
                        "Could not isolate 'Implementors' section. Here is the full doc:\n\n{}",
                        doc.content
                    ))
                }
            }
            Err(e) => Err(e),
        }
    }

    pub async fn get_context_bundle(
        &self,
        crate_name: String,
        version: Option<String>,
    ) -> Result<String, DocsFetchError> {
        // 1. Get crate metadata/README
        let overview = self
            .lookup_crate(crate_name.clone(), version.clone())
            .await?;

        // 2. Get module list
        let modules = self
            .get_crate_modules(crate_name.clone(), version.clone(), Some(20))
            .await?;

        // 3. Get feature flags
        let ver = self.resolve_version(&crate_name, version.clone()).await?;
        let features = self.analyze_feature_flags(crate_name.clone(), ver).await?;

        let bundle = format!(
            "# Context Bundle: {}\n\n## Overview\n{}\n\n## Modules\n{}\n\n## Features\n{}\n",
            crate_name, overview.content, modules, features
        );

        Ok(bundle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analyze_feature_flags_latest() {
        if std::env::var("RUN_REAL_NET_TESTS").is_err() {
            return;
        }
        let client = DocsRsClient::new();
        let result = client
            .analyze_feature_flags("serde".to_string(), "latest".to_string())
            .await;
        assert!(result.is_ok());
    }
}
