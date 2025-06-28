#!/bin/bash
# Docker build script for μNet with multi-stage optimization

set -euo pipefail

# Configuration
IMAGE_NAME="unet"
IMAGE_TAG="${IMAGE_TAG:-latest}"
REGISTRY="${REGISTRY:-}"
BUILD_ARGS="${BUILD_ARGS:-}"
PLATFORM="${PLATFORM:-linux/amd64,linux/arm64}"

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
μNet Docker Build Script

Usage: $0 [OPTIONS] [TARGET]

TARGETS:
    server      Build server image only
    cli         Build CLI image only
    all-in-one  Build all-in-one image (default)
    all         Build all image variants

OPTIONS:
    -t, --tag TAG           Set image tag (default: latest)
    -r, --registry REGISTRY Set registry prefix
    -p, --platform PLATFORM Set build platforms (default: linux/amd64,linux/arm64)
    --push                  Push images to registry after build
    --no-cache              Build without cache
    --build-arg ARG=VALUE   Pass build arguments
    -h, --help              Show this help

EXAMPLES:
    $0                                    # Build all-in-one image
    $0 server -t v1.0.0                 # Build server image with tag v1.0.0
    $0 all -r myregistry.com/unet --push # Build all variants and push
    $0 --no-cache --build-arg RUST_VERSION=1.85
EOF
}

# Parse command line arguments
TARGET="all-in-one"
PUSH=false
NO_CACHE=false
EXTRA_BUILD_ARGS=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -t|--tag)
            IMAGE_TAG="$2"
            shift 2
            ;;
        -r|--registry)
            REGISTRY="$2"
            shift 2
            ;;
        -p|--platform)
            PLATFORM="$2"
            shift 2
            ;;
        --push)
            PUSH=true
            shift
            ;;
        --no-cache)
            NO_CACHE=true
            shift
            ;;
        --build-arg)
            EXTRA_BUILD_ARGS="$EXTRA_BUILD_ARGS --build-arg $2"
            shift 2
            ;;
        server|cli|all-in-one|all)
            TARGET="$1"
            shift
            ;;
        *)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Set full image name
if [[ -n "$REGISTRY" ]]; then
    FULL_IMAGE_NAME="$REGISTRY/$IMAGE_NAME"
else
    FULL_IMAGE_NAME="$IMAGE_NAME"
fi

# Build options
BUILD_OPTS="--platform $PLATFORM"
if [[ "$NO_CACHE" == "true" ]]; then
    BUILD_OPTS="$BUILD_OPTS --no-cache"
fi
if [[ "$PUSH" == "true" ]]; then
    BUILD_OPTS="$BUILD_OPTS --push"
fi

# Add extra build args
if [[ -n "$EXTRA_BUILD_ARGS" ]]; then
    BUILD_OPTS="$BUILD_OPTS $EXTRA_BUILD_ARGS"
fi

# Ensure we're in the right directory
cd "$(dirname "$0")/.."

# Validate Docker buildx
if ! docker buildx version >/dev/null 2>&1; then
    log_error "Docker buildx is required for multi-platform builds"
    exit 1
fi

# Create/use buildx builder
BUILDER_NAME="unet-builder"
if ! docker buildx inspect $BUILDER_NAME >/dev/null 2>&1; then
    log_info "Creating buildx builder: $BUILDER_NAME"
    docker buildx create --name $BUILDER_NAME --platform $PLATFORM
fi

log_info "Using buildx builder: $BUILDER_NAME"
docker buildx use $BUILDER_NAME

# Build function
build_image() {
    local target=$1
    local image_suffix=$2
    
    local full_name="$FULL_IMAGE_NAME$image_suffix:$IMAGE_TAG"
    
    log_info "Building $target image: $full_name"
    log_info "Platform: $PLATFORM"
    log_info "Build options: $BUILD_OPTS"
    
    # Execute build
    if docker buildx build \
        --target $target \
        --tag $full_name \
        $BUILD_OPTS \
        .; then
        log_success "Successfully built $full_name"
    else
        log_error "Failed to build $full_name"
        return 1
    fi
}

# Build targets
case $TARGET in
    server)
        build_image "server" "-server"
        ;;
    cli)
        build_image "cli" "-cli"
        ;;
    all-in-one)
        build_image "all-in-one" ""
        ;;
    all)
        log_info "Building all image variants..."
        build_image "server" "-server"
        build_image "cli" "-cli"  
        build_image "all-in-one" ""
        log_success "All images built successfully"
        ;;
    *)
        log_error "Unknown target: $TARGET"
        exit 1
        ;;
esac

# Image size report
if [[ "$PUSH" == "false" ]]; then
    log_info "Local image sizes:"
    case $TARGET in
        server)
            docker images "$FULL_IMAGE_NAME-server:$IMAGE_TAG" --format "table {{.Repository}}:{{.Tag}}\t{{.Size}}"
            ;;
        cli)
            docker images "$FULL_IMAGE_NAME-cli:$IMAGE_TAG" --format "table {{.Repository}}:{{.Tag}}\t{{.Size}}"
            ;;
        all-in-one)
            docker images "$FULL_IMAGE_NAME:$IMAGE_TAG" --format "table {{.Repository}}:{{.Tag}}\t{{.Size}}"
            ;;
        all)
            docker images | grep -E "^$FULL_IMAGE_NAME" | head -3
            ;;
    esac
fi

log_success "Docker build completed successfully!"

# Usage examples
cat << EOF

🐳 Next Steps:

# Run the server locally:
docker run -p 8080:8080 $FULL_IMAGE_NAME:$IMAGE_TAG

# Run with docker-compose:
docker-compose up unet-server

# Use CLI tools:
docker run --rm -it $FULL_IMAGE_NAME-cli:$IMAGE_TAG nodes list

# Deploy to production:
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d

EOF