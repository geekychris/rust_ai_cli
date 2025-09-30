mod error;
mod llm;
mod tools;
mod prompt;

use std::{collections::HashMap, env, io::{self, Read}, sync::Arc};

use clap::{ArgAction, Parser};
use tools::{ToolRegistry, DirectoryTool, FileTool, ShellTool, ToolArgs};
use llm::{Llm, LlmConfig, LlamaLlm, OllamaLlm};
use error::{Result, Error};

#[derive(Parser, Debug)]
#[command(name = "rustaicli", version, about = "AI CLI with local LLM and tools", long_about = None)]
struct Cli {
    /// Prompt string (if omitted, reads from stdin when piped)
    prompt: Vec<String>,

    /// LLM type (default: ollama)
    #[arg(long, env = "RUSTAICLI_LLM_TYPE", default_value = "ollama")]
    llm_type: String,

    /// Model name/path (for ollama, this is the model name like 'llama2')
    #[arg(long, env = "RUSTAICLI_MODEL_PATH", default_value = "llama2")] 
    model_path: String,

    /// Context size
    #[arg(long, env = "RUSTAICLI_CONTEXT_SIZE", default_value_t = 2048)]
    context_size: usize,

    /// Sampling temperature
    #[arg(long, env = "RUSTAICLI_TEMPERATURE", default_value_t = 0.7)]
    temperature: f32,

    /// List available tools
    #[arg(long, action = ArgAction::SetTrue)]
    list_tools: bool,

    /// Execute a tool by name instead of running the LLM
    #[arg(long)]
    tool: Option<String>,

    /// Key=Value pairs passed to the tool (repeatable)
    #[arg(long = "tool-arg")]
    tool_args: Vec<String>,
}

fn build_tool_registry() -> ToolRegistry {
    let mut reg = ToolRegistry::new();
    reg.register(std::sync::Arc::new(DirectoryTool::new()));
    reg.register(std::sync::Arc::new(FileTool::new()));
    reg.register(std::sync::Arc::new(ShellTool::new()));
    reg
}

fn parse_tool_kv(values: &[String]) -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    for v in values {
        if let Some((k, val)) = v.split_once('=') {
            map.insert(k.to_string(), val.to_string());
        } else {
            return Err(Error::Tool(format!("Invalid --tool-arg '{}', expected key=value", v)));
        }
    }
    Ok(map)
}

fn read_stdin_if_piped() -> Result<Option<String>> {
    if atty::is(atty::Stream::Stdin) {
        return Ok(None); // interactive tty, don't block
    }
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    if buf.trim().is_empty() { Ok(None) } else { Ok(Some(buf)) }
}

fn resolve_model_path(cli: &Cli) -> Result<String> {
    // For Ollama, we want to allow empty model path to default to llama2
    if cli.llm_type == "ollama" {
        if !cli.model_path.is_empty() {
            return Ok(cli.model_path.clone());
        }
        if let Ok(p) = env::var("RUSTAICLI_MODEL_PATH") {
            if !p.is_empty() { return Ok(p); }
        }
        return Ok("llama2".to_string());
    }
    
    // For other LLM types that need a file path
    if !cli.model_path.is_empty() {
        return Ok(cli.model_path.clone());
    }
    if let Ok(p) = env::var("RUSTAICLI_MODEL_PATH") {
        if !p.is_empty() { return Ok(p); }
    }
    Err(Error::Config("Model path not provided. Use --model-path or RUSTAICLI_MODEL_PATH".into()))
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    // Build tools
    let registry = build_tool_registry();

    if cli.list_tools {
        for (name, desc) in registry.list() {
            println!("{} - {}", name, desc);
        }
        return Ok(());
    }

    if let Some(tool_name) = &cli.tool {
        let tool = registry.get(tool_name)
            .ok_or_else(|| Error::Tool(format!("Unknown tool: {}", tool_name)))?;
        let args_map = parse_tool_kv(&cli.tool_args)?;
        let resp = tool.execute(ToolArgs { args: args_map }).await?;
        print!("{}", resp.output);
        return Ok(());
    }

    // Collect prompt input
    let stdin_text = read_stdin_if_piped()?;
    let mut prompt_parts = vec![];
    if !cli.prompt.is_empty() { prompt_parts.push(cli.prompt.join(" ")); }
    if let Some(s) = stdin_text { prompt_parts.push(s); }

    if prompt_parts.is_empty() {
        eprintln!("No input provided. Pass a prompt or pipe data. Try --help for usage.");
        std::process::exit(2);
    }

    let prompt = prompt_parts.join("\n\n");

    // Configure LLM
    let llm_type = cli.llm_type.to_lowercase();
    let config = LlmConfig {
        model_path: resolve_model_path(&cli)?,
        context_size: cli.context_size,
        temperature: cli.temperature,
    };

    // Build tool registry
    let registry = Arc::new(registry);

    match llm_type.as_str() {
        "ollama" => {
            let ollama = OllamaLlm::new(config, registry)?;
            let resp = ollama.generate(&prompt).await?;
            println!("{}", resp.text);
        }
        "llama" => {
            let llama = LlamaLlm::new(config)?;
            let resp = llama.generate(&prompt).await?;
            println!("{}", resp.text);
        }
        other => {
            return Err(Error::Config(format!(
                "Unsupported LLM type: {} (only 'ollama' and 'llama' implemented). You can extend by implementing the Llm trait.",
                other
            )));
        }
    }

    Ok(())
}
