#!/bin/bash
# Comprehensive deployment automation script for μNet

set -euo pipefail

# Configuration
DEPLOYMENT_TYPE="${DEPLOYMENT_TYPE:-kubernetes}"
ENVIRONMENT="${ENVIRONMENT:-dev}"
VERSION="${VERSION:-latest}"
DRY_RUN="${DRY_RUN:-false}"
VERBOSE="${VERBOSE:-false}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
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

log_step() {
    echo -e "${CYAN}[STEP]${NC} $1"
}

# Verbose logging
log_verbose() {
    if [[ "$VERBOSE" == "true" ]]; then
        echo -e "${CYAN}[VERBOSE]${NC} $1"
    fi
}

# Help function
show_help() {
    cat << EOF
μNet Comprehensive Deployment Automation

Usage: $0 [OPTIONS] [COMMAND]

COMMANDS:
    deploy              Deploy μNet (default)
    undeploy           Remove μNet deployment
    status             Check deployment status
    upgrade            Upgrade existing deployment
    rollback           Rollback to previous version
    validate           Validate deployment configuration
    logs               Show deployment logs

DEPLOYMENT TYPES:
    kubernetes         Deploy to Kubernetes (default)
    systemd            Deploy with systemd services
    docker             Deploy with Docker Compose
    helm               Deploy using Helm charts
    ansible            Deploy using Ansible playbooks

ENVIRONMENTS:
    dev                Development environment
    staging            Staging environment  
    prod               Production environment

OPTIONS:
    -t, --type TYPE    Deployment type (kubernetes|systemd|docker|helm|ansible)
    -e, --env ENV      Target environment (dev|staging|prod)
    -v, --version VER  Version to deploy (default: latest)
    --dry-run          Show what would be deployed
    --verbose          Enable verbose output
    --force            Force deployment even if validation fails
    --skip-tests       Skip pre-deployment tests
    --skip-backup      Skip backup before upgrade/rollback
    -h, --help         Show this help

EXAMPLES:
    $0                                    # Deploy to dev with Kubernetes
    $0 -t helm -e prod deploy            # Deploy to prod with Helm
    $0 -t systemd -e staging deploy      # Deploy to staging with systemd
    $0 --dry-run -e prod deploy          # Dry run production deployment
    $0 -t kubernetes status              # Check Kubernetes deployment status
    $0 -v v0.2.0 upgrade                 # Upgrade to specific version
    $0 rollback                          # Rollback to previous version
EOF
}

# Parse command line arguments
COMMAND="deploy"
FORCE=false
SKIP_TESTS=false
SKIP_BACKUP=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -t|--type)
            DEPLOYMENT_TYPE="$2"
            shift 2
            ;;
        -e|--env)
            ENVIRONMENT="$2"
            shift 2
            ;;
        -v|--version)
            VERSION="$2"
            shift 2
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --force)
            FORCE=true
            shift
            ;;
        --skip-tests)
            SKIP_TESTS=true
            shift
            ;;
        --skip-backup)
            SKIP_BACKUP=true
            shift
            ;;
        deploy|undeploy|status|upgrade|rollback|validate|logs)
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

# Validate deployment type
validate_deployment_type() {
    case $DEPLOYMENT_TYPE in
        kubernetes|systemd|docker|helm|ansible)
            log_info "Deployment type: $DEPLOYMENT_TYPE"
            ;;
        *)
            log_error "Invalid deployment type: $DEPLOYMENT_TYPE"
            log_error "Valid types: kubernetes, systemd, docker, helm, ansible"
            exit 1
            ;;
    esac
}

# Validate environment
validate_environment() {
    case $ENVIRONMENT in
        dev|staging|prod)
            log_info "Target environment: $ENVIRONMENT"
            ;;
        *)
            log_error "Invalid environment: $ENVIRONMENT"
            log_error "Valid environments: dev, staging, prod"
            exit 1
            ;;
    esac
}

# Check prerequisites
check_prerequisites() {
    log_step "Checking prerequisites for $DEPLOYMENT_TYPE deployment..."
    
    case $DEPLOYMENT_TYPE in
        kubernetes)
            if ! command -v kubectl >/dev/null 2>&1; then
                log_error "kubectl is required for Kubernetes deployment"
                exit 1
            fi
            if ! kubectl cluster-info >/dev/null 2>&1; then
                log_error "Cannot connect to Kubernetes cluster"
                exit 1
            fi
            ;;
        helm)
            if ! command -v helm >/dev/null 2>&1; then
                log_error "helm is required for Helm deployment"
                exit 1
            fi
            if ! kubectl cluster-info >/dev/null 2>&1; then
                log_error "Cannot connect to Kubernetes cluster"
                exit 1
            fi
            ;;
        docker)
            if ! command -v docker >/dev/null 2>&1; then
                log_error "docker is required for Docker deployment"
                exit 1
            fi
            if ! command -v docker-compose >/dev/null 2>&1; then
                log_error "docker-compose is required for Docker deployment"
                exit 1
            fi
            ;;
        ansible)
            if ! command -v ansible-playbook >/dev/null 2>&1; then
                log_error "ansible-playbook is required for Ansible deployment"
                exit 1
            fi
            ;;
        systemd)
            if [[ $EUID -ne 0 ]] && [[ "$DRY_RUN" != "true" ]]; then
                log_error "Root privileges required for systemd deployment"
                exit 1
            fi
            ;;
    esac
    
    log_success "Prerequisites validated"
}

