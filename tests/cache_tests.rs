use rustools_mcp::cache::{Cache, InMemoryCache};
use rustools_mcp::docs_parser::{CacheKey, DocContent, DocsRsParams};
use tempfile::tempdir;

#[tokio::test]
async fn test_cache_insert_retrieve() {
    let temp_dir = tempdir().unwrap();
    let cache = InMemoryCache::new(temp_dir.path().to_path_buf());

    let key = CacheKey::FetchRawDoc(DocsRsParams {
        crate_name: "test_crate".to_string(),
        version: "1.0.0".to_string(),
        path: "index.html".to_string(),
    });

    let value = DocContent {
        content: "test content".to_string(),
    };

    cache.insert(key.clone(), value.clone()).await;

    let retrieved = cache.get(&key).await;
    assert_eq!(retrieved, Some(value));
}

#[tokio::test]
async fn test_cache_persistence() {
    let temp_dir = tempdir().unwrap();
    let cache_dir = temp_dir.path().to_path_buf();

    let key = CacheKey::FetchRawDoc(DocsRsParams {
        crate_name: "test_crate_2".to_string(),
        version: "1.0.0".to_string(),
        path: "lib.rs".to_string(),
    });

    let value = DocContent {
        content: "persistent content".to_string(),
    };

    // Create first cache instance and save data
    {
        let cache = InMemoryCache::new(cache_dir.clone());
        cache.insert(key.clone(), value.clone()).await;
        // Auto-save is triggered on insert, but we can also explicitly save if needed
        // The implementation says: insert calls save().await
    }

    // Create second cache instance and load data
    let cache2 = InMemoryCache::new(cache_dir);
    cache2.load().await.expect("Failed to load cache");

    let retrieved = cache2.get(&key).await;
    assert_eq!(retrieved, Some(value));
}
