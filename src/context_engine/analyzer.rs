use super::types::*;
use ignore::WalkBuilder;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use syn::{Item, Type, Visibility};

pub struct ProjectAnalyzer {
    root_path: String,
}

impl ProjectAnalyzer {
    pub fn new(root_path: String) -> Self {
        Self { root_path }
    }

    pub fn analyze(&self) -> Result<ProjectContext, Box<dyn std::error::Error>> {
        let mut modules = HashMap::new();
        let mut dependencies = Vec::new();
        let mut total_files = 0;
        let mut total_lines = 0;
        let mut hotspots = Vec::new();

        // 1. Scan Dependencies from Cargo.toml
        if let Ok(cargo_toml) = fs::read_to_string(Path::new(&self.root_path).join("Cargo.toml")) {
            if let Ok(parsed) = cargo_toml.parse::<toml::Table>() {
                if let Some(deps) = parsed.get("dependencies").and_then(|d| d.as_table()) {
                    for (name, value) in deps {
                        let version = value.as_str().unwrap_or("*").to_string();
                        dependencies.push(DependencyInfo {
                            name: name.clone(),
                            version,
                            features: vec![], // TODO: Parse features
                        });
                    }
                }
            }
        }

        // 2. Walk Source Files
        let walker = WalkBuilder::new(&self.root_path)
            .hidden(false)
            .git_ignore(true)
            .build();

        for result in walker {
            match result {
                Ok(entry) => {
                    let path = entry.path();
                    if path.extension().map_or(false, |ext| ext == "rs") {
                        total_files += 1;
                        if let Ok(content) = fs::read_to_string(path) {
                            let lines = content.lines().count();
                            total_lines += lines;

                            // Analyze File Content
                            if let Ok(file_ast) = syn::parse_file(&content) {
                                let module_info = self.analyze_file(&file_ast, path.to_string_lossy().to_string());
                                
                                // Detect Hotspots (Simple Heuristic: Lines > 300 or many functions)
                                if lines > 300 || module_info.functions.len() > 20 {
                                    hotspots.push(Hotspot {
                                        path: path.to_string_lossy().to_string(),
                                        score: (lines / 10) as u32,
                                        reason: format!("Large file ({} lines, {} functions)", lines, module_info.functions.len()),
                                    });
                                }

                                modules.insert(path.to_string_lossy().to_string(), module_info);
                            }
                        }
                    }
                }
                Err(err) => eprintln!("Error walking directory: {}", err),
            }
        }

        // 3. Determine Architecture
        let architecture_type = if Path::new(&self.root_path).join("src/main.rs").exists() {
            ArchitectureType::Binary
        } else if Path::new(&self.root_path).join("src/lib.rs").exists() {
            ArchitectureType::Library
        } else {
            ArchitectureType::Unknown
        };

        // Sort Hotspots
        hotspots.sort_by(|a, b| b.score.cmp(&a.score));
        hotspots.truncate(10); // Top 10

        Ok(ProjectContext {
            overview: ProjectOverview {
                total_files,
                total_lines,
                architecture_type,
            },
            modules,
            dependencies,
            hotspots,
        })
    }

    fn analyze_file(&self, file: &syn::File, path: String) -> ModuleInfo {
        let mut functions = Vec::new();
        let mut structs = Vec::new();
        let mut complexity = 0;

        for item in &file.items {
            match item {
                Item::Fn(func) => {
                    complexity += 1;
                    let name = func.sig.ident.to_string();
                    let visibility = match &func.vis {
                        Visibility::Public(_) => "pub".to_string(),
                        _ => "private".to_string(),
                    };
                    let is_async = func.sig.asyncness.is_some();
                    let return_type = match &func.sig.output {
                        syn::ReturnType::Default => "()".to_string(),
                        syn::ReturnType::Type(_, ty) => self.type_to_string(ty),
                    };
                    
                    let args = func.sig.inputs.iter().map(|arg| match arg {
                        syn::FnArg::Receiver(_) => "self".to_string(),
                        syn::FnArg::Typed(pat_type) => self.type_to_string(&pat_type.ty),
                    }).collect();

                    // Estimate lines (start to end)
                    // syn::spanned::Spanned is needed for accurate lines, but that requires feature "extra-traits" which we disabled?
                    // "full" includes "printing" and "parsing", checking "spanned"...
                    // Without "extra-traits", we can't easily get spans. We'll skip granular function line counts for now.
                    let lines = 0; 

                    functions.push(FunctionInfo {
                        name,
                        visibility,
                        is_async,
                        lines,
                        args,
                        return_type,
                    });
                }
                Item::Struct(st) => {
                    complexity += 1;
                    let name = st.ident.to_string();
                    let visibility = match &st.vis {
                        Visibility::Public(_) => "pub".to_string(),
                        _ => "private".to_string(),
                    };
                    let fields = st.fields.len();
                    structs.push(StructInfo {
                        name,
                        visibility,
                        fields,
                    });
                }
                _ => {}
            }
        }

        ModuleInfo {
            path,
            complexity,
            functions,
            structs,
        }
    }

    fn type_to_string(&self, ty: &Type) -> String {
        // Simple approximation for display
        match ty {
            Type::Path(p) => p.path.segments.last().map(|s| s.ident.to_string()).unwrap_or("?".to_string()),
            Type::Reference(_) => "&T".to_string(),
            _ => "Complex".to_string(),
        }
    }
}
