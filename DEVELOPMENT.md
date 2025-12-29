# Development Guide

This guide explains how to set up and run the Call for Papers application for development.

## Quick Start

For the impatient developer:

```bash
# 1. Install prerequisites
rustup target add wasm32-unknown-unknown
cargo install trunk sqlx-cli --no-default-features --features postgres

# 2. Start database (choose one):
podman-compose up -d              # Using containers (recommended)
# OR
brew services start postgresql@15 # macOS native
# OR
sudo systemctl start postgresql   # Linux native

# 3. Create .env file
cat > .env << 'EOF'
DATABASE_URL=postgres://postgres:postgres@localhost/call_for_papers
JWT_SECRET=dev-secret-change-in-production
RUST_LOG=info,call_for_papers=debug
EOF

# 4. Create database
createdb call_for_papers

# 5. Start development servers (two terminals):
# Terminal 1:
cd frontend && trunk serve

# Terminal 2:
cargo run

# 6. Open http://127.0.0.1:8000
```

That's it! Read below for detailed explanations and troubleshooting.

## Prerequisites

### Backend
- Rust toolchain (1.70 or later)
- PostgreSQL (12 or later) OR Podman/Docker

### Frontend
- Rust toolchain (1.70 or later)
- `wasm32-unknown-unknown` target
- Trunk build tool

### Optional but Recommended
- SQLx CLI (`cargo install sqlx-cli --no-default-features --features postgres`) - For database migrations
- cargo-watch (`cargo install cargo-watch`) - Auto-rebuild on file changes
- beads-cli (`cargo install beads-cli`) - Project issue tracking

## Development Environment Options

You have two options for setting up your development environment:

### Option A: Podman Compose (Recommended for Quick Setup)

Use podman compose to run PostgreSQL in a container while developing the Rust backend and frontend on your host machine. This provides the fastest development experience with hot-reloading.

**Prerequisites:**
- Podman or Docker installed

**Setup:**
```bash
# Start PostgreSQL
podman-compose up -d

# Or with docker-compose
docker-compose up -d

# Verify PostgreSQL is running
podman-compose ps
```

This starts a PostgreSQL container on `localhost:5432` with:
- Database: `call_for_papers`
- Username: `postgres`
- Password: `postgres`

The database data persists in a named volume, so stopping the container won't delete your data.

**Useful commands:**
```bash
# View logs
podman-compose logs -f postgres

# Stop PostgreSQL
podman-compose down

# Stop and remove data (WARNING: deletes database)
podman-compose down -v

# Restart PostgreSQL
podman-compose restart postgres
```

**Optional: pgAdmin**

To enable pgAdmin for database management, uncomment the `pgadmin` service in `compose.yaml` and restart:
```bash
podman-compose up -d
```

Access pgAdmin at http://localhost:5050 with:
- Email: admin@cfp.local
- Password: admin

### Option B: Native PostgreSQL Installation

Install PostgreSQL directly on your system. See the "Setup PostgreSQL" section below.

## Initial Setup

### 1. Install Rust

If you haven't already:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Install Frontend Tools

```bash
# Install WASM target
rustup target add wasm32-unknown-unknown

# Install Trunk
cargo install trunk
```

### 3. Setup PostgreSQL

Make sure PostgreSQL is installed and running:
```bash
# macOS
brew install postgresql
brew services start postgresql

# Ubuntu/Debian
sudo apt-get install postgresql
sudo systemctl start postgresql
```

### 4. Configure Environment

Create a `.env` file in the project root:

```bash
cat > .env << 'EOF'
# Database
DATABASE_URL=postgres://postgres:postgres@localhost/call_for_papers

# Server
HOST=127.0.0.1
PORT=8080

# JWT Secret (generate a random string for production!)
JWT_SECRET=your-secret-key-change-this-in-production

# Log level
RUST_LOG=info,call_for_papers=debug

# Frontend URL (for CORS in development)
FRONTEND_URL=http://127.0.0.1:8000
EOF
```

**Environment Variables Reference:**

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `DATABASE_URL` | PostgreSQL connection string | - | Yes |
| `HOST` | Server bind address | `127.0.0.1` | No |
| `PORT` | Server port | `8080` | No |
| `JWT_SECRET` | Secret key for JWT authentication tokens | - | Yes |
| `RUST_LOG` | Logging level (trace, debug, info, warn, error) | `info` | No |
| `FRONTEND_URL` | Frontend URL for CORS (dev only) | - | No |

**Generate a secure JWT secret:**
```bash
# Linux/macOS
openssl rand -base64 32

# Or use any random string generator
```

