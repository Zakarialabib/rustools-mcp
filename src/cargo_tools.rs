use anyhow::{Result, Context, anyhow};
use std::path::Path;
use tokio::process::Command;

/// Runs `cargo expand` on the given path and item.
/// 
/// # Arguments
/// 
/// * `path` - File path or module path.
/// * `item` - Specific item to expand (optional).
pub async fn run_cargo_expand(
    path: String,
    item: Option<String>,
) -> Result<String> {
    let mut args: Vec<String> = vec!["expand".to_string()];
    
    // Normalize path separators for checking
    let path_str = path.replace("\\", "/");
    
    if path_str.starts_with("examples/") {
        let stem = Path::new(&path).file_stem().unwrap_or_default().to_string_lossy();
        args.push("--example".to_string());
        args.push(stem.into_owned());
        
        if let Some(i) = &item {
            args.push(i.clone());
        }
    } else if path_str.starts_with("tests/") {
        let stem = Path::new(&path).file_stem().unwrap_or_default().to_string_lossy();
        args.push("--test".to_string());
        args.push(stem.into_owned());
        
        if let Some(i) = &item {
            args.push(i.clone());
        }
    } else {
        // Normal module path
        let module_path = if path.ends_with(".rs") {
            convert_file_to_module(&path)?
        } else {
            path
        };

        let target = if let Some(i) = item {
            if module_path.is_empty() {
                i
            } else {
                format!("{}::{}", module_path, i)
            }
        } else {
            module_path
        };

        if !target.is_empty() {
            args.push(target);
        }
    }
    
    args.push("--color".to_string());
    args.push("never".to_string());

    tracing::info!("Running cargo {:?}", args);

    let output = Command::new("cargo")
        .args(&args)
        .output()
        .await
        .context("Failed to execute cargo expand")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("cargo expand failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.to_string())
}

fn convert_file_to_module(path: &str) -> Result<String> {
    let path = Path::new(path);
    let mut components = Vec::new();
    
    for component in path.components() {
        let s = component.as_os_str().to_string_lossy();
        if s == "." || s == ".." { continue; }
        if s == "src" { continue; } // Skip src
        
        // Check if it is the last component (filename)
        if path.ends_with(component) {
             let stem = Path::new(&*s).file_stem().unwrap_or_default().to_string_lossy();
             if stem != "mod" && stem != "lib" && stem != "main" {
                 components.push(stem.to_string());
             }
        } else {
            components.push(s.to_string());
        }
    }
    
    Ok(components.join("::"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_paths() {
        assert_eq!(convert_file_to_module("src/lib.rs").unwrap(), "");
        assert_eq!(convert_file_to_module("src/main.rs").unwrap(), "");
        assert_eq!(convert_file_to_module("src/foo.rs").unwrap(), "foo");
        assert_eq!(convert_file_to_module("src/bar/mod.rs").unwrap(), "bar");
        assert_eq!(convert_file_to_module("src/bar/baz.rs").unwrap(), "bar::baz");
    }
}
