use anyhow::Result;
use clap::{Parser, Subcommand};
use rustools_mcp::server;
use rustools_mcp::mcp::DocFetcher;
use rustools_mcp::cache::InMemoryCache;
use std::sync::Arc;
use std::path::PathBuf;

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
        #[arg(long, default_value = "3000")]
        ui_port: u16,
    },
    /// Run the server with HTTP/SSE interface
    Http {
        /// Address to bind the HTTP server to
        #[arg(short, long, default_value = "127.0.0.1:8080")]
        address: String,
        
        /// Enable debug logging
        #[arg(short, long)]
        debug: bool,

        /// Start web UI on this port (default: 3000)
        #[arg(long, default_value = "3000")]
        ui_port: u16,
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
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Version => {
            println!("cratedocs version {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Commands::Stdio { debug, ui_port } => {
            if debug {
                unsafe { std::env::set_var("RUST_LOG", "debug"); }
            }
            // Spawn UI server in background
            tokio::spawn(async move {
                web_ui::start_ui_server(ui_port).await;
            });
            server::start_stdio_server().await
        }
        Commands::Http { address, debug, ui_port } => {
            if debug {
                unsafe { std::env::set_var("RUST_LOG", "debug"); }
            }
            // Spawn UI server in background
            tokio::spawn(async move {
                web_ui::start_ui_server(ui_port).await;
            });
            println!("Starting SSE server on {}", address);
            server::start_sse_server(&address).await
        }
        Commands::Test { tool, crate_name, item_path, query, version, limit, start_line, end_line, kind } => {
             // Initialize cache with load/save for persistence testing
             let cache = Arc::new(InMemoryCache::new(PathBuf::from(".cache")));
             if let Err(e) = rustools_mcp::cache::Cache::load(cache.as_ref()).await {
                 eprintln!("Warning: Failed to load cache: {}", e);
             }

             let fetcher = DocFetcher::new(cache.clone());
             
             match tool.as_str() {
                 "find_crates" => {
                     let q = query.expect("query is required for find_crates");
                     let res = fetcher.find_crates(q, limit).await?;
                     println!("{}", res.content);
                 },
                 "get_crate_overview" => {
                    let name = crate_name.expect("crate_name is required for get_crate_overview");
                    let res = fetcher.get_crate_overview(name, version).await?;
                    println!("{}", res.content);
                },
                "get_crate_modules" => {
                    let name = crate_name.expect("crate_name is required for get_crate_modules");
                    let res = fetcher.get_crate_modules(name, version, limit.map(|l| l as usize)).await?;
                    println!("{}", res.content);
                },
                "get_symbol_docs" => {
                     let name = crate_name.expect("crate_name is required for get_symbol_docs");
                     let path = item_path.expect("item_path is required for get_symbol_docs");
                     let res = fetcher.get_symbol_docs(name, path, version).await?;
                     println!("{}", res.content);
                 },
                 "read_source_file" => {
                     let name = crate_name.expect("crate_name is required for read_source_file");
                     let path = item_path.expect("item_path (as file path) is required for read_source_file");
                     let res = fetcher.read_source_file(name, path, version, start_line, end_line).await?;
                     println!("{}", res.content);
                 },
                 "get_crate_dependencies" => {
                     let name = crate_name.expect("crate_name is required for get_crate_dependencies");
                     let v = version.expect("version is required for get_crate_dependencies");
                     let res = fetcher.get_crate_dependencies(name, v, kind).await?;
                     println!("{}", res.content);
                 },
                 "get_crate_examples" => {
                     let name = crate_name.expect("crate_name is required for get_crate_examples");
                     let res = fetcher.get_crate_examples(name, version, limit.map(|l| l as usize)).await?;
                     println!("{}", res.content);
                 },
                 "analyze_feature_flags" => {
                    let name = crate_name.expect("crate_name is required for analyze_feature_flags");
                    let v = version.expect("version is required for analyze_feature_flags");
                    let res = fetcher.analyze_feature_flags(name, v).await?;
                    println!("{}", res.content);
                },
                "find_trait_implementors" => {
                    let name = crate_name.expect("crate_name is required for find_trait_implementors");
                    let path = item_path.expect("item_path (as trait path) is required for find_trait_implementors");
                    let res = fetcher.find_trait_implementors(name, path, version, limit.map(|l| l as usize)).await?;
                    println!("{}", res.content);
                },
                "get_context_bundle" => {
                    let name = crate_name.expect("crate_name is required for get_context_bundle");
                    let res = fetcher.get_context_bundle(name, version).await?;
                    println!("{}", res.content);
                },
                "find_by_signature" => {
                    let name = crate_name.expect("crate_name is required for find_by_signature");
                    let pattern = item_path.expect("item_path (as signature pattern) is required for find_by_signature");
                    let res = fetcher.find_by_signature(name, pattern, version).await?;
                    println!("{}", res.content);
                },
                 "_fetch_raw_doc" => {
                     let name = crate_name.expect("crate_name is required for _fetch_raw_doc");
                     let path = item_path.expect("item_path is required for _fetch_raw_doc");
                     let v = version.unwrap_or_else(|| "latest".to_string());
                     let res = fetcher._fetch_raw_doc(name, v, path).await?;
                     println!("{}", res.content);
                 },
                 "doc_workflow_help" => {
                     let res = fetcher.doc_workflow_help().await?;
                     println!("{}", res.content);
                 },
                 "expand_macro" => {
                     let path = item_path.expect("item_path (as file/module path) is required for expand_macro");
                     let item = query; 
                     let res = fetcher.expand_macro(path, item).await?;
                     println!("{}", res.content);
                 },
                 _ => eprintln!("Unknown tool: {}", tool),
             }

             if let Err(e) = rustools_mcp::cache::Cache::save(cache.as_ref()).await {
                 eprintln!("Warning: Failed to save cache: {}", e);
             }
             Ok(())
        }
    }
}
