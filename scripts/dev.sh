#!/usr/bin/env bash

# Development script for RustAICLI
# Usage: ./scripts/dev.sh [command]

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if Rust is installed
check_rust() {
    if ! command -v rustc &> /dev/null; then
        error "Rust not found. Install from https://rustup.rs/"
        exit 1
    fi
    
    local rust_version=$(rustc --version | cut -d' ' -f2)
    log "Using Rust version: $rust_version"
}

# Check if required tools are available
check_tools() {
    local missing_tools=()
    
    if ! command -v cargo &> /dev/null; then
        missing_tools+=("cargo")
    fi
    
    if [[ ${#missing_tools[@]} -gt 0 ]]; then
        error "Missing required tools: ${missing_tools[*]}"
        exit 1
    fi
}

# Run tests
test() {
    log "Running tests..."
    cargo test --all-features
    success "Tests passed"
}

# Run linting
lint() {
    log "Running clippy..."
    cargo clippy --all-targets --all-features -- -D warnings
    
    log "Checking formatting..."
    cargo fmt --all -- --check
    
    success "Linting passed"
}

# Format code
fmt() {
    log "Formatting code..."
    cargo fmt --all
    success "Code formatted"
}

# Build debug version
build() {
    log "Building debug version..."
    cargo build
    success "Debug build completed"
}

# Build release version
release() {
    log "Building release version..."
    cargo build --release
    success "Release build completed"
    
    local binary_size=$(du -h target/release/rustaicli | cut -f1)
    log "Binary size: $binary_size"
}

# Run security audit
audit() {
    log "Running security audit..."
    if ! command -v cargo-audit &> /dev/null; then
        warn "cargo-audit not found, installing..."
        cargo install cargo-audit
    fi
    
    cargo audit
    success "Security audit passed"
}

# Clean build artifacts
clean() {
    log "Cleaning build artifacts..."
    cargo clean
    success "Clean completed"
}

# Setup development environment
setup() {
    log "Setting up development environment..."
    
    # Install development tools
    local tools=("cargo-watch" "cargo-expand" "cargo-audit")
    
    for tool in "${tools[@]}"; do
        if ! cargo install --list | grep -q "^$tool "; then
            log "Installing $tool..."
            cargo install "$tool"
        else
            log "$tool already installed"
        fi
    done
    
    success "Development environment setup completed"
}

# Watch for changes and rebuild
watch() {
    log "Starting file watcher..."
    if ! command -v cargo-watch &> /dev/null; then
        error "cargo-watch not found. Run './scripts/dev.sh setup' first"
        exit 1
    fi
    
    cargo watch -x check -x test -x run
}

# Run all quality checks
check_all() {
    log "Running all quality checks..."
    
    check_rust
    check_tools
    test
    lint
    audit
    
    success "All checks passed!"
}

# Create release build with all targets
release_all() {
    log "Building for all release targets..."
    
    local targets=(
        "x86_64-unknown-linux-gnu"
        "x86_64-unknown-linux-musl"
        "x86_64-apple-darwin"
        "aarch64-apple-darwin"
        "x86_64-pc-windows-gnu"
    )
    
    # Create dist directory
    mkdir -p dist
    
    for target in "${targets[@]}"; do
        log "Building for $target..."
        
        # Add target if not already added
        rustup target add "$target" 2>/dev/null || true
        
        # Build for target
        if cargo build --release --target "$target"; then
            success "Built for $target"
            
            # Copy to dist directory
            local binary_name="rustaicli"
            if [[ $target == *"windows"* ]]; then
                binary_name="rustaicli.exe"
            fi
            
            if [ -f "target/$target/release/$binary_name" ]; then
                mkdir -p "dist/$target"
                cp "target/$target/release/$binary_name" "dist/$target/"
                cp README.md "dist/$target/"
            fi
        else
            warn "Failed to build for $target (skipping)"
        fi
    done
    
    success "Release builds completed. Check dist/ directory"
}

# Show help
help() {
    cat << EOF
RustAICLI Development Script

Usage: $0 [command]

Commands:
    build       Build debug version
    release     Build optimized release version
    test        Run all tests
    lint        Run clippy and format checks
    fmt         Format code
    audit       Run security audit
    clean       Clean build artifacts
    setup       Setup development environment
    watch       Watch for changes and rebuild
    check       Run all quality checks
    release-all Build for all release targets
    help        Show this help message

Examples:
    $0 build
    $0 test
    $0 check
    $0 watch

EOF
}

# Main command dispatcher
main() {
    case "${1:-help}" in
        "build")
            check_rust
            build
            ;;
        "release")
            check_rust
            release
            ;;
        "test")
            check_rust
            test
            ;;
        "lint")
            check_rust
            lint
            ;;
        "fmt")
            check_rust
            fmt
            ;;
        "audit")
            check_rust
            audit
            ;;
        "clean")
            clean
            ;;
        "setup")
            check_rust
            setup
            ;;
        "watch")
            check_rust
            watch
            ;;
        "check")
            check_all
            ;;
        "release-all")
            check_rust
            release_all
            ;;
        "help"|"-h"|"--help")
            help
            ;;
        *)
            error "Unknown command: $1"
            echo
            help
            exit 1
            ;;
    esac
}

main "$@"