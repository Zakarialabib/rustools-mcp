//! Rust documentation fetcher MCP implementation.
//! 
//! This module provides functionality to fetch and cache Rust documentation from docs.rs.
//! It implements the MCP (Machine Control Protocol) server interface to expose documentation
//! fetching capabilities as a service.
//!
//! # LLM Usage Workflow
//! 
//! ## Decision Tree (USE THIS ORDER):
//! 
//! 1. **Finding crates** (Don't know exact name?)
//!    → Use `find_crates("async runtime")`
//!    
//! 2. **Understanding a crate** (Have crate name, need overview?)
//!    → Use `get_crate_overview("tokio")`
//!    
//! 3. **Deep dive into API** (Need specific function/struct/trait docs?)
//!    → Use `get_symbol_docs` with path like `"tokio::net::TcpListener::bind"`
//!    
//! 4. **Raw HTML access** (Advanced/debugging only)
//!    → Use `_fetch_raw_doc` (prefixed with _ to indicate internal use)

use rmcp::model::{Implementation, ListPromptsResult, PaginatedRequestParam, ProtocolVersion, ServerCapabilities};
use rmcp::service::RequestContext;
use rmcp::{RoleServer, Error as McpError, ServerHandler, tool};
use rmcp::model::{Content, IntoContents, ServerInfo};
use std::sync::Arc;
use serde_json::json;
// use rmcp::schemars::JsonSchema;

use crate::cache::{Cache, InMemoryCache};
use crate::docs_parser::{DocsRsClient, DocsRsParams, DocContent, DocsFetchError, CacheKey};
use crate::logger::RequestLogger;
use crate::rust_book::RustBookClient;
use crate::error_index::ErrorIndexClient;

/// Implements conversion from DocContent to MCP Contents.
impl IntoContents for DocContent {
    fn into_contents(self) -> Vec<Content> {
        vec![Content::text(self.content)]
    }
}

/// Implements conversion from DocsFetchError to MCP Contents.
impl IntoContents for DocsFetchError {
    fn into_contents(self) -> Vec<Content> {
        vec![Content::text(self.to_string())]
    }
}

/// Main struct responsible for fetching and caching Rust documentation.
/// 
/// `DocFetcher` provides functionality to fetch documentation from docs.rs
/// and caches the results in memory for faster subsequent access.
/// 
/// # Tool Workflow
/// 
/// **Step 1 - Discovery**: `find_crates` → Returns list of crates  
/// **Step 2 - Overview**: `get_crate_overview` → Returns README + module structure  
/// **Step 3 - Details**: `get_symbol_docs` → Returns specific API documentation  
/// **_Internal**: `_fetch_raw_doc` → Low-level raw HTML fetch (avoid unless debugging)
#[derive(Clone)]
pub struct DocFetcher {
    /// In-memory cache for storing fetched documentation
    cache: Arc<InMemoryCache>,
    /// Shared HTTP client
    client: DocsRsClient,
    /// Request logger
    logger: Arc<RequestLogger>,
    /// Client for the Rust Book
    book_client: RustBookClient,
    /// Client for the Rust Error Index
    error_client: ErrorIndexClient,
}

#[tool(tool_box)]
impl DocFetcher {
    /// Creates a new `DocFetcher` instance with the provided cache.
    pub fn new(cache: Arc<InMemoryCache>) -> Self {
        Self { 
            cache,
            client: DocsRsClient::new(),
            logger: Arc::new(RequestLogger::new("requests.log")),
            book_client: RustBookClient::new(),
            error_client: ErrorIndexClient::new(),
        }
    }

