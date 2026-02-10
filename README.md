# 🧠 Rust AI Assistant MCP Server

> **The Autonomous Rust Partner That Understands Your Project.** 🚀

**`rust-ai-assistant`** is not another tool server—it's a **context-aware, learning Rust expert** that integrates with your AI editor. While existing MCP servers expose tools, this server builds a **real-time project knowledge graph**, proactively identifies issues, and orchestrates intelligent workflows to help you write better Rust code faster.

## 🎯 The Problem & Our Solution

Current Rust MCP servers act as **passive toolboxes**: they give an LLM a wrench but not the blueprint. Your AI assistant lacks project context, can't learn from your patterns, and requires manual prompting for every step.

**This project changes the paradigm.** It's an **Autonomous Development Agent** built on three core layers:

```
┌─────────────────────────────────────────────────────────┐
│                 AI Editor (Claude, Cursor)              │
└──────────────────────────────────┬──────────────────────┘
                                   │
┌──────────────────────────────────▼──────────────────────┐
│            REASONING & ORCHESTRATION LAYER              │
│  • Problem decomposition & tool chaining                │
│  • Context-aware suggestions & proactive monitoring     │
│  • Learning from developer patterns & decisions         │
└──────────────────────────────────┬──────────────────────┘
                                   │
┌──────────────────────────────────▼──────────────────────┐
│              UNIFIED CORE SERVICE LAYER                 │
│  • Single, robust connection to rust-analyzer           │
│  • Isolated dependency & documentation analysis         │
│  • Safe, sandboxed Cargo operation execution           │
└──────────────────────────────────┬──────────────────────┘
                                   │
┌──────────────────────────────────▼──────────────────────┐
│               PROJECT CONTEXT ENGINE                    │
│  • Live code analysis & knowledge graph                 │
│  • Architectural pattern recognition                    │
│  • Historical issue & solution tracking                 │
└─────────────────────────────────────────────────────────┘
```

## ✨ Revolutionary Features

### 🧩 Intelligent Project Context (The Game Changer)
- **`analyze_project_context`**: Generates a comprehensive project knowledge graph in one call—architecture, hotspots, dependency patterns, and tailored suggestions.
- **Proactive Hotspot Detection**: Automatically flags complex functions, common error patterns, and performance traps *before* they become problems.
- **Architectural Pattern Recognition**: Identifies whether you're building a REST API, CLI tool, game engine, or library and adapts suggestions accordingly.

### 🤖 Autonomous Assistant Mode
- **Continuous Monitoring**: Watches `cargo` output, CI results, and file changes to offer context-sensitive help.
- **Chain-of-Thought Tool Orchestration**: The `solve_problem` meta-tool breaks down complex requests, executes the right tool sequence, and synthesizes coherent solutions.
- **Interactive REPL Mode**: Conversational interface that remembers context, asks clarifying questions, and provides educational explanations.