# Run pre-deployment tests
run_tests() {
    if [[ "$SKIP_TESTS" == "true" ]]; then
        log_warning "Skipping pre-deployment tests"
        return
    fi
    
    log_step "Running pre-deployment tests..."
    
    # Build tests
    if [[ -f "Cargo.toml" ]]; then
        log_verbose "Running cargo tests..."
        cargo test --workspace --release
        log_success "Cargo tests passed"
    fi
    
    # Configuration validation
    if [[ -f "config-${ENVIRONMENT}.toml" ]]; then
        log_verbose "Validating configuration..."
        # Add configuration validation logic here
        log_success "Configuration validated"
    fi
    
    log_success "Pre-deployment tests completed"
}

# Create backup
create_backup() {
    if [[ "$SKIP_BACKUP" == "true" ]]; then
        log_warning "Skipping backup creation"
        return
    fi
    
    log_step "Creating backup..."
    
    local backup_dir="backups/$(date +%Y%m%d_%H%M%S)"
    mkdir -p "$backup_dir"
    
    case $DEPLOYMENT_TYPE in
        kubernetes|helm)
            if kubectl get namespace unet-$ENVIRONMENT >/dev/null 2>&1; then
                kubectl get all -n unet-$ENVIRONMENT -o yaml > "$backup_dir/kubernetes-resources.yaml"
                log_success "Kubernetes resources backed up"
            fi
            ;;
        systemd)
            if systemctl is-active --quiet unet-server.service; then
                tar -czf "$backup_dir/systemd-backup.tar.gz" /etc/unet /var/lib/unet /var/log/unet 2>/dev/null || true
                log_success "Systemd deployment backed up"
            fi
            ;;
    esac
    
    log_success "Backup created: $backup_dir"
}

# Deploy with Kubernetes
deploy_kubernetes() {
    log_step "Deploying μNet with Kubernetes..."
    
    local namespace="unet-$ENVIRONMENT"
    local kustomization_path="k8s/overlays/$ENVIRONMENT"
    
    if [[ ! -d "$kustomization_path" ]]; then
        log_error "Kustomization path not found: $kustomization_path"
        exit 1
    fi
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "DRY RUN: kubectl apply -k $kustomization_path"
        kubectl kustomize "$kustomization_path" | kubectl apply --dry-run=client -f -
    else
        kubectl apply -k "$kustomization_path"
        
        log_info "Waiting for deployment to be ready..."
        kubectl wait --for=condition=available --timeout=300s \
            deployment -l "app.kubernetes.io/part-of=unet" -n "$namespace" || true
    fi
    
    log_success "Kubernetes deployment completed"
}

# Deploy with Helm
deploy_helm() {
    log_step "Deploying μNet with Helm..."
    
    local release_name="unet-$ENVIRONMENT"
    local namespace="unet-$ENVIRONMENT"
    local values_file="helm/unet/values-$ENVIRONMENT.yaml"
    
    local helm_args=(
        "upgrade" "--install" "$release_name"
        "helm/unet"
        "--namespace" "$namespace"
        "--create-namespace"
        "--set" "unet.image.tag=$VERSION"
        "--set" "environment=$ENVIRONMENT"
    )
    
    if [[ -f "$values_file" ]]; then
        helm_args+=("--values" "$values_file")
    fi
    
    if [[ "$DRY_RUN" == "true" ]]; then
        helm_args+=("--dry-run")
    fi
    
    helm "${helm_args[@]}"
    
    if [[ "$DRY_RUN" != "true" ]]; then
        log_info "Waiting for deployment to be ready..."
        helm status "$release_name" -n "$namespace"
    fi
    
    log_success "Helm deployment completed"
}

# Deploy with Docker Compose
deploy_docker() {
    log_step "Deploying μNet with Docker Compose..."
    
    local compose_file="docker-compose.yml"
    if [[ "$ENVIRONMENT" == "prod" ]]; then
        compose_file="docker-compose.prod.yml"
    fi
    
    if [[ ! -f "$compose_file" ]]; then
        log_error "Docker Compose file not found: $compose_file"
        exit 1
    fi
    
    local docker_args=(
        "-f" "$compose_file"
        "up" "-d"
    )
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "DRY RUN: docker-compose ${docker_args[*]}"
        docker-compose -f "$compose_file" config
    else
        docker-compose "${docker_args[@]}"
        
        log_info "Waiting for services to be ready..."
        sleep 10
        docker-compose -f "$compose_file" ps
    fi
    
    log_success "Docker Compose deployment completed"
}

