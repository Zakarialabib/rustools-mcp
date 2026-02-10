use crate::mcp::DocFetcher;
use crate::docs_parser::{DocContent, DocsFetchError};
use serde_json::json;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetLanguageConceptArgs {
    /// The concept to search for (e.g., 'ownership', 'smart pointers')
    pub concept: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExplainErrorCodeArgs {
    /// The error code (e.g., 'E0382')
    pub code: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DocWorkflowHelpArgs {}

pub async fn get_language_concept(
    fetcher: &DocFetcher,
    concept: String,
) -> Result<DocContent, DocsFetchError> {
    let args = json!({ "concept": concept });
    
    if concept.trim().is_empty() {
        return Err(DocsFetchError::InvalidInput("Concept cannot be empty".to_string()));
    }

    fetcher.logger
        .log("get_language_concept", &args, &"fetching...")
        .await;

    match fetcher.book_client.search_concept(&concept).await {
        Ok(Some(chapter)) => match fetcher.book_client.get_chapter_content(&chapter.url).await {
            Ok(content) => {
                let result = DocContent {
                    content: format!("# {}\n\n{}", chapter.title, content),
                };
                fetcher.logger
                    .log(
                        "get_language_concept",
                        &args,
                        &Ok::<_, DocsFetchError>(&result),
                    )
                    .await;
                Ok(result)
            }
            Err(e) => {
                let err = DocsFetchError::RequestError(format!(
                    "Failed to fetch chapter content: {}",
                    e
                ));
                fetcher.logger
                    .log("get_language_concept", &args, &Err::<DocContent, _>(&err))
                    .await;
                Err(err)
            }
        },
        Ok(None) => {
            let err = DocsFetchError::ItemNotFound(format!(
                "Concept '{}' not found in the Rust Book",
                concept
            ));
            fetcher.logger
                .log("get_language_concept", &args, &Err::<DocContent, _>(&err))
                .await;
            Err(err)
        }
        Err(e) => {
            let err = DocsFetchError::RequestError(format!("Failed to search book: {}", e));
            fetcher.logger
                .log("get_language_concept", &args, &Err::<DocContent, _>(&err))
                .await;
            Err(err)
        }
    }
}

pub async fn explain_error_code(
    fetcher: &DocFetcher,
    code: String,
) -> Result<DocContent, DocsFetchError> {
    let code = code.trim().to_uppercase();
    let args = json!({ "code": code });

    if code.is_empty() {
        return Err(DocsFetchError::InvalidInput("Error code cannot be empty".to_string()));
    }

    fetcher.logger
        .log("explain_error_code", &args, &"fetching...")
        .await;

    match fetcher.error_client.get_error_explanation(&code).await {
        Ok(content) => {
            let result = DocContent { content };
            fetcher.logger
                .log(
                    "explain_error_code",
                    &args,
                    &Ok::<_, DocsFetchError>(&result),
                )
                .await;
            Ok(result)
        }
        Err(e) => {
            let err = DocsFetchError::RequestError(format!(
                "Failed to fetch error explanation: {}",
                e
            ));
            fetcher.logger
                .log("explain_error_code", &args, &Err::<DocContent, _>(&err))
                .await;
            Err(err)
        }
    }
}

pub async fn doc_workflow_help(
    fetcher: &DocFetcher,
) -> Result<DocContent, DocsFetchError> {
    let help_text = r#"
🧭 RUST DOC MCP - WORKFLOW GUIDE

Follow this decision tree:

1️⃣ NEW CRATE? (Start Here)
   → Use: get_context_bundle("crate") (Replaces get_crate_overview + get_crate_modules)
   → Use: find_crates("query") if you're not sure which crate to use.
   → RETURNS: Bundle of README, Modules, and Feature Flags.
   
2️⃣ NEED TO FIND AN API BY TYPE?
   → Use: find_by_signature("crate", "fn(u32) -> String")
   → Use: find_trait_implementors("crate", "TraitName")
   
3️⃣ EXPLORE STRUCTURE?
   → Use: get_crate_modules("crate_name")
   → Example: get_crate_modules("tokio")
   → Returns: List of modules, structs, enums (API surface)
   
4️⃣ NEED SPECIFIC API DETAILS?
   → Use: get_symbol_docs("crate", "path::to::Symbol")
   → Example: get_symbol_docs("reqwest", "Client::get")
   → Example: get_symbol_docs("std", "vec::Vec::push")
   
5️⃣ LANGUAGE CONCEPTS & SPECS?
   → Use: get_language_concept("concept") (e.g., "ownership", "lifetimes")
   → Use: web_search site:doc.rust-lang.org/reference (for "formal grammar", "memory model")
   → Use: explain_error_code("E0382")

6️⃣ DEEP DIVE?
   → Use: get_crate_dependencies("crate", "version")
   → Use: get_crate_examples("crate")
   → Use: analyze_feature_flags("crate", "version")
   → Use: read_source_file("crate", "path/to/file.rs")
   
7️⃣ EXECUTION & VERIFICATION?
   → Use: cargo_check() (verify compilation)
   → Use: cargo_test() (run tests)
   → Use: cargo_clippy() (linting)
   → Use: cargo_fmt() (formatting)

8️⃣ DEBUGGING/RAW HTML/SOURCE?
   → Use: read_source_file("crate", "path/to/file.rs")
   → Use: expand_macro("path/to/file.rs") (debug macros)
   → Use: _fetch_raw_doc (advanced only)

❌ COMMON MISTAKES TO AVOID:
- Don't guess function names. Use get_crate_modules first.
- Don't read entire files if you just need a function signature.
- Don't forget to check feature flags if code fails to compile.
"#;
    let content = DocContent {
        content: help_text.to_string(),
    };
    fetcher.logger
        .log(
            "doc_workflow_help",
            &json!({}),
            &Ok::<_, DocsFetchError>(&content),
        )
        .await;
    Ok(content)
}
