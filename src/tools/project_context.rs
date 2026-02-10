use crate::context_engine::{ProjectAnalyzer, ProjectContext};
use crate::docs_parser::{DocContent, DocsFetchError};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AnalyzeProjectContextArgs {
    /// The absolute path to the project root
    pub project_path: String,
}

pub async fn analyze_project_context(
    args: AnalyzeProjectContextArgs,
) -> Result<DocContent, DocsFetchError> {
    let analyzer = ProjectAnalyzer::new(args.project_path.clone());
    
    match analyzer.analyze() {
        Ok(context) => {
            let markdown = format_context_as_markdown(&context);
            Ok(DocContent { content: markdown })
        }
        Err(e) => Err(DocsFetchError::CommandError(format!("Analysis failed: {}", e))),
    }
}

fn format_context_as_markdown(context: &ProjectContext) -> String {
    let mut md = String::new();
    
    md.push_str("# Project Context Analysis\n\n");
    
    // Overview
    md.push_str("## Overview\n");
    md.push_str(&format!("- **Total Files**: {}\n", context.overview.total_files));
    md.push_str(&format!("- **Total Lines**: {}\n", context.overview.total_lines));
    md.push_str(&format!("- **Architecture**: {:?}\n\n", context.overview.architecture_type));
    
    // Dependencies
    md.push_str("## Dependencies\n");
    for dep in &context.dependencies {
        md.push_str(&format!("- **{}** ({})\n", dep.name, dep.version));
    }
    md.push_str("\n");
    
    // Modules
    md.push_str("## Modules\n");
    for (name, info) in &context.modules {
        md.push_str(&format!("### {}\n", name));
        md.push_str(&format!("- **Path**: {}\n", info.path));
        md.push_str(&format!("- **Complexity**: {}\n", info.complexity));
        if !info.structs.is_empty() {
            md.push_str("- **Structs**:\n");
            for s in &info.structs {
                md.push_str(&format!("  - {} ({})\n", s.name, s.visibility));
            }
        }
        if !info.functions.is_empty() {
            md.push_str("- **Functions**:\n");
            for f in &info.functions {
                md.push_str(&format!("  - {} ({})\n", f.name, f.visibility));
            }
        }
        md.push_str("\n");
    }
    
    // Hotspots
    md.push_str("## Hotspots\n");
    for hotspot in &context.hotspots {
        md.push_str(&format!("- **{}** (Score: {})\n", hotspot.path, hotspot.score));
        md.push_str(&format!("  - Reason: {}\n", hotspot.reason));
    }
    
    md
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_analyze_self() {
        let current_dir = env::current_dir().unwrap();
        let project_path = current_dir.to_string_lossy().to_string();
        
        let args = AnalyzeProjectContextArgs {
            project_path,
        };

        let result = analyze_project_context(args).await;
        if let Err(e) = &result {
            println!("Analysis failed: {:?}", e);
        }
        assert!(result.is_ok());
        
        let doc_content = result.unwrap();
        println!("{}", doc_content.content);
        
        assert!(doc_content.content.contains("# Project Context Analysis"));
        assert!(doc_content.content.contains("## Modules"));
    }
}
