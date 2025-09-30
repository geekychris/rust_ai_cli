use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::error::{Result, Error};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolArgs {
    pub args: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResponse {
    pub output: String,
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    async fn execute(&self, args: ToolArgs) -> Result<ToolResponse>;
}

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }
    
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }
    
    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    pub fn validate_args(&self, tool_name: &str, args: &ToolArgs) -> Result<()> {
        match tool_name {
            "file" => {
                let operation = args.args.get("operation")
                    .ok_or_else(|| Error::Tool("file tool requires 'operation' argument".to_string()))?;
                
                args.args.get("path")
                    .ok_or_else(|| Error::Tool("file tool requires 'path' argument".to_string()))?;
                
                match operation.as_str() {
                    "write" | "append" => {
                        args.args.get("content")
                            .ok_or_else(|| Error::Tool(format!("file tool {} operation requires 'content' argument", operation)))?;
                    }
                    "read" | "list" => {}
                    _ => return Err(Error::Tool(format!("Unknown file operation: {}", operation)))
                }
            }
            "directory" => {
                args.args.get("path")
                    .ok_or_else(|| Error::Tool("directory tool requires 'path' argument".to_string()))?;
            }
            "shell" => {
                args.args.get("command")
                    .ok_or_else(|| Error::Tool("shell tool requires 'command' argument".to_string()))?;
            }
            _ => return Err(Error::Tool(format!("Unknown tool: {}", tool_name)))
        }
        Ok(())
    }
    
    pub fn list(&self) -> Vec<(String, String)> {
        self.tools
            .iter()
            .map(|(name, tool)| (name.clone(), tool.description().to_string()))
            .collect()
    }
}

// System tools
mod directory;
mod file;
mod shell;

pub use directory::DirectoryTool;
pub use file::FileTool;
pub use shell::ShellTool;