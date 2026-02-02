# Rust MCP Usage Rules (Cognitive Extension)

Follow these rules to maximize efficiency and cognitive power when using the Rust API Documentation MCP.

## 🧭 The Core Workflow (The "Holy Trinity")

For maximum assistance, follow this data flow:

1.  **DISCOVERY (Crates.io)**: 
    *   If you don't know the exact crate name, use `find_crates("keywords")`.
    *   Example: "Find serialization crates"

2.  **ANALYSIS (Docs.rs + Metadata)**:
    *   **Initial Encounter**: Use `get_context_bundle("crate")` for ANY crate you haven't used in this session.
        *   *Why?* It provides README, Modules, and Feature Flags in one go.
    *   **Deep Dive**: Use `get_crate_modules` or `find_by_signature` to narrow down the API surface.
    *   **Dependencies**: Use `get_crate_dependencies` to check weight and reverse deps.

3.  **VALIDATION (Local/Language)**:
    *   Use `get_symbol_docs` for specific types (e.g., `tokio::net::TcpListener`).
    *   Use `get_language_concept` (if available) or `web_search` for conceptual understanding (e.g., "ownership", "async await").

---

## 🗺️ Resource Selection Guide

Select the right tool based on the data source type:

| Intent | Source | Recommended Tool |
|--------|--------|------------------|
| **Concept/Tutorial** | The Rust Book | `get_language_concept("topic")` |
| **Language Spec** | Rust Reference | `web_search site:doc.rust-lang.org/reference` |
| **Library API** | Docs.rs | `get_symbol_docs`, `get_context_bundle` |
| **Crate Popularity** | Crates.io | `find_crates`, `get_crate_dependencies` |
| **Compiler Errors** | Error Index | `explain_error_code("E0382")` |
| **Best Practices** | Rust by Example | `get_crate_examples` (for crates) or `web_search` |

---

## 🛠️ Specialized Rules

### Rule 1: The "No-Guessing" Policy
Never guess if a type implements a trait or if a function exists.
*   **Action**: Use `find_trait_implementors("crate", "Trait")` or `find_by_signature`.

### Rule 2: The Compilation Failure Flow
If the LLM generates code that fails with "module not found" or "no such function":
1.  **Do NOT** just apologize and try again.
2.  **Action**: Call `analyze_feature_flags("crate", "version")`.
3.  **Reason**: Most Rust errors in LLM code are due to missing feature flags in `Cargo.toml`.

### Rule 3: The implementation over docs rule
If the documentation is generic (e.g., "Returns the internal state") or lacks examples:
1.  **Action**: Use `read_source_file("crate", "path/to/file.rs")`.
2.  **Action**: Use `get_crate_examples("crate")`.

### Rule 4: Gap Analysis & Workarounds
Some information is not yet exposed via MCP. Use these workarounds:
*   **Macro Expansion**: Requires `cargo expand` locally (not available via MCP). Use `read_source_file` to infer behavior.
*   **Cross-crate Traits**: Hard to find. Use `find_trait_implementors` on the *implementing* crate, not the definition crate.
*   **Type Inference**: No direct tool. Rely on explicit type annotations in code you generate.

---

## 📉 Token Efficiency Rules

*   **Truncation Awareness**: `get_context_bundle` and `get_crate_modules` utilize smart truncation. Do not ask for "all items" if the list is massive; use targeted queries instead.
*   **Path Precision**: Always use fully qualified paths (`std::vec::Vec`) in `get_symbol_docs` to reduce ambiguity and error-retries.
*   **Signature Search**: If `find_by_signature` returns a link because it requires JS, use `get_crate_modules` on the most likely parent module to see the static export list.

## 🚀 When to use "Power Moves"

*   **Trait Bound navigation**: When finding "what can I use here?", use `find_trait_implementors`.
*   **Manifest-Awareness**: Use `get_crate_dependencies` to check for "heavy" dependencies or version conflicts before suggesting a crate.
