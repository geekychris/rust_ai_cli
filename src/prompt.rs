use std::sync::Arc;
use serde_json::Value;
use crate::tools::{ToolRegistry, ToolArgs};
use crate::error::{Result, Error};

pub struct PromptTemplate {
    tools: Arc<ToolRegistry>,
}

impl PromptTemplate {
    pub fn new(tools: Arc<ToolRegistry>) -> Self {
        Self { tools }
    }

    pub fn format(&self, prompt: &str) -> crate::error::Result<String> {
        let tools = self.tools.list()
            .into_iter()
            .map(|(name, desc)| format!("- {}: {}", name, desc))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(format!(
            r#"You are a helpful AI assistant with access to a set of tools. Use these tools to help answer questions and perform tasks.

Available tools:
{}

To use a tool, format your response like this:
{{
    "tool": "tool_name",
    "args": {{
        // Required arguments per tool:
        // file tool: "operation" (read/write/append/list), "path" (for all ops), "content" (for write/append)
        // directory tool: "path", "recursive" (optional, true/false)
        // shell tool: "command"
        "arg1": "value1",
        "arg2": "value2"
    }}
}}

If you need to use multiple tools, respond with multiple JSON objects, one per line.
After using tools, provide your final response.

User query: {}
"#,
            tools, prompt
        ))
    }

    fn extract_first_json_object(&self, s: &str) -> Option<String> {
        let bytes = s.as_bytes();
        let mut i = 0usize;
        while i < bytes.len() {
            if bytes[i] == b'{' {
                let mut depth = 0i32;
                let mut j = i;
                while j < bytes.len() {
                    match bytes[j] {
                        b'{' => depth += 1,
                        b'}' => {
                            depth -= 1;
                            if depth == 0 {
                                // include j
                                return Some(s[i..=j].to_string());
                            }
                        }
                        _ => {}
                    }
                    j += 1;
                }
            }
            i += 1;
        }
        None
    }

    pub async fn handle_response(&self, response: &str) -> crate::error::Result<Option<(String, bool)>> {
        if let Some(obj) = self.extract_first_json_object(response) {
            if let Ok(json) = serde_json::from_str::<Value>(&obj) {
                if let (Some(tool_name), Some(args)) = (json["tool"].as_str(), json["args"].as_object()) {
                    let tool = self.tools.get(tool_name)
                        .ok_or_else(|| Error::Tool(format!("Unknown tool: {}", tool_name)))?;
                    let args = ToolArgs {
                        args: args.iter()
                            .map(|(k, v)| {
                                let value = match v {
                                    Value::String(s) => s.clone(),
                                    Value::Bool(b) => b.to_string(),
                                    Value::Number(n) => n.to_string(),
                                    Value::Null => "null".to_string(),
                                    _ => v.to_string().trim_matches('"').to_string(),
                                };
                                Ok((k.clone(), value))
                            })
                            .collect::<Result<_>>()?
                    };
                    
                    // Validate arguments before execution
                    self.tools.validate_args(tool_name, &args)?;
                    
                    let result = match tool.execute(args).await {
                            Ok(result) => result.output,
                            Err(e) => format!("Tool execution error: {}", e)
                    };
                    return Ok(Some((result, true)));
                }
            }
        }
        Ok(Some((response.to_string(), false)))
    }
}
