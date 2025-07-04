#!/bin/bash
set -euo pipefail

# μNet Release Testing Script
# Comprehensive testing procedures for release validation

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

# Test results tracking
TESTS_PASSED=0
TESTS_FAILED=0
FAILED_TESTS=()

# Record test result
record_test() {
    local test_name="$1"
    local result="$2"
    
    if [[ "$result" == "PASS" ]]; then
        ((TESTS_PASSED++))
        log_success "✓ $test_name"
    else
        ((TESTS_FAILED++))
        FAILED_TESTS+=("$test_name")
        log_error "✗ $test_name"
    fi
}

# Run a test command and record result
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    log_info "Running: $test_name"
    
    if eval "$test_command" >/dev/null 2>&1; then
        record_test "$test_name" "PASS"
        return 0
    else
        record_test "$test_name" "FAIL"
        return 1
    fi
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    local deps=("cargo" "git" "docker" "curl")
    local missing_deps=()
    
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &> /dev/null; then
            missing_deps+=("$dep")
        fi
    done
    
    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        log_error "Missing required dependencies: ${missing_deps[*]}"
        log_error "Please install missing dependencies and try again"
        exit 1
    fi
    
    log_success "All prerequisites satisfied"
}

# Test workspace integrity
test_workspace() {
    log_info "Testing workspace integrity..."
    
    # Test workspace structure
    run_test "Workspace structure" "test -f '$PROJECT_ROOT/Cargo.toml'"
    run_test "Core crate exists" "test -d '$PROJECT_ROOT/crates/unet-core'"
    run_test "Server crate exists" "test -d '$PROJECT_ROOT/crates/unet-server'"
    run_test "CLI crate exists" "test -d '$PROJECT_ROOT/crates/unet-cli'"
    run_test "Config slicer exists" "test -d '$PROJECT_ROOT/crates/config-slicer'"
    run_test "Migrations exist" "test -d '$PROJECT_ROOT/migrations'"
    
    # Test workspace compilation
    run_test "Workspace check" "cargo check --workspace --quiet"
    run_test "Workspace build" "cargo build --workspace --quiet"
}

# Test code quality
test_code_quality() {
    log_info "Testing code quality..."
    
    # Formatting and linting
    run_test "Code formatting" "cargo fmt --all -- --check"
    run_test "Clippy lints" "cargo clippy --workspace --all-targets --quiet -- -D warnings"
    
    # Security audit
    if command -v cargo-audit &> /dev/null; then
        run_test "Security audit" "cargo audit"
    else
        log_warning "cargo-audit not installed, skipping security audit"
    fi
}

# Test documentation
test_documentation() {
    log_info "Testing documentation..."
    
    # Rust documentation
    run_test "Rust docs build" "cargo doc --workspace --no-deps --document-private-items --quiet"
    
    # mdBook documentation
    if command -v mdbook &> /dev/null; then
        run_test "mdBook build" "mdbook build docs"
        run_test "mdBook test" "mdbook test docs"
    else
        log_warning "mdbook not installed, skipping book tests"
    fi
    
    # Check required documentation files
    run_test "README exists" "test -f '$PROJECT_ROOT/README.md'"
    run_test "CHANGELOG exists" "test -f '$PROJECT_ROOT/CHANGELOG.md'"
    run_test "License file exists" "test -f '$PROJECT_ROOT/LICENSE-MIT'"
}

# Test binaries
test_binaries() {
    log_info "Testing binary compilation..."
    
    # Build release binaries
    run_test "Server binary build" "cargo build --release --bin unet-server --quiet"
    run_test "CLI binary build" "cargo build --release --bin unet-cli --quiet"
    
    # Test binary execution
    if [[ -f "$PROJECT_ROOT/target/release/unet-cli" ]]; then
        run_test "CLI binary execution" "$PROJECT_ROOT/target/release/unet-cli --version"
    fi
    
    if [[ -f "$PROJECT_ROOT/target/release/unet-server" ]]; then
        run_test "Server binary execution" "timeout 5s $PROJECT_ROOT/target/release/unet-server --help"
    fi
}

# Test unit and integration tests
test_suite() {
    log_info "Running test suite..."
    
    # Unit tests
    run_test "Unit tests" "cargo test --workspace --lib --quiet"
    
    # Integration tests
    run_test "Integration tests" "cargo test --workspace --test '*' --quiet"
    
    # Bench tests (if any)
    if find "$PROJECT_ROOT" -name "*.rs" -path "*/benches/*" | grep -q .; then
        run_test "Benchmark compilation" "cargo bench --workspace --no-run --quiet"
    fi
}

# Test Docker build
test_docker() {
    log_info "Testing Docker build..."
    
    if [[ -f "$PROJECT_ROOT/Dockerfile" ]]; then
        run_test "Docker build" "docker build -t unet-test:latest '$PROJECT_ROOT'"
        
        # Test Docker image
        if docker images unet-test:latest | grep -q unet-test; then
            run_test "Docker image created" "docker images unet-test:latest | grep -q unet-test"
            
            # Clean up test image
            docker rmi unet-test:latest >/dev/null 2>&1 || true
        fi
    else
        log_warning "No Dockerfile found, skipping Docker tests"
    fi
}

