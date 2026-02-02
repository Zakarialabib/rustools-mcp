use std::sync::Arc;
use std::path::PathBuf;
use rustools_mcp::mcp::DocFetcher;
use rustools_mcp::cache::InMemoryCache;

#[tokio::main]
async fn main() {
    println!("🚀 Starting MCP Tools Verification...");

    // Setup
    let cache_dir = PathBuf::from("verification_cache");
    let cache = Arc::new(InMemoryCache::new(cache_dir));
    let fetcher = DocFetcher::new(cache);

    // 1. find_crates
    println!("\n1️⃣ Testing find_crates...");
    match fetcher.find_crates("tokio".to_string(), Some(1)).await {
        Ok(res) => println!("✅ Success: Found {} bytes of content", res.content.len()),
        Err(e) => println!("❌ Failed: {}", e),
    }

    // 2. get_crate_overview
    println!("\n2️⃣ Testing get_crate_overview...");
    match fetcher.get_crate_overview("tokio".to_string(), None).await {
        Ok(res) => println!("✅ Success: Found {} bytes of content", res.content.len()),
        Err(e) => println!("❌ Failed: {}", e),
    }

    // 3. get_crate_modules
    println!("\n3️⃣ Testing get_crate_modules...");
    match fetcher.get_crate_modules("tokio".to_string(), None, Some(5)).await {
        Ok(res) => println!("✅ Success: Found {} bytes of content", res.content.len()),
        Err(e) => println!("❌ Failed: {}", e),
    }

    // 4. get_symbol_docs
    println!("\n4️⃣ Testing get_symbol_docs...");
    match fetcher.get_symbol_docs("tokio".to_string(), "net::TcpListener".to_string(), None).await {
        Ok(res) => println!("✅ Success: Found {} bytes of content", res.content.len()),
        Err(e) => println!("❌ Failed: {}", e),
    }

    // 5. get_language_concept
    println!("\n5️⃣ Testing get_language_concept...");
    match fetcher.get_language_concept("ownership".to_string()).await {
        Ok(res) => println!("✅ Success: Found {} bytes of content", res.content.len()),
        Err(e) => println!("❌ Failed: {}", e),
    }

    // 6. explain_error_code
    println!("\n6️⃣ Testing explain_error_code...");
    match fetcher.explain_error_code("E0382".to_string()).await {
        Ok(res) => println!("✅ Success: Found {} bytes of content", res.content.len()),
        Err(e) => println!("❌ Failed: {}", e),
    }

    // 7. get_context_bundle
    println!("\n7️⃣ Testing get_context_bundle...");
    match fetcher.get_context_bundle("serde".to_string(), None).await {
        Ok(res) => println!("✅ Success: Found {} bytes of content", res.content.len()),
        Err(e) => println!("❌ Failed: {}", e),
    }

    // 8. find_by_signature
    println!("\n8️⃣ Testing find_by_signature...");
    match fetcher.find_by_signature("std".to_string(), "fn(usize) -> Option".to_string(), None).await {
        Ok(res) => println!("✅ Success: Found {} bytes of content", res.content.len()),
        Err(e) => println!("❌ Failed: {}", e),
    }

    // 9. find_trait_implementors
    println!("\n9️⃣ Testing find_trait_implementors...");
    match fetcher.find_trait_implementors("serde".to_string(), "Serialize".to_string(), None, Some(5)).await {
        Ok(res) => println!("✅ Success: Found {} bytes of content", res.content.len()),
        Err(e) => println!("❌ Failed: {}", e),
    }

    // 10. analyze_feature_flags
    println!("\n🔟 Testing analyze_feature_flags...");
    match fetcher.analyze_feature_flags("tokio".to_string(), "1.30.0".to_string()).await {
        Ok(res) => println!("✅ Success: Found {} bytes of content", res.content.len()),
        Err(e) => println!("❌ Failed: {}", e),
    }

    // 11. get_crate_dependencies
    println!("\n1️⃣1️⃣ Testing get_crate_dependencies...");
    match fetcher.get_crate_dependencies("tokio".to_string(), "1.30.0".to_string(), None).await {
        Ok(res) => println!("✅ Success: Found {} bytes of content", res.content.len()),
        Err(e) => println!("❌ Failed: {}", e),
    }

    // 12. get_crate_examples
    println!("\n1️⃣2️⃣ Testing get_crate_examples...");
    match fetcher.get_crate_examples("tokio".to_string(), None, Some(1)).await {
        Ok(res) => println!("✅ Success: Found {} bytes of content", res.content.len()),
        Err(e) => println!("❌ Failed: {}", e),
    }

    // 13. read_source_file
    println!("\n1️⃣3️⃣ Testing read_source_file...");
    // Trying to read a known file in tokio
    match fetcher.read_source_file("tokio".to_string(), "src/lib.rs".to_string(), Some("1.30.0".to_string()), Some(1), Some(10)).await {
        Ok(res) => println!("✅ Success: Found {} bytes of content", res.content.len()),
        Err(e) => println!("❌ Failed: {}", e),
    }

    // 14. doc_workflow_help
    println!("\n1️⃣4️⃣ Testing doc_workflow_help...");
    match fetcher.doc_workflow_help().await {
        Ok(res) => println!("✅ Success: Found {} bytes of content", res.content.len()),
        Err(e) => println!("❌ Failed: {}", e),
    }

    println!("\n🏁 Verification Completed.");
}
