.PHONY: help install build build-frontend build-backend test test-unit test-integration test-e2e test-coverage test-coverage-ci test-watch test-frontend run clean dev-frontend dev-backend check fmt setup-hooks db-setup webdriver-setup ci-local

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-20s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

install: ## Install development dependencies
	@echo "Installing Rust toolchain..."
	rustup target add wasm32-unknown-unknown
	@echo "Installing Trunk..."
	cargo install trunk
	@echo "Done! Dependencies installed."

build: build-frontend build-backend ## Build both frontend and backend for production

build-frontend: ## Build frontend for production
	@echo "Building frontend..."
	cd frontend && trunk build --release
	@echo "Frontend built successfully in frontend/dist/"

build-backend: ## Build backend for production
	@echo "Building backend..."
	cargo build --release
	@echo "Backend built successfully at target/release/call-for-papers"

test: ## Run all tests (unit + integration)
	@echo "Running backend tests..."
	cargo test
	@echo "All tests passed!"

test-unit: ## Run unit tests only
	@echo "Running unit tests..."
	SQLX_OFFLINE=true cargo test --lib
	@echo "Unit tests passed!"

test-integration: ## Run integration tests only (requires PostgreSQL)
	@echo "Running integration tests..."
	@echo "Make sure PostgreSQL is running and test database exists!"
	cargo test --test '*' -- --skip e2e
	@echo "Integration tests passed!"

test-e2e: ## Run end-to-end tests (requires PostgreSQL, WebDriver, and built frontend)
	@echo "Running end-to-end tests..."
	@echo "Make sure PostgreSQL is running, test database exists, and WebDriver is running!"
	@echo "Start WebDriver with: make webdriver-setup"
	cargo test --test 'e2e_*' -- --ignored --test-threads=1
	@echo "E2E tests passed!"

test-watch: ## Run tests in watch mode (auto-rerun on changes)
	@echo "Running tests in watch mode..."
	@command -v cargo-watch >/dev/null 2>&1 || { echo "Installing cargo-watch..."; cargo install cargo-watch; }
	SQLX_OFFLINE=true cargo watch -x "test --lib"

test-coverage: ## Generate test coverage report (requires cargo-llvm-cov)
	@echo "Generating test coverage..."
	@command -v cargo-llvm-cov >/dev/null 2>&1 || { echo "Installing cargo-llvm-cov..."; cargo install cargo-llvm-cov; }
	cargo llvm-cov --all-features --workspace --html
	@echo "Coverage report generated at target/llvm-cov/html/index.html"

test-coverage-ci: ## Generate coverage for CI (LCOV format)
	@echo "Generating CI coverage report..."
	@command -v cargo-llvm-cov >/dev/null 2>&1 || { echo "Installing cargo-llvm-cov..."; cargo install cargo-llvm-cov; }
	cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
	@echo "Coverage report generated at lcov.info"

test-frontend: ## Run frontend tests
	@echo "Running frontend tests..."
	cd frontend && wasm-pack test --headless --firefox

run: build-frontend ## Build frontend and run backend
	@echo "Starting backend server..."
	cargo run

dev-frontend: ## Run frontend development server with hot-reload
	@echo "Starting frontend dev server at http://127.0.0.1:8000..."
	cd frontend && trunk serve

dev-backend: ## Run backend in development mode
	@echo "Starting backend server at http://localhost:8080..."
	cargo run

clean: ## Clean build artifacts
	@echo "Cleaning build artifacts..."
	cargo clean
	rm -rf frontend/dist
	@echo "Clean complete!"

check: ## Run clippy and format checks
	@echo "Running cargo fmt check..."
	cargo fmt --all -- --check
	@echo "Running clippy..."
	cargo clippy --all-targets --all-features -- -D warnings
	@echo "All checks passed!"

fmt: ## Format all code
	@echo "Formatting code..."
	cargo fmt --all
	@echo "Code formatted!"

setup-hooks: ## Setup git pre-commit hooks
	@echo "Setting up pre-commit hooks..."
	@command -v pre-commit >/dev/null 2>&1 || { echo "Installing pre-commit (requires Python)..."; pip install pre-commit || pip3 install pre-commit; }
	pre-commit install
	@echo "Pre-commit hooks installed! Run 'pre-commit run --all-files' to test."

db-setup: ## Setup test database
	@echo "Setting up test database..."
	psql -U postgres -c "DROP DATABASE IF EXISTS call_for_papers_test;" || true
	psql -U postgres -c "CREATE DATABASE call_for_papers_test;"
	@command -v sqlx >/dev/null 2>&1 || { echo "Installing sqlx-cli..."; cargo install sqlx-cli --no-default-features --features postgres; }
	sqlx migrate run --database-url "postgres://postgres:postgres@localhost/call_for_papers_test"
	@echo "Test database ready!"

webdriver-setup: ## Start WebDriver for E2E tests (geckodriver)
	@echo "Starting geckodriver on port 4444..."
	@command -v geckodriver >/dev/null 2>&1 || { echo "ERROR: geckodriver not found. Install with: brew install geckodriver (macOS) or download from https://github.com/mozilla/geckodriver/releases"; exit 1; }
	@pkill geckodriver || true
	geckodriver --port 4444 &
	@echo "Geckodriver started. Run 'pkill geckodriver' to stop."

ci-local: check test ## Run CI checks locally (format, lint, test)
	@echo "All CI checks passed locally!"
