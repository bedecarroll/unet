#!/bin/bash
# Systemd deployment script for μNet

set -euo pipefail

# Configuration
UNET_USER="${UNET_USER:-unet}"
UNET_GROUP="${UNET_GROUP:-unet}"
POSTGRES_USER="${POSTGRES_USER:-postgres}"
REDIS_USER="${REDIS_USER:-redis}"
INSTALL_DIR="${INSTALL_DIR:-/usr/bin}"
CONFIG_DIR="${CONFIG_DIR:-/etc/unet}"
DATA_DIR="${DATA_DIR:-/var/lib/unet}"
LOG_DIR="${LOG_DIR:-/var/log/unet}"
BACKUP_DIR="${BACKUP_DIR:-/var/backups/unet}"
SYSTEMD_DIR="${SYSTEMD_DIR:-/etc/systemd/system}"

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
μNet Systemd Deployment Script

Usage: $0 [OPTIONS] [COMMAND]

COMMANDS:
    install     Install and configure μNet services (default)
    uninstall   Remove μNet services
    start       Start μNet services
    stop        Stop μNet services
    restart     Restart μNet services
    status      Show service status
    logs        Show service logs
    upgrade     Upgrade μNet installation

OPTIONS:
    --user USER         System user for μNet (default: unet)
    --postgres-user USER PostgreSQL user (default: postgres)
    --redis-user USER   Redis user (default: redis)
    --install-dir DIR   Installation directory (default: /usr/bin)
    --config-dir DIR    Configuration directory (default: /etc/unet)
    --data-dir DIR      Data directory (default: /var/lib/unet)
    --log-dir DIR       Log directory (default: /var/log/unet)
    --backup-dir DIR    Backup directory (default: /var/backups/unet)
    --skip-postgres     Skip PostgreSQL installation
    --skip-redis        Skip Redis installation
    --dry-run           Show what would be done
    -h, --help          Show this help

EXAMPLES:
    $0                                  # Install with default settings
    $0 --skip-postgres install         # Install without PostgreSQL
    $0 --dry-run install              # Show what would be installed
    $0 status                         # Check service status
    $0 logs                           # Show logs
EOF
}

# Parse command line arguments
COMMAND="install"
SKIP_POSTGRES=false
SKIP_REDIS=false
DRY_RUN=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        --user)
            UNET_USER="$2"
            shift 2
            ;;
        --postgres-user)
            POSTGRES_USER="$2"
            shift 2
            ;;
        --redis-user)
            REDIS_USER="$2"
            shift 2
            ;;
        --install-dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        --config-dir)
            CONFIG_DIR="$2"
            shift 2
            ;;
        --data-dir)
            DATA_DIR="$2"
            shift 2
            ;;
        --log-dir)
            LOG_DIR="$2"
            shift 2
            ;;
        --backup-dir)
            BACKUP_DIR="$2"
            shift 2
            ;;
        --skip-postgres)
            SKIP_POSTGRES=true
            shift
            ;;
        --skip-redis)
            SKIP_REDIS=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        install|uninstall|start|stop|restart|status|logs|upgrade)
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

# Ensure we're running as root
check_root() {
    if [[ $EUID -ne 0 ]] && [[ "$DRY_RUN" != "true" ]]; then
        log_error "This script must be run as root (or use --dry-run)"
        exit 1
    fi
}

# Execute command with dry-run support
execute() {
    local cmd="$1"
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "DRY RUN: $cmd"
    else
        log_info "Executing: $cmd"
        eval "$cmd"
    fi
}

