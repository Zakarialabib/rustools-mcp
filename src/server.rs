use rmcp;
use std::sync::Arc;
use crate::mcp::DocFetcher;
use crate::cache::InMemoryCache;
use axum::Router;

pub async fn start_server(addr: std::net::SocketAddr) -> anyhow::Result<()> {
    // 1. Session Manager
    let session_manager = rmcp::transport::streamable_http_server::session::local::LocalSessionManager::default();
    let session_manager_arc = Arc::new(session_manager);
    
    // 2. Config
    let config = rmcp::transport::streamable_http_server::StreamableHttpServerConfig::default();
    
    // 3. Service Factory
    let factory = || { 
        let cache_dir = std::env::current_dir().unwrap().join(".cache");
        let cache = InMemoryCache::new(cache_dir);
        Ok::<_, std::io::Error>(DocFetcher::new(cache)) 
    };
    
    // 4. Create Service
    let service = rmcp::transport::streamable_http_server::StreamableHttpService::new(
        factory,
        session_manager_arc,
        config
    );
    
    // 5. Serve with Axum
    // We nest the service under "/mcp" so clients connect to http://localhost:8080/mcp
    let app = Router::new().nest_service("/mcp", service);

    println!("Listening on http://{}", addr);
    println!("MCP endpoint is at http://{}/mcp", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

pub async fn start_stdio_server() -> anyhow::Result<()> {
    // Use the Stdio transport
    // Since rmcp 0.14 doesn't have a high-level StdioServer helper that manages the loop easily (it might, but we are using low-level components),
    // let's look at how to run it.
    // Actually, rmcp::server::Server might be what we want for Stdio if we aren't using StreamableHttpService.
    // But let's check what rmcp provides for stdio.
    
    // For now, let's leave this as a placeholder or try to implement it if we find the right API.
    // The focus was on HTTP/SSE server.
    Ok(())
}
