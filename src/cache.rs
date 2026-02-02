use crate::docs_parser::{DocContent, CacheKey, DocsRsParams};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use std::io;
use async_trait::async_trait;
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// Trait defining the caching behavior for documentation.
#[async_trait]
pub trait Cache: Send + Sync {
    /// Retrieves content from the cache.
    async fn get(&self, key: &CacheKey) -> Option<DocContent>;

    /// Inserts content into the cache.
    async fn insert(&self, key: CacheKey, value: DocContent);

    /// Checks if a key exists in the cache.
    async fn contains_key(&self, key: &CacheKey) -> bool;

    /// Clears the entire cache.
    async fn clear(&self);

    /// Saves the cache state to persistent storage.
    ///
    /// # Errors
    ///
    /// Returns an IO error if the save operation fails.
    async fn save(&self) -> Result<(), io::Error>;

    /// Loads the cache state from persistent storage.
    ///
    /// # Errors
    ///
    /// Returns an IO error if the load operation fails.
    async fn load(&self) -> Result<(), io::Error>;
}

/// Cache data for a single crate, mapping normalized key to content.
type CrateCacheData = HashMap<String, DocContent>;

/// In-memory representation of the entire cache.
#[derive(Debug, Serialize, Deserialize, Default)]
struct CacheData {
    /// Maps cache keys to their content
    data: HashMap<CacheKey, DocContent>,
}

/// Helper to extract crate name for file organization
fn get_crate_name(key: &CacheKey) -> String {
    match key {
        CacheKey::FetchRawDoc(p) => p.crate_name.clone(),
        CacheKey::FindCrates { .. } => "__global__".to_string(),
        CacheKey::GetCrateOverview { crate_name, .. } => crate_name.clone(),
        CacheKey::GetCrateModules { crate_name, .. } => crate_name.clone(),
        CacheKey::GetSymbolDocs { crate_name, .. } => crate_name.clone(),
        CacheKey::ReadSourceFile { crate_name, .. } => crate_name.clone(),
        CacheKey::GetCrateDependencies { crate_name, .. } => crate_name.clone(),
        CacheKey::GetCrateExamples { crate_name, .. } => crate_name.clone(),
        CacheKey::AnalyzeFeatureFlags { crate_name, .. } => crate_name.clone(),
        CacheKey::FindTraitImplementors { crate_name, .. } => crate_name.clone(),
        CacheKey::GetContextBundle { crate_name, .. } => crate_name.clone(),
        CacheKey::FindBySignature { crate_name, .. } => crate_name.clone(),
    }
}

/// Normalizes cache key into a string for storage within a crate file.
fn normalize_key(key: &CacheKey) -> String {
    match key {
        CacheKey::FetchRawDoc(p) => format!("doc::{}::{}", p.version, p.path),
        CacheKey::FindCrates { query, limit } => format!("search::{}::{}", query, limit.unwrap_or(0)),
        CacheKey::GetCrateOverview { version, .. } => format!("landing::{}", version.as_deref().unwrap_or("latest")),
        CacheKey::GetCrateModules { version, limit, .. } => format!("modules::{}::{}", version.as_deref().unwrap_or("latest"), limit.unwrap_or(0)),
        CacheKey::GetSymbolDocs { version, symbol_path, .. } => format!("item::{}::{}", version.as_deref().unwrap_or("latest"), symbol_path),
        CacheKey::ReadSourceFile { version, path, start_line, end_line, .. } => {
             let s = start_line.unwrap_or(0);
             let e = end_line.unwrap_or(0);
             format!("source::{}::{}::{}::{}", version.as_deref().unwrap_or("latest"), s, e, path)
        },
        CacheKey::GetCrateDependencies { version, kind, .. } => format!("deps::{}::{}", version, kind.as_deref().unwrap_or("all")),
        CacheKey::GetCrateExamples { version, limit, .. } => format!("examples::{}::{}", version.as_deref().unwrap_or("latest"), limit.unwrap_or(0)),
        CacheKey::AnalyzeFeatureFlags { version, .. } => format!("features::{}", version),
        CacheKey::FindTraitImplementors { version, trait_path, limit, .. } => format!("trait_impls::{}::{}::{}", version.as_deref().unwrap_or("latest"), limit.unwrap_or(0), trait_path),
        CacheKey::GetContextBundle { version, .. } => format!("bundle::{}", version.as_deref().unwrap_or("latest")),
        CacheKey::FindBySignature { version, signature_pattern, .. } => format!("signature::{}::{}", version.as_deref().unwrap_or("latest"), signature_pattern),
    }
}

