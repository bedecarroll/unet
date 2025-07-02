#!/bin/bash

# Build script for creating RPM packages for μNet
# Usage: ./build-rpm.sh [clean]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_ROOT/target/rpm-build"
RPM_BUILD_ROOT="$BUILD_DIR/rpmbuild"

echo "Building RPM packages for μNet..."
echo "Project root: $PROJECT_ROOT"
echo "Build directory: $BUILD_DIR"

# Function to check for required tools
check_dependencies() {
    local missing=()
    
    if ! command -v rpmbuild >/dev/null 2>&1; then
        missing+=("rpm-build")
    fi
    
    if ! command -v cargo >/dev/null 2>&1; then
        missing+=("cargo")
    fi
    
    if ! command -v rustc >/dev/null 2>&1; then
        missing+=("rustc")
    fi
    
    if [ ${#missing[@]} -ne 0 ]; then
        echo "Error: Missing required dependencies: ${missing[*]}"
        if command -v dnf >/dev/null 2>&1; then
            echo "Install with: sudo dnf install ${missing[*]} gcc openssl-devel sqlite-devel postgresql-devel pkgconfig"
        elif command -v yum >/dev/null 2>&1; then
            echo "Install with: sudo yum install ${missing[*]} gcc openssl-devel sqlite-devel postgresql-devel pkgconfig"
        else
            echo "Install the missing dependencies using your package manager."
        fi
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
    
    # Create RPM build directory structure
    mkdir -p "$RPM_BUILD_ROOT"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
    
    # Copy spec file
    cp "$PROJECT_ROOT/packaging/rpm/SPECS/unet.spec" "$RPM_BUILD_ROOT/SPECS/"
    
    # Create source tarball
    cd "$PROJECT_ROOT"
    tar --exclude-vcs --exclude='target' --exclude='packaging/rpm-build' \
        --transform 's,^,unet-0.1.0/,' \
        -czf "$RPM_BUILD_ROOT/SOURCES/unet-0.1.0.tar.gz" .
    
    # Copy additional files needed for build
    cp packaging/rpm/unet-server.service "$RPM_BUILD_ROOT/SOURCES/"
}

# Function to build packages
build_packages() {
    echo "Building RPM packages..."
    
    # Build the RPM packages
    rpmbuild --define "_topdir $RPM_BUILD_ROOT" -ba "$RPM_BUILD_ROOT/SPECS/unet.spec"
    
    echo "Build completed successfully!"
    echo "Packages created in: $RPM_BUILD_ROOT/RPMS/"
    find "$RPM_BUILD_ROOT/RPMS" -name "*.rpm" -exec ls -la {} \;
}

# Function to test packages
test_packages() {
    echo "Testing package installation..."
    cd "$RPM_BUILD_ROOT/RPMS"
    
    # Test package metadata
    for rpm in $(find . -name "*.rpm"); do
        echo "Testing package: $rpm"
        rpm -qip "$rpm"
        rpm -qlp "$rpm" | head -20
        echo ""
    done
    
    # Check for rpmlint issues (if available)
    if command -v rpmlint >/dev/null 2>&1; then
        echo "Running rpmlint checks..."
        find . -name "*.rpm" -exec rpmlint {} \; || true
    fi
}

# Function to create installation script
create_install_script() {
    cat > "$RPM_BUILD_ROOT/install-unet.sh" << 'EOF'
#!/bin/bash

# Installation script for μNet RPM packages

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RPMS_DIR="$SCRIPT_DIR/RPMS"

echo "Installing μNet packages..."

# Find the architecture directory
ARCH_DIR=$(find "$RPMS_DIR" -maxdepth 1 -type d -name "*86*" -o -name "aarch64" -o -name "noarch" | head -1)
if [ -z "$ARCH_DIR" ]; then
    echo "Error: Could not find RPM architecture directory"
    exit 1
fi

# Install packages
if command -v dnf >/dev/null 2>&1; then
    sudo dnf install -y "$ARCH_DIR"/unet-*.rpm
elif command -v yum >/dev/null 2>&1; then
    sudo yum install -y "$ARCH_DIR"/unet-*.rpm
else
    sudo rpm -ivh "$ARCH_DIR"/unet-*.rpm
fi

echo ""
echo "μNet installed successfully!"
echo "Next steps:"
echo "1. Review and customize /etc/unet/config.toml"
echo "2. Initialize the database: sudo -u unet unet migrate"
echo "3. Start the service: sudo systemctl start unet-server"
echo "4. Enable on boot: sudo systemctl enable unet-server"
EOF
    chmod +x "$RPM_BUILD_ROOT/install-unet.sh"
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
    
    # Create installation script
    create_install_script
    
    echo ""
    echo "RPM packages built successfully!"
    echo "Packages located in: $RPM_BUILD_ROOT/RPMS/"
    echo "Install with: cd $RPM_BUILD_ROOT && ./install-unet.sh"
    echo "Or manually: sudo rpm -ivh $RPM_BUILD_ROOT/RPMS/*/*.rpm"
}

# Run main function
main "$@"