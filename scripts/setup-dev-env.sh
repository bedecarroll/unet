#!/bin/bash
# μNet Development Environment Setup Script
# This script sets up a complete development environment for μNet

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "🦀 Setting up μNet development environment..."
echo "Project root: $PROJECT_ROOT"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_step() {
    echo -e "${BLUE}📦 $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
print_step "Checking prerequisites..."

# Check for Rust
if ! command_exists rustc; then
    print_error "Rust is not installed. Please install Rust from https://rustup.rs/"
    exit 1
fi

# Check Rust version
rust_version=$(rustc --version | cut -d' ' -f2)
print_success "Found Rust $rust_version"

# Check for required Rust components
print_step "Checking Rust components..."
if ! rustup component list --installed | grep -q "clippy"; then
    print_step "Installing clippy..."
    rustup component add clippy
fi

if ! rustup component list --installed | grep -q "rustfmt"; then
    print_step "Installing rustfmt..."
    rustup component add rustfmt
fi

if ! rustup component list --installed | grep -q "rust-src"; then
    print_step "Installing rust-src..."
    rustup component add rust-src
fi

print_success "All required Rust components are installed"

# Change to project directory
cd "$PROJECT_ROOT"

# Check dependencies
print_step "Checking and fetching dependencies..."
if ! cargo fetch; then
    print_error "Failed to fetch dependencies"
    exit 1
fi

# Run initial build to ensure everything works
print_step "Running initial build..."
if ! cargo check --workspace; then
    print_error "Initial build failed"
    exit 1
fi

print_success "Initial build successful"

# Set up git hooks
print_step "Setting up git hooks..."
if [ -f ".git/hooks/pre-commit" ]; then
    print_success "Pre-commit hook already exists"
else
    print_warning "Pre-commit hook not found - please check installation"
fi

# Check for additional tools
print_step "Checking for additional development tools..."

# mdbook for documentation
if command_exists mdbook; then
    print_success "mdbook is available for documentation building"
else
    print_warning "mdbook not found - install with 'cargo install mdbook' for documentation"
fi

# sea-orm-cli for database migrations
if command_exists sea-orm-cli; then
    print_success "sea-orm-cli is available for database migrations"
else
    print_warning "sea-orm-cli not found - install with 'cargo install sea-orm-cli' for database work"
fi

# cargo-watch for development
if command_exists cargo-watch; then
    print_success "cargo-watch is available for auto-rebuild"
else
    print_warning "cargo-watch not found - install with 'cargo install cargo-watch' for development"
fi

# Run tests to ensure everything is working
print_step "Running test suite..."
if cargo test --workspace; then
    print_success "All tests passed"
else
    print_warning "Some tests failed - this might be expected in development"
fi

# Create database and run migrations
print_step "Setting up database..."
if [ -f "migrations/src/lib.rs" ]; then
    if ./scripts/migrate.sh; then
        print_success "Database migration completed"
    else
        print_warning "Database migration failed - you may need to set up manually"
    fi
else
    print_warning "Migration script not found"
fi

# VS Code setup check
if [ -d ".vscode" ]; then
    print_success "VS Code configuration found"
    if command_exists code; then
        print_step "You can open the project in VS Code with: code ."
    fi
else
    print_warning "No VS Code configuration found"
fi

echo ""
echo "🎉 Development environment setup complete!"
echo ""
echo "📚 Quick start commands:"
echo "  cargo run --bin unet-server              # Start the server"
echo "  cargo run --bin unet -- nodes list       # Use the CLI"
echo "  cargo test --workspace                    # Run all tests"
echo "  cargo fmt                                 # Format code"
echo "  cargo clippy                              # Run linter"
echo "  mdbook serve docs                         # Serve documentation"
echo ""
echo "📖 See README.md and docs/ for more information"
echo "🐛 Report issues at: https://github.com/your-org/unet/issues"