use chrono::Utc;
use serde::Serialize;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct RequestLogger {
    file: Option<Arc<Mutex<File>>>,
}

impl RequestLogger {
    pub fn new(path: &str) -> Self {
        // We open synchronously at startup to ensure the file exists and we have permissions
        // Then we convert to tokio::fs::File for async writing
        let file = match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            Ok(std_file) => Some(Arc::new(Mutex::new(File::from_std(std_file)))),
            Err(e) => {
                eprintln!("Failed to open log file '{}': {}", path, e);
                None
            }
        };

        Self { file }
    }

    pub async fn log<A: Serialize, R: Serialize>(&self, tool: &str, args: &A, result: &R) {
        let entry = LogEntry {
            timestamp: Utc::now().to_rfc3339(),
            tool: tool.to_string(),
            args,
            result,
        };

        if let Some(file_mutex) = &self.file {
            if let Ok(line) = serde_json::to_string(&entry) {
                let mut file = file_mutex.lock().await;
                let _ = file.write_all(format!("{}\n", line).as_bytes()).await;
            }
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