### 🛠️ Unified Tool Integration (Best of All Worlds)
- **Stable rust-analyzer Foundation**: Uses a battle-tested LSP client (inspired by Zeenix's approach) for reliable code analysis.
- **Live, Version-Locked Documentation**: Isolated documentation fetcher (like Terhechte's) that provides accurate, project-specific crate docs.
- **Safe Cargo Orchestration**: Sandboxed execution of build, test, and audit commands with intelligent output parsing.
- **Intelligent Caching**: Multi-layer caching (`moka` + project-aware invalidation) for instant responses without staleness.

### 📚 Learning & Adaptation System
- **Pattern Recognition**: Learns your team's conventions, preferred crates, and error-handling patterns.
- **Decision Feedback Loop**: Improves suggestions based on which fixes you accept or reject.
- **Team Knowledge Sharing**: Optional sharing of anonymized patterns across organization to elevate collective code quality.

---

## 🚀 Quick Start

### Installation
```bash
# Install from crates.io
cargo install rust-ai-assistant

# Or build from source
git clone https://github.com/yourusername/rust-ai-assistant.git
cd rust-ai-assistant
cargo build --release

# The binary includes both MCP server and interactive REPL
rust-ai-assistant --help
```

### Editor Configuration
Add to your MCP client configuration (Claude Desktop, Cursor, etc.):

```json
{
  "mcpServers": {
    "rust-ai-assistant": {
      "command": "rust-ai-assistant",
      "args": ["server", "--project-path", "/path/to/your/project"],
      "env": {
        "RUST_LOG": "info",
        "RUST_AI_ASSISTANT_MODE": "autonomous"
      }
    }
  }
}
```

### First Run - Project Onboarding
```bash
# 1. Let the assistant analyze your project
curl -X POST http://localhost:8080/tools/analyze_project_context \
  -H "Content-Type: application/json" \
  -d '{"scan_depth": "deep", "include_suggestions": true}'

# Response includes architectural insights, hotspots, and initial recommendations
```

---

## 🧭 Usage: The Autonomous Workflow

### Scenario 1: You Just Cloned a New Codebase
**Traditional Approach**: Manually explore directories, run `cargo check`, guess at architecture.

**With Rust AI Assistant**:
```rust
// Your AI assistant automatically runs:
1. analyze_project_context() → Gets architecture, key files, dependencies
2. cargo_check() → Identifies compilation issues  
3. detect_patterns() → Flags unusual patterns or potential bugs
4. generate_onboarding_summary() → Creates tailored project overview

// Before you even ask, you get:
"📋 Project Analysis Complete:
 • Architecture: Axum web server with SSE, JWT auth, PostgreSQL
 • Hotspots: `src/auth/jwt.rs` (high complexity), `src/api/mod.rs` (circular ref risk)
 • Dependencies: 3 outdated, 1 with security advisory (chrono)
 • Test Coverage: 68% - missing auth integration tests
 Ready to help you navigate!"
```

### Scenario 2: Debugging a Complex Borrow Checker Error
**Traditional Approach**: Manual `rustc` error deciphering, trial-and-error fixes.

**With Rust AI Assistant**:
```rust
// When cargo check fails with E0597:
[ASSISTANT PROACTIVE INTERVENTION]
"⚠️  Detected lifetime error in `src/processor.rs:127`
 The issue: Cannot borrow `*self` as mutable because `self.queue` is also borrowed.

 Context: This pattern appears 3 times in your codebase.
 Previous solutions that worked for your team:
 1. Use `Rc<RefCell<T>>` for shared mutable state (used in `src/cache.rs`)
 2. Restructure to separate borrowing (used in `src/engine/mod.rs`)
 3. Use async-friendly mutex (recommended for Tokio context)

 Suggested fix for this context (Tokio + async):
   - let item = self.queue.pop_front().await;  // ← Change to async pop
   - self.process(item).await;

 Apply this fix? [Y/n/explain_more]"
```

### Scenario 3: Adding a New Feature with Dependencies
```rust
// You: "I need to add JSON Web Token authentication"
[ASSISTANT ORCHESTRATION]
1. Searches crates: `find_crates("jwt auth async")` → Finds `jsonwebtoken`, `jwt-compact`
2. Checks compatibility: `analyze_dependency_compatibility("jsonwebtoken", current_dependencies)`
3. Reviews patterns: `find_similar_implementations("auth", "jwt")` in your org's codebase
4. Generates implementation: `generate_auth_module()` with your team's preferred error handling
5. Sets up monitoring: Adds watch for common JWT security pitfalls

// Result: Complete, context-appropriate implementation in 1 minute
```

---

## 🛠️ Available Tools & Capabilities

### Core Intelligence Tools
| Tool | Input | Output | Autonomous Trigger |
|------|-------|--------|-------------------|
| `analyze_project_context` | `{scan_depth, include_suggestions}` | ProjectKnowledgeGraph | On project load, major changes |
| `solve_problem` | `{description: "My async task is deadlocking"}` | StepwiseSolution | Complex user questions |
| `proactive_suggestion_engine` | `{context: current_file, recent_errors}` | SuggestionBatch | File save, compile error |
| `learn_from_decision` | `{issue, chosen_fix, rejected_fixes}` | UpdatedProfile | After user accepts/rejects help |

### Unified Development Tools
| Category | Tools | Source Inspiration |
|----------|-------|-------------------|
| **Code Analysis** | `hover`, `definition`, `references`, `diagnostics`, `symbol_search` | Zeenix's stable LSP client |
| **Documentation** | `get_documentation`, `get_context_bundle`, `find_by_signature` | Terhechte's version-locked docs |
| **Cargo Operations** | `cargo_check`, `cargo_test`, `cargo_clippy`, `cargo_audit`, `cargo_expand` | Robust, sandboxed execution |
| **Dependency Mgmt** | `dependency_graph`, `check_updates`, `security_audit`, `semver_checks` | Combined intelligence |

### Monitoring & Alerting Tools
| Tool | Purpose | Autonomous Action |
|------|---------|-------------------|
| `watch_compilation` | Monitor build outputs | Auto-explain new errors |
| `track_performance` | Profile regressions | Flag >10% slowdowns |
| `security_monitor` | Watch for new advisories | Alert + suggest patches |
| `api_compatibility` | Detect breaking changes | Warn before updating |

---

## ⚙️ Configuration & Customization

### Project Configuration (`rust-ai-assistant.toml`)
```toml
[assistant]
mode = "autonomous"  # autonomous, interactive, or passive
intervention_level = "suggest"  # suggest, confirm, or auto_fix
learning_enabled = true
team_knowledge_sharing = false  # Enable for organizations

[project_context]
scan_on_startup = true
watch_for_changes = true
hotspot_threshold = 10  # Cyclomatic complexity threshold

[tools]
enable_cargo_operations = true
sandbox_level = "strict"  # strict, moderate, or permissive
timeout_seconds = 30

[cache]
strategy = "aggressive"
ttl_hours = 24
max_size_mb = 1024
```

### Team Knowledge Base
Share patterns across your organization:
```bash
# Export your team's successful patterns
rust-ai-assistant patterns export --output team-patterns.json

# Import into new projects
rust-ai-assistant patterns import --file team-patterns.json
```

---

## 🏗️ Architecture: Building the Future

### The Three-Layer Architecture
```
1. Project Context Engine (Stateful)
   - In-memory knowledge graph of your project
   - Real-time AST analysis via rust-analyzer
   - Historical decision tracking
   
2. Unified Core Service Layer (Stateless)
   - Single point of truth for rust-analyzer
   - Isolated dependency analysis
   - Safe sandbox for Cargo operations
   
3. Reasoning & Orchestration Layer (Intelligent)
   - Problem decomposition engine
   - Tool chaining & workflow management
   - Learning & adaptation system
```

### Integration with Existing MCP Servers
```rust
// Instead of replacing, we orchestrate
pub struct AssistantOrchestrator {
    lsp_client: ZeenixStyleClient,      // Stable analysis
    doc_fetcher: TerhechteStyleFetcher, // Accurate docs
    cargo_executor: SafeSandbox,        // Safe operations
    context_engine: ProjectKnowledgeGraph, // Our innovation
    learning_engine: PatternRecognizer,    // Our innovation
}

// The assistant chooses the best tool for each task
async fn handle_complex_request(&self, request: Problem) -> Solution {
    match request.category {
        ProblemCategory::CodeAnalysis => self.lsp_client.analyze(request),
        ProblemCategory::Dependency => self.doc_fetcher.fetch(request),
        ProblemCategory::Build => self.cargo_executor.execute(request),
        ProblemCategory::Architecture => self.context_engine.solve(request),
    }
}
```

### Extensibility: Plugin System
```rust
// Add your own analyzers
#[derive(AssistantPlugin)]
struct SecurityAnalyzer {
    rules: Vec<SecurityRule>,
}

impl ToolProvider for SecurityAnalyzer {
    fn tools(&self) -> Vec<Tool> {
        vec![
            tool! { check_crypto_usage() },
            tool! { audit_unsafe_blocks() },
            tool! { detect_hardcoded_secrets() },
        ]
    }
}

// Register with the assistant
assistant.register_plugin(SecurityAnalyzer::new(rules));
```

---

## 📊 Performance & Reliability

### Caching Strategy
- **L1 Cache**: In-memory, project-specific (clears on file change)
- **L2 Cache**: Disk-based, versioned responses (TTL: 24 hours)
- **L3 Cache**: Pre-fetched common crate documentation
- **Intelligent Invalidation**: Cache busting on dependency changes

### Resource Management
```toml
[resources]
max_memory_mb = 2048
max_concurrent_analyses = 4
rust_analyzer_memory_limit = "2GB"
network_timeout_seconds = 10
retry_attempts = 3
```

### Monitoring & Metrics
```bash
# Built-in observability
rust-ai-assistant metrics --format=prometheus
# Exposes: tool_call_count, cache_hit_rate, avg_response_time, proactive_interventions

# Health checks
rust-ai-assistant health
# Checks: rust-analyzer connection, cargo availability, cache health
```

---

## 🚀 Getting Started for Contributors

### Prerequisites
```bash
# Required for development
rustup toolchain install nightly
cargo install cargo-watch
cargo install rust-analyzer

# For testing the full stack
cargo install mcp-cli  # MCP client for testing
```

### Development Setup
```bash
# 1. Clone and setup
git clone https://github.com/yourusername/rust-ai-assistant.git
cd rust-ai-assistant

# 2. Build with all features
cargo build --all-features

# 3. Run tests (including integration tests)
cargo test -- --test-threads=1

# 4. Start in development mode
cargo run -- server --dev --project-path examples/sample-project
```

### Project Structure
```
src/
├── context_engine/     # Project knowledge graph & analysis
├── reasoning_layer/    # Problem solving & tool orchestration  
├── core_services/      # Unified LSP, docs, cargo clients
├── learning/           # Pattern recognition & adaptation
├── plugins/            # Extensibility system
└── server/             # MCP server implementation
```

---

## 📈 Roadmap: The Path to Autonomous Development

### Phase 1: Intelligent Foundation (Current)
- ✅ Project context engine with knowledge graphs
- ✅ Unified tool orchestration layer
- ✅ Basic learning from user decisions

### Phase 2: Deep Integration (Next 3 Months)
- 🔄 Cross-repository pattern analysis
- 🔄 CI/CD pipeline integration
- 🔄 Real-time collaboration features

### Phase 3: Full Autonomy (6+ Months)
- ◻️ Predictive code generation based on patterns
- ◻️ Automated refactoring with safety guarantees  
- ◻️ Self-improving suggestion engine

---

## 🤝 Contributing & Community

We're building the future of Rust development with AI. Join us:

1. **Report Issues**: Bug reports, feature requests, and documentation improvements
2. **Share Patterns**: Contribute your team's successful Rust patterns
3. **Build Plugins**: Extend the assistant with domain-specific tools
4. **Improve Intelligence**: Help train the suggestion engine

### Code of Conduct
This project follows the Rust Code of Conduct. We're building an inclusive, helpful assistant—let's build an inclusive, helpful community too.

---

## 📄 License & Acknowledgments

**License**: MIT or Apache 2.0 (your choice)

**Built Upon**:
- The stability of Zeenix's rust-analyzer-mcp
- The accuracy of Terhechte's documentation system
- The vision of Dex's comprehensive toolset
- The Rust community's incredible ecosystem

**Special Thanks**: To every Rust developer who ever wished their AI assistant actually understood their project. This is for you.

---

## 🚨 Ready to Transform Your Rust Development?

```bash
# Start your autonomous Rust partner today
cargo install rust-ai-assistant
cd your-project
rust-ai-assistant init --mode=autonomous
```

**Experience the future of Rust development—where your AI assistant doesn't just run tools, but understands your project, learns your patterns, and proactively helps you write better code.**

---
*"The best tool is the one that understands what you're trying to build."*