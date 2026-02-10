use crate::docs_parser::{DocContent, DocsFetchError, DocsRsClient};
use std::path::PathBuf;
use std::fs;

#[derive(schemars::JsonSchema, serde::Deserialize)]
pub struct GetLocalDocArgs {
    pub path: String,
    pub cwd: Option<String>,
}

pub fn get_local_doc(args: GetLocalDocArgs) -> Result<DocContent, DocsFetchError> {
    let mut path = PathBuf::from(&args.path);
    
    if let Some(cwd) = args.cwd {
        if !path.is_absolute() {
            path = PathBuf::from(cwd).join(path);
        }
    }

    if !path.exists() {
        return Err(DocsFetchError::DocsNotFound);
    }
    
    let html = fs::read_to_string(&path).map_err(|e| DocsFetchError::RequestError(e.to_string()))?;
    
    // We treat the file path as the URL for reference
    let url = format!("file://{}", path.display());
    
    let content = DocsRsClient::extract_content(&html, &url)?;
    
    Ok(DocContent { content })
}
