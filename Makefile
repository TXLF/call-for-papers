.PHONY: help install build build-frontend build-backend test run clean dev-frontend

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

test: ## Run all tests
	@echo "Running backend tests..."
	cargo test
	@echo "All tests passed!"

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
