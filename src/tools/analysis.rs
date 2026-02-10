use crate::mcp::DocFetcher;
use crate::docs_parser::{CacheKey, DocContent, DocsFetchError};
use crate::cache::Cache;
use serde_json::json;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetCrateDependenciesArgs {
    /// Crate name
    pub crate_name: String,
    /// Version (Required)
    pub version: String,
    /// Dependency kind filter (dev, build, normal). If omitted, shows all.
    pub kind: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AnalyzeFeatureFlagsArgs {
    /// Crate name
    pub crate_name: String,
    /// Version (Required)
    pub version: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FindTraitImplementorsArgs {
    /// Crate name
    pub crate_name: String,
    /// Trait path (e.g., 'Serialize' or 'serde::Serialize')
    pub trait_path: String,
    /// Version (Optional, defaults to latest)
    pub version: Option<String>,
    /// Max results (default: 30)
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetCrateExamplesArgs {
    /// Crate name
    pub crate_name: String,
    /// Version (Optional)
    pub version: Option<String>,
    /// Limit number of examples (Optional)
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FindBySignatureArgs {
    /// Crate name
    pub crate_name: String,
    /// Signature pattern (e.g., 'fn(&str) -> Result')
    pub signature_pattern: String,
    /// Version (Optional, defaults to latest)
    pub version: Option<String>,
}

pub async fn get_crate_dependencies(
    fetcher: &DocFetcher,
    crate_name: String,
    version: String,
    kind: Option<String>,
) -> Result<DocContent, DocsFetchError> {
    let key = CacheKey::GetCrateDependencies {
        crate_name: crate_name.clone(),
        version: version.clone(),
        kind: kind.clone(),
    };
    let args = json!({ "crate_name": crate_name, "version": version, "kind": kind });

    if let Some(cached_content) = fetcher.cache.get(&key).await {
        tracing::info!("Cache hit for get_crate_dependencies");
        fetcher.logger
            .log(
                "get_crate_dependencies",
                &args,
                &Ok::<_, DocsFetchError>(&cached_content),
            )
            .await;
        return Ok(cached_content);
    }

    tracing::info!("Fetching dependencies for: {}", crate_name);
    let result = fetcher
        .client
        .get_crate_dependencies(crate_name, version, kind)
        .await
        .map(|content_str| DocContent {
            content: content_str,
        });

    if let Ok(content) = &result {
        fetcher.cache.insert(key, content.clone()).await;
    }

    fetcher.logger
        .log("get_crate_dependencies", &args, &result)
        .await;
    result
}

pub async fn analyze_feature_flags(
    fetcher: &DocFetcher,
    crate_name: String,
    version: String,
) -> Result<DocContent, DocsFetchError> {
    let key = CacheKey::AnalyzeFeatureFlags {
        crate_name: crate_name.clone(),
        version: version.clone(),
    };
    let args = json!({ "crate_name": crate_name, "version": version });

    if let Some(content) = fetcher.cache.get(&key).await {
        fetcher.logger
            .log(
                "analyze_feature_flags",
                &args,
                &Ok::<_, DocsFetchError>(&content),
            )
            .await;
        return Ok(content);
    }

    tracing::info!("Analyzing feature flags for: {} {}", crate_name, version);
    let result = fetcher
        .client
        .analyze_feature_flags(crate_name, version)
        .await
        .map(|content_str| DocContent {
            content: content_str,
        });

    if let Ok(content) = &result {
        fetcher.cache.insert(key, content.clone()).await;
    }

    fetcher.logger
        .log("analyze_feature_flags", &args, &result)
        .await;
    result
}

pub async fn find_trait_implementors(
    fetcher: &DocFetcher,
    crate_name: String,
    trait_path: String,
    version: Option<String>,
    limit: Option<usize>,
) -> Result<DocContent, DocsFetchError> {
    let key = CacheKey::FindTraitImplementors {
        crate_name: crate_name.clone(),
        trait_path: trait_path.clone(),
        version: version.clone(),
        limit,
    };
    let args = json!({ "crate_name": crate_name, "trait_path": trait_path, "version": version, "limit": limit });

    if let Some(content) = fetcher.cache.get(&key).await {
        fetcher.logger
            .log(
                "find_trait_implementors",
                &args,
                &Ok::<_, DocsFetchError>(&content),
            )
            .await;
        return Ok(content);
    }

    tracing::info!(
        "Finding trait implementors for: {} in {}",
        trait_path,
        crate_name
    );
    let result = fetcher
        .client
        .find_trait_implementors(crate_name, trait_path, version, limit)
        .await
        .map(|content_str| DocContent {
            content: content_str,
        });

    if let Ok(content) = &result {
        fetcher.cache.insert(key, content.clone()).await;
    }

    fetcher.logger
        .log("find_trait_implementors", &args, &result)
        .await;
    result
}

pub async fn get_crate_examples(
    fetcher: &DocFetcher,
    crate_name: String,
    version: Option<String>,
    limit: Option<usize>,
) -> Result<DocContent, DocsFetchError> {
    let key = CacheKey::GetCrateExamples {
        crate_name: crate_name.clone(),
        version: version.clone(),
        limit,
    };
    let args = json!({ "crate_name": crate_name, "version": version, "limit": limit });

    if let Some(content) = fetcher.cache.get(&key).await {
        fetcher.logger
            .log(
                "get_crate_examples",
                &args,
                &Ok::<_, DocsFetchError>(&content),
            )
            .await;
        return Ok(content);
    }

    tracing::info!(
        "Fetching examples for: {} {}",
        crate_name,
        version.as_deref().unwrap_or("latest")
    );
    let result = fetcher
        .client
        .get_crate_examples(crate_name, version, limit)
        .await
        .map(|content_str| DocContent {
            content: content_str,
        });

    if let Ok(content) = &result {
        fetcher.cache.insert(key, content.clone()).await;
    }

    fetcher.logger.log("get_crate_examples", &args, &result).await;
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

pub async fn find_by_signature(
    fetcher: &DocFetcher,
    crate_name: String,
    signature_pattern: String,
    version: Option<String>,
) -> Result<DocContent, DocsFetchError> {
    let key = CacheKey::FindBySignature {
        crate_name: crate_name.clone(),
        signature_pattern: signature_pattern.clone(),
        version: version.clone(),
    };
    let args = json!({ "crate_name": crate_name, "signature_pattern": signature_pattern, "version": version });

    if let Some(content) = fetcher.cache.get(&key).await {
        fetcher.logger
            .log(
                "find_by_signature",
                &args,
                &Ok::<_, DocsFetchError>(&content),
            )
            .await;
        return Ok(content);
    }

    tracing::info!(
        "Searching by signature: {} in {}",
        signature_pattern,
        crate_name
    );
    let result = fetcher
        .client
        .find_by_signature(crate_name, signature_pattern, version)
        .await
        .map(|content_str| DocContent {
            content: content_str,
        });

    if let Ok(content) = &result {
        fetcher.cache.insert(key, content.clone()).await;
    }

    fetcher.logger.log("find_by_signature", &args, &result).await;
    result
}
