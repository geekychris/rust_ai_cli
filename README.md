# RustAICLI

A high-performance, extensible command-line tool for AI prompt processing with pluggable LLM backends and a comprehensive system tools integration. Built in Rust for speed, safety, and reliability.

## Features

- **Flexible Input Modes**: Interactive prompt mode and pipe input support
- **Multiple LLM Backends**: 
  - Ollama integration for easy model management
  - Local llama.cpp for direct model execution
  - Extensible trait-based architecture for custom backends
- **Intelligent Tool Integration**: AI can automatically invoke system tools based on user requests
- **Built-in System Tools**:
  - Directory operations (list, recursive list)
  - File operations (read, write, append)
  - Shell command execution
- **Configuration Flexibility**: Environment variables and command-line configuration
- **Async/Await Architecture**: Non-blocking I/O for better performance

## Quick Start

### Installation

```bash
# Install from source
cargo install --path .

# Or build locally
cargo build --release
./target/release/rustaicli --help
```

### Prerequisites

- **Rust**: 1.70+ (uses async/await and modern Rust features)
- **Ollama** (recommended): For easy LLM management
  ```bash
  # Install Ollama (macOS)
  brew install ollama
  # Pull a model
  ollama pull llama2
  ```
- **llama.cpp models** (optional): For direct model execution

## Usage Examples

### Basic AI Queries

```bash
# Simple question
rustaicli "What is the difference between async and sync programming?"

# Process piped input
echo "Review this code and suggest improvements" | rustaicli

# File analysis with automatic tool usage
rustaicli "Analyze the structure of my current directory"
# The AI will automatically use the directory tool to list files
```

### Advanced Usage with Tools

```bash
# Direct tool execution
rustaicli --tool directory --tool-arg path=./src --tool-arg recursive=true
rustaicli --tool file --tool-arg operation=read --tool-arg path=./README.md
rustaicli --tool shell --tool-arg command="ls -la"

# List available tools
rustaicli --list-tools
```

### Configuration Options

#### Environment Variables
```bash
export RUSTAICLI_LLM_TYPE=ollama          # or "llama"
export RUSTAICLI_MODEL_PATH=llama2         # for Ollama: model name, for llama: file path
export RUSTAICLI_CONTEXT_SIZE=4096         # Token context window
export RUSTAICLI_TEMPERATURE=0.7           # Sampling temperature (0.0-1.0)
```

#### Command-line Flags
```bash
# Use specific LLM backend
rustaicli --llm-type ollama --model-path codellama "Help me debug this code"
rustaicli --llm-type llama --model-path ./models/llama-7b.gguf "Explain async Rust"

# Adjust generation parameters
rustaicli --temperature 0.3 --context-size 2048 "Write production-ready code"
```

## System Architecture

RustAICLI is built with a modular, trait-based architecture that emphasizes extensibility, performance, and type safety.

### Core Design Principles

- **Plugin Architecture**: Both LLM backends and tools use trait-based interfaces
- **Async-First**: Built on Tokio for non-blocking I/O and concurrent operations
- **Type Safety**: Extensive use of Rust's type system to prevent runtime errors
- **Zero-Copy**: Minimal string allocation and efficient data handling
- **Error Handling**: Comprehensive error types with context using `thiserror`

### System Components

```
┌─────────────────────────────────────────────────────────────┐
│                     CLI Interface (main.rs)                 │
│  • Argument parsing (clap)                                  │
│  • Input handling (stdin/direct)                           │
│  • Configuration management                                 │
│  • Tool registry initialization                            │
└─────────────────┬───────────────────────────┬───────────────┘
                  │                           │
┌─────────────────▼───────────────┐  ┌────────▼──────────────┐
│         LLM Interface           │  │     Tool System       │
│  • Trait-based backend design  │  │  • Dynamic registry   │
│  • Ollama HTTP client          │  │  • Built-in tools     │
│  • llama.cpp integration       │  │  • Validation system  │
│  • Async generation            │  │  • JSON tool calling  │
└─────────────────┬───────────────┘  └───────────────────────┘
                  │                           
┌─────────────────▼───────────────┐           
│       Prompt Template           │           
│  • Tool-aware prompt formatting │           
│  • JSON parsing for tool calls  │           
│  • Multi-turn conversations     │           
└─────────────────────────────────┘           
```

