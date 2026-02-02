# Developer Documentation: `mcp_rustools`

This document details the architecture, internal logic, and development workflows for the `mcp_rustools` (crate: `rustools-mcp`) project.

## 1. System Architecture

The project is a **Model Context Protocol (MCP)** server implemented in Rust. It bridges the gap between Large Language Models (LLMs) and the Rust ecosystem.

```mermaid
graph TD
    User[LLM / Client] <-->|JSON-RPC (Stdio/SSE)| Server[Server Layer]
    Server -->|Dispatches| Handler[Tool Handler (mcp.rs)]
    
    subgraph Core Logic
        Handler -->|1. Check| Cache[InMemoryCache (cache.rs)]
        Handler -->|2. Miss| Fetcher[DocFetcher (mcp.rs)]
        Fetcher -->|3. Request| Parser[DocsParser (docs_parser.rs)]
    end
    
    subgraph Data Sources
        Parser <-->|HTTP| DocsRS[docs.rs]
        Parser <-->|HTTP| CratesIO[crates.io]
        Parser <-->|HTTP| RustBook[doc.rust-lang.org]
    end
    
    Cache -->|Persist| Disk[.cache/ directory]
    Parser -->|Return| Markdown[Optimized Markdown]
```

### Key Components

1.  **Server Layer (`src/server.rs`)**:
    *   Uses `rmcp` crate to handle MCP protocol details.
    *   Supports **Stdio** (default, recommended for local) and **SSE** (Server-Sent Events) transports.
    *   Spawns a background task for the Web UI (port 3000).

2.  **Tool Handler (`src/mcp.rs`)**:
    *   The "Brain" of the server.
    *   Maps MCP tool calls (e.g., `get_context_bundle`) to internal functions.
    *   Implements the **Cognitive Triggers** logic (Discovery -> Analysis -> Validation).
    *   Handles logging via `RequestLogger`.

3.  **Docs Parser (`src/docs_parser.rs`)**:
    *   The "Engine".
    *   `DocsRsClient`: Manages HTTP sessions, User-Agents, and timeouts.
    *   `extract_content`: Uses `scraper` to isolate `#main-content` and `html2md` to convert it to clean Markdown.
    *   **Context Bundling**: Aggregates README, Features, and Dependencies into a single response.

4.  **Caching (`src/cache.rs`)**:
    *   `InMemoryCache`: `Arc<RwLock<HashMap<CacheKey, DocContent>>>`.
    *   **Persistence**: Automatically saves to/loads from disk on startup/shutdown.
    *   Keys are structured enums (`CacheKey`) to prevent collisions.

## 2. Tool Design Philosophy

The tools are designed with **Agentic Workflows** in mind. We use "Chain-of-Thought" prompting in the tool descriptions to guide the LLM.

### The "Holy Trinity" Pattern
Every interaction is modeled as a 3-step process:
1.  **Discovery**: `find_crates`, `get_language_concept`.
2.  **Analysis**: `get_context_bundle`, `analyze_feature_flags`.
3.  **Validation**: `get_symbol_docs`, `read_source_file`.

### Agent-Friendly Descriptions
We explicitly tell the agent:
- **WHEN** to use a tool.
- **WHAT** it returns.
- **NEXT STEPS** to take.
- **PITFALLS** to avoid.

*Example (`src/mcp.rs`):*
```rust
#[tool(description = 
r#"STEP 2 (Alternative) - OVERVIEW: ...
USE THIS WHEN: ...
NEXT STEP: Use `get_crate_modules`..."#)]
```

## 3. Observability & Debugging

### Logs
- **`requests.log`**: JSON-line formatted log of every tool call, arguments, and response (truncated).
- **Console Output**: Uses `tracing` for debug/info logs (stderr).

### Testing Tools
The binary includes a `test` subcommand to verify logic without running the full server:

```bash
# Test a specific tool
cargo run --release -- test --tool get_context_bundle --crate-name tokio

# Test local cache
cargo run --release -- test --tool find_crates --query "async"
```

### `verify_mcp.rs`
A standalone script to validate the server's health and tool availability.

## 4. Development Workflow

1.  **Add a new tool**:
    -   Define the method in `DocsRsClient` (`docs_parser.rs`).
    -   Add the `CacheKey` variant (`docs_parser.rs`).
    -   Implement the tool in `DocFetcher` (`mcp.rs`) with the `#[tool]` macro.
    -   Add a test case in `src/main.rs` (CLI) and `mcp.rs` (Unit Test).

2.  **Update dependencies**:
    -   Check `Cargo.toml`.
    -   Run `cargo build` to ensure no conflicts.

3.  **Release**:
    -   Update version in `Cargo.toml`.
    -   Run `cargo build --release`.

## 5. Troubleshooting

| Symptom | Cause | Solution |
| :--- | :--- | :--- |
| **Response `null`** | Tool panic or empty result. | Check `requests.log`. Ensure crate exists. |
| **"Crate not found"** | Typo or std lib confusion. | Use `find_crates` first. Std lib is `std`, `core`, `alloc`. |
| **Connection Refused** | CORS or Port conflict. | Use **Stdio** mode instead of SSE. |
| **Slow Response** | Cache miss + large crate. | Subsequent calls will be instant. |