    /// **STEP 1: DISCOVERY** - Find crates matching your keywords.
    /// 
    /// Use this when you DON'T know the exact crate name yet.
    /// After finding a crate here, proceed to `get_crate_overview`.
    /// 
    /// # When NOT to use
    /// - If you already know the exact crate name (e.g., "serde_json")
    /// - If you need specific function documentation (use `get_symbol_docs` instead)
    #[tool(description = 
r#"STEP 1 - DISCOVERY: Find crates on crates.io matching a query.

USE THIS WHEN: You don't know the exact crate name yet and need to discover options.
EXAMPLE: "Find http client crates" or "Search for json serialization"

NEXT STEP: After identifying a crate here, use `get_crate_overview` to see its README.

WHEN NOT TO USE: 
- If you already know the exact name (e.g., "tokio"), skip to `get_crate_overview`
- Never use for specific functions/types (that's `get_symbol_docs`)"#)]
    pub async fn find_crates(
        &self,
        #[tool(param)]
        #[schemars(description = "Search query (e.g., 'async runtime', 'json parser', 'web framework')")]
        query: String,
        
        #[tool(param)]
        #[schemars(description = "Maximum results (1-100, default: 10)")]
        limit: Option<u32>,
    ) -> Result<DocContent, DocsFetchError> {
        let key = CacheKey::FindCrates { query: query.clone(), limit };
        let args = json!({ "query": query, "limit": limit });

        if let Some(cached_content) = self.cache.get(&key).await {
            tracing::info!("Cache hit for find_crates: {}", query);
            self.logger.log("find_crates", &args, &Ok::<_, DocsFetchError>(&cached_content)).await;
            return Ok(cached_content);
        }

        tracing::info!("Cache miss for find_crates: {}. Searching...", query);
        let result = self.client.search_crates(query, limit).await.map(|res| DocContent { content: res });
        
        if let Ok(content) = &result {
             self.cache.insert(key, content.clone()).await;
        }
        
        self.logger.log("find_crates", &args, &result).await;
        result
    }

    /// **STEP 2 (Alternative): OVERVIEW** - Get the main landing page for a crate.
    /// 
    /// Use this to understand what a crate does, see its README, and explore
    /// the top-level module structure. This is your starting point for any
    /// crate you want to use.
    /// 
    /// # When NOT to use  
    /// - If you haven't identified the crate yet (use `find_crates` first)
    /// - If you need docs for a specific type/function (use `get_symbol_docs`)
    /// - If you just want to see available modules/exports (use `get_crate_modules`)
    #[tool(description = 
r#"STEP 2 (Alternative) - OVERVIEW: Get ONLY the main documentation (README) for a crate.

USE THIS WHEN: You specifically want only the description/README and don't need modules or features.
PREFER: `get_context_bundle` for most cases as it provides more context in one go.

EXAMPLE: "How do I use serde?" or "Show me the tokio crate overview"

RETURNS: README, description.

NEXT STEP: 
- Use `get_crate_modules` to navigate the crate structure.
- Use `get_symbol_docs` for specific APIs.

WHEN NOT TO USE:
- If you want a full picture of the crate (use `get_context_bundle` instead)
- If you only have a vague idea of what you need (use `find_crates` first)
- For specific function signatures like `Vec::push` (use `get_symbol_docs`)"#)]
    pub async fn get_crate_overview(
        &self,
        #[tool(param)]
        #[schemars(description = "Crate name (e.g., 'tokio', 'serde', 'reqwest')")]
        crate_name: String,
        
        #[tool(param)]
        #[schemars(description = "Version like '1.0.0'. Omit for latest.")]
        version: Option<String>,
    ) -> Result<DocContent, DocsFetchError> {
        let key = CacheKey::GetCrateOverview { crate_name: crate_name.clone(), version: version.clone() };
        let args = json!({ "crate_name": crate_name, "version": version });

        if let Some(content) = self.cache.get(&key).await {
            self.logger.log("get_crate_overview", &args, &Ok::<_, DocsFetchError>(&content)).await;
            return Ok(content);
        }

        tracing::info!("Fetching overview for: {} {}", crate_name, version.as_deref().unwrap_or("latest"));
        let result = self.client.lookup_crate(crate_name, version).await;
        
        if let Ok(content) = &result {
            self.cache.insert(key, content.clone()).await;
        }

        self.logger.log("get_crate_overview", &args, &result).await;
        result
    }

    /// **STEP 2.5: STRUCTURE** - List modules, structs, enums, traits, and re-exports.
    /// 
    /// Use this to explore the crate structure without loading the full README.
    /// Useful for navigation and discovering what's available.
    #[tool(description = 
r#"STEP 2.5 - STRUCTURE: List modules, structs, enums, traits, and re-exports.

USE THIS WHEN: You want to explore the API surface after seeing the overview, or if `get_context_bundle` didn't provide enough depth.
EXAMPLE: "What modules are in tokio?" or "List exports of serde"

RETURNS: List of modules and items in the crate root.

NEXT STEP: Use `get_symbol_docs` to drill down into specific items."#)]
    pub async fn get_crate_modules(
        &self,
        #[tool(param)]
        #[schemars(description = "Crate name")]
        crate_name: String,
        
        #[tool(param)]
        #[schemars(description = "Version")]
        version: Option<String>,

        #[tool(param)]
        #[schemars(description = "Max items per section (default: unlimited). Use this for large crates.")]
        limit: Option<usize>,
    ) -> Result<DocContent, DocsFetchError> {
        let key = CacheKey::GetCrateModules { crate_name: crate_name.clone(), version: version.clone(), limit };
        let args = json!({ "crate_name": crate_name, "version": version, "limit": limit });

        if let Some(cached_content) = self.cache.get(&key).await {
            tracing::info!("Cache hit for get_crate_modules: {}", crate_name);
            self.logger.log("get_crate_modules", &args, &Ok::<_, DocsFetchError>(&cached_content)).await;
            return Ok(cached_content);
        }

        tracing::info!("Fetching modules for crate: {}", crate_name);
        let result = self.client.extract_modules(crate_name, version, limit).await;
        
        if let Ok(content) = &result {
            self.cache.insert(key, content.clone()).await;
        }

        self.logger.log("get_crate_modules", &args, &result).await;
        result
    }

    /// **STEP 3: DETAILS** - Get documentation for a specific symbol.
    /// 
    /// Use this for structs, enums, functions, traits, methods, etc.
    /// You MUST provide a fully qualified path or the symbol name.
    /// 
    /// # Path formats
    /// - `std::vec::Vec` (full path - PREFERRED)
    /// - `Vec::push` (method)
    /// - `tokio::net::TcpListener::bind` (function in module)
    /// 
    /// # When NOT to use
    /// - If you don't know the crate yet (use `find_crates`)
    /// - If you need the crate's general purpose (use `get_crate_overview`)
    /// - If you only have a search term, not a symbol path
    #[tool(description = 
r#"STEP 3 - DETAILS: Get docs for a specific symbol (struct, fn, trait, enum).

USE THIS WHEN: You need API docs for a specific type or function.
EXAMPLES: 
  - "Explain std::vec::Vec::push"
  - "What does tokio::net::TcpListener do?"
  - "Show me Option::map"

REQUIRES: Full path like "crate::module::Item::method" (Preferred) or "Item::method"

WHEN NOT TO USE:
- If you don't know the crate name yet → use `find_crates`
- If you want the crate's README/overview → use `get_crate_overview`
- For vague searches like "something for json" → use `find_crates`"#)]
    pub async fn get_symbol_docs(
        &self,
        #[tool(param)]
        #[schemars(description = "Crate name (e.g., 'std', 'tokio', 'serde')")]
        crate_name: String,
        
        #[tool(param)]
        #[schemars(description = "Symbol path (e.g., 'vec::Vec', 'net::TcpListener::bind', 'Option::map')")]
        symbol_path: String,
        
        #[tool(param)]
        #[schemars(description = "Version like '1.0.0'. Omit for latest.")]
        version: Option<String>,
    ) -> Result<DocContent, DocsFetchError> {
        // Normalize path separators (accept :: or /)
        let normalized_path = symbol_path.replace("::", "/");
        let args = json!({ "crate_name": crate_name, "symbol_path": symbol_path, "version": version });
        
        let key = CacheKey::GetSymbolDocs { 
            crate_name: crate_name.clone(), 
            symbol_path: normalized_path.clone(), 
            version: version.clone() 
        };

        if let Some(cached_content) = self.cache.get(&key).await {
            tracing::info!("Cache hit for get_symbol_docs: {}::{}", crate_name, normalized_path);
            self.logger.log("get_symbol_docs", &args, &Ok::<_, DocsFetchError>(&cached_content)).await;
            return Ok(cached_content);
        }

        tracing::info!("Fetching symbol docs: {}::{}", crate_name, normalized_path);
        let result = self.client.lookup_item(crate_name, normalized_path, version).await;
        
        if let Ok(content) = &result {
            self.cache.insert(key, content.clone()).await;
        }

        self.logger.log("get_symbol_docs", &args, &result).await;
        result
    }

    /// **INTERNAL**: Fetch raw HTML from a specific docs.rs path.
    /// 
    /// ⚠️ This is prefixed with `_` to indicate it's for internal/advanced use.
    /// 
    /// Use only when you need raw HTML for debugging or when building
    /// custom documentation parsers. Prefer `get_symbol_docs` for normal use.
    #[tool(description = 
r#"[INTERNAL] Fetch raw HTML documentation from a specific path.

⚠️ ADVANCED USE ONLY - Prefer `get_symbol_docs` for normal queries.

USE THIS WHEN: You need raw HTML for debugging or specific parsing needs.
EXAMPLE: "_fetch_raw_doc('tokio', '1.0', 'tokio/net/struct.TcpListener.html')"

WHEN NOT TO USE:
- For normal documentation reading (use `get_symbol_docs` instead)
- If you don't know the exact HTML path structure"#)]
    pub async fn _fetch_raw_doc(
        &self,
        #[tool(param)]
        #[schemars(description = "Crate name")]
        crate_name: String,

        #[tool(param)]
        #[schemars(description = "Version like '1.0.0'")]
        version: String,

        #[tool(param)]
        #[schemars(description = "HTML path (e.g., 'tokio/net/struct.TcpListener.html')")]
        path: String,
    ) -> Result<DocContent, DocsFetchError> {
        let params = DocsRsParams {
            crate_name,
            version,
            path,
        };
        let key = CacheKey::FetchRawDoc(params.clone());
        let args = json!({ "params": params });

        if let Some(cached_content) = self.cache.get(&key).await {
            tracing::info!("Cache hit for _fetch_raw_doc");
            self.logger.log("_fetch_raw_doc", &args, &Ok::<_, DocsFetchError>(&cached_content)).await;
            return Ok(cached_content);
        }
        
        tracing::info!("Fetching raw doc: {:?}", key);
        let result = self.client.fetch_docs(params).await;
        
        if let Ok(content) = &result {
            self.cache.insert(key, content.clone()).await;
        }

        self.logger.log("_fetch_raw_doc", &args, &result).await;
        result
    }

    /// **STEP 3 (Deep Dive): SOURCE** - Read raw source code from a file.
    #[tool(description = 
r#"STEP 3 (Deep Dive) - SOURCE: Read raw source code from a file in the crate.

USE THIS WHEN: 
- You need to see the implementation details (how it works under the hood).
- You want to see full examples that aren't in the docs.
- You need to debug a macro or complex trait behavior.

EXAMPLE: "read_source_file('tokio', 'net/tcp/listener.rs')"

RETURNS: The raw source code text."#)]
    pub async fn read_source_file(
        &self,
        #[tool(param)]
        #[schemars(description = "Crate name")]
        crate_name: String,
        
        #[tool(param)]
        #[schemars(description = "Path to file (e.g., 'src/lib.rs', 'examples/echo.rs')")]
        path: String,
        
        #[tool(param)]
        #[schemars(description = "Version")]
        version: Option<String>,

        #[tool(param)]
        #[schemars(description = "Start line number (1-based, optional)")]
        start_line: Option<usize>,

        #[tool(param)]
        #[schemars(description = "End line number (1-based, optional)")]
        end_line: Option<usize>,
    ) -> Result<DocContent, DocsFetchError> {
        let key = CacheKey::ReadSourceFile { 
            crate_name: crate_name.clone(), 
            path: path.clone(), 
            version: version.clone(),
            start_line,
            end_line,
        };
        let args = json!({ 
            "crate_name": crate_name, "path": path, "version": version, 
            "start_line": start_line, "end_line": end_line 
        });

        if let Some(cached_content) = self.cache.get(&key).await {
            tracing::info!("Cache hit for read_source_file: {}", path);
            self.logger.log("read_source_file", &args, &Ok::<_, DocsFetchError>(&cached_content)).await;
            return Ok(cached_content);
        }

        tracing::info!("Reading source file: {}", path);
        let result = self.client.read_source_file(crate_name, path, version, start_line, end_line).await;
        
        if let Ok(content) = &result {
            self.cache.insert(key, content.clone()).await;
        }

        self.logger.log("read_source_file", &args, &result).await;
        result
    }

    /// **DEPENDENCIES**: Get dependencies of a crate.
    #[tool(description = 
r#"Get dependencies of a crate from crates.io.

USE THIS WHEN: You want to know what a crate depends on (heavy vs light).
EXAMPLE: "get_crate_dependencies('tokio', '1.0.0')"

RETURNS: A table of dependencies."#)]
    pub async fn get_crate_dependencies(
        &self,
        #[tool(param)]
        #[schemars(description = "Crate name")]
        crate_name: String,
        
        #[tool(param)]
        #[schemars(description = "Version (Required)")]
        version: String,

        #[tool(param)]
        #[schemars(description = "Dependency kind filter (dev, build, normal). If omitted, shows all.")]
        kind: Option<String>,
    ) -> Result<DocContent, DocsFetchError> {
        let key = CacheKey::GetCrateDependencies { 
            crate_name: crate_name.clone(), 
            version: version.clone(),
            kind: kind.clone(),
        };
        let args = json!({ "crate_name": crate_name, "version": version, "kind": kind });

        if let Some(cached_content) = self.cache.get(&key).await {
            tracing::info!("Cache hit for get_crate_dependencies");
            self.logger.log("get_crate_dependencies", &args, &Ok::<_, DocsFetchError>(&cached_content)).await;
            return Ok(cached_content);
        }

        tracing::info!("Fetching dependencies for: {}", crate_name);
        let result = self.client.get_crate_dependencies(crate_name, version, kind).await.map(|content_str| DocContent { content: content_str });
        
        if let Ok(content) = &result {
            self.cache.insert(key, content.clone()).await;
        }

        self.logger.log("get_crate_dependencies", &args, &result).await;
        result
    }

    /// **EXAMPLES**: List examples.
    #[tool(description = 
r#"List examples from the crate's source or documentation.

USE THIS WHEN: You want to see code examples provided by the crate authors.
EXAMPLE: "get_crate_examples('tokio')"

RETURNS: A list of example files or extracted code blocks."#)]
    pub async fn get_crate_examples(
        &self,
        #[tool(param)]
        #[schemars(description = "Crate name")]
        crate_name: String,
        
        #[tool(param)]
        #[schemars(description = "Version")]
        version: Option<String>,

        #[tool(param)]
        #[schemars(description = "Max number of examples (default: 5). Use this to reduce tokens.")]
        limit: Option<usize>,
    ) -> Result<DocContent, DocsFetchError> {
        let key = CacheKey::GetCrateExamples { 
            crate_name: crate_name.clone(), 
            version: version.clone(),
            limit,
        };
        let args = json!({ "crate_name": crate_name, "version": version, "limit": limit });

        if let Some(cached_content) = self.cache.get(&key).await {
            tracing::info!("Cache hit for get_crate_examples");
            self.logger.log("get_crate_examples", &args, &Ok::<_, DocsFetchError>(&cached_content)).await;
            return Ok(cached_content);
        }

        tracing::info!("Fetching examples for crate: {}", crate_name);
        let result = self.client.get_crate_examples(crate_name, version, limit).await.map(|content_str| DocContent { content: content_str });
        
        if let Ok(content) = &result {
            self.cache.insert(key, content.clone()).await;
        }

        self.logger.log("get_crate_examples", &args, &result).await;
        result
    }

    /// **FEATURES**: Analyze feature flags.
    #[tool(description = r#"
Analyze feature flags for a crate. Essential for Rust development.

USE WHEN: You see cfg(feature = "...") in code, need to know 
which features enable specific modules, or debugging "module/function not found" errors.

EXAMPLE: "What features does tokio need for full async runtime?" or "analyze_feature_flags('tokio', '1.0.0')"

RETURNS: A table of features, their default status, and what they enable.
"#)]
    pub async fn analyze_feature_flags(
        &self,
        #[tool(param)]
        #[schemars(description = "Crate name")]
        crate_name: String,
        
        #[tool(param)]
        #[schemars(description = "Version (Required)")]
        version: String,
    ) -> Result<DocContent, DocsFetchError> {
        let key = CacheKey::AnalyzeFeatureFlags {
            crate_name: crate_name.clone(),
            version: version.clone(),
        };
        let args = json!({ "crate_name": crate_name, "version": version });

        if let Some(content) = self.cache.get(&key).await {
            self.logger.log("analyze_feature_flags", &args, &Ok::<_, DocsFetchError>(&content)).await;
            return Ok(content);
        }

        tracing::info!("Analyzing feature flags for: {} {}", crate_name, version);
        let result = self.client.analyze_feature_flags(crate_name, version).await.map(|content_str| DocContent { content: content_str });
        
        if let Ok(content) = &result {
            self.cache.insert(key, content.clone()).await;
        }

        self.logger.log("analyze_feature_flags", &args, &result).await;
        result
    }

    /// **TRAITS**: Find all types implementing a specific trait.
    #[tool(description = r#"
Find all types implementing a specific trait in a crate or dependency tree.

USE WHEN: You need to know "what types can I use here?" or 
finding implementations of Display, Serialize, FromStr, etc.

EXAMPLE: find_trait_implementors("serde", "Serialize", "1.0")

RETURNS: A list of direct, auto, and blanket implementations, cleaned of HTML artifacts.
"#)]
    pub async fn find_trait_implementors(
        &self,
        #[tool(param)]
        #[schemars(description = "Crate name")]
        crate_name: String,
        
        #[tool(param)]
        #[schemars(description = "Trait path (e.g., 'Serialize' or 'serde::Serialize')")]
        trait_path: String,

        #[tool(param)]
        #[schemars(description = "Version (Optional, defaults to latest)")]
        version: Option<String>,

        #[tool(param)]
        #[schemars(description = "Max results (default: 30). Use this to reduce tokens.")]
        limit: Option<usize>,
    ) -> Result<DocContent, DocsFetchError> {
        let key = CacheKey::FindTraitImplementors {
            crate_name: crate_name.clone(),
            trait_path: trait_path.clone(),
            version: version.clone(),
            limit,
        };
        let args = json!({ "crate_name": crate_name, "trait_path": trait_path, "version": version, "limit": limit });

        if let Some(content) = self.cache.get(&key).await {
            self.logger.log("find_trait_implementors", &args, &Ok::<_, DocsFetchError>(&content)).await;
            return Ok(content);
        }

        tracing::info!("Finding trait implementors for: {} in {}", trait_path, crate_name);
        let result = self.client.find_trait_implementors(crate_name, trait_path, version, limit).await.map(|content_str| DocContent { content: content_str });
        
        if let Ok(content) = &result {
            self.cache.insert(key, content.clone()).await;
        }

        self.logger.log("find_trait_implementors", &args, &result).await;
        result
    }

    /// **STEP 2: ANALYSIS (Recommended)** - Get a complete context bundle for a crate.
    #[tool(description = r#"
STEP 2 - ANALYSIS (Recommended): Get a complete context bundle for a crate.
Combines README, Modules, and Feature Flags in one call.

USE THIS WHEN: You first encounter a crate. It replaces `get_crate_overview` and `get_crate_modules` for the initial look.
Reduces token waste and round trips.

EXAMPLE: "get_context_bundle('tokio', '1.0.0')"

RETURNS: A structured markdown report including Overview (README snippet), Module tree skeleton, and Feature Flags.
"#)]
    pub async fn get_context_bundle(
        &self,
        #[tool(param)]
        #[schemars(description = "Crate name")]
        crate_name: String,
        
        #[tool(param)]
        #[schemars(description = "Version (Optional, defaults to latest)")]
        version: Option<String>,
    ) -> Result<DocContent, DocsFetchError> {
        let key = CacheKey::GetContextBundle {
            crate_name: crate_name.clone(),
            version: version.clone(),
        };
        let args = json!({ "crate_name": crate_name, "version": version });

        if let Some(content) = self.cache.get(&key).await {
            self.logger.log("get_context_bundle", &args, &Ok::<_, DocsFetchError>(&content)).await;
            return Ok(content);
        }

        tracing::info!("Fetching context bundle for: {} {}", crate_name, version.as_deref().unwrap_or("latest"));
        let result = self.client.get_context_bundle(crate_name, version).await.map(|content_str| DocContent { content: content_str });
        
        if let Ok(content) = &result {
            self.cache.insert(key, content.clone()).await;
        }

        self.logger.log("get_context_bundle", &args, &result).await;
        result
    }

    /// **SIGNATURE**: Find items by type signature.
    #[tool(description = r#"
Semantic search: Find functions or methods matching a specific type signature pattern.

USE WHEN: You know the type transformation you need but don't know the name.
EXAMPLE: find_by_signature("std", "fn(&str) -> Result<_, _>")

RETURNS: A list of matching items or a link to the search results if JS is required.
"#)]
    pub async fn find_by_signature(
        &self,
        #[tool(param)]
        #[schemars(description = "Crate name")]
        crate_name: String,
        
        #[tool(param)]
        #[schemars(description = "Signature pattern (e.g., 'fn(&str) -> Result')")]
        signature_pattern: String,
        
        #[tool(param)]
        #[schemars(description = "Version (Optional, defaults to latest)")]
        version: Option<String>,
    ) -> Result<DocContent, DocsFetchError> {
        let key = CacheKey::FindBySignature {
            crate_name: crate_name.clone(),
            signature_pattern: signature_pattern.clone(),
            version: version.clone(),
        };
        let args = json!({ "crate_name": crate_name, "signature_pattern": signature_pattern, "version": version });

        if let Some(content) = self.cache.get(&key).await {
            self.logger.log("find_by_signature", &args, &Ok::<_, DocsFetchError>(&content)).await;
            return Ok(content);
        }

        tracing::info!("Searching by signature: {} in {}", signature_pattern, crate_name);
        let result = self.client.find_by_signature(crate_name, signature_pattern, version).await.map(|content_str| DocContent { content: content_str });
        
        if let Ok(content) = &result {
            self.cache.insert(key, content.clone()).await;
        }

        self.logger.log("find_by_signature", &args, &result).await;
        result
    }

    /// **LANGUAGE CONCEPT**: Get conceptual explanation from the Rust Book.
    #[tool(description = 
r#"Get a conceptual explanation from The Rust Programming Language (the Book).

USE THIS WHEN: You need to understand a core language concept.
EXAMPLE: "ownership", "borrowing", "lifetimes", "traits", "async await"

RETURNS: The content of the relevant chapter from the Rust Book."#)]
    pub async fn get_language_concept(
        &self,
        #[tool(param)]
        #[schemars(description = "The concept to search for (e.g., 'ownership', 'smart pointers')")]
        concept: String,
    ) -> Result<DocContent, DocsFetchError> {
        let args = json!({ "concept": concept });
        self.logger.log("get_language_concept", &args, &"fetching...").await;
        
        match self.book_client.search_concept(&concept).await {
            Ok(Some(chapter)) => {
                 match self.book_client.get_chapter_content(&chapter.url).await {
                    Ok(content) => {
                        let result = DocContent {
                            content: format!("# {}\n\n{}", chapter.title, content),
                        };
                         self.logger.log("get_language_concept", &args, &Ok::<_, DocsFetchError>(&result)).await;
                        Ok(result)
                    },
                    Err(e) => {
                         let err = DocsFetchError::RequestError(format!("Failed to fetch chapter content: {}", e));
                         self.logger.log("get_language_concept", &args, &Err::<DocContent, _>(&err)).await;
                         Err(err)
                    }
                 }
            },
            Ok(None) => {
                 let err = DocsFetchError::ItemNotFound(format!("Concept '{}' not found in the Rust Book", concept));
                 self.logger.log("get_language_concept", &args, &Err::<DocContent, _>(&err)).await;
                 Err(err)
            },
            Err(e) => {
                let err = DocsFetchError::RequestError(format!("Failed to search book: {}", e));
                self.logger.log("get_language_concept", &args, &Err::<DocContent, _>(&err)).await;
                Err(err)
            }
        }
    }

    /// **ERROR CODE**: Explain a Rust compiler error code.
    #[tool(description = 
r#"Get a detailed explanation for a Rust compiler error code (e.g., E0382).

USE THIS WHEN: You encounter a compiler error and need to understand why it happened and how to fix it.
EXAMPLE: "E0382", "E0507"

RETURNS: The explanation from the official Rust error index."#)]
    pub async fn explain_error_code(
        &self,
        #[tool(param)]
        #[schemars(description = "The error code (e.g., 'E0382')")]
        code: String,
    ) -> Result<DocContent, DocsFetchError> {
        let code = code.trim().to_uppercase();
        let args = json!({ "code": code });
        self.logger.log("explain_error_code", &args, &"fetching...").await;

        match self.error_client.get_error_explanation(&code).await {
            Ok(content) => {
                let result = DocContent { content };
                self.logger.log("explain_error_code", &args, &Ok::<_, DocsFetchError>(&result)).await;
                Ok(result)
            }
            Err(e) => {
                let err = DocsFetchError::RequestError(format!("Failed to fetch error explanation: {}", e));
                self.logger.log("explain_error_code", &args, &Err::<DocContent, _>(&err)).await;
                Err(err)
            }
        }
    }

    /// Utility: Get workflow guidance for LLMs.
    /// 
    /// Returns text explaining which tool to use in which situation.
    #[tool(description = 
r#"Get help on which tool to use for your documentation needs.

USE THIS IF: You're confused about whether to use find_crates vs get_crate_overview vs get_symbol_docs.

Returns a decision tree guide."#)]
    pub async fn doc_workflow_help(&self) -> Result<DocContent, DocsFetchError> {
        let help_text = r#"
🧭 RUST DOC MCP - WORKFLOW GUIDE

Follow this decision tree:

1️⃣ NEW CRATE? (Start Here)
   → Use: get_context_bundle("crate") (Replaces get_crate_overview + get_crate_modules)
   → Use: find_crates("query") if you're not sure which crate to use.
   → RETURNS: Bundle of README, Modules, and Feature Flags.
   
2️⃣ NEED TO FIND AN API BY TYPE?
   → Use: find_by_signature("crate", "fn(u32) -> String")
   → Use: find_trait_implementors("crate", "TraitName")
   
3️⃣ EXPLORE STRUCTURE?
   → Use: get_crate_modules("crate_name")
   → Example: get_crate_modules("tokio")
   → Returns: List of modules, structs, enums (API surface)
   
4️⃣ NEED SPECIFIC API DETAILS?
   → Use: get_symbol_docs("crate", "path::to::Symbol")
   → Example: get_symbol_docs("reqwest", "Client::get")
   → Example: get_symbol_docs("std", "vec::Vec::push")
   
5️⃣ LANGUAGE CONCEPTS & SPECS?
   → Use: get_language_concept("concept") (e.g., "ownership", "lifetimes")
   → Use: web_search site:doc.rust-lang.org/reference (for "formal grammar", "memory model")
   → Use: explain_error_code("E0382")

6️⃣ DEEP DIVE?
   → Use: get_crate_dependencies("crate", "version")
   → Use: get_crate_examples("crate")
   → Use: analyze_feature_flags("crate", "version")
   → Use: read_source_file("crate", "path/to/file.rs")
   
7️⃣ DEBUGGING/RAW HTML/SOURCE?
   → Use: read_source_file("crate", "path/to/file.rs")
   → Use: expand_macro("path/to/file.rs") (debug macros)
   → Use: _fetch_raw_doc (advanced only)

❌ COMMON MISTAKES TO AVOID:
- Don't guess function names. Use get_crate_modules first.
- Don't read entire files if you just need a function signature.
- Don't forget to check feature flags if code fails to compile.
"#;
        let content = DocContent { content: help_text.to_string() };
        self.logger.log("doc_workflow_help", &json!({}), &Ok::<_, DocsFetchError>(&content)).await;
        Ok(content)
    }

    /// **MACRO EXPANSION**: Expand macros to see generated code.
    #[tool(description = 
r#"Expands Rust macros (like #[derive], lazy_static, tokio::main) to show the actual generated code.

USE WHEN: You need to debug macro behavior or see hidden trait implementations.
REQUIRES: 'cargo-expand' installed locally."#)]
    pub async fn expand_macro(
        &self,
        #[tool(param)]
        #[schemars(description = "File path or module path to expand")]
        path: String,
        
        #[tool(param)]
        #[schemars(description = "Specific item within the file to expand (optional)")]
        item: Option<String>,
    ) -> Result<DocContent, DocsFetchError> {
        let args = json!({ "path": path, "item": item });
        self.logger.log("expand_macro", &args, &"running...").await;

        match crate::cargo_tools::run_cargo_expand(path, item).await {
            Ok(content) => {
                let result = DocContent { content };
                self.logger.log("expand_macro", &args, &Ok::<_, DocsFetchError>(&result)).await;
                Ok(result)
            }
            Err(e) => {
                let err = DocsFetchError::RequestError(format!("Failed to expand macro: {}", e));
                self.logger.log("expand_macro", &args, &Err::<DocContent, _>(&err)).await;
                Err(err)
            }
        }
    }
}

#[tool(tool_box)]
impl ServerHandler for DocFetcher {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::default(),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
r#"Rust Documentation MCP Server - Ecosystem & Language Intelligence

📋 THE HOLY TRINITY WORKFLOW (Use in this order):
1. DISCOVERY: find_crates("query") or get_language_concept("concept")
2. ANALYSIS: get_context_bundle("crate") (Replaces get_crate_overview + get_crate_modules)
3. VALIDATION: get_symbol_docs("path") or read_source_file("path")

🗺️ RESOURCE MAP:
- Language Concepts: get_language_concept("topic")
- Language Spec: web_search site:doc.rust-lang.org/reference
- Crate Docs: get_symbol_docs, get_context_bundle
- Compiler Errors: explain_error_code("code")
- Macro Debugging: expand_macro("path")

💡 TIP: Use analyze_feature_flags if code fails to compile.
This server caches results for performance."#.to_string()
            ),
        }
    }

    async fn list_prompts(
        &self,
        _request: PaginatedRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        Ok(ListPromptsResult {
            next_cursor: None,
            prompts: vec![],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::time::Instant;
    use std::fs;

    fn setup_test_fetcher() -> (DocFetcher, Arc<InMemoryCache>) {
        let temp_dir = tempdir().unwrap();
        let cache_dir = temp_dir.path().to_path_buf();
        fs::create_dir_all(&cache_dir).unwrap(); 

        let cache = Arc::new(InMemoryCache::new(cache_dir));
        let fetcher = DocFetcher::new(cache.clone());
        (fetcher, cache)
    }

    #[tokio::test]
    async fn test_find_crates() {
        let (fetcher, _cache) = setup_test_fetcher();
        let result = fetcher.find_crates(
            "serialization".to_string(),
            Some(5),
        ).await.unwrap();

        assert!(!result.content.is_empty());
        assert!(result.content.contains("serde") || result.content.contains("serde_json"));
    }

    #[tokio::test]
    async fn test_get_crate_overview() {
        let (fetcher, _cache) = setup_test_fetcher();
        let result = fetcher.get_crate_overview(
            "rand".to_string(),
            Some("0.9.0".to_string()),
        ).await.unwrap();

        assert!(!result.content.is_empty());
        assert!(result.content.contains("Random number generators"));
    }

    #[tokio::test]
    async fn test_get_symbol_docs() {
        let (fetcher, _cache) = setup_test_fetcher();
        // Test with both :: and / separators
        let result = fetcher.get_symbol_docs(
            "rand".to_string(),
            "Rng::gen".to_string(),  // Using :: separator
            Some("0.9.0".to_string()),
        ).await.unwrap();

        assert!(!result.content.is_empty());
    }

    #[tokio::test]
    async fn test_workflow_decision_tree() {
        let (fetcher, _cache) = setup_test_fetcher();
        
        // Simulate typical workflow:
        // 1. Find crates
        let search = fetcher.find_crates("random number".to_string(), Some(3)).await;
        assert!(search.is_ok());
        
        // 2. Get overview of found crate  
        let overview = fetcher.get_crate_overview("rand".to_string(), None).await;
        assert!(overview.is_ok());
        
        // 3. Get specific symbol
        let symbol = fetcher.get_symbol_docs("rand".to_string(), "thread_rng".to_string(), None).await;
        assert!(symbol.is_ok());
    }

    #[tokio::test]
    async fn test_cache_efficiency() {
        let (fetcher, _cache) = setup_test_fetcher();
        
        // First call - cache miss
        let start1 = Instant::now();
        fetcher.get_crate_overview("serde".to_string(), None).await.unwrap();
        let duration1 = start1.elapsed();
        
        // Second call - cache hit
        let start2 = Instant::now();
        fetcher.get_crate_overview("serde".to_string(), None).await.unwrap();
        let duration2 = start2.elapsed();
        
        assert!(duration2 < duration1 / 10 || duration2.as_millis() < 5, 
                "Cache hit was not faster than cache miss");
    }
}