### LLM Backend Architecture

LLM backends implement the `Llm` trait for consistent interfaces:

```rust
#[async_trait]
pub trait Llm: Send + Sync {
    fn name(&self) -> &str;
    async fn generate(&self, prompt: &str) -> Result<LlmResponse>;
    fn config(&self) -> &LlmConfig;
}
```

**Available Backends:**

1. **OllamaLlm**: HTTP client for Ollama server
   - Auto-discovery of localhost:11434
   - Tool integration via prompt templates
   - Multi-turn conversations for tool usage
   - Configurable temperature and model selection

2. **LlamaLlm**: Direct integration with llama.cpp
   - Local model file loading
   - Memory-efficient inference
   - Streaming token generation

### Tool System Architecture

The tool system enables AI agents to perform system operations through a safe, validated interface:

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    async fn execute(&self, args: ToolArgs) -> Result<ToolResponse>;
}
```

**Built-in Tools:**

- **DirectoryTool**: List directory contents with optional recursion
- **FileTool**: Read, write, append files with validation
- **ShellTool**: Execute shell commands with safety constraints

**Tool Registry**: Central registry with validation and discovery

```rust
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}
```

## Building and Development

### System Requirements

- **Rust**: 1.70 or later (MSRV)
- **Operating System**: 
  - Linux (tested on Ubuntu 20.04+)
  - macOS 11+ (Intel and Apple Silicon)
  - Windows 10+ (limited testing)

### Dependencies

#### Core Dependencies
```toml
[dependencies]
clap = { version = "4.4.6", features = ["derive", "env"] }  # CLI parsing
tokio = { version = "1.32.0", features = ["full"] }         # Async runtime
serde = { version = "1.0", features = ["derive"] }          # Serialization
serde_json = "1.0"                                          # JSON handling
anyhow = "1.0"                                              # Error context
thiserror = "1.0"                                           # Error types
tracing = "0.1"                                             # Logging
tracing-subscriber = { version = "0.3.20", features = ["env-filter"] }
llm = { version = "0.1.1" }                                 # LLM base
llm-base = "0.1.1"                                          # LLM foundation
llm-llama = "0.1.1"                                         # Llama.cpp integration
async-trait = "0.1.75"                                      # Async traits
reqwest = { version = "0.11", features = ["json"] }        # HTTP client
```

### Build Process

#### Development Build
```bash
# Clone the repository
git clone <repository-url>
cd rustaicli

# Check Rust version
rustc --version  # Should be 1.70+

# Run tests (recommended)
cargo test

# Development build with debug symbols
cargo build

# Run locally
./target/debug/rustaicli --help
```

#### Optimized Build
```bash
# Release build (optimized)
cargo build --release

# The binary will be in target/release/rustaicli
ls -la target/release/rustaicli
```

#### Build Configuration Options

```bash
# Build with specific features
cargo build --release --features "full-features"

# Cross-compilation (example for Linux on macOS)
rustup target add x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu

# Static linking (for distribution)
RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target x86_64-unknown-linux-musl
```

### Release Build Process

#### Automated Release Pipeline

1. **Version Management**:
   ```bash
   # Update version in Cargo.toml
   sed -i 's/version = "0.1.0"/version = "0.2.0"/' Cargo.toml
   
   # Update version references in README
   # Commit version bump
   git commit -am "Bump version to 0.2.0"
   git tag v0.2.0
   ```

2. **Multi-platform Builds**:
   ```bash
   # Build for multiple targets
   declare -a targets=("x86_64-unknown-linux-gnu" 
                      "x86_64-unknown-linux-musl"
                      "x86_64-apple-darwin"
                      "aarch64-apple-darwin"
                      "x86_64-pc-windows-gnu")
   
   for target in "${targets[@]}"; do
     echo "Building for $target"
     rustup target add $target
     cargo build --release --target $target
   done
   ```

3. **Distribution Packaging**:
   ```bash
   # Create distribution directory
   mkdir -p dist/
   
   # Copy binaries and create archives
   for target in "${targets[@]}"; do
     binary_name="rustaicli"
     if [[ $target == *"windows"* ]]; then
       binary_name="rustaicli.exe"
     fi
     
     mkdir -p "dist/rustaicli-$target"
     cp "target/$target/release/$binary_name" "dist/rustaicli-$target/"
     cp README.md LICENSE "dist/rustaicli-$target/"
     
     cd dist
     tar czf "rustaicli-$target.tar.gz" "rustaicli-$target/"
     cd ..
   done
   ```

#### GitHub Actions CI/CD (Recommended)

Create `.github/workflows/release.yml`:
```yaml
name: Release