/// Reconstructs CacheKey from a normalized string and crate name.
fn denormalize_key(crate_name: &str, normalized_key: &str) -> Result<CacheKey, String> {
    if crate_name == "__global__" {
        if let Some(rest) = normalized_key.strip_prefix("search::") {
             let parts: Vec<&str> = rest.splitn(2, "::").collect();
             if parts.len() == 2 {
                 let query = parts[0].to_string();
                 let limit = parts[1].parse::<u32>().ok().map(|l| if l == 0 { None } else { Some(l) });
                 return Ok(CacheKey::FindCrates { query, limit: limit.flatten() });
             }
        }
        return Err(format!("Invalid global key: {}", normalized_key));
    }

    if let Some(rest) = normalized_key.strip_prefix("doc::") {
        let parts: Vec<&str> = rest.splitn(2, "::").collect();
        if parts.len() == 2 {
            return Ok(CacheKey::FetchRawDoc(DocsRsParams {
                crate_name: crate_name.to_string(),
                version: parts[0].to_string(),
                path: parts[1].to_string(),
            }));
        }
    } else if let Some(rest) = normalized_key.strip_prefix("landing::") {
         let version = if rest == "latest" { None } else { Some(rest.to_string()) };
         return Ok(CacheKey::GetCrateOverview {
             crate_name: crate_name.to_string(),
             version,
         });
    } else if let Some(rest) = normalized_key.strip_prefix("modules::") {
         let parts: Vec<&str> = rest.splitn(2, "::").collect();
         if parts.len() == 2 {
             let version = if parts[0] == "latest" { None } else { Some(parts[0].to_string()) };
             let limit = parts[1].parse::<usize>().ok().filter(|&l| l > 0);
             return Ok(CacheKey::GetCrateModules {
                 crate_name: crate_name.to_string(),
                 version,
                 limit,
             });
         } else {
             // Legacy fallback
             let version = if rest == "latest" { None } else { Some(rest.to_string()) };
             return Ok(CacheKey::GetCrateModules {
                 crate_name: crate_name.to_string(),
                 version,
                 limit: None,
             });
         }
    } else if let Some(rest) = normalized_key.strip_prefix("item::") {
        let parts: Vec<&str> = rest.splitn(2, "::").collect();
        if parts.len() == 2 {
            let version = if parts[0] == "latest" { None } else { Some(parts[0].to_string()) };
            return Ok(CacheKey::GetSymbolDocs {
                crate_name: crate_name.to_string(),
                symbol_path: parts[1].to_string(),
                version,
            });
        }
    } else if let Some(rest) = normalized_key.strip_prefix("source::") {
        let parts: Vec<&str> = rest.splitn(4, "::").collect();
        if parts.len() == 4 {
            let version = if parts[0] == "latest" { None } else { Some(parts[0].to_string()) };
            let start_line = parts[1].parse::<usize>().ok().filter(|&x| x > 0);
            let end_line = parts[2].parse::<usize>().ok().filter(|&x| x > 0);
            let path = parts[3].to_string();
            return Ok(CacheKey::ReadSourceFile {
                crate_name: crate_name.to_string(),
                path,
                version,
                start_line,
                end_line,
            });
        } else if parts.len() == 2 {
            // Legacy key support: source::version::path
            let version = if parts[0] == "latest" { None } else { Some(parts[0].to_string()) };
            let path = parts[1].to_string();
             return Ok(CacheKey::ReadSourceFile {
                crate_name: crate_name.to_string(),
                path,
                version,
                start_line: None,
                end_line: None,
            });
        }
    } else if let Some(rest) = normalized_key.strip_prefix("deps::") {
        let parts: Vec<&str> = rest.splitn(2, "::").collect();
        if parts.len() == 2 {
            let version = parts[0].to_string();
            let kind = if parts[1] == "all" { None } else { Some(parts[1].to_string()) };
            return Ok(CacheKey::GetCrateDependencies { crate_name: crate_name.to_string(), version, kind });
        } else {
             // Legacy fallback
             let version = rest.to_string();
             return Ok(CacheKey::GetCrateDependencies { crate_name: crate_name.to_string(), version, kind: None });
        }
    } else if let Some(rest) = normalized_key.strip_prefix("examples::") {
         let parts: Vec<&str> = rest.splitn(2, "::").collect();
         if parts.len() == 2 {
             let version = if parts[0] == "latest" { None } else { Some(parts[0].to_string()) };
             let limit = parts[1].parse::<usize>().ok().filter(|&l| l > 0);
             return Ok(CacheKey::GetCrateExamples {
                 crate_name: crate_name.to_string(),
                 version,
                 limit,
             });
         } else {
             // Legacy fallback
             let version = if rest == "latest" { None } else { Some(rest.to_string()) };
             return Ok(CacheKey::GetCrateExamples {
                 crate_name: crate_name.to_string(),
                 version,
                 limit: None,
             });
         }
    } else if let Some(version) = normalized_key.strip_prefix("features::") {
         return Ok(CacheKey::AnalyzeFeatureFlags {
             crate_name: crate_name.to_string(),
             version: version.to_string(),
         });
    } else if let Some(rest) = normalized_key.strip_prefix("trait_impls::") {
         let parts: Vec<&str> = rest.splitn(3, "::").collect();
         if parts.len() == 3 {
             let version = if parts[0] == "latest" { None } else { Some(parts[0].to_string()) };
             let limit = parts[1].parse::<usize>().ok().filter(|&l| l > 0);
             let trait_path = parts[2].to_string();
             return Ok(CacheKey::FindTraitImplementors {
                 crate_name: crate_name.to_string(),
                 trait_path,
                 version,
                 limit,
             });
         } else if parts.len() == 2 {
             // Legacy fallback
             let version = if parts[0] == "latest" { None } else { Some(parts[0].to_string()) };
             let trait_path = parts[1].to_string();
             return Ok(CacheKey::FindTraitImplementors {
                 crate_name: crate_name.to_string(),
                 trait_path,
                 version,
                 limit: None,
             });
         }
     } else if let Some(rest) = normalized_key.strip_prefix("bundle::") {
          let version = if rest == "latest" { None } else { Some(rest.to_string()) };
          return Ok(CacheKey::GetContextBundle {
              crate_name: crate_name.to_string(),
               version,
           });
      } else if let Some(rest) = normalized_key.strip_prefix("signature::") {
          let parts: Vec<&str> = rest.splitn(2, "::").collect();
          if parts.len() == 2 {
              let version = if parts[0] == "latest" { None } else { Some(parts[0].to_string()) };
              let signature_pattern = parts[1].to_string();
              return Ok(CacheKey::FindBySignature {
                  crate_name: crate_name.to_string(),
                  signature_pattern,
                  version,
              });
          }
      } else {
        // Legacy fallback: assume it's FetchRawDoc (version::path)
        let parts: Vec<&str> = normalized_key.splitn(2, "::").collect();
        if parts.len() == 2 {
            return Ok(CacheKey::FetchRawDoc(DocsRsParams {
                crate_name: crate_name.to_string(),
                version: parts[0].to_string(),
                path: parts[1].to_string(),
            }));
        }
    }

    Err(format!("Invalid normalized key format for crate {}: {}", crate_name, normalized_key))
}

