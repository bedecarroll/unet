#!/bin/bash
set -euo pipefail

# μNet Installation Script
# Quick and easy installation of μNet from GitHub releases

# Configuration
REPO="example/unet"
GITHUB_API="https://api.github.com/repos/$REPO"
GITHUB_RELEASES="https://github.com/$REPO/releases"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
TEMP_DIR=$(mktemp -d)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Log functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Cleanup function
cleanup() {
    if [[ -d "$TEMP_DIR" ]]; then
        rm -rf "$TEMP_DIR"
    fi
}

# Set up cleanup trap
trap cleanup EXIT

# Help function
show_help() {
    cat << EOF
μNet Installation Script

Usage: $0 [OPTIONS]

Options:
  -h, --help              Show this help message
  -v, --version VERSION   Install specific version (default: latest)
  -d, --dir DIRECTORY     Installation directory (default: /usr/local/bin)
  --user                  Install to user directory (~/.local/bin)
  --system                Install system-wide (requires sudo)
  --force                 Force installation even if already installed
  --no-verify             Skip checksum verification
  --dry-run               Show what would be installed without installing

Examples:
  $0                      Install latest version to /usr/local/bin
  $0 --user               Install to ~/.local/bin (no sudo required)
  $0 -v 0.2.0             Install specific version 0.2.0
  $0 --system             Install system-wide with sudo
  $0 --dry-run            Preview installation without making changes

Environment Variables:
  INSTALL_DIR             Override default installation directory
  GITHUB_TOKEN            GitHub API token for higher rate limits

EOF
}

# Parse command line arguments
VERSION=""
USER_INSTALL=false
SYSTEM_INSTALL=false
FORCE=false
VERIFY=true
DRY_RUN=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -v|--version)
            VERSION="$2"
            shift 2
            ;;
        -d|--dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        --user)
            USER_INSTALL=true
            shift
            ;;
        --system)
            SYSTEM_INSTALL=true
            shift
            ;;
        --force)
            FORCE=true
            shift
            ;;
        --no-verify)
            VERIFY=false
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        -*)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
        *)
            log_error "Unexpected argument: $1"
            show_help
            exit 1
            ;;
    esac
done

# Detect platform
detect_platform() {
    local os arch
    
    case "$(uname -s)" in
        Linux*)
            os="linux"
            ;;
        Darwin*)
            os="macos"
            ;;
        CYGWIN*|MINGW*|MSYS*)
            os="windows"
            ;;
        *)
            log_error "Unsupported operating system: $(uname -s)"
            exit 1
            ;;
    esac
    
    case "$(uname -m)" in
        x86_64|amd64)
            arch="x86_64"
            ;;
        aarch64|arm64)
            if [[ "$os" == "macos" ]]; then
                arch="aarch64"
            else
                log_error "ARM64 not yet supported on $os"
                exit 1
            fi
            ;;
        *)
            log_error "Unsupported architecture: $(uname -m)"
            exit 1
            ;;
    esac
    
    # Determine file extension and specific platform string
    local platform_string
    local file_ext
    
    case "$os" in
        linux)
            # Detect if we should use musl
            if ldd --version 2>&1 | grep -q musl; then
                platform_string="linux-x86_64-musl"
            else
                platform_string="linux-x86_64"
            fi
            file_ext="tar.gz"
            ;;
        macos)
            platform_string="macos-$arch"
            file_ext="tar.gz"
            ;;
        windows)
            platform_string="windows-x86_64"
            file_ext="zip"
            ;;
    esac
    
    echo "$platform_string:$file_ext"
}

# Get latest version from GitHub API
get_latest_version() {
    local auth_header=""
    if [[ -n "${GITHUB_TOKEN:-}" ]]; then
        auth_header="Authorization: token $GITHUB_TOKEN"
    fi
    
    local version
    if command -v curl >/dev/null 2>&1; then
        version=$(curl -s ${auth_header:+-H "$auth_header"} "$GITHUB_API/releases/latest" | grep '"tag_name"' | cut -d'"' -f4)
    elif command -v wget >/dev/null 2>&1; then
        version=$(wget -q ${auth_header:+--header="$auth_header"} -O- "$GITHUB_API/releases/latest" | grep '"tag_name"' | cut -d'"' -f4)
    else
        log_error "Neither curl nor wget found. Please install one of them."
        exit 1
    fi
    
    if [[ -z "$version" ]]; then
        log_error "Failed to get latest version from GitHub API"
        exit 1
    fi
    
    echo "$version"
}

# Download file
download_file() {
    local url="$1"
    local output="$2"
    
    log_info "Downloading from $url"
    
    if command -v curl >/dev/null 2>&1; then
        curl -L -o "$output" "$url"
    elif command -v wget >/dev/null 2>&1; then
        wget -O "$output" "$url"
    else
        log_error "Neither curl nor wget found. Please install one of them."
        exit 1
    fi
}

