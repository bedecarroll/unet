#!/bin/bash
set -euo pipefail

# μNet Version Bump Script
# Automatically updates version across all workspace crates and prepares for release

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
μNet Version Bump Script

Usage: $0 [OPTIONS] <VERSION>

Arguments:
  VERSION                 New version to set (e.g., 0.2.0, 1.0.0-beta.1)

Options:
  -h, --help             Show this help message
  -d, --dry-run          Show what would be changed without making changes
  -c, --check            Check current versions across workspace
  --no-git               Skip git operations (tag creation)
  --changelog-only       Only update changelog, don't bump versions

Examples:
  $0 0.2.0               Bump to version 0.2.0
  $0 1.0.0-rc.1          Bump to release candidate 1.0.0-rc.1
  $0 --dry-run 0.2.0     Preview changes for version 0.2.0
  $0 --check             Check current versions
  $0 --changelog-only    Update changelog only

EOF
}

# Parse command line arguments
DRY_RUN=false
CHECK_ONLY=false
NO_GIT=false
CHANGELOG_ONLY=false
NEW_VERSION=""

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
        -c|--check)
            CHECK_ONLY=true
            shift
            ;;
        --no-git)
            NO_GIT=true
            shift
            ;;
        --changelog-only)
            CHANGELOG_ONLY=true
            shift
            ;;
        -*)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
        *)
            if [[ -z "$NEW_VERSION" ]]; then
                NEW_VERSION="$1"
            else
                log_error "Multiple versions specified: $NEW_VERSION and $1"
                exit 1
            fi
            shift
            ;;
    esac
done

# Validate dependencies
check_dependencies() {
    local deps=("cargo" "git" "grep" "sed")
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &> /dev/null; then
            log_error "Required dependency '$dep' not found"
            exit 1
        fi
    done
}

# Get current version from workspace Cargo.toml
get_current_version() {
    grep '^version = ' "$PROJECT_ROOT/Cargo.toml" | sed 's/version = "\(.*\)"/\1/'
}

# Validate version format
validate_version() {
    local version="$1"
    # Basic semver validation
    if [[ ! $version =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.-]+)?(\+[a-zA-Z0-9.-]+)?$ ]]; then
        log_error "Invalid version format: $version"
        log_error "Version must follow semantic versioning (e.g., 1.0.0, 1.0.0-beta.1)"
        exit 1
    fi
}

# Check current versions across workspace
check_versions() {
    log_info "Checking current versions across workspace..."
    
    local workspace_version
    workspace_version=$(get_current_version)
    log_info "Workspace version: $workspace_version"
    
    # Check individual crate versions
    local crates=("crates/unet-core" "crates/unet-server" "crates/unet-cli" "crates/config-slicer" "migrations")
    for crate in "${crates[@]}"; do
        if [[ -f "$PROJECT_ROOT/$crate/Cargo.toml" ]]; then
            local crate_version
            crate_version=$(grep '^version = ' "$PROJECT_ROOT/$crate/Cargo.toml" | sed 's/version = "\(.*\)"/\1/' || echo "No version found")
            log_info "  $crate: $crate_version"
        fi
    done
    
    # Check if git tag exists for current version
    if git tag -l | grep -q "^v$workspace_version$"; then
        log_warning "Git tag v$workspace_version already exists"
    else
        log_info "Git tag v$workspace_version does not exist"
    fi
}

# Update version in Cargo.toml files
update_cargo_versions() {
    local new_version="$1"
    local files=(
        "$PROJECT_ROOT/Cargo.toml"
        "$PROJECT_ROOT/crates/unet-core/Cargo.toml"
        "$PROJECT_ROOT/crates/unet-server/Cargo.toml"
        "$PROJECT_ROOT/crates/unet-cli/Cargo.toml"
        "$PROJECT_ROOT/crates/config-slicer/Cargo.toml"
        "$PROJECT_ROOT/migrations/Cargo.toml"
    )
    
    for file in "${files[@]}"; do
        if [[ -f "$file" ]]; then
            log_info "Updating version in $(basename "$(dirname "$file")")/$(basename "$file")"
            if [[ "$DRY_RUN" == "true" ]]; then
                log_info "  Would change version to: $new_version"
            else
                # Update version line
                sed -i.bak "s/^version = \".*\"/version = \"$new_version\"/" "$file"
                rm -f "$file.bak"
                log_success "  Updated to version $new_version"
            fi
        else
            log_warning "File not found: $file"
        fi
    done
}

