# Development Guide

This guide explains how to set up and run the Call for Papers application for development.

## Prerequisites

### Backend
- Rust toolchain (1.70 or later)
- PostgreSQL (12 or later)

### Frontend
- Rust toolchain (1.70 or later)
- `wasm32-unknown-unknown` target
- Trunk build tool

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

```bash
cp .env.example .env
```

Edit `.env` to set your database credentials if needed. Default:
```
DATABASE_URL=postgres://postgres:postgres@localhost/call_for_papers
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
   sqlx migrate run
   ```

## Building for Production

### 1. Build Frontend
```bash
cd frontend
trunk build --release
cd ..
```

### 2. Build Backend
```bash
cargo build --release
```

The release binary will be in `target/release/call-for-papers`

### 3. Deploy
- Copy the binary
- Copy the `migrations/` directory
- Copy the `frontend/dist/` directory
- Set environment variables
- Run the binary

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
- Ensure PostgreSQL is running: `pg_isready`
- Check your DATABASE_URL in `.env`
- Verify the database exists or let the app create it

### Frontend Build Errors
- Ensure `wasm32-unknown-unknown` target is installed
- Ensure Trunk is installed and up to date
- Clear cache: `rm -rf frontend/dist frontend/target`

### Port Already in Use
- Backend (8080): Change `SERVER_PORT` in `.env`
- Frontend dev server (8000): Use `trunk serve --port 8001`

### CORS Issues
- When developing, use `trunk serve` for the frontend
- Or build the frontend and run through the backend

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
