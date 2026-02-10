use axum::{
    extract::Path,
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tokio::fs;
use tower_http::cors::CorsLayer;

pub async fn start_ui_server(port: u16) {
    let app = Router::new()
        .route("/", get(index))
        .route("/api/logs", get(get_logs))
        .route("/api/cache", get(list_cache))
        .route("/api/cache/{name}", get(get_cache_item))
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Dashboard running at http://localhost:{}", port);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> Html<&'static str> {
    Html(include_str!("dashboard.html"))
}

async fn get_logs() -> impl IntoResponse {
    match fs::read_to_string("requests.log").await {
        Ok(content) => {
            // Convert JSONL to JSON Array
            let lines: Vec<Value> = content
                .lines()
                .filter_map(|line| serde_json::from_str(line).ok())
                .collect();
            Json(lines).into_response()
        }
        Err(_) => Json(Vec::<Value>::new()).into_response(),
    }
}

async fn list_cache() -> impl IntoResponse {
    let mut files = Vec::new();
    if let Ok(mut entries) = fs::read_dir(".cache").await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(file_name) = entry.file_name().into_string() {
                if file_name.ends_with(".json") {
                    files.push(file_name);
                }
            }
        }
    }
    Json(files).into_response()
}

async fn get_cache_item(Path(name): Path<String>) -> impl IntoResponse {
    let path = std::path::Path::new(".cache").join(&name);
    // Security check: prevent directory traversal
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Json(json!({"error": "Invalid filename"})).into_response();
    }

    match fs::read_to_string(path).await {
        Ok(content) => {
            if let Ok(json) = serde_json::from_str::<Value>(&content) {
                Json(json).into_response()
            } else {
                Json(json!({"error": "Invalid JSON in cache file"})).into_response()
            }
        }
        Err(_) => Json(json!({"error": "File not found"})).into_response(),
    }
}
