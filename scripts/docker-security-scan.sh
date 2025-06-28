#!/bin/bash
# Docker security scanning script for μNet images

set -euo pipefail

# Configuration
IMAGE_NAME="${IMAGE_NAME:-unet}"
IMAGE_TAG="${IMAGE_TAG:-latest}"
SCAN_OUTPUT_DIR="${SCAN_OUTPUT_DIR:-./security-reports}"
SEVERITY_THRESHOLD="${SEVERITY_THRESHOLD:-HIGH}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
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
μNet Docker Security Scanner

Usage: $0 [OPTIONS] [IMAGE]

OPTIONS:
    -i, --image IMAGE       Docker image name (default: unet)
    -t, --tag TAG          Image tag (default: latest)
    -s, --severity LEVEL   Minimum severity level (LOW|MEDIUM|HIGH|CRITICAL)
    -o, --output DIR       Output directory for reports
    --skip-trivy           Skip Trivy vulnerability scanning
    --skip-docker-bench    Skip Docker Bench security check
    --skip-hadolint        Skip Dockerfile linting
    -h, --help             Show this help

EXAMPLES:
    $0                     # Scan default image with all tools
    $0 -i myapp -t v1.0.0  # Scan specific image
    $0 --severity CRITICAL # Only show critical vulnerabilities
EOF
}

# Parse command line arguments
SKIP_TRIVY=false
SKIP_DOCKER_BENCH=false
SKIP_HADOLINT=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -i|--image)
            IMAGE_NAME="$2"
            shift 2
            ;;
        -t|--tag)
            IMAGE_TAG="$2"
            shift 2
            ;;
        -s|--severity)
            SEVERITY_THRESHOLD="$2"
            shift 2
            ;;
        -o|--output)
            SCAN_OUTPUT_DIR="$2"
            shift 2
            ;;
        --skip-trivy)
            SKIP_TRIVY=true
            shift
            ;;
        --skip-docker-bench)
            SKIP_DOCKER_BENCH=true
            shift
            ;;
        --skip-hadolint)
            SKIP_HADOLINT=true
            shift
            ;;
        *)
            if [[ ! "$1" =~ ^- ]]; then
                IMAGE_NAME="$1"
            else
                log_error "Unknown option: $1"
                show_help
                exit 1
            fi
            shift
            ;;
    esac
done

# Create output directory
mkdir -p "$SCAN_OUTPUT_DIR"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

# Full image name
FULL_IMAGE="$IMAGE_NAME:$IMAGE_TAG"

log_info "Starting security scan for: $FULL_IMAGE"
log_info "Output directory: $SCAN_OUTPUT_DIR"

# Check if image exists
if ! docker image inspect "$FULL_IMAGE" >/dev/null 2>&1; then
    log_error "Image not found: $FULL_IMAGE"
    log_info "Available images:"
    docker images "$IMAGE_NAME" || true
    exit 1
fi

# 1. Hadolint - Dockerfile linting
if [[ "$SKIP_HADOLINT" == "false" ]]; then
    log_info "Running Hadolint Dockerfile analysis..."
    
    HADOLINT_OUTPUT="$SCAN_OUTPUT_DIR/hadolint_${TIMESTAMP}.txt"
    
    if command -v hadolint >/dev/null 2>&1; then
        if hadolint Dockerfile > "$HADOLINT_OUTPUT" 2>&1; then
            log_success "Hadolint: No issues found"
        else
            HADOLINT_ISSUES=$(wc -l < "$HADOLINT_OUTPUT")
            log_warning "Hadolint: Found $HADOLINT_ISSUES issues (see $HADOLINT_OUTPUT)"
        fi
    else
        log_warning "Hadolint not installed. Installing via Docker..."
        if docker run --rm -i hadolint/hadolint < Dockerfile > "$HADOLINT_OUTPUT" 2>&1; then
            log_success "Hadolint: No issues found"
        else
            HADOLINT_ISSUES=$(wc -l < "$HADOLINT_OUTPUT")
            log_warning "Hadolint: Found $HADOLINT_ISSUES issues (see $HADOLINT_OUTPUT)"
        fi
    fi
fi

