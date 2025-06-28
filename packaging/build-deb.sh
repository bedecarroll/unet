#!/bin/bash

# Build script for creating Debian packages for μNet
# Usage: ./build-deb.sh [clean]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_ROOT/target/debian-build"

echo "Building Debian packages for μNet..."
echo "Project root: $PROJECT_ROOT"
echo "Build directory: $BUILD_DIR"

# Function to check for required tools
check_dependencies() {
    local missing=()
    
    if ! command -v dpkg-buildpackage >/dev/null 2>&1; then
        missing+=("dpkg-dev")
    fi
    
    if ! command -v dh >/dev/null 2>&1; then
        missing+=("debhelper")
    fi
    
    if ! command -v cargo >/dev/null 2>&1; then
        missing+=("cargo")
    fi
    
    if ! command -v rustc >/dev/null 2>&1; then
        missing+=("rustc")
    fi
    
    if [ ${#missing[@]} -ne 0 ]; then
        echo "Error: Missing required dependencies: ${missing[*]}"
        echo "Install with: sudo apt-get install ${missing[*]}"
        exit 1
    fi
}

# Function to clean previous builds
clean_build() {
    echo "Cleaning previous build artifacts..."
    rm -rf "$BUILD_DIR"
    cd "$PROJECT_ROOT"
    cargo clean
}

# Function to prepare build environment
prepare_build() {
    echo "Preparing build environment..."
    
    # Create build directory
    mkdir -p "$BUILD_DIR"
    
    # Copy source code to build directory
    cd "$PROJECT_ROOT"
    tar --exclude-vcs --exclude='target' --exclude='packaging/debian-build' \
        -czf "$BUILD_DIR/unet_0.1.0.orig.tar.gz" .
    
    # Extract source for building
    cd "$BUILD_DIR"
    tar -xzf unet_0.1.0.orig.tar.gz
    mv unet_0.1.0.orig.tar.gz unet_0.1.0.orig.tar.gz.bak || true
    
    # Copy Debian packaging files
    mkdir -p debian
    cp -r "$PROJECT_ROOT/packaging/debian"/* debian/
}

# Function to build packages
build_packages() {
    echo "Building Debian packages..."
    cd "$BUILD_DIR"
    
    # Ensure we have the required Rust version
    echo "Checking Rust version..."
    RUST_VERSION=$(rustc --version | awk '{print $2}')
    echo "Rust version: $RUST_VERSION"
    
    # Build packages
    dpkg-buildpackage -us -uc -b
    
    echo "Build completed successfully!"
    echo "Packages created in: $BUILD_DIR/../"
    ls -la "$BUILD_DIR"/../*.deb
}

# Function to test packages
test_packages() {
    echo "Testing package installation..."
    cd "$BUILD_DIR/.."
    
    # Test package metadata
    for deb in *.deb; do
        echo "Testing package: $deb"
        dpkg-deb --info "$deb"
        dpkg-deb --contents "$deb" | head -20
        echo ""
    done
    
    # Check for lintian issues (if available)
    if command -v lintian >/dev/null 2>&1; then
        echo "Running lintian checks..."
        lintian *.deb || true
    fi
}

# Main execution
main() {
    cd "$PROJECT_ROOT"
    
    # Check if clean was requested
    if [ "$1" = "clean" ]; then
        clean_build
        echo "Clean completed."
        exit 0
    fi
    
    # Check dependencies
    check_dependencies
    
    # Clean previous builds
    clean_build
    
    # Prepare build environment
    prepare_build
    
    # Build packages
    build_packages
    
    # Test packages
    test_packages
    
    echo ""
    echo "Debian packages built successfully!"
    echo "Install with:"
    echo "  sudo dpkg -i $BUILD_DIR/../unet_0.1.0-1_*.deb $BUILD_DIR/../unet-server_0.1.0-1_*.deb"
    echo "  sudo apt-get install -f  # Fix any dependency issues"
}

# Run main function
main "$@"