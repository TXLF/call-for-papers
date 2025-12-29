# Call for Papers - Just Commands
# https://github.com/casey/just

# Default recipe to display help information
default:
    @just --list

# Setup development environment
setup:
    @echo "Setting up development environment..."
    rustup target add wasm32-unknown-unknown
    cargo install trunk --version 0.18.8
    cargo install sqlx-cli --no-default-features --features postgres
    cp .env.example .env
    @echo "Setup complete! Edit .env with your configuration."

# Start PostgreSQL using podman-compose
db-start:
    podman-compose up -d postgres

# Stop PostgreSQL
db-stop:
    podman-compose down

# View database logs
db-logs:
    podman-compose logs -f postgres

# Run database migrations
migrate:
    sqlx migrate run

# Create a new migration
migrate-create name:
    sqlx migrate add {{name}}

# Revert last migration
migrate-revert:
    sqlx migrate revert

# Build backend (debug)
build:
    cargo build

# Build backend (release)
build-release:
    cargo build --release

# Build frontend (debug)
build-frontend:
    cd frontend && trunk build

# Build frontend (release)
build-frontend-release:
    cd frontend && trunk build --release

# Build both backend and frontend (release)
build-all: build-frontend-release build-release

# Run backend server
run:
    cargo run

# Run backend with watch (auto-reload on changes)
watch:
    cargo watch -x run

# Run frontend dev server
run-frontend:
    cd frontend && trunk serve

# Run frontend dev server on specific port
run-frontend-port port:
    cd frontend && trunk serve --port {{port}}

# Run both backend and frontend in development mode
dev:
    @echo "Start backend in one terminal: just run"
    @echo "Start frontend in another terminal: just run-frontend"

# Run tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Run frontend tests
test-frontend:
    cd frontend && cargo test --target wasm32-unknown-unknown

# Run all tests (backend + frontend)
test-all: test test-frontend

# Format code
fmt:
    cargo fmt --all

# Check code formatting
fmt-check:
    cargo fmt --all -- --check

# Run clippy linter
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Run security audit
audit:
    cargo audit

# Clean build artifacts
clean:
    cargo clean
    cd frontend && trunk clean

# Full check (format, lint, test)
check: fmt-check lint test

# Build Docker image
docker-build:
    podman build -t call-for-papers:latest .

# Build Docker image with cache
docker-build-cached:
    podman build --cache-from call-for-papers:latest -t call-for-papers:latest .

# Run Docker container (requires PostgreSQL)
docker-run:
    podman run -d \
        --name cfp-app \
        -p 8080:8080 \
        -e DATABASE_URL=postgres://postgres:postgres@host.containers.internal:5432/call_for_papers \
        call-for-papers:latest

# Stop Docker container
docker-stop:
    podman stop cfp-app
    podman rm cfp-app

# View Docker logs
docker-logs:
    podman logs -f cfp-app

# Start full stack with Docker Compose
up:
    podman-compose up -d

# Stop full stack
down:
    podman-compose down

# Restart full stack
restart:
    podman-compose restart

# View all logs
logs:
    podman-compose logs -f

# Create uploads directory
create-uploads:
    mkdir -p uploads

# Initialize project (setup + db + migrations + uploads)
init: setup create-uploads db-start
    @echo "Waiting for PostgreSQL to be ready..."
    @sleep 5
    just migrate
    @echo "Initialization complete! Run 'just dev' to start developing."

# Production build check
prod-check: clean check build-all
    @echo "Production build successful!"

# Show project status
status:
    @echo "=== Git Status ==="
    @git status --short
    @echo ""
    @echo "=== Docker Containers ==="
    @podman-compose ps
    @echo ""
    @echo "=== Build Artifacts ==="
    @ls -lh target/release/call-for-papers 2>/dev/null || echo "No release binary"
    @ls -lh frontend/dist/index.html 2>/dev/null || echo "No frontend build"