on:
  push:
    tags: ['v*']

jobs:
  build:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-msvc

    steps:
    - uses: actions/checkout@v3
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        target: ${{ matrix.target }}
    - name: Build
      run: cargo build --release --target ${{ matrix.target }}
    - name: Upload artifact
      uses: actions/upload-artifact@v3
      with:
        name: rustaicli-${{ matrix.target }}
        path: target/${{ matrix.target }}/release/rustaicli*
```

## Extending the System

### Adding New Tools

The tool system is designed for easy extension. Here's how to add a new tool:

#### 1. Create Tool Implementation

```rust
// src/tools/my_tool.rs
use async_trait::async_trait;
use crate::error::{Result, Error};
use super::{Tool, ToolArgs, ToolResponse};

pub struct MyCustomTool;

impl MyCustomTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for MyCustomTool {
    fn name(&self) -> &str {
        "my_tool"
    }

    fn description(&self) -> &str {
        "My custom tool that does amazing things"
    }

    async fn execute(&self, args: ToolArgs) -> Result<ToolResponse> {
        // Validate required arguments
        let input = args.args.get("input")
            .ok_or_else(|| Error::Tool("input argument required".to_string()))?;
        
        // Optional argument with default
        let format = args.args.get("format").unwrap_or(&"default".to_string());
        
        // Tool logic here
        let result = perform_custom_operation(input, format).await?;
        
        Ok(ToolResponse {
            output: result,
        })
    }
}

async fn perform_custom_operation(input: &str, format: &str) -> Result<String> {
    // Your custom logic here
    Ok(format!("Processed: {} with format: {}", input, format))
}
```

#### 2. Register the Tool

```rust
// src/tools/mod.rs
mod my_tool;
pub use my_tool::MyCustomTool;

// In src/main.rs, update build_tool_registry()
fn build_tool_registry() -> ToolRegistry {
    let mut reg = ToolRegistry::new();
    reg.register(std::sync::Arc::new(DirectoryTool::new()));
    reg.register(std::sync::Arc::new(FileTool::new()));
    reg.register(std::sync::Arc::new(ShellTool::new()));
    reg.register(std::sync::Arc::new(MyCustomTool::new())); // Add your tool
    reg
}
```

#### 3. Add Validation (Optional)

```rust
// In src/tools/mod.rs, update validate_args method
pub fn validate_args(&self, tool_name: &str, args: &ToolArgs) -> Result<()> {
    match tool_name {
        "my_tool" => {
            args.args.get("input")
                .ok_or_else(|| Error::Tool("my_tool requires 'input' argument".to_string()))?;
        }
        // ... existing validations
        _ => return Err(Error::Tool(format!("Unknown tool: {}", tool_name)))
    }
    Ok(())
}
```

### Adding New LLM Backends

To add support for a new LLM backend (e.g., OpenAI, Anthropic, or custom inference servers):

#### 1. Implement the Llm Trait

```rust
// src/llm/openai.rs
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use crate::error::{Result, Error};
use super::{Llm, LlmConfig, LlmResponse};

pub struct OpenAiLlm {
    config: LlmConfig,
    client: Client,
    api_key: String,
}

impl OpenAiLlm {
    pub fn new(config: LlmConfig) -> Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| Error::Config("OPENAI_API_KEY environment variable required".into()))?;
        
        Ok(Self {
            config,
            client: Client::new(),
            api_key,
        })
    }
}

#[async_trait]
impl Llm for OpenAiLlm {
    fn name(&self) -> &str {
        "openai"
    }
    
