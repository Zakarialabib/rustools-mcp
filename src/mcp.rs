use rmcp::{RoleServer, ServerHandler, service::RequestContext, model::{CallToolRequestParams, CallToolResult, InitializeRequestParams, InitializeResult, ListToolsResult, PaginatedRequestParams}, ErrorData};

use crate::logger::RequestLogger;
use crate::docs_parser::DocsRsClient;
use crate::cache::InMemoryCache;
use crate::rust_book::RustBookClient;
use crate::error_index::ErrorIndexClient;
use crate::tools::{discovery, cargo, analysis, learning, structure};
use crate::tools::discovery::{FindCratesArgs, GetCrateOverviewArgs, GetContextBundleArgs};
use crate::tools::structure::{GetCrateModulesArgs, ReadSourceFileArgs, GetSymbolDocsArgs, FetchRawDocArgs};
use crate::tools::analysis::{GetCrateDependenciesArgs, AnalyzeFeatureFlagsArgs, FindTraitImplementorsArgs, GetCrateExamplesArgs, FindBySignatureArgs};
use crate::tools::learning::{GetLanguageConceptArgs, ExplainErrorCodeArgs, DocWorkflowHelpArgs};
use crate::tools::local::{self, GetLocalDocArgs};
use crate::tools::project_context::{analyze_project_context, AnalyzeProjectContextArgs};
use crate::tools::cargo::{CargoCheckArgs, CargoTestArgs, CargoClippyArgs, CargoFmtArgs, ExpandMacroArgs, CargoTreeArgs, CargoBenchArgs, CargoSemverChecksArgs, CargoAuditArgs};
use crate::prompts::SYSTEM_PROMPT;

#[derive(Clone)]
pub struct DocFetcher {
    pub book_client: RustBookClient,
    pub error_client: ErrorIndexClient,
    pub logger: RequestLogger,
    pub client: DocsRsClient,
    pub cache: InMemoryCache,
}

impl DocFetcher {
    pub fn new(cache: InMemoryCache) -> Self {
        Self {
            book_client: RustBookClient::new(),
            error_client: ErrorIndexClient::new(),
            logger: RequestLogger::new("requests.log"),
            client: DocsRsClient::new(),
            cache,
        }
    }

    pub async fn find_crates(&self, query: String, limit: Option<u32>, fuzzy: Option<bool>) -> Result<crate::docs_parser::DocContent, Box<dyn std::error::Error + Send + Sync>> {
        Ok(discovery::find_crates(self, query, limit, fuzzy).await?)
    }

    pub async fn get_crate_overview(&self, crate_name: String, version: Option<String>) -> Result<crate::docs_parser::DocContent, Box<dyn std::error::Error + Send + Sync>> {
        Ok(discovery::get_crate_overview(self, crate_name, version).await?)
    }

    pub async fn get_context_bundle(&self, crate_name: String, version: Option<String>) -> Result<crate::docs_parser::DocContent, Box<dyn std::error::Error + Send + Sync>> {
        Ok(analysis::get_context_bundle(self, crate_name, version).await?)
    }

    pub async fn cargo_check(&self, args: Option<Vec<String>>, cwd: Option<String>) -> Result<crate::docs_parser::DocContent, Box<dyn std::error::Error + Send + Sync>> {
        Ok(cargo::cargo_check(self, args, cwd).await?)
    }

    pub async fn cargo_test(&self, args: Option<Vec<String>>, cwd: Option<String>) -> Result<crate::docs_parser::DocContent, Box<dyn std::error::Error + Send + Sync>> {
        Ok(cargo::cargo_test(self, args, cwd).await?)
    }

    pub async fn cargo_clippy(&self, args: Option<Vec<String>>, cwd: Option<String>) -> Result<crate::docs_parser::DocContent, Box<dyn std::error::Error + Send + Sync>> {
        Ok(cargo::cargo_clippy(self, args, cwd).await?)
    }

    pub async fn cargo_fmt(&self, args: Option<Vec<String>>, cwd: Option<String>) -> Result<crate::docs_parser::DocContent, Box<dyn std::error::Error + Send + Sync>> {
        Ok(cargo::cargo_fmt(self, args, cwd).await?)
    }

