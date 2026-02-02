use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use std::sync::Arc;
use serde::Serialize;
use chrono::Utc;

#[derive(Clone)]
pub struct RequestLogger {
    file: Arc<Mutex<File>>,
}

impl RequestLogger {
    pub fn new(path: &str) -> Self {
        // We open synchronously at startup to ensure the file exists and we have permissions
        // Then we convert to tokio::fs::File for async writing
        let std_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .expect("Failed to open log file");
        
        let file = File::from_std(std_file);
        
        Self {
            file: Arc::new(Mutex::new(file)),
        }
    }

    pub async fn log<A: Serialize, R: Serialize>(&self, tool: &str, args: &A, result: &R) {
        let entry = LogEntry {
            timestamp: Utc::now().to_rfc3339(),
            tool: tool.to_string(),
            args,
            result,
        };
        
        if let Ok(line) = serde_json::to_string(&entry) {
            let mut file = self.file.lock().await;
            let _ = file.write_all(format!("{}\n", line).as_bytes()).await;
        }
    }
}

#[derive(Serialize)]
struct LogEntry<'a, A, R> {
    timestamp: String,
    tool: String,
    args: &'a A,
    result: &'a R,
}