# Create system users
create_users() {
    log_info "Creating system users..."
    
    if ! id "$UNET_USER" >/dev/null 2>&1; then
        execute "useradd --system --create-home --shell /bin/bash --user-group '$UNET_USER'"
        log_success "Created user: $UNET_USER"
    else
        log_info "User $UNET_USER already exists"
    fi
    
    if [[ "$SKIP_POSTGRES" != "true" ]] && ! id "$POSTGRES_USER" >/dev/null 2>&1; then
        execute "useradd --system --create-home --shell /bin/bash --user-group '$POSTGRES_USER'"
        log_success "Created user: $POSTGRES_USER"
    fi
    
    if [[ "$SKIP_REDIS" != "true" ]] && ! id "$REDIS_USER" >/dev/null 2>&1; then
        execute "useradd --system --create-home --shell /bin/bash --user-group '$REDIS_USER'"
        log_success "Created user: $REDIS_USER"
    fi
}

# Create directories
create_directories() {
    log_info "Creating directories..."
    
    local dirs=("$CONFIG_DIR" "$DATA_DIR" "$LOG_DIR" "$BACKUP_DIR")
    
    for dir in "${dirs[@]}"; do
        execute "mkdir -p '$dir'"
        execute "chown '$UNET_USER:$UNET_GROUP' '$dir'"
        execute "chmod 755 '$dir'"
    done
    
    if [[ "$SKIP_POSTGRES" != "true" ]]; then
        execute "mkdir -p /var/lib/postgresql/data/unet"
        execute "chown '$POSTGRES_USER:$POSTGRES_USER' /var/lib/postgresql/data/unet"
        execute "chmod 700 /var/lib/postgresql/data/unet"
    fi
    
    if [[ "$SKIP_REDIS" != "true" ]]; then
        execute "mkdir -p /var/lib/redis"
        execute "chown '$REDIS_USER:$REDIS_USER' /var/lib/redis"
        execute "chmod 755 /var/lib/redis"
    fi
    
    log_success "Directories created"
}

# Install systemd services
install_services() {
    log_info "Installing systemd services..."
    
    local services=("unet-server.service" "unet.target" "unet-backup.service" "unet-backup.timer" "unet-healthcheck.service" "unet-healthcheck.timer")
    
    if [[ "$SKIP_POSTGRES" != "true" ]]; then
        services+=("unet-postgres.service")
    fi
    
    if [[ "$SKIP_REDIS" != "true" ]]; then
        services+=("unet-redis.service")
    fi
    
    for service in "${services[@]}"; do
        if [[ -f "systemd/$service" ]]; then
            execute "cp 'systemd/$service' '$SYSTEMD_DIR/'"
            execute "chmod 644 '$SYSTEMD_DIR/$service'"
            log_success "Installed: $service"
        else
            log_warning "Service file not found: systemd/$service"
        fi
    done
    
    execute "systemctl daemon-reload"
    log_success "Systemd services installed"
}

# Install configuration files
install_config() {
    log_info "Installing configuration files..."
    
    if [[ -f "config.toml" ]]; then
        execute "cp config.toml '$CONFIG_DIR/'"
        execute "chown '$UNET_USER:$UNET_GROUP' '$CONFIG_DIR/config.toml'"
        execute "chmod 640 '$CONFIG_DIR/config.toml'"
        log_success "Installed configuration file"
    else
        log_warning "Configuration file not found: config.toml"
    fi
}

# Install binaries
install_binaries() {
    log_info "Installing binaries..."
    
    local binaries=("unet-server" "unet")
    
    for binary in "${binaries[@]}"; do
        if [[ -f "target/release/$binary" ]]; then
            execute "cp 'target/release/$binary' '$INSTALL_DIR/'"
            execute "chmod 755 '$INSTALL_DIR/$binary'"
            log_success "Installed: $binary"
        else
            log_warning "Binary not found: target/release/$binary"
        fi
    done
}

# Start services
start_services() {
    log_info "Starting μNet services..."
    
    if [[ "$SKIP_POSTGRES" != "true" ]]; then
        execute "systemctl enable unet-postgres.service"
        execute "systemctl start unet-postgres.service"
    fi
    
    if [[ "$SKIP_REDIS" != "true" ]]; then
        execute "systemctl enable unet-redis.service"
        execute "systemctl start unet-redis.service"
    fi
    
    execute "systemctl enable unet-server.service"
    execute "systemctl start unet-server.service"
    execute "systemctl enable unet-backup.timer"
    execute "systemctl start unet-backup.timer"
    execute "systemctl enable unet-healthcheck.timer"
    execute "systemctl start unet-healthcheck.timer"
    
    log_success "Services started"
}

