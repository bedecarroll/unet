#!/bin/bash
set -euo pipefail

# μNet Release Automation Script
# Complete release preparation and execution workflow

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

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

# Help function
show_help() {
    cat << EOF
μNet Release Automation Script

Usage: $0 [OPTIONS] <COMMAND> [VERSION]

Commands:
  prepare VERSION        Prepare release (version bump, changelog, testing)
  publish VERSION        Publish release (create tag, trigger CI/CD)
  announce VERSION       Generate release announcements
  full VERSION           Complete release workflow (prepare + publish)

Arguments:
  VERSION               Version to release (e.g., 0.2.0, 1.0.0-beta.1)

Options:
  -h, --help            Show this help message
  -d, --dry-run         Preview changes without making them
  --skip-tests          Skip release testing (not recommended)
  --force               Force release even if tests fail
  --prerelease          Mark as pre-release

Examples:
  $0 prepare 0.2.0      Prepare version 0.2.0 for release
  $0 publish 0.2.0      Publish prepared release 0.2.0
  $0 full 0.2.0         Complete release workflow for 0.2.0
  $0 announce 0.2.0     Generate announcement templates
  $0 --dry-run full 0.2.0   Preview complete release workflow

Release Workflow:
  1. prepare: Version bump, changelog update, release testing
  2. publish: Git tag creation, GitHub release trigger
  3. announce: Generate announcement templates and materials

EOF
}

# Parse command line arguments
DRY_RUN=false
SKIP_TESTS=false
FORCE=false
PRERELEASE=false
COMMAND=""
VERSION=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -d|--dry-run)
            DRY_RUN=true
            shift
            ;;
        --skip-tests)
            SKIP_TESTS=true
            shift
            ;;
        --force)
            FORCE=true
            shift
            ;;
        --prerelease)
            PRERELEASE=true
            shift
            ;;
        prepare|publish|announce|full)
            if [[ -z "$COMMAND" ]]; then
                COMMAND="$1"
            else
                log_error "Multiple commands specified: $COMMAND and $1"
                exit 1
            fi
            shift
            ;;
        -*)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
        *)
            if [[ -z "$VERSION" ]]; then
                VERSION="$1"
            else
                log_error "Multiple versions specified: $VERSION and $1"
                exit 1
            fi
            shift
            ;;
    esac
done

# Validate inputs
if [[ -z "$COMMAND" ]]; then
    log_error "Command required. Use --help for usage information."
    exit 1
fi

if [[ -z "$VERSION" && "$COMMAND" != "announce" ]]; then
    log_error "Version required for command: $COMMAND"
    exit 1
fi

# Check dependencies
check_dependencies() {
    local deps=("cargo" "git")
    local missing_deps=()
    
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &> /dev/null; then
            missing_deps+=("$dep")
        fi
    done
    
    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        log_error "Missing required dependencies: ${missing_deps[*]}"
        exit 1
    fi
}

# Validate version format
validate_version() {
    local version="$1"
    if [[ ! $version =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.-]+)?(\+[a-zA-Z0-9.-]+)?$ ]]; then
        log_error "Invalid version format: $version"
        log_error "Version must follow semantic versioning (e.g., 1.0.0, 1.0.0-beta.1)"
        exit 1
    fi
}

# Check git status
check_git_status() {
    if [[ "$DRY_RUN" == "true" ]]; then
        return 0
    fi
    
    # Check if we're in a git repository
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        log_error "Not in a git repository"
        exit 1
    fi
    
    # Check for uncommitted changes
    if ! git diff-index --quiet HEAD --; then
        log_error "Uncommitted changes detected. Please commit or stash changes before releasing."
        git status --short
        exit 1
    fi
    
    # Check if we're on main/master branch
    local current_branch
    current_branch=$(git branch --show-current)
    if [[ "$current_branch" != "main" && "$current_branch" != "master" ]]; then
        log_warning "Current branch is '$current_branch', not 'main' or 'master'"
        read -p "Continue anyway? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "Release cancelled"
            exit 0
        fi
    fi
}