    pub async fn cargo_tree(&self, args: Option<Vec<String>>, cwd: Option<String>) -> Result<crate::docs_parser::DocContent, Box<dyn std::error::Error + Send + Sync>> {
        Ok(cargo::cargo_tree(self, args, cwd).await?)
    }

    pub async fn cargo_bench(&self, args: Option<Vec<String>>, cwd: Option<String>) -> Result<crate::docs_parser::DocContent, Box<dyn std::error::Error + Send + Sync>> {
        Ok(cargo::cargo_bench(self, args, cwd).await?)
    }

    pub async fn cargo_semver_checks(&self, args: Option<Vec<String>>, cwd: Option<String>) -> Result<crate::docs_parser::DocContent, Box<dyn std::error::Error + Send + Sync>> {
        Ok(cargo::cargo_semver_checks(self, args, cwd).await?)
    }

    pub async fn cargo_audit(&self, args: Option<Vec<String>>, cwd: Option<String>) -> Result<crate::docs_parser::DocContent, Box<dyn std::error::Error + Send + Sync>> {
        Ok(cargo::cargo_audit(self, args, cwd).await?)
    }

    pub async fn expand_macro(&self, path: String, item: Option<String>, cwd: Option<String>) -> Result<crate::docs_parser::DocContent, Box<dyn std::error::Error + Send + Sync>> {
        Ok(cargo::expand_macro(self, path, item, cwd).await?)
    }

    pub async fn get_language_concept(&self, concept: String) -> Result<crate::docs_parser::DocContent, Box<dyn std::error::Error + Send + Sync>> {
        Ok(learning::get_language_concept(self, concept).await?)
    }

    pub async fn explain_error_code(&self, code: String) -> Result<crate::docs_parser::DocContent, Box<dyn std::error::Error + Send + Sync>> {
        Ok(learning::explain_error_code(self, code).await?)
    }

    pub async fn doc_workflow_help(&self) -> Result<crate::docs_parser::DocContent, Box<dyn std::error::Error + Send + Sync>> {
        Ok(learning::doc_workflow_help(self).await?)
    }

    pub async fn _fetch_raw_doc(&self, crate_name: String, version: String, path: String) -> Result<crate::docs_parser::DocContent, Box<dyn std::error::Error + Send + Sync>> {
        Ok(structure::_fetch_raw_doc(self, crate_name, version, path).await?)
    }

    pub async fn read_source_file(&self, crate_name: String, path: String, version: Option<String>, start_line: Option<usize>, end_line: Option<usize>) -> Result<crate::docs_parser::DocContent, Box<dyn std::error::Error + Send + Sync>> {
        Ok(structure::read_source_file(self, crate_name, path, version, start_line, end_line).await?)
    }

    pub async fn get_crate_modules(&self, crate_name: String, version: Option<String>, limit: Option<usize>) -> Result<crate::docs_parser::DocContent, Box<dyn std::error::Error + Send + Sync>> {
        Ok(structure::get_crate_modules(self, crate_name, version, limit).await?)
    }

    pub async fn get_symbol_docs(&self, crate_name: String, symbol_path: String, version: Option<String>) -> Result<crate::docs_parser::DocContent, Box<dyn std::error::Error + Send + Sync>> {
        Ok(structure::get_symbol_docs(self, crate_name, symbol_path, version).await?)
    }

    pub async fn get_crate_dependencies(&self, crate_name: String, version: String, kind: Option<String>) -> Result<crate::docs_parser::DocContent, Box<dyn std::error::Error + Send + Sync>> {
        Ok(analysis::get_crate_dependencies(self, crate_name, version, kind).await?)
    }

    pub async fn analyze_feature_flags(&self, crate_name: String, version: String) -> Result<crate::docs_parser::DocContent, Box<dyn std::error::Error + Send + Sync>> {
        Ok(analysis::analyze_feature_flags(self, crate_name, version).await?)
    }

    pub async fn find_trait_implementors(&self, crate_name: String, trait_path: String, version: Option<String>, limit: Option<usize>) -> Result<crate::docs_parser::DocContent, Box<dyn std::error::Error + Send + Sync>> {
        Ok(analysis::find_trait_implementors(self, crate_name, trait_path, version, limit).await?)
    }