# 2. Trivy - Vulnerability scanning
if [[ "$SKIP_TRIVY" == "false" ]]; then
    log_info "Running Trivy vulnerability scan..."
    
    TRIVY_OUTPUT="$SCAN_OUTPUT_DIR/trivy_${TIMESTAMP}.json"
    TRIVY_SUMMARY="$SCAN_OUTPUT_DIR/trivy_summary_${TIMESTAMP}.txt"
    
    if command -v trivy >/dev/null 2>&1; then
        # JSON output for detailed analysis
        trivy image --format json --output "$TRIVY_OUTPUT" "$FULL_IMAGE" || true
        
        # Human-readable summary
        trivy image --severity "$SEVERITY_THRESHOLD" --format table "$FULL_IMAGE" > "$TRIVY_SUMMARY" 2>&1 || true
        
        # Count vulnerabilities
        if [[ -f "$TRIVY_OUTPUT" ]]; then
            CRITICAL=$(jq -r '.Results[]?.Vulnerabilities[]? | select(.Severity == "CRITICAL") | .VulnerabilityID' "$TRIVY_OUTPUT" 2>/dev/null | wc -l || echo "0")
            HIGH=$(jq -r '.Results[]?.Vulnerabilities[]? | select(.Severity == "HIGH") | .VulnerabilityID' "$TRIVY_OUTPUT" 2>/dev/null | wc -l || echo "0")
            MEDIUM=$(jq -r '.Results[]?.Vulnerabilities[]? | select(.Severity == "MEDIUM") | .VulnerabilityID' "$TRIVY_OUTPUT" 2>/dev/null | wc -l || echo "0")
            LOW=$(jq -r '.Results[]?.Vulnerabilities[]? | select(.Severity == "LOW") | .VulnerabilityID' "$TRIVY_OUTPUT" 2>/dev/null | wc -l || echo "0")
            
            log_info "Trivy Results: CRITICAL=$CRITICAL, HIGH=$HIGH, MEDIUM=$MEDIUM, LOW=$LOW"
            
            if [[ $CRITICAL -gt 0 ]]; then
                log_error "Found $CRITICAL CRITICAL vulnerabilities!"
            elif [[ $HIGH -gt 0 ]]; then
                log_warning "Found $HIGH HIGH severity vulnerabilities"
            else
                log_success "No high-severity vulnerabilities found"
            fi
        fi
    else
        log_warning "Trivy not installed. Installing via Docker..."
        docker run --rm -v "$(pwd)/$SCAN_OUTPUT_DIR:/output" \
            aquasec/trivy:latest image --format json --output "/output/trivy_${TIMESTAMP}.json" "$FULL_IMAGE" || true
        docker run --rm -v "$(pwd)/$SCAN_OUTPUT_DIR:/output" \
            aquasec/trivy:latest image --severity "$SEVERITY_THRESHOLD" --format table "$FULL_IMAGE" > "$TRIVY_SUMMARY" 2>&1 || true
    fi
fi

# 3. Docker Bench Security
if [[ "$SKIP_DOCKER_BENCH" == "false" ]]; then
    log_info "Running Docker Bench Security check..."
    
    DOCKER_BENCH_OUTPUT="$SCAN_OUTPUT_DIR/docker_bench_${TIMESTAMP}.log"
    
    if [[ -d "/tmp/docker-bench-security" ]]; then
        rm -rf /tmp/docker-bench-security
    fi
    
    if git clone https://github.com/docker/docker-bench-security.git /tmp/docker-bench-security >/dev/null 2>&1; then
        cd /tmp/docker-bench-security
        if sudo ./docker-bench-security.sh > "$DOCKER_BENCH_OUTPUT" 2>&1; then
            cd - >/dev/null
            WARNINGS=$(grep -c "WARN" "$DOCKER_BENCH_OUTPUT" || echo "0")
            INFOS=$(grep -c "INFO" "$DOCKER_BENCH_OUTPUT" || echo "0")
            log_info "Docker Bench: $WARNINGS warnings, $INFOS info items (see $DOCKER_BENCH_OUTPUT)"
        else
            cd - >/dev/null
            log_warning "Docker Bench Security check failed"
        fi
        rm -rf /tmp/docker-bench-security
    else
        log_warning "Failed to download Docker Bench Security"
    fi
