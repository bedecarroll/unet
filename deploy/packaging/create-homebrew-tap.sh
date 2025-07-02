#!/bin/bash

# Script to create and manage a Homebrew tap for μNet
# Usage: ./create-homebrew-tap.sh [init|update|test]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TAP_NAME="homebrew-unet"
TAP_DIR="$PROJECT_ROOT/target/$TAP_NAME"
FORMULA_FILE="$SCRIPT_DIR/homebrew/unet.rb"

echo "Managing Homebrew tap for μNet..."

# Function to initialize a new tap repository
init_tap() {
    echo "Initializing Homebrew tap repository..."
    
    if [ -d "$TAP_DIR" ]; then
        echo "Warning: Tap directory already exists at $TAP_DIR"
        echo "Remove it manually if you want to start fresh."
        exit 1
    fi
    
    # Create tap directory structure
    mkdir -p "$TAP_DIR"
    cd "$TAP_DIR"
    
    # Initialize git repository
    git init
    
    # Create basic tap structure
    mkdir -p Formula
    
    # Copy formula file
    cp "$FORMULA_FILE" Formula/unet.rb
    
    # Create README
    cat > README.md << 'EOF'
# Homebrew Tap for μNet

This is the official Homebrew tap for μNet, a network configuration management and automation tool.

## Installation

```bash
# Add the tap
brew tap example/unet

# Install μNet
brew install unet

# Install with PostgreSQL support
brew install unet --with-postgresql
```

## Usage

```bash
# Start the server service
brew services start unet-server

# Stop the server service
brew services stop unet-server

# Check service status
brew services list | grep unet

# Use the CLI tool
unet --help
unet nodes list
unet health
```

## Configuration

Default configuration is installed at:
- `/opt/homebrew/etc/unet/config.toml` (Apple Silicon)
- `/usr/local/etc/unet/config.toml` (Intel)

Data and logs are stored in:
- `/opt/homebrew/var/lib/unet/` (Apple Silicon)
- `/opt/homebrew/var/log/unet/` (Apple Silicon)
- `/usr/local/var/lib/unet/` (Intel)
- `/usr/local/var/log/unet/` (Intel)

## Support

- Project: https://github.com/example/unet
- Issues: https://github.com/example/unet/issues
- Documentation: https://github.com/example/unet/docs
EOF
    
    # Create initial commit
    git add .
    git commit -m "Initial commit: Add μNet formula"
    
    echo "Homebrew tap initialized at: $TAP_DIR"
    echo "To publish this tap:"
    echo "1. Create a GitHub repository named 'homebrew-unet'"
    echo "2. Push this directory to the repository:"
    echo "   cd $TAP_DIR"
    echo "   git remote add origin https://github.com/YOUR_USERNAME/homebrew-unet.git"
    echo "   git branch -M main"
    echo "   git push -u origin main"
}

# Function to update formula with new version
update_formula() {
    echo "Updating formula..."
    
    if [ ! -d "$TAP_DIR" ]; then
        echo "Error: Tap directory not found. Run '$0 init' first."
        exit 1
    fi
    
    # Copy updated formula
    cp "$FORMULA_FILE" "$TAP_DIR/Formula/unet.rb"
    
    cd "$TAP_DIR"
    
    # Check if there are changes
    if git diff --quiet; then
        echo "No changes to commit."
        return
    fi
    
    # Get version from formula
    VERSION=$(grep 'url.*v[0-9]' Formula/unet.rb | sed 's/.*v\([0-9.]*\).*/\1/')
    
    # Commit changes
    git add Formula/unet.rb
    git commit -m "Update μNet to version $VERSION"
    
    echo "Formula updated to version $VERSION"
    echo "Push changes with: cd $TAP_DIR && git push"
}

# Function to test formula locally
test_formula() {
    echo "Testing formula locally..."
    
    if [ ! -f "$FORMULA_FILE" ]; then
        echo "Error: Formula file not found at $FORMULA_FILE"
        exit 1
    fi
    
    # Check formula syntax
    if command -v brew >/dev/null 2>&1; then
        echo "Testing formula syntax..."
        brew ruby -e "
          require 'formula'
          load '$FORMULA_FILE'
          puts 'Formula syntax is valid'
        "
        
        # Run brew audit if available
        if [ -d "$TAP_DIR" ]; then
            cd "$TAP_DIR"
            echo "Running brew audit..."
            brew audit --formula Formula/unet.rb || true
        fi
        
        echo "Formula testing completed."
    else
        echo "Homebrew not installed. Skipping formula testing."
        echo "Install Homebrew: /bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
    fi
}

# Function to calculate SHA256 for source archive
calculate_sha256() {
    echo "Calculating SHA256 for source archive..."
    
    VERSION="0.1.0"
    TARBALL_URL="https://github.com/example/unet/archive/v${VERSION}.tar.gz"
    
    echo "Download the source archive and calculate SHA256:"
    echo "curl -L '$TARBALL_URL' | shasum -a 256"
    echo ""
    echo "Then update the sha256 value in the formula file."
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  init      Initialize a new Homebrew tap repository"
    echo "  update    Update formula with latest changes"
    echo "  test      Test formula syntax and run audit"
    echo "  sha256    Show instructions for calculating source SHA256"
    echo "  help      Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 init       # Create new tap repository"
    echo "  $0 update     # Update formula and commit changes"
    echo "  $0 test       # Test formula locally"
}

# Main execution
case "${1:-help}" in
    init)
        init_tap
        ;;
    update)
        update_formula
        ;;
    test)
        test_formula
        ;;
    sha256)
        calculate_sha256
        ;;
    help|--help|-h)
        show_usage
        ;;
    *)
        echo "Error: Unknown command '$1'"
        echo ""
        show_usage
        exit 1
        ;;
esac