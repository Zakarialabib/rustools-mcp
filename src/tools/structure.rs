use crate::mcp::DocFetcher;
use crate::docs_parser::{CacheKey, DocContent, DocsFetchError, DocsRsParams};
use crate::cache::Cache;
use serde_json::json;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetCrateModulesArgs {
    /// Crate name
    pub crate_name: String,
    /// Version (Optional, defaults to latest)
    pub version: Option<String>,
    /// Max items per section (default: unlimited)
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReadSourceFileArgs {
    /// Crate name
    pub crate_name: String,
    /// Path to file (e.g., 'src/lib.rs')
    pub path: String,
    /// Version
    pub version: Option<String>,
    /// Start line number (1-based, optional)
    pub start_line: Option<usize>,
    /// End line number (1-based, optional)
    pub end_line: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetSymbolDocsArgs {
    /// Crate name (e.g., 'std', 'tokio')
    pub crate_name: String,
    /// Symbol path (e.g., 'vec::Vec', 'net::TcpListener::bind')
    pub symbol_path: String,
    /// Version like '1.0.0'. Omit for latest.
    pub version: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FetchRawDocArgs {
    pub crate_name: String,
    pub version: String,
    pub path: String,
}

pub async fn get_crate_modules(
    fetcher: &DocFetcher,
    crate_name: String,
    version: Option<String>,
    limit: Option<usize>,
) -> Result<DocContent, DocsFetchError> {
    let key = CacheKey::GetCrateModules {
        crate_name: crate_name.clone(),
        version: version.clone(),
        limit,
    };
    let args = json!({ "crate_name": crate_name, "version": version, "limit": limit });

    if crate_name.trim().is_empty() {
        return Err(DocsFetchError::InvalidInput("Crate name cannot be empty".to_string()));
    }

    if let Some(cached_content) = fetcher.cache.get(&key).await {
        tracing::info!("Cache hit for get_crate_modules: {}", crate_name);
        fetcher.logger
            .log(
                "get_crate_modules",
                &args,
                &Ok::<_, DocsFetchError>(&cached_content),
            )
            .await;
        return Ok(cached_content);
    }

    tracing::info!("Fetching modules for crate: {}", crate_name);
    let result = fetcher
        .client
        .get_crate_modules(crate_name, version, limit)
        .await
        .map(|content_str| DocContent {
            content: content_str,
        });

    if let Ok(content) = &result {
        fetcher.cache.insert(key, content.clone()).await;
    }

    fetcher.logger.log("get_crate_modules", &args, &result).await;
    result
}

pub async fn get_symbol_docs(
    fetcher: &DocFetcher,
    crate_name: String,
    symbol_path: String,
    version: Option<String>,
) -> Result<DocContent, DocsFetchError> {
    // Normalize path separators (accept :: or /)
    let normalized_path = symbol_path.replace("::", "/");
    let args =
        json!({ "crate_name": crate_name, "symbol_path": symbol_path, "version": version });

    if crate_name.trim().is_empty() {
        return Err(DocsFetchError::InvalidInput("Crate name cannot be empty".to_string()));
    }
    if symbol_path.trim().is_empty() {
        return Err(DocsFetchError::InvalidInput("Symbol path cannot be empty".to_string()));
    }

    let key = CacheKey::GetSymbolDocs {
        crate_name: crate_name.clone(),
        symbol_path: normalized_path.clone(),
        version: version.clone(),
    };

    if let Some(cached_content) = fetcher.cache.get(&key).await {
        tracing::info!(
            "Cache hit for get_symbol_docs: {}::{}",
            crate_name,
            normalized_path
        );
        fetcher.logger
            .log(
                "get_symbol_docs",
                &args,
                &Ok::<_, DocsFetchError>(&cached_content),
            )
            .await;
        return Ok(cached_content);
    }

    tracing::info!("Fetching symbol docs: {}::{}", crate_name, normalized_path);
    let result = fetcher
        .client
        .lookup_item(crate_name, normalized_path, version)
        .await;

    if let Ok(content) = &result {
        fetcher.cache.insert(key, content.clone()).await;
    }

    fetcher.logger.log("get_symbol_docs", &args, &result).await;
    result
}

pub async fn _fetch_raw_doc(
    fetcher: &DocFetcher,
    crate_name: String,
    version: String,
    path: String,
) -> Result<DocContent, DocsFetchError> {
    let params = DocsRsParams {
        crate_name,
        version,
        path,
    };
    let key = CacheKey::FetchRawDoc(params.clone());
    let args = json!({ "params": params });

    if let Some(cached_content) = fetcher.cache.get(&key).await {
        tracing::info!("Cache hit for _fetch_raw_doc");
        fetcher.logger
            .log(
                "_fetch_raw_doc",
                &args,
                &Ok::<_, DocsFetchError>(&cached_content),
            )
            .await;
        return Ok(cached_content);
    }

    tracing::info!("Fetching raw doc: {:?}", key);
    let result = fetcher.client.fetch_docs(params).await;

    if let Ok(content) = &result {
        fetcher.cache.insert(key, content.clone()).await;
    }

    fetcher.logger.log("_fetch_raw_doc", &args, &result).await;
    result
}

pub async fn read_source_file(
    fetcher: &DocFetcher,
    crate_name: String,
    path: String,
    version: Option<String>,
    start_line: Option<usize>,
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

    if let Some(cached_content) = fetcher.cache.get(&key).await {
        tracing::info!("Cache hit for read_source_file: {}", path);
        fetcher.logger
            .log(
                "read_source_file",
                &args,
                &Ok::<_, DocsFetchError>(&cached_content),
            )
            .await;
        return Ok(cached_content);
    }

    tracing::info!("Reading source file: {}", path);
    let result = fetcher
        .client
        .read_source_file(crate_name, path, version, start_line, end_line)
        .await;

    if let Ok(content) = &result {
        fetcher.cache.insert(key, content.clone()).await;
    }

    fetcher.logger.log("read_source_file", &args, &result).await;
    result
}