# Test packaging
test_packaging() {
    log_info "Testing packaging scripts..."
    
    local packaging_scripts=(
        "scripts/docker-build.sh"
        "packaging/build-deb.sh"
        "packaging/build-rpm.sh"
    )
    
    for script in "${packaging_scripts[@]}"; do
        if [[ -f "$PROJECT_ROOT/$script" && -x "$PROJECT_ROOT/$script" ]]; then
            run_test "Script executable: $(basename "$script")" "test -x '$PROJECT_ROOT/$script'"
        fi
    done
}

# Test configuration files
test_configuration() {
    log_info "Testing configuration files..."
    
    # Test TOML files are valid
    local toml_files=()
    mapfile -t toml_files < <(find "$PROJECT_ROOT" -name "*.toml" -not -path "*/target/*")
    
    for toml_file in "${toml_files[@]}"; do
        if command -v toml &> /dev/null; then
            run_test "TOML syntax: $(basename "$toml_file")" "toml check '$toml_file'"
        fi
    done
    
    # Test example configurations
    local config_dirs=(
        "configs/environments/development"
        "configs/environments/staging"
        "configs/environments/production"
    )
    
    for config_dir in "${config_dirs[@]}"; do
        run_test "Config directory: $config_dir" "test -d '$PROJECT_ROOT/$config_dir'"
    done
}

# Test version consistency
test_version_consistency() {
    log_info "Testing version consistency..."
    
    local workspace_version
    workspace_version=$(grep '^version = ' "$PROJECT_ROOT/Cargo.toml" | sed 's/version = "\(.*\)"/\1/')
    
    local crates=("crates/unet-core" "crates/unet-server" "crates/unet-cli" "crates/config-slicer" "migrations")
    local version_consistent=true
    
    for crate in "${crates[@]}"; do
        if [[ -f "$PROJECT_ROOT/$crate/Cargo.toml" ]]; then
            local crate_version
            crate_version=$(grep '^version = ' "$PROJECT_ROOT/$crate/Cargo.toml" | sed 's/version = "\(.*\)"/\1/' || echo "")
            
            if [[ "$crate_version" != "$workspace_version" && -n "$crate_version" ]]; then
                version_consistent=false
                log_error "Version mismatch in $crate: $crate_version (expected: $workspace_version)"
            fi
        fi
    done
    
    if [[ "$version_consistent" == "true" ]]; then
        record_test "Version consistency" "PASS"
    else
        record_test "Version consistency" "FAIL"
    fi
}

# Generate test report
generate_report() {
    local total_tests=$((TESTS_PASSED + TESTS_FAILED))
    
    echo
    log_info "════════════════════════════════════════"
    log_info "μNet Release Testing Report"
    log_info "════════════════════════════════════════"
    echo
    log_info "Total tests: $total_tests"
    log_success "Passed: $TESTS_PASSED"
    
    if [[ $TESTS_FAILED -gt 0 ]]; then
        log_error "Failed: $TESTS_FAILED"
        echo
        log_error "Failed tests:"
        for test in "${FAILED_TESTS[@]}"; do
            log_error "  - $test"
        done
        echo
        log_error "Release testing FAILED. Please fix the issues above before releasing."
        return 1
    else
        echo
        log_success "All tests passed! Release is ready for deployment."
        return 0
    fi
}

# Main test execution
main() {
    log_info "μNet Release Testing Script"
    log_info "============================"
    echo
    
    # Change to project root
    cd "$PROJECT_ROOT"
    
    # Check prerequisites
    check_prerequisites
    echo
    
    # Run all tests
    test_workspace
    test_code_quality
    test_documentation
    test_binaries
    test_suite
    test_docker
    test_packaging
    test_configuration
    test_version_consistency
    
    # Generate final report
    generate_report
}

# Show help
show_help() {
    cat << EOF
μNet Release Testing Script

Usage: $0 [OPTIONS]

Options:
  -h, --help             Show this help message
  --skip-docker          Skip Docker-related tests
  --skip-packaging       Skip packaging tests
  --quiet                Reduce output verbosity

Description:
  Runs comprehensive tests to validate that the μNet codebase is ready for release.
  This includes:
  
  - Workspace integrity and compilation
  - Code quality (formatting, linting, security)
  - Documentation building and testing
  - Binary compilation and execution
  - Unit and integration test suite
  - Docker build validation
  - Packaging script validation
  - Configuration file validation
  - Version consistency checks

Example:
  $0                     # Run all tests
  $0 --skip-docker       # Skip Docker tests
  $0 --quiet             # Reduce output verbosity

EOF
}

# Parse command line arguments
SKIP_DOCKER=false
SKIP_PACKAGING=false
QUIET=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        --skip-docker)
            SKIP_DOCKER=true
            shift
            ;;
        --skip-packaging)
            SKIP_PACKAGING=true
            shift
            ;;
        --quiet)
            QUIET=true
            shift
            ;;
        *)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Override test functions if skipping
if [[ "$SKIP_DOCKER" == "true" ]]; then
    test_docker() {
        log_info "Skipping Docker tests (--skip-docker specified)"
    }
fi

if [[ "$SKIP_PACKAGING" == "true" ]]; then
    test_packaging() {
        log_info "Skipping packaging tests (--skip-packaging specified)"
    }
fi

# Run main function
main "$@"