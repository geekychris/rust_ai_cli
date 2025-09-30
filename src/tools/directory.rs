use async_trait::async_trait;
use std::path::Path;
use tokio::fs;
use crate::error::{Result, Error};
use super::{Tool, ToolArgs, ToolResponse};

pub struct DirectoryTool;

impl DirectoryTool {
    pub fn new() -> Self {
        Self
    }
    
    async fn list_directory_inner(path: &Path, recursive: bool, entries: &mut Vec<String>) -> Result<()> {
        let mut read_dir = fs::read_dir(path).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();
            entries.push(path.display().to_string());
            
            if recursive && path.is_dir() {
                let fut = Box::pin(Self::list_directory_inner(&path, true, entries));
                fut.await?
            }
        }
        Ok(())
    }

    async fn list_directory(path: &Path, recursive: bool) -> Result<Vec<String>> {
        let mut entries = Vec::new();
        Self::list_directory_inner(path, recursive, &mut entries).await?;
        Ok(entries)
    }
}

#[async_trait]
impl Tool for DirectoryTool {
    fn name(&self) -> &str {
        "directory"
    }
    
    fn description(&self) -> &str {
        "List directory contents, optionally recursively"
    }
    
    async fn execute(&self, args: ToolArgs) -> Result<ToolResponse> {
        let path = args.args.get("path")
            .ok_or_else(|| Error::Tool("path argument required".to_string()))?;
            
        let recursive = args.args.get("recursive")
            .map(|v| v.parse::<bool>().unwrap_or_else(|_| v.to_lowercase() == "true"))
            .unwrap_or(false);
            
        let entries = Self::list_directory(Path::new(path), recursive).await?;
        
        Ok(ToolResponse {
            output: entries.join("\n"),
        })
    }
}