use async_trait::async_trait;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use crate::error::{Result, Error};
use super::{Tool, ToolArgs, ToolResponse};

pub struct FileTool;

impl FileTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for FileTool {
    fn name(&self) -> &str {
        "file"
    }
    
    fn description(&self) -> &str {
        "Read, write, append, or list directory entries"
    }
    
    async fn execute(&self, args: ToolArgs) -> Result<ToolResponse> {
        let operation = args.args.get("operation")
            .ok_or_else(|| Error::Tool("operation argument required".to_string()))?;
        
        match operation.as_str() {
            "read" => {
                let path = args.args.get("path")
                    .ok_or_else(|| Error::Tool("path argument required".to_string()))?;
                let content = fs::read_to_string(path).await?;
                Ok(ToolResponse { output: content })
            }
            
            "write" => {
                let path = args.args.get("path")
                    .ok_or_else(|| Error::Tool("path argument required".to_string()))?;
                let content = args.args.get("content")
                    .ok_or_else(|| Error::Tool("content argument required for write".to_string()))?;
                
                let mut file = fs::File::create(path).await?;
                file.write_all(content.as_bytes()).await?;
                
                Ok(ToolResponse {
                    output: format!("File written: {}", path),
                })
            }
            
            "append" => {
                let path = args.args.get("path")
                    .ok_or_else(|| Error::Tool("path argument required".to_string()))?;
                let content = args.args.get("content")
                    .ok_or_else(|| Error::Tool("content argument required for append".to_string()))?;
                
                let mut file = fs::OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(path)
                    .await?;
                
                file.write_all(content.as_bytes()).await?;
                
                Ok(ToolResponse {
                    output: format!("Content appended to: {}", path),
                })
            }
            
            "list" => {
                let path = args.args.get("path").map(|s| s.as_str()).unwrap_or(".");
                let mut entries = fs::read_dir(path).await?;
                let mut names: Vec<String> = Vec::new();
                while let Some(entry) = entries.next_entry().await? {
                    let file_name = entry.file_name();
                    names.push(file_name.to_string_lossy().into_owned());
                }
                names.sort();
                Ok(ToolResponse { output: names.join("\n") })
            }
            
            _ => Err(Error::Tool(format!("Unknown operation: {}", operation)))
        }
    }
}