# Prepare release
prepare_release() {
    local version="$1"
    
    log_info "Preparing release $version..."
    
    # Validate version
    validate_version "$version"
    
    # Check git status
    check_git_status
    
    # Version bump
    log_info "Updating version to $version..."
    local version_args=()
    if [[ "$DRY_RUN" == "true" ]]; then
        version_args+=("--dry-run")
    fi
    
    if ! "$SCRIPT_DIR/version-bump.sh" "${version_args[@]}" "$version"; then
        log_error "Version bump failed"
        exit 1
    fi
    
    # Run release tests
    if [[ "$SKIP_TESTS" == "false" ]]; then
        log_info "Running release tests..."
        
        local test_args=()
        if [[ "$DRY_RUN" == "true" ]]; then
            log_info "Skipping tests in dry-run mode"
        else
            if ! "$SCRIPT_DIR/test-release.sh" "${test_args[@]}"; then
                if [[ "$FORCE" == "true" ]]; then
                    log_warning "Tests failed but continuing due to --force flag"
                else
                    log_error "Release tests failed. Use --force to override or fix issues first."
                    exit 1
                fi
            fi
        fi
    else
        log_warning "Skipping release tests (--skip-tests specified)"
    fi
    
    log_success "Release $version prepared successfully!"
    
    if [[ "$DRY_RUN" == "false" ]]; then
        log_info "Next steps:"
        log_info "  1. Review changes: git diff"
        log_info "  2. Commit changes: git add . && git commit -m 'chore: prepare release $version'"
        log_info "  3. Publish release: $0 publish $version"
    fi
}

# Publish release
publish_release() {
    local version="$1"
    
    log_info "Publishing release $version..."
    
    # Validate version
    validate_version "$version"
    
    # Check git status
    check_git_status
    
    local tag_name="v$version"
    
    # Check if tag already exists
    if git tag -l | grep -q "^$tag_name$"; then
        log_error "Git tag $tag_name already exists"
        exit 1
    fi
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "Would create git tag: $tag_name"
        log_info "Would push tag to trigger release workflow"
        return 0
    fi
    
    # Create and push tag
    log_info "Creating git tag: $tag_name"
    local tag_message="Release $version"
    if [[ "$PRERELEASE" == "true" ]]; then
        tag_message="Pre-release $version"
    fi
    
    git tag -a "$tag_name" -m "$tag_message"
    
    log_info "Pushing tag to trigger release workflow..."
    git push origin "$tag_name"
    
    log_success "Release $version published successfully!"
    log_info "Monitor the release workflow at: https://github.com/example/unet/actions"
    log_info "Release will be available at: https://github.com/example/unet/releases/tag/$tag_name"
}

