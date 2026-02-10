use crate::mcp::DocFetcher;
use crate::docs_parser::{CacheKey, DocContent, DocsFetchError};
use crate::cache::Cache;
use serde_json::json;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FindCratesArgs {
    /// Search query (e.g., 'async runtime', 'json parser', 'web framework')
    pub query: String,
    /// Maximum results (1-100, default: 10)
    pub limit: Option<u32>,
    /// Enable fuzzy matching for better results (defaults to false)
    pub fuzzy: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetCrateOverviewArgs {
    /// Crate name (e.g., 'tokio', 'serde', 'reqwest')
    pub crate_name: String,
    /// Version like '1.0.0'. Omit for latest.
    pub version: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetContextBundleArgs {
    /// Crate name
    pub crate_name: String,
    /// Version (Optional, defaults to latest)
    pub version: Option<String>,
}

pub async fn find_crates(
    fetcher: &DocFetcher,
    query: String,
    limit: Option<u32>,
    fuzzy: Option<bool>,
) -> Result<DocContent, DocsFetchError> {
    let key = CacheKey::FindCrates {
        query: query.clone(),
        limit,
        fuzzy,
    };
    let args = json!({ "query": query, "limit": limit, "fuzzy": fuzzy });

    if query.trim().is_empty() {
        return Err(DocsFetchError::InvalidInput("Query cannot be empty".to_string()));
    }

    if let Some(cached_content) = fetcher.cache.get(&key).await {
        tracing::info!("Cache hit for find_crates: {}", query);
        fetcher.logger
            .log(
                "find_crates",
                &args,
                &Ok::<_, DocsFetchError>(&cached_content),
            )
            .await;
        return Ok(cached_content);
    }

    tracing::info!("Cache miss for find_crates: {}. Searching...", query);
    let result = fetcher
        .client
        .search_crates(query, limit, fuzzy.unwrap_or(false))
        .await
        .map(|res| DocContent { content: res });

    if let Ok(content) = &result {
        fetcher.cache.insert(key, content.clone()).await;
    }

    fetcher.logger.log("find_crates", &args, &result).await;
    result
}

pub async fn get_crate_overview(
    fetcher: &DocFetcher,
    crate_name: String,
    version: Option<String>,
) -> Result<DocContent, DocsFetchError> {
    let key = CacheKey::GetCrateOverview {
        crate_name: crate_name.clone(),
        version: version.clone(),
    };
    let args = json!({ "crate_name": crate_name, "version": version });

    if crate_name.trim().is_empty() {
        return Err(DocsFetchError::InvalidInput("Crate name cannot be empty".to_string()));
    }

    if let Some(content) = fetcher.cache.get(&key).await {
        fetcher.logger
            .log(
                "get_crate_overview",
                &args,
                &Ok::<_, DocsFetchError>(&content),
            )
            .await;
        return Ok(content);
    }

    tracing::info!(
        "Fetching overview for: {} {}",
        crate_name,
        version.as_deref().unwrap_or("latest")
    );
    let result = fetcher.client.lookup_crate(crate_name, version).await;

    if let Ok(content) = &result {
        fetcher.cache.insert(key, content.clone()).await;
    }

    fetcher.logger.log("get_crate_overview", &args, &result).await;
    result
}

pub async fn get_context_bundle(
    fetcher: &DocFetcher,
    crate_name: String,
    version: Option<String>,
) -> Result<DocContent, DocsFetchError> {
    let key = CacheKey::GetContextBundle {
        crate_name: crate_name.clone(),
        version: version.clone(),
    };
    let args = json!({ "crate_name": crate_name, "version": version });

    if let Some(content) = fetcher.cache.get(&key).await {
        fetcher.logger
            .log(
                "get_context_bundle",
                &args,
                &Ok::<_, DocsFetchError>(&content),
            )
            .await;
        return Ok(content);
    }

    tracing::info!(
        "Fetching context bundle for: {} {}",
        crate_name,
        version.as_deref().unwrap_or("latest")
    );
    let result = fetcher
        .client
        .get_context_bundle(crate_name, version)
        .await
        .map(|content_str| DocContent {
            content: content_str,
        });

    if let Ok(content) = &result {
        fetcher.cache.insert(key, content.clone()).await;
    }

    fetcher.logger.log("get_context_bundle", &args, &result).await;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::InMemoryCache;
    use std::path::PathBuf;

    async fn create_test_fetcher() -> DocFetcher {
        let cache = InMemoryCache::new(PathBuf::from("test_cache"));
        DocFetcher::new(cache)
    }

    #[tokio::test]
    async fn test_find_crates_empty_query() {
        let fetcher = create_test_fetcher().await;
        let result = find_crates(&fetcher, "".to_string(), None, None).await;
        assert!(matches!(result, Err(DocsFetchError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn test_get_crate_overview_empty_name() {
        let fetcher = create_test_fetcher().await;
        let result = get_crate_overview(&fetcher, "".to_string(), None).await;
        assert!(matches!(result, Err(DocsFetchError::InvalidInput(_))));
    }
}