    async fn generate(&self, prompt: &str) -> Result<LlmResponse> {
        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&json!({
                "model": self.config.model_path,
                "messages": [
                    {"role": "user", "content": prompt}
                ],
                "temperature": self.config.temperature,
                "max_tokens": self.config.context_size
            }))
            .send()
            .await
            .map_err(|e| Error::Llm(format!("OpenAI request failed: {}", e)))?;
            
        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::Llm(format!("Failed to parse OpenAI response: {}", e)))?;
        
        let text = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| Error::Llm("Invalid OpenAI response format".into()))?
            .to_string();
        
        Ok(LlmResponse { text })
    }
    
    fn config(&self) -> &LlmConfig {
        &self.config
    }
}
```

#### 2. Register the Backend

```rust
// src/llm/mod.rs
pub mod openai;
pub use openai::OpenAiLlm;

// In src/main.rs, update the match statement
match llm_type.as_str() {
    "ollama" => { /* existing */ }
    "llama" => { /* existing */ }
    "openai" => {
        let openai = OpenAiLlm::new(config)?;
        let resp = openai.generate(&prompt).await?;
        println!("{}", resp.text);
    }
    // ...
}
```

### Advanced Extension Patterns

#### Custom Configuration

```rust
// Extend LlmConfig for backend-specific options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomLlmConfig {
    pub base: LlmConfig,
    pub custom_endpoint: String,
    pub auth_token: Option<String>,
    pub retry_attempts: u32,
}
```

#### Plugin System (Future Enhancement)

```rust
// Future: Dynamic loading of tools
pub trait PluginLoader {
    fn load_tools(&self) -> Result<Vec<Box<dyn Tool>>>;
    fn load_llm_backends(&self) -> Result<Vec<Box<dyn LlmBackendFactory>>>;
}
```

## Development Workflow

### Testing Strategy

#### Unit Tests
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test tools::tests

# Test with different features
cargo test --all-features
```

#### Integration Tests
Create tests that verify end-to-end functionality:

```rust
// tests/integration_test.rs
use rustaicli::tools::*;
use tokio_test;

#[tokio::test]
async fn test_file_tool_integration() {
    let tool = FileTool::new();
    let args = ToolArgs {
        args: [(
            ("operation".to_string(), "read".to_string()),
            ("path".to_string(), "Cargo.toml".to_string()),
        )].iter().cloned().collect(),
    };
    
    let result = tool.execute(args).await;
    assert!(result.is_ok());
    assert!(result.unwrap().output.contains("[package]"));
}
```

### Debugging and Development Tips

#### Logging
```bash
# Enable debug logging
RUST_LOG=debug cargo run -- "test prompt"

# Trace-level logging for specific modules
RUST_LOG=rustaicli::llm=trace,rustaicli::tools=debug cargo run

# Log to file
RUST_LOG=debug cargo run -- "test" 2> debug.log
```

#### Profiling
```bash
# Profile release build
cargo build --release
perf record --call-graph=dwarf ./target/release/rustaicli "complex prompt"
perf report

# Memory usage analysis
valgrind --tool=massif ./target/release/rustaicli "prompt"
```

#### Development Environment Setup

```bash
# Install development tools
cargo install cargo-watch     # Auto-rebuild on changes
cargo install cargo-expand    # Macro expansion
cargo install cargo-audit     # Security audit

# Development workflow
cargo watch -x check -x test -x run

# Code formatting and linting
cargo fmt
cargo clippy -- -D warnings

# Security audit
cargo audit
```

### Code Quality and Best Practices

#### Code Style
- Follow Rust standard formatting (`cargo fmt`)
- Address all Clippy warnings (`cargo clippy`)
- Use meaningful variable and function names
- Add comprehensive documentation (`cargo doc`)

#### Error Handling
```rust
// Prefer specific error types
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("Invalid configuration: {msg}")]
    Config { msg: String },
}

// Use Result<T> consistently
pub type Result<T> = std::result::Result<T, MyError>;
```

#### Async Best Practices
```rust
// Use async/await properly
pub async fn process_data(data: &str) -> Result<String> {
    let result = expensive_operation(data).await?;
    let processed = transform_data(result).await?;
    Ok(processed)
}

// Handle concurrent operations
use tokio::task::JoinSet;

pub async fn process_multiple(items: Vec<String>) -> Result<Vec<String>> {
    let mut set = JoinSet::new();
    
    for item in items {
        set.spawn(async move {
            process_single_item(item).await
        });
    }
    
    let mut results = Vec::new();
    while let Some(res) = set.join_next().await {
        results.push(res??);
    }
    
    Ok(results)
}
```