# Generate announcements
generate_announcements() {
    local version="$1"
    local announcements_dir="$PROJECT_ROOT/release-announcements-$version"
    
    log_info "Generating release announcements for version $version..."
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "Would create announcements in: $announcements_dir"
        return 0
    fi
    
    # Create announcements directory
    mkdir -p "$announcements_dir"
    
    # Extract changelog for this version
    local changelog_section=""
    if [[ -f "$PROJECT_ROOT/CHANGELOG.md" ]]; then
        changelog_section=$(awk "/^## \[$version\]/, /^## \[/ { if (/^## \[/ && !/^## \[$version\]/) exit; if (!/^## \[$version\]/) print }" "$PROJECT_ROOT/CHANGELOG.md" || echo "")
    fi
    
    # Copy announcement templates
    if [[ -d "$PROJECT_ROOT/templates" ]]; then
        cp "$PROJECT_ROOT/templates/release-announcement-"*.md "$announcements_dir/" 2>/dev/null || true
    fi
    
    # Create a release info file
    cat > "$announcements_dir/release-info.json" << EOF
{
  "version": "$version",
  "tag_name": "v$version",
  "release_date": "$(date +"%Y-%m-%d")",
  "is_prerelease": $PRERELEASE,
  "changelog": $(echo "$changelog_section" | jq -R -s .),
  "download_urls": {
    "linux_x86_64": "https://github.com/example/unet/releases/download/v$version/unet-linux-x86_64.tar.gz",
    "linux_musl": "https://github.com/example/unet/releases/download/v$version/unet-linux-x86_64-musl.tar.gz",
    "macos_x86_64": "https://github.com/example/unet/releases/download/v$version/unet-macos-x86_64.tar.gz",
    "macos_aarch64": "https://github.com/example/unet/releases/download/v$version/unet-macos-aarch64.tar.gz",
    "windows": "https://github.com/example/unet/releases/download/v$version/unet-windows-x86_64.zip"
  }
}
EOF
    
    # Create a README for the announcements
    cat > "$announcements_dir/README.md" << EOF
# μNet $version Release Announcements

This directory contains announcement templates and materials for μNet $version release.

## Files

- \`release-info.json\` - Release metadata and information
- \`release-announcement-github.md\` - GitHub release announcement template
- \`release-announcement-email.md\` - Email announcement template  
- \`release-announcement-social.md\` - Social media announcement templates

## Usage

1. Review and customize the templates with specific release information
2. Update placeholders with actual feature descriptions and changes
3. Publish announcements through appropriate channels

## Customization

The templates use placeholder syntax like \`{{version}}\` and \`{{features}}\`. 
Replace these with actual content before publishing.

## Channels

- **GitHub**: Use \`release-announcement-github.md\` as release description
- **Email**: Customize \`release-announcement-email.md\` for mailing lists
- **Social Media**: Use relevant sections from \`release-announcement-social.md\`

## Release Information

- **Version**: $version
- **Tag**: v$version
- **Date**: $(date +"%Y-%m-%d")
- **Type**: $(if [[ "$PRERELEASE" == "true" ]]; then echo "Pre-release"; else echo "Stable release"; fi)
EOF
    
    log_success "Release announcements generated in: $announcements_dir"
    log_info "Next steps:"
    log_info "  1. Review and customize announcement templates"
    log_info "  2. Update placeholders with specific release information"  
    log_info "  3. Publish announcements through appropriate channels"
}

# Full release workflow
full_release() {
    local version="$1"
    
    log_info "Starting full release workflow for version $version..."
    
    # Prepare release
    prepare_release "$version"
    
    if [[ "$DRY_RUN" == "false" ]]; then
        # Confirm before publishing
        echo
        log_warning "Ready to publish release $version"
        log_info "This will create a git tag and trigger the release workflow"
        read -p "Continue with publication? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "Stopping before publication. Release prepared but not published."
            log_info "To publish later, run: $0 publish $version"
            exit 0
        fi
    fi
    
    # Publish release
    publish_release "$version"
    
    # Generate announcements
    generate_announcements "$version"
    
    log_success "Full release workflow completed for version $version!"
}

# Main execution
main() {
    log_info "μNet Release Automation Script"
    log_info "================================"
    
    # Change to project root
    cd "$PROJECT_ROOT"
    
    # Check dependencies
    check_dependencies
    
    # Show configuration
    log_info "Configuration:"
    log_info "  Command: $COMMAND"
    log_info "  Version: ${VERSION:-"N/A"}"
    log_info "  Dry run: $DRY_RUN"
    log_info "  Skip tests: $SKIP_TESTS"
    log_info "  Force: $FORCE"
    log_info "  Pre-release: $PRERELEASE"
    echo
    
    # Execute command
    case "$COMMAND" in
        prepare)
            prepare_release "$VERSION"
            ;;
        publish)
            publish_release "$VERSION"
            ;;
        announce)
            if [[ -z "$VERSION" ]]; then
                # Get version from Cargo.toml if not specified
                VERSION=$(grep '^version = ' "$PROJECT_ROOT/Cargo.toml" | sed 's/version = "\(.*\)"/\1/')
                log_info "Using current version: $VERSION"
            fi
            generate_announcements "$VERSION"
            ;;
        full)
            full_release "$VERSION"
            ;;
        *)
            log_error "Unknown command: $COMMAND"
            show_help
            exit 1
            ;;
    esac
}

# Run main function
main "$@"