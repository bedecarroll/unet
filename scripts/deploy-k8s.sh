#!/bin/bash
# Kubernetes deployment script for μNet

set -euo pipefail

# Configuration
ENVIRONMENT="${ENVIRONMENT:-dev}"
NAMESPACE="${NAMESPACE:-unet-${ENVIRONMENT}}"
KUSTOMIZATION_PATH="${KUSTOMIZATION_PATH:-k8s/overlays/${ENVIRONMENT}}"
DRY_RUN="${DRY_RUN:-false}"
WAIT_TIMEOUT="${WAIT_TIMEOUT:-300s}"

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
μNet Kubernetes Deployment Script

Usage: $0 [OPTIONS] [COMMAND]

COMMANDS:
    deploy      Deploy μNet to Kubernetes (default)
    delete      Delete μNet from Kubernetes
    status      Check deployment status
    logs        Tail application logs
    port-forward Forward local port to service

OPTIONS:
    -e, --environment ENV   Target environment (dev|staging|prod)
    -n, --namespace NS      Kubernetes namespace
    -k, --kustomization PATH Path to kustomization
    --dry-run              Show what would be deployed
    --wait-timeout TIMEOUT Timeout for deployment wait
    -h, --help             Show this help

EXAMPLES:
    $0                                # Deploy to dev environment
    $0 -e prod deploy                 # Deploy to production
    $0 --dry-run -e staging deploy    # Dry run for staging
    $0 delete -e dev                  # Delete dev deployment
    $0 status -e prod                 # Check production status
EOF
}

# Parse command line arguments
COMMAND="deploy"

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -e|--environment)
            ENVIRONMENT="$2"
            NAMESPACE="unet-${ENVIRONMENT}"
            KUSTOMIZATION_PATH="k8s/overlays/${ENVIRONMENT}"
            shift 2
            ;;
        -n|--namespace)
            NAMESPACE="$2"
            shift 2
            ;;
        -k|--kustomization)
            KUSTOMIZATION_PATH="$2"
            shift 2
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --wait-timeout)
            WAIT_TIMEOUT="$2"
            shift 2
            ;;
        deploy|delete|status|logs|port-forward)
            COMMAND="$1"
            shift
            ;;
        *)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Ensure we're in the right directory
cd "$(dirname "$0")/.."

# Validate prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check kubectl
    if ! command -v kubectl >/dev/null 2>&1; then
        log_error "kubectl is required but not installed"
        exit 1
    fi
    
    # Check kustomize
    if ! command -v kustomize >/dev/null 2>&1; then
        log_warning "kustomize not found, using kubectl built-in"
    fi
    
    # Check cluster connection
    if ! kubectl cluster-info >/dev/null 2>&1; then
        log_error "Cannot connect to Kubernetes cluster"
        exit 1
    fi
    
    # Validate kustomization path
    if [[ ! -f "$KUSTOMIZATION_PATH/kustomization.yaml" ]]; then
        log_error "Kustomization file not found: $KUSTOMIZATION_PATH/kustomization.yaml"
        exit 1
    fi
    
    log_success "Prerequisites validated"
}

# Deploy function
deploy() {
    log_info "Deploying μNet to environment: $ENVIRONMENT"
    log_info "Namespace: $NAMESPACE"
    log_info "Kustomization: $KUSTOMIZATION_PATH"
    
    # Apply kustomization
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "Dry run mode - showing what would be deployed:"
        kubectl kustomize "$KUSTOMIZATION_PATH" | kubectl apply --dry-run=client -f -
    else
        log_info "Applying Kubernetes manifests..."
        kubectl apply -k "$KUSTOMIZATION_PATH"
        
        log_info "Waiting for deployment to be ready..."
        kubectl wait --for=condition=available --timeout="$WAIT_TIMEOUT" \
            deployment -l "app.kubernetes.io/part-of=unet" -n "$NAMESPACE" || true
        
        log_info "Waiting for pods to be ready..."
        kubectl wait --for=condition=ready --timeout="$WAIT_TIMEOUT" \
            pod -l "app.kubernetes.io/part-of=unet" -n "$NAMESPACE" || true
    fi
    
    log_success "Deployment completed"
}

# Delete function
delete() {
    log_warning "Deleting μNet from environment: $ENVIRONMENT"
    
    read -p "Are you sure you want to delete the deployment? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Deletion cancelled"
        exit 0
    fi
    
    kubectl delete -k "$KUSTOMIZATION_PATH" || true
    log_success "Deletion completed"
}

# Status function
status() {
    log_info "Checking deployment status for environment: $ENVIRONMENT"
    
    echo ""
    log_info "Namespace:"
    kubectl get namespace "$NAMESPACE" 2>/dev/null || echo "Namespace not found"
    
    echo ""
    log_info "Deployments:"
    kubectl get deployments -n "$NAMESPACE" -l "app.kubernetes.io/part-of=unet" 2>/dev/null || echo "No deployments found"
    
    echo ""
    log_info "Pods:"
    kubectl get pods -n "$NAMESPACE" -l "app.kubernetes.io/part-of=unet" 2>/dev/null || echo "No pods found"
    
    echo ""
    log_info "Services:"
    kubectl get services -n "$NAMESPACE" -l "app.kubernetes.io/part-of=unet" 2>/dev/null || echo "No services found"
    
    echo ""
    log_info "Ingresses:"
    kubectl get ingress -n "$NAMESPACE" 2>/dev/null || echo "No ingresses found"
    
    echo ""
    log_info "PVCs:"
    kubectl get pvc -n "$NAMESPACE" 2>/dev/null || echo "No PVCs found"
}

# Logs function
logs() {
    log_info "Streaming logs from μNet pods..."
    kubectl logs -f -l "app.kubernetes.io/name=unet-server" -n "$NAMESPACE" --max-log-requests=10
}

# Port forward function
port_forward() {
    local LOCAL_PORT="${LOCAL_PORT:-8080}"
    log_info "Port forwarding $LOCAL_PORT -> 8080 for μNet service..."
    kubectl port-forward service/unet-server-internal "$LOCAL_PORT:8080" -n "$NAMESPACE"
}

# Main execution
main() {
    case $COMMAND in
        deploy)
            check_prerequisites
            deploy
            status
            ;;
        delete)
            check_prerequisites
            delete
            ;;
        status)
            check_prerequisites
            status
            ;;
        logs)
            check_prerequisites
            logs
            ;;
        port-forward)
            check_prerequisites
            port_forward
            ;;
        *)
            log_error "Unknown command: $COMMAND"
            show_help
            exit 1
            ;;
    esac
}

# Run main function
main

# Post-deployment information
if [[ "$COMMAND" == "deploy" && "$DRY_RUN" == "false" ]]; then
    echo ""
    log_info "🚀 Deployment Information:"
    echo "   📍 Environment: $ENVIRONMENT"
    echo "   🏷️  Namespace: $NAMESPACE"
    echo "   📁 Kustomization: $KUSTOMIZATION_PATH"
    echo ""
    log_info "📋 Next Steps:"
    echo "   1. Check status: $0 status -e $ENVIRONMENT"
    echo "   2. View logs: $0 logs -e $ENVIRONMENT"
    echo "   3. Port forward: $0 port-forward -e $ENVIRONMENT"
    echo "   4. Access service: kubectl get ingress -n $NAMESPACE"
fi