# Update changelog
update_changelog() {
    local new_version="$1"
    local changelog_file="$PROJECT_ROOT/CHANGELOG.md"
    local date=$(date +"%Y-%m-%d")
    
    if [[ ! -f "$changelog_file" ]]; then
        log_error "CHANGELOG.md not found at $changelog_file"
        exit 1
    fi
    
    log_info "Updating CHANGELOG.md for version $new_version"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "  Would add release entry for version $new_version on $date"
        return
    fi
    
    # Create temporary file with updated changelog
    local temp_file=$(mktemp)
    
    # Read the changelog and update the unreleased section
    awk -v version="$new_version" -v date="$date" '
    /^## \[Unreleased\]/ {
        print $0
        print ""
        print "### Added"
        print "- (Add new features here)"
        print ""
        print "### Changed"
        print "- (Add changes here)"
        print ""
        print "### Fixed"
        print "- (Add bug fixes here)"
        print ""
        print "## [" version "] - " date
        next
    }
    /^\[Unreleased\]:/ {
        print "[Unreleased]: https://github.com/example/unet/compare/v" version "...HEAD"
        print "[" version "]: https://github.com/example/unet/releases/tag/v" version
        next
    }
    { print }
    ' "$changelog_file" > "$temp_file"
    
    mv "$temp_file" "$changelog_file"
    log_success "  Updated CHANGELOG.md"
}

# Create git tag
create_git_tag() {
    local new_version="$1"
    local tag_name="v$new_version"
    
    if [[ "$NO_GIT" == "true" ]]; then
        log_info "Skipping git tag creation (--no-git specified)"
        return
    fi
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "Would create git tag: $tag_name"
        return
    fi
    
    # Check if tag already exists
    if git tag -l | grep -q "^$tag_name$"; then
        log_error "Git tag $tag_name already exists"
        exit 1
    fi
    
    log_info "Creating git tag: $tag_name"
    git tag -a "$tag_name" -m "Release $new_version"
    log_success "Created git tag: $tag_name"
    
    log_info "To push the tag, run: git push origin $tag_name"
}

# Verify workspace after version bump
verify_workspace() {
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "Skipping workspace verification in dry-run mode"
        return
    fi
    
    log_info "Verifying workspace integrity..."
    
    # Check that workspace builds
    if ! cargo check --workspace --quiet; then
        log_error "Workspace failed to build after version update"
        exit 1
    fi
    
    log_success "Workspace verification passed"
}

# Main execution
main() {
    log_info "μNet Version Bump Script"
    log_info "========================="
    
    # Change to project root
    cd "$PROJECT_ROOT"
    
    # Check dependencies
    check_dependencies
    
    # Handle check-only mode
    if [[ "$CHECK_ONLY" == "true" ]]; then
        check_versions
        exit 0
    fi
    
    # Validate inputs
    if [[ -z "$NEW_VERSION" && "$CHANGELOG_ONLY" == "false" ]]; then
        log_error "Version argument required (use --help for usage)"
        exit 1
    fi
    
    if [[ -n "$NEW_VERSION" ]]; then
        validate_version "$NEW_VERSION"
    fi
    
    # Show current state
    log_info "Current state:"
    check_versions
    echo
    
    # Handle changelog-only mode
    if [[ "$CHANGELOG_ONLY" == "true" ]]; then
        if [[ -z "$NEW_VERSION" ]]; then
            NEW_VERSION=$(get_current_version)
        fi
        update_changelog "$NEW_VERSION"
        log_success "Changelog updated successfully"
        exit 0
    fi
    
    # Confirm changes in non-dry-run mode
    if [[ "$DRY_RUN" == "false" ]]; then
        echo
        log_warning "This will update versions to $NEW_VERSION across the entire workspace"
        read -p "Continue? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "Operation cancelled"
            exit 0
        fi
    fi
    
    # Perform version bump
    log_info "Starting version bump to $NEW_VERSION..."
    
    # Update Cargo.toml files
    update_cargo_versions "$NEW_VERSION"
    
    # Update changelog
    update_changelog "$NEW_VERSION"
    
    # Verify workspace
    verify_workspace
    
    # Create git tag
    create_git_tag "$NEW_VERSION"
    
    # Success message
    echo
    if [[ "$DRY_RUN" == "true" ]]; then
        log_success "Dry run completed successfully"
        log_info "No changes were made. Run without --dry-run to apply changes."
    else
        log_success "Version bump to $NEW_VERSION completed successfully!"
        log_info "Next steps:"
        log_info "  1. Review the changes: git diff"
        log_info "  2. Commit the changes: git add . && git commit -m 'chore: bump version to $NEW_VERSION'"
        log_info "  3. Push the tag: git push origin v$NEW_VERSION"
        log_info "  4. The release workflow will trigger automatically"
    fi
}

# Run main function
main "$@"