# Stop services
stop_services() {
    log_info "Stopping μNet services..."
    
    execute "systemctl stop unet-healthcheck.timer || true"
    execute "systemctl stop unet-backup.timer || true"
    execute "systemctl stop unet-server.service || true"
    
    if [[ "$SKIP_REDIS" != "true" ]]; then
        execute "systemctl stop unet-redis.service || true"
    fi
    
    if [[ "$SKIP_POSTGRES" != "true" ]]; then
        execute "systemctl stop unet-postgres.service || true"
    fi
    
    log_success "Services stopped"
}

# Show service status
show_status() {
    log_info "Service status:"
    
    local services=("unet-server.service")
    
    if [[ "$SKIP_POSTGRES" != "true" ]]; then
        services+=("unet-postgres.service")
    fi
    
    if [[ "$SKIP_REDIS" != "true" ]]; then
        services+=("unet-redis.service")
    fi
    
    services+=("unet-backup.timer" "unet-healthcheck.timer")
    
    for service in "${services[@]}"; do
        echo ""
        echo "=== $service ==="
        if systemctl is-active --quiet "$service"; then
            echo -e "${GREEN}Active${NC}"
        else
            echo -e "${RED}Inactive${NC}"
        fi
        systemctl status "$service" --no-pager -l || true
    done
}

# Show logs
show_logs() {
    log_info "Showing μNet logs..."
    journalctl -u unet-server.service -f --no-pager
}

# Uninstall services
uninstall_services() {
    log_warning "Uninstalling μNet services..."
    
    stop_services
    
    local services=("unet-server.service" "unet-postgres.service" "unet-redis.service" "unet.target" "unet-backup.service" "unet-backup.timer" "unet-healthcheck.service" "unet-healthcheck.timer")
    
    for service in "${services[@]}"; do
        execute "systemctl disable '$service' || true"
        execute "rm -f '$SYSTEMD_DIR/$service'"
        log_success "Removed: $service"
    done
    
    execute "systemctl daemon-reload"
    log_success "Services uninstalled"
}

# Main execution
main() {
    case $COMMAND in
        install)
            check_root
            log_info "Installing μNet with systemd..."
            create_users
            create_directories
            install_binaries
            install_config
            install_services
            start_services
            log_success "μNet installation completed!"
            ;;
        uninstall)
            check_root
            uninstall_services
            log_success "μNet uninstalled"
            ;;
        start)
            check_root
            start_services
            ;;
        stop)
            check_root
            stop_services
            ;;
        restart)
            check_root
            stop_services
            sleep 2
            start_services
            ;;
        status)
            show_status
            ;;
        logs)
            show_logs
            ;;
        upgrade)
            check_root
            log_info "Upgrading μNet..."
            stop_services
            install_binaries
            install_config
            install_services
            start_services
            log_success "μNet upgrade completed!"
            ;;
        *)
            log_error "Unknown command: $COMMAND"
            show_help
            exit 1
            ;;
    esac
}

# Change to script directory
cd "$(dirname "$0")/.."

# Run main function
main

# Post-installation information
if [[ "$COMMAND" == "install" && "$DRY_RUN" != "true" ]]; then
    echo ""
    log_info "🚀 Installation Information:"
    echo "   📍 Configuration: $CONFIG_DIR/config.toml"
    echo "   📁 Data directory: $DATA_DIR"
    echo "   📝 Log directory: $LOG_DIR"
    echo "   💾 Backup directory: $BACKUP_DIR"
    echo ""
    log_info "📋 Next Steps:"
    echo "   1. Edit configuration: sudo vim $CONFIG_DIR/config.toml"
    echo "   2. Check status: $0 status"
    echo "   3. View logs: $0 logs"
    echo "   4. Test API: curl http://localhost:8080/health"
fi