# Extract archive
extract_archive() {
    local archive="$1"
    local dest_dir="$2"
    local file_ext="$3"
    
    case "$file_ext" in
        tar.gz)
            tar -xzf "$archive" -C "$dest_dir"
            ;;
        zip)
            if command -v unzip >/dev/null 2>&1; then
                unzip -q "$archive" -d "$dest_dir"
            else
                log_error "unzip not found. Please install it to extract Windows archives."
                exit 1
            fi
            ;;
        *)
            log_error "Unsupported archive format: $file_ext"
            exit 1
            ;;
    esac
}

# Verify installation directory
setup_install_dir() {
    # Handle user install
    if [[ "$USER_INSTALL" == "true" ]]; then
        INSTALL_DIR="$HOME/.local/bin"
        SYSTEM_INSTALL=false
    fi
    
    # Create install directory if it doesn't exist
    if [[ ! -d "$INSTALL_DIR" ]]; then
        if [[ "$DRY_RUN" == "true" ]]; then
            log_info "Would create directory: $INSTALL_DIR"
        else
            log_info "Creating directory: $INSTALL_DIR"
            if [[ "$SYSTEM_INSTALL" == "true" ]] || [[ "$INSTALL_DIR" == "/usr/local/bin" ]] || [[ "$INSTALL_DIR" == "/usr/bin" ]]; then
                sudo mkdir -p "$INSTALL_DIR"
            else
                mkdir -p "$INSTALL_DIR"
            fi
        fi
    fi
    
    # Check if directory is writable
    if [[ "$DRY_RUN" == "false" ]]; then
        if [[ ! -w "$INSTALL_DIR" ]]; then
            if [[ "$SYSTEM_INSTALL" == "true" ]] || [[ "$INSTALL_DIR" == "/usr/local/bin" ]] || [[ "$INSTALL_DIR" == "/usr/bin" ]]; then
                log_info "Directory $INSTALL_DIR requires sudo for installation"
            else
                log_error "Directory $INSTALL_DIR is not writable"
                exit 1
            fi
        fi
    fi
}