## Running the Application

### Option 1: Full Stack (Recommended for Production Testing)

This runs the backend and serves the pre-built frontend:

1. **Build the frontend:**
   ```bash
   cd frontend
   trunk build --release
   cd ..
   ```

2. **Run the backend:**
   ```bash
   cargo run
   ```

3. **Access the app:**
   - Frontend: http://localhost:8080
   - API Health: http://localhost:8080/api/health
   - DB Health: http://localhost:8080/api/health/db

### Option 2: Separate Development Servers (Recommended for Development)

This provides hot-reloading for both frontend and backend:

1. **Terminal 1 - Run the backend:**
   ```bash
   cargo run
   ```
   Backend runs on http://localhost:8080

2. **Terminal 2 - Run the frontend dev server:**
   ```bash
   cd frontend
   trunk serve
   ```
   Frontend runs on http://127.0.0.1:8000

   **Note:** In this mode, API calls from the frontend will go to `http://localhost:8080/api/*`

## Project Structure

```
call-for-papers/
├── src/                    # Backend Rust code
│   ├── main.rs            # Entry point
│   ├── api/               # API routes
│   ├── db/                # Database connection
│   ├── models/            # Data models
│   └── handlers/          # Request handlers
├── migrations/            # Database migrations
├── frontend/              # Frontend Yew.rs app
│   ├── src/
│   ├── public/
│   ├── index.html
│   └── Trunk.toml
├── Cargo.toml             # Backend dependencies
└── .env                   # Environment configuration
```

## Development Workflow

### Making Backend Changes

1. Edit files in `src/`
2. The backend will need to be restarted manually
3. Run `cargo build` to check for errors
4. Test with `cargo run`

### Making Frontend Changes

When using `trunk serve`:
- Changes auto-reload in the browser
- Check the terminal for build errors
- Styles can be edited in `frontend/public/styles.css`

### Database Changes

1. Create a new migration:
   ```bash
   sqlx migrate add <migration_name>
   ```

2. Edit the migration file in `migrations/`

3. Migrations run automatically on backend startup

4. To manually run migrations:
   ```bash
   sqlx migrate run --database-url "postgres://postgres:postgres@localhost/call_for_papers"
   ```

5. Revert the last migration:
   ```bash
   sqlx migrate revert --database-url "postgres://postgres:postgres@localhost/call_for_papers"
   ```

6. Check migration status:
   ```bash
   sqlx migrate info
   ```

**Note:** See [migrations/README.md](migrations/README.md) for detailed schema documentation.

### Issue Tracking with Beads

This project uses `bd` (beads) for issue tracking. Common workflow:

```bash
# View available work (tasks with no blockers)
bd ready

# Show details for a specific issue
bd show cfp-xxx

# Claim an issue to work on it
bd update cfp-xxx --status=in_progress

# Close completed work
bd close cfp-xxx --reason "Implemented feature X"

# View all open issues
bd list --status=open

# View project statistics
bd stats

# Sync changes with git remote
bd sync
```

**Workflow tips:**
- Run `bd prime` to see the full workflow guide
- Issues are tracked in `.beads/issues.jsonl`
- Always run `bd sync` at the end of your session
- Use `bd create` to report new issues

## Building for Production

### Option 1: Native Build

#### 1. Build Frontend
```bash
cd frontend
trunk build --release
cd ..
```

#### 2. Build Backend
```bash
cargo build --release
```

The release binary will be in `target/release/call-for-papers`

#### 3. Deploy
- Copy the binary
- Copy the `migrations/` directory
- Copy the `frontend/dist/` directory
- Set environment variables
- Run the binary

### Option 2: Container Build

Build and run the entire application in containers using the provided Dockerfile:

```bash
# Build the container image
podman build -t call-for-papers:latest .

# Run with existing PostgreSQL from compose.yaml
podman run -d \
  --name cfp-app \
  -p 8080:8080 \
  -e DATABASE_URL=postgres://postgres:postgres@host.containers.internal:5432/call_for_papers \
  call-for-papers:latest

# Or add to compose.yaml for a complete containerized setup
```

To add the backend to `compose.yaml`, you can uncomment or add:
```yaml
  backend:
    build: .
    container_name: cfp-backend
    environment:
      DATABASE_URL: postgres://postgres:postgres@postgres:5432/call_for_papers
      SERVER_HOST: 0.0.0.0
      SERVER_PORT: 8080
    ports:
      - "8080:8080"
    depends_on:
      postgres:
        condition: service_healthy
    restart: unless-stopped
```

## Testing

