use async_trait::async_trait;
use tokio::process::Command;
use crate::error::{Result, Error};
use super::{Tool, ToolArgs, ToolResponse};

pub struct ShellTool;

impl ShellTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for ShellTool {
    fn name(&self) -> &str {
        "shell"
    }
    
    fn description(&self) -> &str {
        "Execute shell commands"
    }
    
    async fn execute(&self, args: ToolArgs) -> Result<ToolResponse> {
        let command = args.args.get("command")
            .ok_or_else(|| Error::Tool("command argument required".to_string()))?;
            
        let shell = if cfg!(target_os = "windows") { "cmd" } else { "sh" };
        let shell_arg = if cfg!(target_os = "windows") { "/C" } else { "-c" };
        
        let output = Command::new(shell)
            .arg(shell_arg)
            .arg(command)
            .output()
            .await?;
            
        let mut response = String::new();
        
        if !output.stdout.is_empty() {
            response.push_str(&String::from_utf8_lossy(&output.stdout));
        }
        
        if !output.stderr.is_empty() {
            if !response.is_empty() {
                response.push_str("\n\n");
            }
            response.push_str("Error output:\n");
            response.push_str(&String::from_utf8_lossy(&output.stderr));
        }
        
        if response.is_empty() {
            response = "Command executed successfully with no output".to_string();
        }
        
        Ok(ToolResponse { output: response })
    }
}