# Check if already installed
check_existing_installation() {
    local binaries=("unet-server" "unet-cli")
    local installed_binaries=()
    
    for binary in "${binaries[@]}"; do
        if command -v "$binary" >/dev/null 2>&1; then
            installed_binaries+=("$binary")
        fi
    done
    
    if [[ ${#installed_binaries[@]} -gt 0 ]]; then
        log_info "Found existing μNet installation:"
        for binary in "${installed_binaries[@]}"; do
            local existing_path
            existing_path=$(command -v "$binary")
            local existing_version
            existing_version=$("$binary" --version 2>/dev/null | head -n1 || echo "unknown")
            log_info "  $binary: $existing_path ($existing_version)"
        done
        
        if [[ "$FORCE" == "false" ]]; then
            echo
            log_warning "μNet is already installed. Use --force to reinstall."
            read -p "Continue with installation? (y/N): " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                log_info "Installation cancelled"
                exit 0
            fi
        fi
    fi
}

# Install binaries
install_binaries() {
    local extract_dir="$1"
    local need_sudo=false
    
    # Determine if we need sudo
    if [[ "$SYSTEM_INSTALL" == "true" ]] || [[ "$INSTALL_DIR" == "/usr/local/bin" ]] || [[ "$INSTALL_DIR" == "/usr/bin" ]]; then
        need_sudo=true
    fi
    
    # Find binaries in extract directory
    local server_binary=""
    local cli_binary=""
    
    # Look for binaries
    if [[ -f "$extract_dir/unet-server" ]]; then
        server_binary="$extract_dir/unet-server"
    elif [[ -f "$extract_dir/release/unet-server" ]]; then
        server_binary="$extract_dir/release/unet-server"
    fi
    
    if [[ -f "$extract_dir/unet-cli" ]]; then
        cli_binary="$extract_dir/unet-cli"
    elif [[ -f "$extract_dir/release/unet-cli" ]]; then
        cli_binary="$extract_dir/release/unet-cli"
    fi
    
    # Handle Windows executables
    if [[ -f "$extract_dir/unet-server.exe" ]]; then
        server_binary="$extract_dir/unet-server.exe"
    elif [[ -f "$extract_dir/release/unet-server.exe" ]]; then
        server_binary="$extract_dir/release/unet-server.exe"
    fi
    
    if [[ -f "$extract_dir/unet-cli.exe" ]]; then
        cli_binary="$extract_dir/unet-cli.exe"
    elif [[ -f "$extract_dir/release/unet-cli.exe" ]]; then
        cli_binary="$extract_dir/release/unet-cli.exe"
    fi
    
    # Install binaries
    local installed_binaries=()
    
    if [[ -n "$server_binary" ]]; then
        local dest="$INSTALL_DIR/$(basename "$server_binary")"
        if [[ "$DRY_RUN" == "true" ]]; then
            log_info "Would install unet-server to: $dest"
        else
            log_info "Installing unet-server to: $dest"
            if [[ "$need_sudo" == "true" ]]; then
                sudo cp "$server_binary" "$dest"
                sudo chmod +x "$dest"
            else
                cp "$server_binary" "$dest"
                chmod +x "$dest"
            fi
        fi
        installed_binaries+=("unet-server")
    fi
    
    if [[ -n "$cli_binary" ]]; then
        local dest="$INSTALL_DIR/$(basename "$cli_binary")"
        if [[ "$DRY_RUN" == "true" ]]; then
            log_info "Would install unet-cli to: $dest"
        else
            log_info "Installing unet-cli to: $dest"
            if [[ "$need_sudo" == "true" ]]; then
                sudo cp "$cli_binary" "$dest"
                sudo chmod +x "$dest"
            else
                cp "$cli_binary" "$dest"
                chmod +x "$dest"
            fi
        fi
        installed_binaries+=("unet-cli")
    fi
    
    if [[ ${#installed_binaries[@]} -eq 0 ]]; then
        log_error "No μNet binaries found in downloaded archive"
        exit 1
    fi
    
    echo "${installed_binaries[@]}"
}

# Verify installation
verify_installation() {
    local binaries=("$@")
    
    log_info "Verifying installation..."
    
    for binary in "${binaries[@]}"; do
        if command -v "$binary" >/dev/null 2>&1; then
            local version
            version=$("$binary" --version 2>/dev/null | head -n1 || echo "unknown")
            log_success "$binary installed successfully ($version)"
        else
            log_error "$binary not found in PATH"
            return 1
        fi
    done
}

# Add to PATH instructions
show_path_instructions() {
    if [[ "$USER_INSTALL" == "true" ]]; then
        if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
            echo
            log_info "Add ~/.local/bin to your PATH by adding this line to your shell profile:"
            echo "export PATH=\"\$HOME/.local/bin:\$PATH\""
            echo
            log_info "Then reload your shell or run: source ~/.bashrc"
        fi
    fi
}

# Main installation function
main() {
    log_info "μNet Installation Script"
    log_info "========================="
    
    # Detect platform
    local platform_info
    platform_info=$(detect_platform)
    local platform="${platform_info%:*}"
    local file_ext="${platform_info#*:}"
    
    log_info "Detected platform: $platform"
    
    # Get version
    if [[ -z "$VERSION" ]]; then
        log_info "Getting latest version..."
        VERSION=$(get_latest_version)
    fi
    
    # Ensure version has 'v' prefix for URLs
    if [[ "$VERSION" != v* ]]; then
        VERSION="v$VERSION"
    fi
    
    log_info "Installing μNet $VERSION"
    
    # Setup installation directory
    setup_install_dir
    log_info "Installation directory: $INSTALL_DIR"
    
    # Check existing installation
    if [[ "$FORCE" == "false" ]]; then
        check_existing_installation
    fi
    
    # Prepare download
    local filename="unet-$platform.$file_ext"
    local download_url="$GITHUB_RELEASES/download/$VERSION/$filename"
    local archive_path="$TEMP_DIR/$filename"
    
    log_info "Download URL: $download_url"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "Dry run - would download and install μNet $VERSION"
        log_info "  Platform: $platform"
        log_info "  Archive: $filename"
        log_info "  Install directory: $INSTALL_DIR"
        exit 0
    fi
    
    # Download archive
    download_file "$download_url" "$archive_path"
    
    # Extract archive
    log_info "Extracting archive..."
    extract_archive "$archive_path" "$TEMP_DIR" "$file_ext"
    
    # Install binaries
    log_info "Installing binaries..."
    local installed_binaries
    installed_binaries=($(install_binaries "$TEMP_DIR"))
    
    # Verify installation
    verify_installation "${installed_binaries[@]}"
    
    # Show PATH instructions if needed
    show_path_instructions
    
    # Success message
    echo
    log_success "μNet $VERSION installed successfully!"
    log_info "Installed binaries: ${installed_binaries[*]}"
    log_info "Installation directory: $INSTALL_DIR"
    echo
    log_info "Get started with:"
    log_info "  unet-cli --help     # View CLI help"
    log_info "  unet-server --help  # View server help"
    echo
    log_info "Documentation: https://example.github.io/unet/"
}

# Run main function
main "$@"