### Contributing Guidelines

1. **Fork and Branch**: Create a feature branch from `main`
2. **Code Quality**: Ensure tests pass and code follows style guidelines
3. **Documentation**: Update README and add inline documentation for new features
4. **Testing**: Add unit tests for new functionality
5. **Performance**: Consider performance implications of changes
6. **Backward Compatibility**: Avoid breaking changes when possible

#### Pull Request Process

```bash
# Before submitting PR
cargo test                    # All tests pass
cargo clippy -- -D warnings  # No warnings
cargo fmt                     # Code formatted
cargo audit                   # No security issues

# Update documentation if needed
cargo doc --no-deps --open
```

### Performance Optimization

#### Profiling Bottlenecks
```bash
# CPU profiling
cargo build --release
./target/release/rustaicli "complex prompt" &
perf top -p $(pgrep rustaicli)

# Memory profiling with heaptrack
heaptrack ./target/release/rustaicli "prompt"
```

#### Common Optimization Patterns
```rust
// Use Arc<str> for shared strings
use std::sync::Arc;
pub type SharedString = Arc<str>;

// Avoid unnecessary allocations
pub fn process_efficient(data: &str) -> String {
    // Use with_capacity when size is known
    let mut result = String::with_capacity(data.len() * 2);
    
    // Use iterators instead of collections when possible
    for line in data.lines() {
        result.push_str(&line.to_uppercase());
        result.push('\n');
    }
    
    result
}

// Use async streaming for large data
use tokio_stream::{Stream, StreamExt};

pub async fn process_stream<S>(mut stream: S) -> Result<Vec<String>>
where
    S: Stream<Item = String> + Unpin,
{
    let mut results = Vec::new();
    while let Some(item) = stream.next().await {
        results.push(process_item(item).await?);
    }
    Ok(results)
}
```

## Troubleshooting

### Common Issues

#### Ollama Connection Failed
```bash
# Ensure Ollama is running
ollama serve

# Test connection
curl http://localhost:11434/api/version

# Check if model is available
ollama list
```

#### llama.cpp Model Loading Failed
```bash
# Verify model file exists and is readable
ls -la /path/to/model.gguf

# Check model compatibility
file /path/to/model.gguf
```

#### Tool Execution Errors
```bash
# Enable debug logging to see tool execution details
RUST_LOG=rustaicli::tools=debug rustaicli "your command"

# Test tool directly
rustaicli --tool file --tool-arg operation=read --tool-arg path=./test.txt
```

### Performance Tuning

#### Memory Usage
- Adjust `context_size` based on available RAM
- Use smaller models for faster inference
- Consider model quantization for reduced memory usage

#### CPU Optimization
- Enable CPU-specific optimizations during build:
  ```bash
  RUSTFLAGS="-C target-cpu=native" cargo build --release
  ```

## Roadmap

### Planned Features
- [ ] Plugin system for dynamic tool loading
- [ ] Streaming responses for better UX
- [ ] Multiple conversation contexts
- [ ] Web UI for easier interaction
- [ ] Integration with more LLM providers
- [ ] Tool marketplace/registry
- [ ] Configuration profiles
- [ ] Batch processing mode

### Performance Goals
- Sub-second startup time
- Memory usage under 100MB for basic operations
- Support for models up to 70B parameters

## Security Considerations

- **Shell Tool**: Commands are executed in the user's environment with their permissions
- **File Tool**: Limited to readable/writable paths based on user permissions
- **Network**: HTTP requests made to configured endpoints only
- **Secrets**: Environment variables used for API keys - never logged

## License

MIT License - see LICENSE file for details.

## Acknowledgments

- [llama.cpp](https://github.com/ggerganov/llama.cpp) for efficient local inference
- [Ollama](https://ollama.ai/) for easy model management
- [Tokio](https://tokio.rs/) for async runtime
- [clap](https://clap.rs/) for CLI parsing

---

**Version**: 0.1.0  
**Minimum Rust Version**: 1.70  
**Status**: Alpha - API may change

For questions, issues, or contributions, please visit the [repository](https://github.com/your-username/rustaicli).