fi

# 4. Image Analysis Summary
log_info "Analyzing image properties..."

IMAGE_ANALYSIS="$SCAN_OUTPUT_DIR/image_analysis_${TIMESTAMP}.txt"

cat > "$IMAGE_ANALYSIS" << EOF
μNet Docker Image Security Analysis Report
==========================================
Date: $(date)
Image: $FULL_IMAGE
Scan Directory: $SCAN_OUTPUT_DIR

Image Properties:
-----------------
$(docker image inspect "$FULL_IMAGE" --format '
Size: {{.Size}} bytes ({{div .Size 1048576}} MB)
Architecture: {{.Architecture}}
OS: {{.Os}}
Created: {{.Created}}
RootFS Type: {{.RootFS.Type}}
Layers: {{len .RootFS.Layers}}
')

Image History (last 10 layers):
-------------------------------
$(docker history "$FULL_IMAGE" --no-trunc --format "table {{.CreatedBy}}" | head -10)

Security Assessment:
-------------------
EOF

# Add vulnerability summary if available
if [[ -f "$TRIVY_OUTPUT" ]]; then
    echo "Vulnerability Summary (via Trivy):" >> "$IMAGE_ANALYSIS"
    echo "- Critical: $(jq -r '.Results[]?.Vulnerabilities[]? | select(.Severity == "CRITICAL") | .VulnerabilityID' "$TRIVY_OUTPUT" 2>/dev/null | wc -l || echo "0")" >> "$IMAGE_ANALYSIS"
    echo "- High: $(jq -r '.Results[]?.Vulnerabilities[]? | select(.Severity == "HIGH") | .VulnerabilityID' "$TRIVY_OUTPUT" 2>/dev/null | wc -l || echo "0")" >> "$IMAGE_ANALYSIS"
    echo "- Medium: $(jq -r '.Results[]?.Vulnerabilities[]? | select(.Severity == "MEDIUM") | .VulnerabilityID' "$TRIVY_OUTPUT" 2>/dev/null | wc -l || echo "0")" >> "$IMAGE_ANALYSIS"
    echo "- Low: $(jq -r '.Results[]?.Vulnerabilities[]? | select(.Severity == "LOW") | .VulnerabilityID' "$TRIVY_OUTPUT" 2>/dev/null | wc -l || echo "0")" >> "$IMAGE_ANALYSIS"
    echo "" >> "$IMAGE_ANALYSIS"
fi

echo "Recommendations:" >> "$IMAGE_ANALYSIS"
echo "- Regularly update base images" >> "$IMAGE_ANALYSIS"
echo "- Use minimal base images (distroless, alpine)" >> "$IMAGE_ANALYSIS"
echo "- Implement multi-stage builds to reduce final image size" >> "$IMAGE_ANALYSIS"
echo "- Run containers as non-root user" >> "$IMAGE_ANALYSIS"
echo "- Use specific image tags, not 'latest'" >> "$IMAGE_ANALYSIS"
echo "- Scan images in CI/CD pipeline" >> "$IMAGE_ANALYSIS"

log_success "Security scan completed successfully!"

# Summary
echo ""
log_info "📊 Security Scan Summary:"
echo "   📁 Reports saved to: $SCAN_OUTPUT_DIR"
echo "   📋 Analysis summary: $IMAGE_ANALYSIS"

if [[ -f "$TRIVY_OUTPUT" ]]; then
    echo "   🔍 Trivy report: $TRIVY_OUTPUT"
    echo "   📖 Trivy summary: $TRIVY_SUMMARY"
fi

if [[ -f "$HADOLINT_OUTPUT" ]]; then
    echo "   📝 Hadolint report: $HADOLINT_OUTPUT"
fi

if [[ -f "$DOCKER_BENCH_OUTPUT" ]]; then
    echo "   🔒 Docker Bench: $DOCKER_BENCH_OUTPUT"
fi

echo ""
log_info "🚀 Next steps:"
echo "   1. Review security reports"
echo "   2. Address critical/high severity vulnerabilities"
echo "   3. Implement fixes in Dockerfile"
echo "   4. Re-run security scan to verify fixes"
echo "   5. Integrate scanning into CI/CD pipeline"