### Backend Tests
```bash
cargo test
```

### Frontend Tests
```bash
cd frontend
cargo test --target wasm32-unknown-unknown
```

## Troubleshooting

### Database Connection Errors

**Issue:** `connection refused` or `database "call_for_papers" does not exist`

**Solutions:**
```bash
# 1. Check if PostgreSQL is running
pg_isready

# 2. Start PostgreSQL if not running
# macOS (Homebrew)
brew services start postgresql@15

# Linux (systemd)
sudo systemctl start postgresql

# With Podman/Docker
podman-compose up -d

# 3. Verify database exists
psql -l | grep call_for_papers

# 4. Create database if needed
psql -c "CREATE DATABASE call_for_papers;"

# 5. Test connection
psql postgres://postgres:postgres@localhost/call_for_papers -c "SELECT 1;"
```

### Migration Errors

**Issue:** `migration has already been applied` or migration conflicts

**Solutions:**
```bash
# Check current migration status
sqlx migrate info

# If migrations are out of sync, reset database (WARNING: destroys data)
psql -c "DROP DATABASE call_for_papers;"
psql -c "CREATE DATABASE call_for_papers;"
sqlx migrate run
```

### Frontend Build Errors

**Issue:** `error: target wasm32-unknown-unknown not found`

**Solution:**
```bash
rustup target add wasm32-unknown-unknown
```

**Issue:** `trunk: command not found`

**Solution:**
```bash
cargo install trunk
# Add cargo bin to PATH if needed
export PATH="$HOME/.cargo/bin:$PATH"
```

**Issue:** Build errors after pulling changes

**Solution:**
```bash
# Clean and rebuild
cd frontend
trunk clean
rm -rf dist target
trunk build
```

### SQLx Compile-Time Verification Errors

**Issue:** `DATABASE_URL not found` during compilation

**Solutions:**
```bash
# Option 1: Set environment variable
export DATABASE_URL="postgres://postgres:postgres@localhost/call_for_papers"

# Option 2: Use offline mode (requires prepared sqlx-data.json)
cargo sqlx prepare
```

### Port Already in Use

**Issue:** `address already in use` error

**Solutions:**
```bash
# Find process using port 8080
lsof -i :8080

# Kill the process
kill -9 <PID>

# Or change the port
# Backend: Edit PORT in .env
# Frontend: trunk serve --port 8001
```

### CORS Issues in Development

**Issue:** `CORS policy: No 'Access-Control-Allow-Origin' header`

**Solutions:**
- Always access frontend via `trunk serve` (http://127.0.0.1:8000), not backend directly
- Make sure `FRONTEND_URL` is set in `.env`
- The backend automatically configures CORS for the frontend URL

### First User Setup

After starting the application, you'll need an organizer account:

```bash
# 1. Sign up via the web interface at http://127.0.0.1:8000/signup
# 2. Promote your user to organizer in the database:

psql postgres://postgres:postgres@localhost/call_for_papers

# In psql:
UPDATE users SET is_organizer = true WHERE email = 'your@email.com';
\q
```

### Complete Environment Reset

If things are completely broken, reset everything:

```bash
# 1. Stop all running services
pkill -f "trunk serve"
pkill -f "cargo run"

# 2. Clean build artifacts
cargo clean
cd frontend && trunk clean && cd ..
rm -rf frontend/dist frontend/target

# 3. Reset database
psql -c "DROP DATABASE IF EXISTS call_for_papers;"
psql -c "CREATE DATABASE call_for_papers;"

# 4. Rebuild from scratch
cargo build
cd frontend && trunk build && cd ..
cargo run
```

## Useful Commands

```bash
# Backend
cargo check              # Quick compile check
cargo build             # Build debug
cargo build --release   # Build optimized
cargo run               # Run backend
cargo test              # Run tests

# Frontend
cd frontend
trunk serve             # Dev server with hot reload
trunk build             # Build debug
trunk build --release   # Build optimized
trunk clean             # Clean build artifacts

# Database
sqlx migrate run        # Run migrations
sqlx migrate revert     # Revert last migration
sqlx migrate info       # Show migration status
```

## IDE Setup

### VS Code
Recommended extensions:
- rust-analyzer
- Even Better TOML
- crates

### IntelliJ/CLion
- Rust plugin
- Database Tools and SQL plugin

## Additional Resources

- [Yew Documentation](https://yew.rs/)
- [Axum Documentation](https://docs.rs/axum/)
- [SQLx Documentation](https://docs.rs/sqlx/)
- [Trunk Documentation](https://trunkrs.dev/)
