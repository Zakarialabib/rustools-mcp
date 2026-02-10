use serde::Deserialize;
use std::path::PathBuf;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_cache_dir")]
    pub cache_dir: PathBuf,
    
    #[serde(default = "default_log_level")]
    pub log_level: String,
    
    #[serde(default = "default_ui_port")]
    pub ui_port: u16,
    
    #[serde(default = "default_server_address")]
    pub server_address: String,
}

fn default_cache_dir() -> PathBuf {
    PathBuf::from(".cache")
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_ui_port() -> u16 {
    3000
}

fn default_server_address() -> String {
    "127.0.0.1:8080".to_string()
}

impl Config {
    pub fn load() -> Self {
        // 1. Try config.toml
        if let Ok(content) = fs::read_to_string("config.toml") {
            if let Ok(config) = toml::from_str(&content) {
                return config;
            }
        }
        
        // 2. Try environment variables (could be expanded)
        
        // 3. Default
        Config {
            cache_dir: default_cache_dir(),
            log_level: default_log_level(),
            ui_port: default_ui_port(),
            server_address: default_server_address(),
        }
    }
}