    pub async fn get_crate_examples(&self, crate_name: String, version: Option<String>, limit: Option<usize>) -> Result<crate::docs_parser::DocContent, Box<dyn std::error::Error + Send + Sync>> {
        Ok(crate::tools::analysis::get_crate_examples(self, crate_name, version, limit).await?)
    }

    pub async fn find_by_signature(&self, crate_name: String, signature_pattern: String, version: Option<String>) -> Result<crate::docs_parser::DocContent, Box<dyn std::error::Error + Send + Sync>> {
        Ok(crate::tools::analysis::find_by_signature(self, crate_name, signature_pattern, version).await?)
    }

    pub async fn get_local_doc(&self, path: String, cwd: Option<String>) -> Result<crate::docs_parser::DocContent, Box<dyn std::error::Error + Send + Sync>> {
        Ok(local::get_local_doc(GetLocalDocArgs { path, cwd })?)
    }
}

impl ServerHandler for DocFetcher {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        rmcp::model::ServerInfo {
            protocol_version: rmcp::model::ProtocolVersion::LATEST,
            capabilities: rmcp::model::ServerCapabilities::default(),
            server_info: rmcp::model::Implementation {
                name: "rustools-mcp".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                ..rmcp::model::Implementation::default()
            },
            instructions: Some(SYSTEM_PROMPT.into()),
        }
    }

    async fn initialize(
        &self,
        _params: InitializeRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, ErrorData> {
        Ok(InitializeResult::default())
    }

    async fn list_tools(
        &self,
        _req: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult {
            tools: vec![
                rmcp::model::Tool {
                    name: "find_crates".into(),
                    description: Some("Find crates on crates.io".into()),
                    input_schema: schema_to_arc_map(schemars::schema_for!(FindCratesArgs)),
                    annotations: None, icons: None, meta: None, title: None, output_schema: None,
                },
                rmcp::model::Tool {
                    name: "get_crate_overview".into(),
                    description: Some("Get crate overview (README)".into()),
                    input_schema: schema_to_arc_map(schemars::schema_for!(GetCrateOverviewArgs)),
                    annotations: None, icons: None, meta: None, title: None, output_schema: None,
                },
                rmcp::model::Tool {
                    name: "get_context_bundle".into(),
                    description: Some("Get crate context (README, modules, features)".into()),
                    input_schema: schema_to_arc_map(schemars::schema_for!(GetContextBundleArgs)),
                    annotations: None, icons: None, meta: None, title: None, output_schema: None,
                },
                rmcp::model::Tool {
                    name: "cargo_check".into(),
                    description: Some("Run cargo check".into()),
                    input_schema: schema_to_arc_map(schemars::schema_for!(CargoCheckArgs)),
                    annotations: None, icons: None, meta: None, title: None, output_schema: None,
                },
                rmcp::model::Tool {
                    name: "cargo_test".into(),
                    description: Some("Run cargo test".into()),
                    input_schema: schema_to_arc_map(schemars::schema_for!(CargoTestArgs)),
                    annotations: None, icons: None, meta: None, title: None, output_schema: None,
                },
                rmcp::model::Tool {
                    name: "cargo_clippy".into(),
                    description: Some("Run cargo clippy".into()),
                    input_schema: schema_to_arc_map(schemars::schema_for!(CargoClippyArgs)),
                    annotations: None, icons: None, meta: None, title: None, output_schema: None,
                },
                rmcp::model::Tool {
                    name: "cargo_fmt".into(),
                    description: Some("Run cargo fmt".into()),
                    input_schema: schema_to_arc_map(schemars::schema_for!(CargoFmtArgs)),
                    annotations: None, icons: None, meta: None, title: None, output_schema: None,
                },
                rmcp::model::Tool {
                    name: "cargo_tree".into(),
                    description: Some("Run cargo tree".into()),
                    input_schema: schema_to_arc_map(schemars::schema_for!(CargoTreeArgs)),
                    annotations: None, icons: None, meta: None, title: None, output_schema: None,
                },
                rmcp::model::Tool {
                    name: "cargo_bench".into(),
                    description: Some("Run cargo bench".into()),
                    input_schema: schema_to_arc_map(schemars::schema_for!(CargoBenchArgs)),
                    annotations: None, icons: None, meta: None, title: None, output_schema: None,
                },
                rmcp::model::Tool {
                    name: "cargo_semver_checks".into(),
                    description: Some("Run cargo semver-checks to check for breaking changes".into()),
                    input_schema: schema_to_arc_map(schemars::schema_for!(CargoSemverChecksArgs)),
                    annotations: None, icons: None, meta: None, title: None, output_schema: None,
                },
                rmcp::model::Tool {
                    name: "cargo_audit".into(),
                    description: Some("Run cargo audit to check for security vulnerabilities".into()),
                    input_schema: schema_to_arc_map(schemars::schema_for!(CargoAuditArgs)),
                    annotations: None, icons: None, meta: None, title: None, output_schema: None,
                },
                rmcp::model::Tool {
                    name: "expand_macro".into(),
                    description: Some("Expand macros using cargo-expand".into()),
                    input_schema: schema_to_arc_map(schemars::schema_for!(ExpandMacroArgs)),
                    annotations: None, icons: None, meta: None, title: None, output_schema: None,
                },
                rmcp::model::Tool {
                    name: "get_language_concept".into(),
                    description: Some("Get explanation of a Rust concept".into()),
                    input_schema: schema_to_arc_map(schemars::schema_for!(GetLanguageConceptArgs)),
                    annotations: None, icons: None, meta: None, title: None, output_schema: None,
                },
                rmcp::model::Tool {
                    name: "explain_error_code".into(),
                    description: Some("Explain a Rust error code".into()),
                    input_schema: schema_to_arc_map(schemars::schema_for!(ExplainErrorCodeArgs)),
                    annotations: None, icons: None, meta: None, title: None, output_schema: None,
                },
                rmcp::model::Tool {
                    name: "doc_workflow_help".into(),
                    description: Some("Get help on how to use the documentation tools".into()),
                    input_schema: schema_to_arc_map(schemars::schema_for!(DocWorkflowHelpArgs)),
                    annotations: None, icons: None, meta: None, title: None, output_schema: None,
                },
                rmcp::model::Tool {
                    name: "read_source_file".into(),
                    description: Some("Read source file content".into()),
                    input_schema: schema_to_arc_map(schemars::schema_for!(ReadSourceFileArgs)),
                    annotations: None, icons: None, meta: None, title: None, output_schema: None,
                },
                rmcp::model::Tool {
                    name: "get_crate_modules".into(),
                    description: Some("Get crate modules structure".into()),
                    input_schema: schema_to_arc_map(schemars::schema_for!(GetCrateModulesArgs)),
                    annotations: None, icons: None, meta: None, title: None, output_schema: None,
                },
                rmcp::model::Tool {
                    name: "get_symbol_docs".into(),
                    description: Some("Get documentation for a specific symbol".into()),
                    input_schema: schema_to_arc_map(schemars::schema_for!(GetSymbolDocsArgs)),
                    annotations: None, icons: None, meta: None, title: None, output_schema: None,
                },
                rmcp::model::Tool {
                    name: "get_crate_dependencies".into(),
                    description: Some("Get crate dependencies".into()),
                    input_schema: schema_to_arc_map(schemars::schema_for!(GetCrateDependenciesArgs)),
                    annotations: None, icons: None, meta: None, title: None, output_schema: None,
                },
                rmcp::model::Tool {
                    name: "analyze_feature_flags".into(),
                    description: Some("Analyze feature flags".into()),
                    input_schema: schema_to_arc_map(schemars::schema_for!(AnalyzeFeatureFlagsArgs)),
                    annotations: None, icons: None, meta: None, title: None, output_schema: None,
                },
                rmcp::model::Tool {
                    name: "find_trait_implementors".into(),
                    description: Some("Find implementors of a trait".into()),
                    input_schema: schema_to_arc_map(schemars::schema_for!(FindTraitImplementorsArgs)),
                    annotations: None, icons: None, meta: None, title: None, output_schema: None,
                },
                rmcp::model::Tool {
                    name: "get_crate_examples".into(),
                    description: Some("Get crate examples".into()),
                    input_schema: schema_to_arc_map(schemars::schema_for!(GetCrateExamplesArgs)),
                    annotations: None, icons: None, meta: None, title: None, output_schema: None,
                },
                rmcp::model::Tool {
                    name: "find_by_signature".into(),
                    description: Some("Find functions by signature".into()),
                    input_schema: schema_to_arc_map(schemars::schema_for!(FindBySignatureArgs)),
                    annotations: None, icons: None, meta: None, title: None, output_schema: None,
                },
                rmcp::model::Tool {
                    name: "_fetch_raw_doc".into(),
                    description: Some("[INTERNAL] Fetch raw HTML documentation".into()),
                    input_schema: schema_to_arc_map(schemars::schema_for!(FetchRawDocArgs)),
                    annotations: None, icons: None, meta: None, title: None, output_schema: None,
                },
                rmcp::model::Tool {
                    name: "get_local_doc".into(),
                    description: Some("Get documentation from a local HTML file".into()),
                    input_schema: schema_to_arc_map(schemars::schema_for!(GetLocalDocArgs)),
                    annotations: None, icons: None, meta: None, title: None, output_schema: None,
                },
                rmcp::model::Tool {
                    name: "analyze_project_context".into(),
                    description: Some("Analyze project structure and dependencies".into()),
                    input_schema: schema_to_arc_map(schemars::schema_for!(AnalyzeProjectContextArgs)),
                    annotations: None, icons: None, meta: None, title: None, output_schema: None,
                },
            ],
            next_cursor: None,
            meta: None,
        })
    }

    async fn call_tool(
        &self,
        req: CallToolRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let name = req.name;
        let arguments = req.arguments.unwrap_or_default();

        match name.as_ref() {
            "find_crates" => {
                let args: FindCratesArgs = serde_json::from_value(serde_json::Value::Object(arguments)).map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32602),
                    message: format!("Invalid arguments for find_crates: {}", e).into(),
                    data: None,
                })?;
                let result = discovery::find_crates(self, args.query, args.limit, args.fuzzy).await.map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32000),
                    message: e.to_string().into(),
                    data: None,
                })?;
                Ok(CallToolResult { content: vec![rmcp::model::Content::text(result.content)], is_error: None, meta: None, structured_content: None })
            }
            "get_crate_overview" => {
                let args: GetCrateOverviewArgs = serde_json::from_value(serde_json::Value::Object(arguments)).map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32602),
                    message: format!("Invalid arguments for get_crate_overview: {}", e).into(),
                    data: None,
                })?;
                let result = self.get_crate_overview(args.crate_name, args.version).await.map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32000),
                    message: e.to_string().into(),
                    data: None,
                })?;
                Ok(CallToolResult { content: vec![rmcp::model::Content::text(result.content)], is_error: None, meta: None, structured_content: None })
            }
            "get_context_bundle" => {
                let args: GetContextBundleArgs = serde_json::from_value(serde_json::Value::Object(arguments)).map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32602),
                    message: format!("Invalid arguments for get_context_bundle: {}", e).into(),
                    data: None,
                })?;
                let result = self.get_context_bundle(args.crate_name, args.version).await.map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32000),
                    message: e.to_string().into(),
                    data: None,
                })?;
                Ok(CallToolResult { content: vec![rmcp::model::Content::text(result.content)], is_error: None, meta: None, structured_content: None })
            }
            "cargo_check" => {
                let args: CargoCheckArgs = serde_json::from_value(serde_json::Value::Object(arguments)).map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32602),
                    message: format!("Invalid arguments for cargo_check: {}", e).into(),
                    data: None,
                })?;
                let result = self.cargo_check(args.args, args.cwd).await.map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32000),
                    message: e.to_string().into(),
                    data: None,
                })?;
                Ok(CallToolResult { content: vec![rmcp::model::Content::text(result.content)], is_error: None, meta: None, structured_content: None })
            }
            "cargo_test" => {
                let args: CargoTestArgs = serde_json::from_value(serde_json::Value::Object(arguments)).map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32602),
                    message: format!("Invalid arguments for cargo_test: {}", e).into(),
                    data: None,
                })?;
                let result = self.cargo_test(args.args, args.cwd).await.map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32000),
                    message: e.to_string().into(),
                    data: None,
                })?;
                Ok(CallToolResult { content: vec![rmcp::model::Content::text(result.content)], is_error: None, meta: None, structured_content: None })
            }
            "cargo_clippy" => {
                let args: CargoClippyArgs = serde_json::from_value(serde_json::Value::Object(arguments)).map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32602),
                    message: format!("Invalid arguments for cargo_clippy: {}", e).into(),
                    data: None,
                })?;
                let result = self.cargo_clippy(args.args, args.cwd).await.map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32000),
                    message: e.to_string().into(),
                    data: None,
                })?;
                Ok(CallToolResult { content: vec![rmcp::model::Content::text(result.content)], is_error: None, meta: None, structured_content: None })
            }
            "cargo_fmt" => {
                let args: CargoFmtArgs = serde_json::from_value(serde_json::Value::Object(arguments)).map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32602),
                    message: format!("Invalid arguments for cargo_fmt: {}", e).into(),
                    data: None,
                })?;
                let result = self.cargo_fmt(args.args, args.cwd).await.map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32000),
                    message: e.to_string().into(),
                    data: None,
                })?;
                Ok(CallToolResult { content: vec![rmcp::model::Content::text(result.content)], is_error: None, meta: None, structured_content: None })
            }
            "cargo_tree" => {
                let args: CargoTreeArgs = serde_json::from_value(serde_json::Value::Object(arguments)).map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32602),
                    message: format!("Invalid arguments for cargo_tree: {}", e).into(),
                    data: None,
                })?;
                let result = self.cargo_tree(args.args, args.cwd).await.map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32000),
                    message: e.to_string().into(),
                    data: None,
                })?;
                Ok(CallToolResult { content: vec![rmcp::model::Content::text(result.content)], is_error: None, meta: None, structured_content: None })
            }
            "cargo_bench" => {
                let args: CargoBenchArgs = serde_json::from_value(serde_json::Value::Object(arguments)).map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32602),
                    message: format!("Invalid arguments for cargo_bench: {}", e).into(),
                    data: None,
                })?;
                let result = self.cargo_bench(args.args, args.cwd).await.map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32000),
                    message: e.to_string().into(),
                    data: None,
                })?;
                Ok(CallToolResult { content: vec![rmcp::model::Content::text(result.content)], is_error: None, meta: None, structured_content: None })
            }
            "cargo_semver_checks" => {
                let args: CargoSemverChecksArgs = serde_json::from_value(serde_json::Value::Object(arguments)).map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32602),
                    message: format!("Invalid arguments for cargo_semver_checks: {}", e).into(),
                    data: None,
                })?;
                let result = self.cargo_semver_checks(args.args, args.cwd).await.map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32000),
                    message: e.to_string().into(),
                    data: None,
                })?;
                Ok(CallToolResult { content: vec![rmcp::model::Content::text(result.content)], is_error: None, meta: None, structured_content: None })
            }
            "cargo_audit" => {
                let args: CargoAuditArgs = serde_json::from_value(serde_json::Value::Object(arguments)).map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32602),
                    message: format!("Invalid arguments for cargo_audit: {}", e).into(),
                    data: None,
                })?;
                let result = self.cargo_audit(args.args, args.cwd).await.map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32000),
                    message: e.to_string().into(),
                    data: None,
                })?;
                Ok(CallToolResult { content: vec![rmcp::model::Content::text(result.content)], is_error: None, meta: None, structured_content: None })
            }
            "expand_macro" => {
                let args: ExpandMacroArgs = serde_json::from_value(serde_json::Value::Object(arguments)).map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32602),
                    message: format!("Invalid arguments for expand_macro: {}", e).into(),
                    data: None,
                })?;
                let result = self.expand_macro(args.path, args.item, args.cwd).await.map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32000),
                    message: e.to_string().into(),
                    data: None,
                })?;
                Ok(CallToolResult { content: vec![rmcp::model::Content::text(result.content)], is_error: None, meta: None, structured_content: None })
            }
            "get_language_concept" => {
                let args: GetLanguageConceptArgs = serde_json::from_value(serde_json::Value::Object(arguments)).map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32602),
                    message: format!("Invalid arguments for get_language_concept: {}", e).into(),
                    data: None,
                })?;
                let result = self.get_language_concept(args.concept).await.map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32000),
                    message: e.to_string().into(),
                    data: None,
                })?;
                Ok(CallToolResult { content: vec![rmcp::model::Content::text(result.content)], is_error: None, meta: None, structured_content: None })
            }
            "explain_error_code" => {
                let args: ExplainErrorCodeArgs = serde_json::from_value(serde_json::Value::Object(arguments)).map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32602),
                    message: format!("Invalid arguments for explain_error_code: {}", e).into(),
                    data: None,
                })?;
                let result = self.explain_error_code(args.code).await.map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32000),
                    message: e.to_string().into(),
                    data: None,
                })?;
                Ok(CallToolResult { content: vec![rmcp::model::Content::text(result.content)], is_error: None, meta: None, structured_content: None })
            }
            "doc_workflow_help" => {
                let _args: DocWorkflowHelpArgs = serde_json::from_value(serde_json::Value::Object(arguments)).map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32602),
                    message: format!("Invalid arguments for doc_workflow_help: {}", e).into(),
                    data: None,
                })?;
                let result = self.doc_workflow_help().await.map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32000),
                    message: e.to_string().into(),
                    data: None,
                })?;
                Ok(CallToolResult { content: vec![rmcp::model::Content::text(result.content)], is_error: None, meta: None, structured_content: None })
            }
            "read_source_file" => {
                let args: ReadSourceFileArgs = serde_json::from_value(serde_json::Value::Object(arguments)).map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32602),
                    message: format!("Invalid arguments for read_source_file: {}", e).into(),
                    data: None,
                })?;
                let result = self.read_source_file(args.crate_name, args.path, args.version, args.start_line, args.end_line).await.map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32000),
                    message: e.to_string().into(),
                    data: None,
                })?;
                Ok(CallToolResult { content: vec![rmcp::model::Content::text(result.content)], is_error: None, meta: None, structured_content: None })
            }
            "get_crate_modules" => {
                let args: GetCrateModulesArgs = serde_json::from_value(serde_json::Value::Object(arguments)).map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32602),
                    message: format!("Invalid arguments for get_crate_modules: {}", e).into(),
                    data: None,
                })?;
                let result = self.get_crate_modules(args.crate_name, args.version, args.limit).await.map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32000),
                    message: e.to_string().into(),
                    data: None,
                })?;
                Ok(CallToolResult { content: vec![rmcp::model::Content::text(result.content)], is_error: None, meta: None, structured_content: None })
            }
            "get_symbol_docs" => {
                let args: GetSymbolDocsArgs = serde_json::from_value(serde_json::Value::Object(arguments)).map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32602),
                    message: format!("Invalid arguments for get_symbol_docs: {}", e).into(),
                    data: None,
                })?;
                let result = self.get_symbol_docs(args.crate_name, args.symbol_path, args.version).await.map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32000),
                    message: e.to_string().into(),
                    data: None,
                })?;
                Ok(CallToolResult { content: vec![rmcp::model::Content::text(result.content)], is_error: None, meta: None, structured_content: None })
            }
            "get_crate_dependencies" => {
                let args: GetCrateDependenciesArgs = serde_json::from_value(serde_json::Value::Object(arguments)).map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32602),
                    message: format!("Invalid arguments for get_crate_dependencies: {}", e).into(),
                    data: None,
                })?;
                let result = self.get_crate_dependencies(args.crate_name, args.version, args.kind).await.map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32000),
                    message: e.to_string().into(),
                    data: None,
                })?;
                Ok(CallToolResult { content: vec![rmcp::model::Content::text(result.content)], is_error: None, meta: None, structured_content: None })
            }
            "analyze_feature_flags" => {
                let args: AnalyzeFeatureFlagsArgs = serde_json::from_value(serde_json::Value::Object(arguments)).map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32602),
                    message: format!("Invalid arguments for analyze_feature_flags: {}", e).into(),
                    data: None,
                })?;
                let result = self.analyze_feature_flags(args.crate_name, args.version).await.map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32000),
                    message: e.to_string().into(),
                    data: None,
                })?;
                Ok(CallToolResult { content: vec![rmcp::model::Content::text(result.content)], is_error: None, meta: None, structured_content: None })
            }
            "find_trait_implementors" => {
                let args: FindTraitImplementorsArgs = serde_json::from_value(serde_json::Value::Object(arguments)).map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32602),
                    message: format!("Invalid arguments for find_trait_implementors: {}", e).into(),
                    data: None,
                })?;
                let result = self.find_trait_implementors(args.crate_name, args.trait_path, args.version, args.limit).await.map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32000),
                    message: e.to_string().into(),
                    data: None,
                })?;
                Ok(CallToolResult { content: vec![rmcp::model::Content::text(result.content)], is_error: None, meta: None, structured_content: None })
            }
            "get_crate_examples" => {
                let args: GetCrateExamplesArgs = serde_json::from_value(serde_json::Value::Object(arguments)).map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32602),
                    message: format!("Invalid arguments for get_crate_examples: {}", e).into(),
                    data: None,
                })?;
                let result = self.get_crate_examples(args.crate_name, args.version, args.limit).await.map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32000),
                    message: e.to_string().into(),
                    data: None,
                })?;
                Ok(CallToolResult { content: vec![rmcp::model::Content::text(result.content)], is_error: None, meta: None, structured_content: None })
            }
            "find_by_signature" => {
                let args: FindBySignatureArgs = serde_json::from_value(serde_json::Value::Object(arguments)).map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32602),
                    message: format!("Invalid arguments for find_by_signature: {}", e).into(),
                    data: None,
                })?;
                let result = self.find_by_signature(args.crate_name, args.signature_pattern, args.version).await.map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32000),
                    message: e.to_string().into(),
                    data: None,
                })?;
                Ok(CallToolResult { content: vec![rmcp::model::Content::text(result.content)], is_error: None, meta: None, structured_content: None })
            }
            "_fetch_raw_doc" => {
                let args: FetchRawDocArgs = serde_json::from_value(serde_json::Value::Object(arguments)).map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32602),
                    message: format!("Invalid arguments for _fetch_raw_doc: {}", e).into(),
                    data: None,
                })?;
                let result = self._fetch_raw_doc(args.crate_name, args.version, args.path).await.map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32000),
                    message: e.to_string().into(),
                    data: None,
                })?;
                Ok(CallToolResult { content: vec![rmcp::model::Content::text(result.content)], is_error: None, meta: None, structured_content: None })
            }
            "get_local_doc" => {
                let args: GetLocalDocArgs = serde_json::from_value(serde_json::Value::Object(arguments)).map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32602),
                    message: format!("Invalid arguments for get_local_doc: {}", e).into(),
                    data: None,
                })?;
                let result = self.get_local_doc(args.path, args.cwd).await.map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32000),
                    message: e.to_string().into(),
                    data: None,
                })?;
                Ok(CallToolResult { content: vec![rmcp::model::Content::text(result.content)], is_error: None, meta: None, structured_content: None })
            }
            "analyze_project_context" => {
                let args: AnalyzeProjectContextArgs = serde_json::from_value(serde_json::Value::Object(arguments)).map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32602),
                    message: format!("Invalid arguments for analyze_project_context: {}", e).into(),
                    data: None,
                })?;
                let result = analyze_project_context(args).await.map_err(|e| ErrorData {
                    code: rmcp::model::ErrorCode(-32000),
                    message: e.to_string().into(),
                    data: None,
                })?;
                Ok(CallToolResult { content: vec![rmcp::model::Content::text(result.content)], is_error: None, meta: None, structured_content: None })
            }
            _ => Err(ErrorData {
                code: rmcp::model::ErrorCode(-32601),
                message: format!("Tool not found: {}", name).into(),
                data: None,
            }),
        }
    }
}

fn schema_to_arc_map<T: serde::Serialize>(schema: T) -> std::sync::Arc<serde_json::Map<String, serde_json::Value>> {
    let value = serde_json::to_value(schema).unwrap_or(serde_json::Value::Null);
    match value {
        serde_json::Value::Object(map) => std::sync::Arc::new(map),
        _ => std::sync::Arc::new(serde_json::Map::new()),
    }
}