/// Thread-safe cache implementation with disk persistence.
#[derive(Debug, Clone)]
pub struct InMemoryCache {
    /// Thread-safe storage for cache data
    cache: Arc<RwLock<CacheData>>,
    /// Directory where cache files are stored
    cache_dir: PathBuf,
}

impl InMemoryCache {
    /// Creates a new cache instance using the specified directory for persistence.
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            cache: Arc::new(RwLock::new(CacheData::default())),
            cache_dir,
        }
    }
}

#[async_trait]
impl Cache for InMemoryCache {
    async fn get(&self, key: &CacheKey) -> Option<DocContent> {
        self.cache.read().await.data.get(key).cloned()
    }

    async fn insert(&self, key: CacheKey, value: DocContent) {
        self.cache.write().await.data.insert(key, value);
        // Auto-save to ensure persistence even if process is killed
        if let Err(e) = self.save().await {
            tracing::error!("Failed to auto-save cache: {}", e);
        }
    }

    async fn contains_key(&self, key: &CacheKey) -> bool {
        self.cache.read().await.data.contains_key(key)
    }

    async fn clear(&self) {
        self.cache.write().await.data.clear();
    }

    async fn save(&self) -> Result<(), io::Error> {
        let dir_path = &self.cache_dir;
        
        let data_to_save: HashMap<String, CrateCacheData> = {
            let cache_guard = self.cache.read().await;
            let data_map = &cache_guard.data;
            
            // Group by crate
            let mut groups: HashMap<String, CrateCacheData> = HashMap::new();
            for (key, content) in data_map {
                let crate_name = get_crate_name(key);
                let normalized = normalize_key(key);
                groups.entry(crate_name)
                    .or_default()
                    .insert(normalized, content.clone());
            }
            groups
        };

        if fs::metadata(dir_path).await.is_err() {
            fs::create_dir_all(dir_path).await?;
        }

        for (crate_name, crate_data) in data_to_save {
            let file_path = dir_path.join(format!("{}.json", crate_name));
            // Serialize to string first
            let content = serde_json::to_string(&crate_data)?;
            let mut file = fs::File::create(file_path).await?;
            file.write_all(content.as_bytes()).await?;
        }

        Ok(())
    }

    async fn load(&self) -> Result<(), io::Error> {
        let dir_path = &self.cache_dir;
        if fs::metadata(dir_path).await.is_err() {
            return Ok(());
        }

        let mut entries = fs::read_dir(dir_path).await?;
        let mut loaded_data = HashMap::new();

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let crate_name = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .to_string();

                if let Ok(content) = fs::read_to_string(&path).await {
                    if let Ok(crate_data) = serde_json::from_str::<CrateCacheData>(&content) {
                        for (normalized_key, content) in crate_data {
                            if let Ok(key) = denormalize_key(&crate_name, &normalized_key) {
                                loaded_data.insert(key, content);
                            }
                        }
                    }
                }
            }
        }

        let mut cache_guard = self.cache.write().await;
        cache_guard.data = loaded_data;
        Ok(())
    }
}
