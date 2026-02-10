use crate::mcp::DocFetcher;
use crate::docs_parser::{DocContent, DocsFetchError};
use serde_json::json;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CargoCheckArgs {
    /// Optional arguments (e.g., ['--all-features', '--lib'])
    pub args: Option<Vec<String>>,
    /// The directory to run the command in
    pub cwd: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CargoTestArgs {
    /// Optional arguments (e.g., ['test_name'])
    pub args: Option<Vec<String>>,
    /// The directory to run the command in
    pub cwd: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CargoClippyArgs {
    /// Optional arguments
    pub args: Option<Vec<String>>,
    /// The directory to run the command in
    pub cwd: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CargoFmtArgs {
    /// Optional arguments
    pub args: Option<Vec<String>>,
    /// The directory to run the command in
    pub cwd: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CargoTreeArgs {
    /// Optional arguments (e.g., ['--invert', '--depth', '2'])
    pub args: Option<Vec<String>>,
    /// The directory to run the command in
    pub cwd: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CargoBenchArgs {
    /// Optional arguments (e.g., ['bench_name'])
    pub args: Option<Vec<String>>,
    /// The directory to run the command in
    pub cwd: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CargoSemverChecksArgs {
    /// Optional arguments (e.g., ['--check-build'])
    pub args: Option<Vec<String>>,
    /// The directory to run the command in
    pub cwd: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CargoAuditArgs {
    /// Optional arguments (e.g., ['--deny', 'warnings'])
    pub args: Option<Vec<String>>,
    /// The directory to run the command in
    pub cwd: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExpandMacroArgs {
    /// File path or module path to expand
    pub path: String,
    /// Specific item within the file to expand (optional)
    pub item: Option<String>,
    /// The directory to run the command in
    pub cwd: Option<String>,
}

pub async fn cargo_check(
    fetcher: &DocFetcher,
    args: Option<Vec<String>>,
    cwd: Option<String>,
) -> Result<DocContent, DocsFetchError> {
    let args = args.unwrap_or_default();
    let args_json = json!({ "args": args, "cwd": cwd });
    fetcher.logger
        .log("cargo_check", &args_json, &"running...")
        .await;

    match crate::cargo_tools::run_cargo_check(args, cwd).await {
        Ok(content) => {
            let result = DocContent { content };
            fetcher.logger
                .log("cargo_check", &args_json, &Ok::<_, DocsFetchError>(&result))
                .await;
            Ok(result)
        }
        Err(e) => {
            let err = DocsFetchError::CommandError(e.to_string());
            fetcher.logger
                .log("cargo_check", &args_json, &Err::<DocContent, _>(&err))
                .await;
            Err(err)
        }
    }
}

pub async fn cargo_audit(
    fetcher: &DocFetcher,
    args: Option<Vec<String>>,
    cwd: Option<String>,
) -> Result<DocContent, DocsFetchError> {
    let args = args.unwrap_or_default();
    let args_json = json!({ "args": args, "cwd": cwd });
    fetcher.logger
        .log("cargo_audit", &args_json, &"running...")
        .await;

    match crate::cargo_tools::run_cargo_audit(args, cwd).await {
        Ok(content) => {
            let result = DocContent { content };
            fetcher.logger
                .log("cargo_audit", &args_json, &Ok::<_, DocsFetchError>(&result))
                .await;
            Ok(result)
        }
        Err(e) => {
            let err = DocsFetchError::CommandError(e.to_string());
            fetcher.logger
                .log("cargo_audit", &args_json, &Err::<DocContent, _>(&err))
                .await;
            Err(err)
        }
    }
}

pub async fn cargo_clippy(
    fetcher: &DocFetcher,
    args: Option<Vec<String>>,
    cwd: Option<String>,
) -> Result<DocContent, DocsFetchError> {
    let args = args.unwrap_or_default();
    let args_json = json!({ "args": args, "cwd": cwd });
    fetcher.logger
        .log("cargo_clippy", &args_json, &"running...")
        .await;

    match crate::cargo_tools::run_cargo_clippy(args, cwd).await {
        Ok(content) => {
            let result = DocContent { content };
            fetcher.logger
                .log(
                    "cargo_clippy",
                    &args_json,
                    &Ok::<_, DocsFetchError>(&result),
                )
                .await;
            Ok(result)
        }
        Err(e) => {
            let err = DocsFetchError::CommandError(e.to_string());
            fetcher.logger
                .log(
                    "cargo_clippy",
                    &args_json,
                    &Err::<DocContent, _>(&err),
                )
                .await;
            Err(err)
        }
    }
}

pub async fn cargo_fmt(
    fetcher: &DocFetcher,
    args: Option<Vec<String>>,
    cwd: Option<String>,
) -> Result<DocContent, DocsFetchError> {
    let args = args.unwrap_or_default();
    let args_json = json!({ "args": args, "cwd": cwd });
    fetcher.logger
        .log("cargo_fmt", &args_json, &"running...")
        .await;

    match crate::cargo_tools::run_cargo_fmt(args, cwd).await {
        Ok(content) => {
            let result = DocContent { content };
            fetcher.logger
                .log("cargo_fmt", &args_json, &Ok::<_, DocsFetchError>(&result))
                .await;
            Ok(result)
        }
        Err(e) => {
            let err = DocsFetchError::CommandError(e.to_string());
            fetcher.logger
                .log("cargo_fmt", &args_json, &Err::<DocContent, _>(&err))
                .await;
            Err(err)
        }
    }
}

pub async fn cargo_test(
    fetcher: &DocFetcher,
    args: Option<Vec<String>>,
    cwd: Option<String>,
) -> Result<DocContent, DocsFetchError> {
    let args = args.unwrap_or_default();
    let args_json = json!({ "args": args, "cwd": cwd });
    fetcher.logger
        .log("cargo_test", &args_json, &"running...")
        .await;

    match crate::cargo_tools::run_cargo_test(args, cwd).await {
        Ok(content) => {
            let result = DocContent { content };
            fetcher.logger
                .log("cargo_test", &args_json, &Ok::<_, DocsFetchError>(&result))
                .await;
            Ok(result)
        }
        Err(e) => {
            let err = DocsFetchError::CommandError(e.to_string());
            fetcher.logger
                .log("cargo_test", &args_json, &Err::<DocContent, _>(&err))
                .await;
            Err(err)
        }
    }
}

pub async fn cargo_tree(
    fetcher: &DocFetcher,
    args: Option<Vec<String>>,
    cwd: Option<String>,
) -> Result<DocContent, DocsFetchError> {
    let args = args.unwrap_or_default();
    let args_json = json!({ "args": args, "cwd": cwd });
    fetcher.logger
        .log("cargo_tree", &args_json, &"running...")
        .await;

    match crate::cargo_tools::run_cargo_tree(args, cwd).await {
        Ok(content) => {
            let result = DocContent { content };
            fetcher.logger
                .log("cargo_tree", &args_json, &Ok::<_, DocsFetchError>(&result))
                .await;
            Ok(result)
        }
        Err(e) => {
            let err = DocsFetchError::CommandError(e.to_string());
            fetcher.logger
                .log("cargo_tree", &args_json, &Err::<DocContent, _>(&err))
                .await;
            Err(err)
        }
    }
}

pub async fn cargo_bench(
    fetcher: &DocFetcher,
    args: Option<Vec<String>>,
    cwd: Option<String>,
) -> Result<DocContent, DocsFetchError> {
    let args = args.unwrap_or_default();
    let args_json = json!({ "args": args, "cwd": cwd });
    fetcher.logger
        .log("cargo_bench", &args_json, &"running...")
        .await;

    match crate::cargo_tools::run_cargo_bench(args, cwd).await {
        Ok(content) => {
            let result = DocContent { content };
            fetcher.logger
                .log("cargo_bench", &args_json, &Ok::<_, DocsFetchError>(&result))
                .await;
            Ok(result)
        }
        Err(e) => {
            let err = DocsFetchError::CommandError(e.to_string());
            fetcher.logger
                .log("cargo_bench", &args_json, &Err::<DocContent, _>(&err))
                .await;
            Err(err)
        }
    }
}

pub async fn cargo_semver_checks(
    fetcher: &DocFetcher,
    args: Option<Vec<String>>,
    cwd: Option<String>,
) -> Result<DocContent, DocsFetchError> {
    let args = args.unwrap_or_default();
    let args_json = json!({ "args": args, "cwd": cwd });
    fetcher.logger
        .log("cargo_semver_checks", &args_json, &"running...")
        .await;

    match crate::cargo_tools::run_cargo_semver_checks(args, cwd).await {
        Ok(content) => {
            let result = DocContent { content };
            fetcher.logger
                .log(
                    "cargo_semver_checks",
                    &args_json,
                    &Ok::<_, DocsFetchError>(&result),
                )
                .await;
            Ok(result)
        }
        Err(e) => {
            let err = DocsFetchError::CommandError(e.to_string());
            fetcher.logger
                .log(
                    "cargo_semver_checks",
                    &args_json,
                    &Err::<DocContent, _>(&err),
                )
                .await;
            Err(err)
        }
    }
}

pub async fn expand_macro(
    fetcher: &DocFetcher,
    path: String,
    item: Option<String>,
    cwd: Option<String>,
) -> Result<DocContent, DocsFetchError> {
    let args_json = json!({ "path": path, "item": item, "cwd": cwd });
    fetcher.logger
        .log("expand_macro", &args_json, &"running...")
        .await;

    match crate::cargo_tools::run_cargo_expand(path, item, cwd).await {
        Ok(content) => {
            let result = DocContent { content };
            fetcher.logger
                .log("expand_macro", &args_json, &Ok::<_, DocsFetchError>(&result))
                .await;
            Ok(result)
        }
        Err(e) => {
            let err = DocsFetchError::CommandError(e.to_string());
            fetcher.logger
                .log("expand_macro", &args_json, &Err::<DocContent, _>(&err))
                .await;
            Err(err)
        }
    }
}
