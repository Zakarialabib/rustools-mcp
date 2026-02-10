use anyhow::Result;
use clap::{Parser, Subcommand};
use rustools_mcp::cache::InMemoryCache;
use rustools_mcp::mcp::DocFetcher;
use rustools_mcp::server;
use rustools_mcp::config::Config;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

use rustools_mcp::web_ui;

#[derive(Subcommand)]
enum Commands {
    /// Output the version and exit
    Version,
    /// Run the server in stdin/stdout mode
    Stdio {
        /// Enable debug logging
        #[arg(short, long)]
        debug: bool,

        /// Start web UI on this port (default: 3000)
        #[arg(long)]
        ui_port: Option<u16>,
    },
    /// Run the server with HTTP/SSE interface
    Http {
        /// Address to bind the HTTP server to
        #[arg(short, long)]
        address: Option<String>,

        /// Enable debug logging
        #[arg(short, long)]
        debug: bool,

        /// Start web UI on this port (default: 3000)
        #[arg(long)]
        ui_port: Option<u16>,
    },
    /// Test tools directly from the CLI
    Test {
        /// The tool to test (find_crates, get_crate_overview, get_crate_modules, get_symbol_docs, read_source_file, get_crate_dependencies, get_crate_examples, _fetch_raw_doc, doc_workflow_help)
        #[arg(long, default_value = "get_crate_overview")]
        tool: String,

        /// Crate name
        #[arg(long)]
        crate_name: Option<String>,

        /// Symbol/Item/File path
        #[arg(long)]
        item_path: Option<String>,

        /// Search query (for find_crates)
        #[arg(long)]
        query: Option<String>,

        /// Crate version (optional)
        #[arg(long)]
        version: Option<String>,

        /// Result limit (for find_crates)
        #[arg(long)]
        limit: Option<u32>,

        /// Start line (for read_source_file)
        #[arg(long)]
        start_line: Option<usize>,

        /// End line (for read_source_file)
        #[arg(long)]
        end_line: Option<usize>,

        /// Dependency kind (for get_crate_dependencies)
        #[arg(long)]
        kind: Option<String>,

        /// Arguments for cargo tools (e.g. cargo_check, cargo_test)
        #[arg(long, num_args = 0..)]
        args: Option<Vec<String>>,

        /// Working directory for cargo tools
        #[arg(long)]
        cwd: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load();

    match cli.command {
        Commands::Version => {
            println!("cratedocs version {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Commands::Stdio { debug, ui_port } => {
            if debug || config.log_level == "debug" {
                unsafe {
                    std::env::set_var("RUST_LOG", "debug");
                }
            }
            // Spawn UI server in background
            let port = ui_port.unwrap_or(config.ui_port);
            tokio::spawn(async move {
                web_ui::start_ui_server(port).await;
            });
            server::start_stdio_server().await
        }
        Commands::Http {
            address,
            debug,
            ui_port,
        } => {
            if debug || config.log_level == "debug" {
                unsafe {
                    std::env::set_var("RUST_LOG", "debug");
                }
            }
            // Spawn UI server in background
            let port = ui_port.unwrap_or(config.ui_port);
            tokio::spawn(async move {
                web_ui::start_ui_server(port).await;
            });
            let addr_str = address.unwrap_or(config.server_address);
            println!("Starting SSE server on {}", addr_str);
            let addr: std::net::SocketAddr = addr_str.parse().map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;
            server::start_server(addr).await
        }
        Commands::Test {
            tool,
            crate_name,
            item_path,
            query,
            version,
            limit,
            start_line,
            end_line,
            kind,
            args,
            cwd,
        } => {
            // Initialize cache with load/save for persistence testing
            let cache = InMemoryCache::new(config.cache_dir);
            if let Err(e) = rustools_mcp::cache::Cache::load(&cache).await {
                eprintln!("Warning: Failed to load cache: {}", e);
            }

            let fetcher = DocFetcher::new(cache.clone());

            match tool.as_str() {
                "find_crates" => {
                    let q = query.expect("query is required for find_crates");
                    let res = fetcher.find_crates(q, limit, None).await.map_err(|e| anyhow::anyhow!(e))?;
                    println!("{}", res.content);
                }
                "get_crate_overview" => {
                    let name = crate_name.expect("crate_name is required");
                    let res = fetcher.get_crate_overview(name, version).await.map_err(|e| anyhow::anyhow!(e))?;
                    println!("{}", res.content);
                }
                "get_crate_modules" => {
                    let name = crate_name.expect("crate_name is required");
                    let res = fetcher.get_crate_modules(name, version, limit.map(|l| l as usize)).await.map_err(|e| anyhow::anyhow!(e))?;
                    println!("{}", res.content);
                }
                "get_symbol_docs" => {
                    let name = crate_name.expect("crate_name is required");
                    let path = item_path.expect("item_path is required");
                    let res = fetcher.get_symbol_docs(name, path, version).await.map_err(|e| anyhow::anyhow!(e))?;
                    println!("{}", res.content);
                }
                "read_source_file" => {
                    let name = crate_name.expect("crate_name is required");
                    let path = item_path.expect("item_path is required");
                    let res = fetcher.read_source_file(name, path, version, start_line, end_line).await.map_err(|e| anyhow::anyhow!(e))?;
                    println!("{}", res.content);
                }
                "get_crate_dependencies" => {
                    let name = crate_name.expect("crate_name is required");
                    let ver = version.expect("version is required for dependencies");
                    let res = fetcher.get_crate_dependencies(name, ver, kind).await.map_err(|e| anyhow::anyhow!(e))?;
                    println!("{}", res.content);
                }
                "get_crate_examples" => {
                    let name = crate_name.expect("crate_name is required");
                    let res = fetcher.get_crate_examples(name, version, limit.map(|l| l as usize)).await.map_err(|e| anyhow::anyhow!(e))?;
                    println!("{}", res.content);
                }
                "analyze_feature_flags" => {
                    let name = crate_name.expect("crate_name is required");
                    let ver = version.expect("version is required for feature flags");
                    let res = fetcher.analyze_feature_flags(name, ver).await.map_err(|e| anyhow::anyhow!(e))?;
                    println!("{}", res.content);
                }
                "find_trait_implementors" => {
                    let name = crate_name.expect("crate_name is required");
                    let trait_path = item_path.expect("item_path (trait) is required");
                    let res = fetcher.find_trait_implementors(name, trait_path, version, limit.map(|l| l as usize)).await.map_err(|e| anyhow::anyhow!(e))?;
                    println!("{}", res.content);
                }
                "get_context_bundle" => {
                    let name = crate_name.expect("crate_name is required");
                    let res = fetcher.get_context_bundle(name, version).await.map_err(|e| anyhow::anyhow!(e))?;
                    println!("{}", res.content);
                }
                "find_by_signature" => {
                    let name = crate_name.expect("crate_name is required");
                    let pattern = query.expect("query (signature pattern) is required");
                    let res = fetcher.find_by_signature(name, pattern, version).await.map_err(|e| anyhow::anyhow!(e))?;
                    println!("{}", res.content);
                }
                "_fetch_raw_doc" => {
                    let name = crate_name.expect("crate_name is required");
                    let ver = version.expect("version is required");
                    let path = item_path.expect("item_path is required");
                    let res = fetcher._fetch_raw_doc(name, ver, path).await.map_err(|e| anyhow::anyhow!(e))?;
                    println!("{}", res.content);
                }
                "doc_workflow_help" => {
                    let res = fetcher.doc_workflow_help().await.map_err(|e| anyhow::anyhow!(e))?;
                    println!("{}", res.content);
                }
                "cargo_check" => {
                    let res = fetcher.cargo_check(args, cwd).await.map_err(|e| anyhow::anyhow!(e))?;
                    println!("{}", res.content);
                }
                "cargo_test" => {
                    let res = fetcher.cargo_test(args, cwd).await.map_err(|e| anyhow::anyhow!(e))?;
                    println!("{}", res.content);
                }
                "cargo_clippy" => {
                    let res = fetcher.cargo_clippy(args, cwd).await.map_err(|e| anyhow::anyhow!(e))?;
                    println!("{}", res.content);
                }
                "cargo_fmt" => {
                    let res = fetcher.cargo_fmt(args, cwd).await.map_err(|e| anyhow::anyhow!(e))?;
                    println!("{}", res.content);
                }
                "expand_macro" => {
                    let path = item_path.expect("item_path is required for expand_macro");
                    let item = query; // reuse query for item if needed, or maybe I should check arguments mapping
                    let res = fetcher.expand_macro(path, item, cwd).await.map_err(|e| anyhow::anyhow!(e))?;
                    println!("{}", res.content);
                }
                "get_language_concept" => {
                    let concept = query.expect("query is required for get_language_concept");
                    let res = fetcher.get_language_concept(concept).await.map_err(|e| anyhow::anyhow!(e))?;
                    println!("{}", res.content);
                }
                "explain_error_code" => {
                    let code = query.expect("query is required for explain_error_code");
                    let res = fetcher.explain_error_code(code).await.map_err(|e| anyhow::anyhow!(e))?;
                    println!("{}", res.content);
                }
                "cargo_tree" => {
                    let res = fetcher.cargo_tree(args, cwd).await.map_err(|e| anyhow::anyhow!(e))?;
                    println!("{}", res.content);
                }
                "cargo_bench" => {
                    let res = fetcher.cargo_bench(args, cwd).await.map_err(|e| anyhow::anyhow!(e))?;
                    println!("{}", res.content);
                }
                "cargo_semver_checks" => {
                    let res = fetcher.cargo_semver_checks(args, cwd).await.map_err(|e| anyhow::anyhow!(e))?;
                    println!("{}", res.content);
                }
                "get_local_doc" => {
                    let path = item_path.expect("item_path is required for get_local_doc");
                    let res = fetcher.get_local_doc(path, cwd).await.map_err(|e| anyhow::anyhow!(e))?;
                    println!("{}", res.content);
                }
                _ => {
                    println!("Unknown tool: {}", tool);
                }
            }
            Ok(())
        }
    }
}