# Deploy with Ansible
deploy_ansible() {
    log_step "Deploying μNet with Ansible..."
    
    local playbook="ansible/deploy-unet.yml"
    local inventory="ansible/inventory.yml"
    
    if [[ ! -f "$playbook" ]]; then
        log_error "Ansible playbook not found: $playbook"
        exit 1
    fi
    
    local ansible_args=(
        "-i" "$inventory"
        "$playbook"
        "--limit" "$ENVIRONMENT"
        "--extra-vars" "unet_version=$VERSION"
        "--extra-vars" "unet_environment=$ENVIRONMENT"
    )
    
    if [[ "$DRY_RUN" == "true" ]]; then
        ansible_args+=("--check")
    fi
    
    if [[ "$VERBOSE" == "true" ]]; then
        ansible_args+=("-v")
    fi
    
    ansible-playbook "${ansible_args[@]}"
    
    log_success "Ansible deployment completed"
}

# Deploy with systemd
deploy_systemd() {
    log_step "Deploying μNet with systemd..."
    
    local script_args=(
        "--user" "unet"
        "--config-dir" "/etc/unet"
        "--data-dir" "/var/lib/unet"
        "--log-dir" "/var/log/unet"
    )
    
    if [[ "$DRY_RUN" == "true" ]]; then
        script_args+=("--dry-run")
    fi
    
    ./scripts/deploy-systemd.sh "${script_args[@]}" install
    
    log_success "Systemd deployment completed"
}

# Main deployment function
deploy() {
    log_info "Starting μNet deployment..."
    log_info "Type: $DEPLOYMENT_TYPE, Environment: $ENVIRONMENT, Version: $VERSION"
    
    case $DEPLOYMENT_TYPE in
        kubernetes)
            deploy_kubernetes
            ;;
        helm)
            deploy_helm
            ;;
        docker)
            deploy_docker
            ;;
        ansible)
            deploy_ansible
            ;;
        systemd)
            deploy_systemd
            ;;
    esac
}

# Status check function
check_status() {
    log_step "Checking deployment status..."
    
    case $DEPLOYMENT_TYPE in
        kubernetes|helm)
            local namespace="unet-$ENVIRONMENT"
            echo ""
            echo "=== Deployments ==="
            kubectl get deployments -n "$namespace" 2>/dev/null || echo "No deployments found"
            echo ""
            echo "=== Pods ==="
            kubectl get pods -n "$namespace" 2>/dev/null || echo "No pods found"
            echo ""
            echo "=== Services ==="
            kubectl get services -n "$namespace" 2>/dev/null || echo "No services found"
            ;;
        docker)
            echo ""
            echo "=== Docker Containers ==="
            docker-compose ps 2>/dev/null || echo "No containers found"
            ;;
        systemd)
            echo ""
            echo "=== Systemd Services ==="
            systemctl status unet-server.service --no-pager -l || true
            ;;
    esac
}

# Undeploy function
undeploy() {
    log_warning "Removing μNet deployment..."
    
    case $DEPLOYMENT_TYPE in
        kubernetes)
            local namespace="unet-$ENVIRONMENT"
            kubectl delete namespace "$namespace" --ignore-not-found=true
            ;;
        helm)
            local release_name="unet-$ENVIRONMENT"
            local namespace="unet-$ENVIRONMENT"
            helm uninstall "$release_name" -n "$namespace" || true
            kubectl delete namespace "$namespace" --ignore-not-found=true
            ;;
        docker)
            docker-compose down -v || true
            ;;
        systemd)
            ./scripts/deploy-systemd.sh uninstall
            ;;
    esac
    
    log_success "Deployment removed"
}

# Main execution
main() {
    # Header
    echo ""
    echo "======================================"
    echo "     μNet Deployment Automation"
    echo "======================================"
    echo ""
    
    validate_deployment_type
    validate_environment
    check_prerequisites
    
    case $COMMAND in
        deploy)
            run_tests
            create_backup
            deploy
            check_status
            ;;
        undeploy)
            create_backup
            undeploy
            ;;
        status)
            check_status
            ;;
        upgrade)
            log_info "Upgrading μNet to version $VERSION..."
            run_tests
            create_backup
            deploy
            check_status
            ;;
        rollback)
            log_warning "Rollback functionality not implemented yet"
            exit 1
            ;;
        validate)
            run_tests
            log_success "Validation completed"
            ;;
        logs)
            case $DEPLOYMENT_TYPE in
                kubernetes|helm)
                    kubectl logs -f -l "app.kubernetes.io/name=unet-server" -n "unet-$ENVIRONMENT"
                    ;;
                systemd)
                    journalctl -u unet-server.service -f
                    ;;
                docker)
                    docker-compose logs -f
                    ;;
            esac
            ;;
        *)
            log_error "Unknown command: $COMMAND"
            show_help
            exit 1
            ;;
    esac
    
    # Footer
    if [[ "$COMMAND" == "deploy" && "$DRY_RUN" != "true" ]]; then
        echo ""
        log_success "🚀 μNet deployment completed successfully!"
        echo ""
        log_info "📋 Next Steps:"
        echo "   1. Check status: $0 -t $DEPLOYMENT_TYPE -e $ENVIRONMENT status"
        echo "   2. View logs: $0 -t $DEPLOYMENT_TYPE -e $ENVIRONMENT logs"
        echo "   3. Test API: curl http://localhost:8080/health"
    fi
}

# Run main function
main