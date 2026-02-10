use rustools_mcp::cache::InMemoryCache;
use rustools_mcp::mcp::DocFetcher;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    println!("🚀 Starting MCP Tools Verification...");

    // Setup
    let cache_dir = PathBuf::from("verification_cache");
    let cache = InMemoryCache::new(cache_dir);
    let fetcher = DocFetcher::new(cache);

    // 1. find_crates
    println!("\n1️⃣ Testing find_crates...");
    match fetcher.find_crates("tokio".to_string(), Some(1), None).await {
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
    match fetcher
        .get_crate_modules("tokio".to_string(), None, Some(5))
        .await
    {
        Ok(res) => println!("✅ Success: Found {} bytes of content", res.content.len()),
        Err(e) => println!("❌ Failed: {}", e),
    }

    // 4. get_symbol_docs
    println!("\n4️⃣ Testing get_symbol_docs...");
    match fetcher
        .get_symbol_docs("tokio".to_string(), "net::TcpListener".to_string(), None)
        .await
    {
        Ok(res) => println!("✅ Success: Found {} bytes of content", res.content.len()),
        Err(e) => println!("❌ Failed: {}", e),
    }

    // 5. get_language_concept
    println!("\n5️⃣ Testing get_language_concept...");
    match fetcher.get_language_concept("async".to_string()).await {
        Ok(res) => println!("✅ Success: Found {} bytes of content", res.content.len()),
        Err(e) => println!("❌ Failed: {}", e),
    }

    // 6. cargo_tree
    println!("\n6️⃣ Testing cargo_tree...");
    match fetcher.cargo_tree(None, None).await {
        Ok(res) => println!("✅ Success: Found {} bytes of content", res.content.len()),
        Err(e) => println!("❌ Failed: {}", e),
    }

    // 7. cargo_semver_checks
    println!("\n7️⃣ Testing cargo_semver_checks...");
    match fetcher.cargo_semver_checks(None, None).await {
        Ok(res) => println!("✅ Success: Found {} bytes of content", res.content.len()),
        Err(e) => println!("❌ Failed (Expected if not installed): {}", e),
    }
}
