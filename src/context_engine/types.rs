use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectContext {
    pub overview: ProjectOverview,
    pub modules: HashMap<String, ModuleInfo>,
    pub dependencies: Vec<DependencyInfo>,
    pub hotspots: Vec<Hotspot>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectOverview {
    pub total_files: usize,
    pub total_lines: usize,
    pub architecture_type: ArchitectureType,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ArchitectureType {
    Binary,
    Library,
    Workspace,
    Mixed,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModuleInfo {
    pub path: String,
    pub complexity: u32,
    pub functions: Vec<FunctionInfo>,
    pub structs: Vec<StructInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub visibility: String,
    pub is_async: bool,
    pub lines: usize,
    pub args: Vec<String>,
    pub return_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StructInfo {
    pub name: String,
    pub visibility: String,
    pub fields: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DependencyInfo {
    pub name: String,
    pub version: String,
    pub features: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Hotspot {
    pub path: String,
    pub score: u32,
    pub reason: String,
}
