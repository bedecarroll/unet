# μNet Development Makefile
# Provides convenient shortcuts for common development tasks

.PHONY: help setup build test fmt clippy docs clean run-server run-cli migrate dev-env

# Default target
help: ## Show this help message
	@echo "μNet Development Commands:"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}'

setup: ## Set up development environment
	@./scripts/setup-dev-env.sh

build: ## Build all crates
	@echo "🔨 Building workspace..."
	@cargo build --workspace

test: ## Run all tests
	@echo "🧪 Running tests..."
	@cargo test --workspace

fmt: ## Format all code
	@echo "📝 Formatting code..."
	@cargo fmt --all

clippy: ## Run clippy lints
	@echo "🔍 Running clippy..."
	@cargo clippy --workspace --all-targets --all-features -- -D warnings

docs: ## Build and serve documentation
	@echo "📚 Building documentation..."
	@cd docs && mdbook serve --open

clean: ## Clean build artifacts
	@echo "🧹 Cleaning build artifacts..."
	@cargo clean
	@rm -f *.db test_*.db

run-server: ## Run the μNet server
	@echo "🚀 Starting μNet server..."
	@cargo run --bin unet-server

run-cli: ## Run the μNet CLI (with 'help' command)
	@echo "💻 Running μNet CLI..."
	@cargo run --bin unet -- help

migrate: ## Run database migrations
	@echo "🗄️ Running database migrations..."
	@./scripts/migrate.sh

dev-env: ## Quick development environment check
	@echo "🔍 Development environment status:"
	@echo "Rust version: $$(rustc --version)"
	@echo "Cargo version: $$(cargo --version)"
	@echo "Git hooks: $$(if [ -x .git/hooks/pre-commit ]; then echo 'installed'; else echo 'not installed'; fi)"
	@echo "VS Code config: $$(if [ -d .vscode ]; then echo 'present'; else echo 'missing'; fi)"

ci: ## Run CI-like checks locally
	@echo "🤖 Running CI checks..."
	@make fmt
	@make clippy
	@make test
	@echo "✅ All CI checks passed!"

install-tools: ## Install additional development tools
	@echo "🛠️ Installing development tools..."
	@cargo install cargo-watch
	@cargo install mdbook
	@cargo install sea-orm-cli
	@echo "✅ Tools installed!"

watch: ## Watch for changes and auto-rebuild
	@echo "👀 Watching for changes..."
	@cargo watch -x "check --workspace"

watch-test: ## Watch for changes and auto-test
	@echo "👀 Watching for changes and running tests..."
	@cargo watch -x "test --workspace"

release: ## Build release version
	@echo "📦 Building release..."
	@cargo build --workspace --release

# Database management
db-reset: ## Reset database (delete and recreate)
	@echo "🗄️ Resetting database..."
	@rm -f unet.db test_*.db
	@make migrate

db-backup: ## Backup current database
	@echo "💾 Backing up database..."
	@cp unet.db unet_backup_$$(date +%Y%m%d_%H%M%S).db
	@echo "Database backed up"

# Development shortcuts
dev: ## Start development mode (server with auto-reload)
	@echo "🔥 Starting development mode..."
	@cargo watch -x "run --bin unet-server"

check-all: ## Run all quality checks
	@echo "🔍 Running comprehensive checks..."
	@make fmt
	@make clippy
	@make test
	@cargo audit || echo "⚠️ Audit issues found"
	@echo "✅ All checks completed!"