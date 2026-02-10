use anyhow::{anyhow, Context, Result};
use std::path::Path;
use tokio::process::Command;

/// Runs `cargo check` (or clippy) on the project.
pub async fn run_cargo_check(args: Vec<String>, cwd: Option<String>) -> Result<String> {
    run_cargo_command("check", &args, cwd).await
}

/// Runs `cargo clippy` on the project.
pub async fn run_cargo_clippy(args: Vec<String>, cwd: Option<String>) -> Result<String> {
    run_cargo_command("clippy", &args, cwd).await
}

/// Runs `cargo fmt` on the project.
pub async fn run_cargo_fmt(args: Vec<String>, cwd: Option<String>) -> Result<String> {
    run_cargo_command("fmt", &args, cwd).await
}

/// Runs `cargo test` on the project.
pub async fn run_cargo_test(args: Vec<String>, cwd: Option<String>) -> Result<String> {
    run_cargo_command("test", &args, cwd).await
}

/// Runs `cargo tree` on the project.
pub async fn run_cargo_tree(args: Vec<String>, cwd: Option<String>) -> Result<String> {
    run_cargo_command("tree", &args, cwd).await
}

/// Runs `cargo bench` on the project.
pub async fn run_cargo_bench(args: Vec<String>, cwd: Option<String>) -> Result<String> {
    run_cargo_command("bench", &args, cwd).await
}

/// Helper to run arbitrary cargo commands
async fn run_cargo_command(command: &str, args: &[String], cwd: Option<String>) -> Result<String> {
    let mut cmd_args = vec![command.to_string()];
    cmd_args.extend_from_slice(args);

    // Add color=never to ensure clean output for LLM
    if command != "fmt" {
        // fmt doesn't support --color
        cmd_args.push("--color".to_string());
        cmd_args.push("never".to_string());
    }

    tracing::info!("Running cargo {:?} in {:?}", cmd_args, cwd);

    let mut cmd = Command::new("cargo");
    cmd.args(&cmd_args);
    
    if let Some(path) = cwd {
        cmd.current_dir(path);
    }

    let output = cmd
        .output()
        .await
        .context(format!("Failed to execute cargo {}", command))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let combined = if output.status.success() {
        if stdout.trim().is_empty() {
            // For check/clippy, success often means empty stdout but stderr has "Finished"
            if stderr.trim().is_empty() {
                "Command executed successfully (no output).".to_string()
            } else {
                stderr.to_string()
            }
        } else {
            format!("{}\n{}", stdout, stderr)
        }
    } else {
        format!("Command failed:\n{}\n{}", stdout, stderr)
    };

    Ok(combined.trim().to_string())
}

/// Runs `cargo expand` on the given path and item.
///
/// # Arguments
///
/// * `path` - File path or module path.
/// * `item` - Specific item to expand (optional).
/// * `cwd` - The directory to run the command in (optional).
pub async fn run_cargo_expand(path: String, item: Option<String>, cwd: Option<String>) -> Result<String> {
    let mut args: Vec<String> = vec!["expand".to_string()];

    // Normalize path separators for checking
    let path_str = path.replace("\\", "/");

    if path_str.starts_with("examples/") {
        let stem = Path::new(&path)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        args.push("--example".to_string());
        args.push(stem.into_owned());

        if let Some(i) = &item {
            args.push(i.clone());
        }
    } else if path_str.starts_with("tests/") {
        let stem = Path::new(&path)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
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

    tracing::info!("Running cargo {:?} in {:?}", args, cwd);

    let mut cmd = Command::new("cargo");
    cmd.args(&args);

    if let Some(path) = cwd {
        cmd.current_dir(path);
    }

    let output = cmd
        .output()
        .await
        .context("Failed to execute cargo expand")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("no such command: `expand`") {
            return Err(anyhow!(
                "`cargo expand` is not installed. Please install it via: `cargo install cargo-expand`"
            ));
        }
        return Err(anyhow!("cargo expand failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.to_string())
}

/// Runs `cargo semver-checks` on the project.
pub async fn run_cargo_semver_checks(args: Vec<String>, cwd: Option<String>) -> Result<String> {
    // Check if cargo-semver-checks is installed
    let version_check = Command::new("cargo")
        .args(&["semver-checks", "--version"])
        .output()
        .await;

    if version_check.is_err() || !version_check.unwrap().status.success() {
        return Err(anyhow!("cargo-semver-checks is not installed. Please install it with `cargo install cargo-semver-checks`."));
    }

    run_cargo_command("semver-checks", &args, cwd).await
}

/// Runs `cargo audit` on the project.
pub async fn run_cargo_audit(args: Vec<String>, cwd: Option<String>) -> Result<String> {
    // Check if cargo-audit is installed
    let version_check = Command::new("cargo")
        .args(&["audit", "--version"])
        .output()
        .await;

    if version_check.is_err() || !version_check.unwrap().status.success() {
        return Err(anyhow!("cargo-audit is not installed. Please install it with `cargo install cargo-audit`."));
    }

    run_cargo_command("audit", &args, cwd).await
}

fn convert_file_to_module(path: &str) -> Result<String> {
    let path = Path::new(path);
    let mut components = Vec::new();

    for component in path.components() {
        let s = component.as_os_str().to_string_lossy();
        if s == "." || s == ".." {
            continue;
        }
        if s == "src" {
            continue;
        } // Skip src

        // Check if it is the last component (filename)
        if path.ends_with(component) {
            let stem = Path::new(&*s)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy();
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
        assert_eq!(
            convert_file_to_module("src/bar/baz.rs").unwrap(),
            "bar::baz"
        );
    }
}
