# Database Setup Guide

This guide explains how the database migration system works in the Call for Papers application.

## Prerequisites

- PostgreSQL 12 or higher installed and running
- A PostgreSQL user with database creation privileges

## Quick Start

1. **Copy the environment template:**
   ```bash
   cp .env.example .env
   ```

2. **Configure your database URL in `.env`:**
   ```bash
   DATABASE_URL=postgres://username:password@localhost/call_for_papers
   ```

3. **Run the application:**
   ```bash
   cargo run
   ```

The application will:
- Automatically create the database if it doesn't exist
- Run all pending migrations
- Start the web server

## How It Works

### Automatic Migration System

The application uses SQLx's built-in migration system with automatic execution:

1. **On startup**, the application checks if the database exists
2. If not, it **creates the database**
3. It then **runs all pending migrations** from the `migrations/` directory
4. Finally, it **starts the web server**

### Migration Files

Migrations are located in `migrations/` and are executed in order:

- `20241224000001_create_users_and_auth.sql` - User authentication system
- `20241224000002_create_talks_and_labels.sql` - Talk submission and ratings
- `20241224000003_create_schedule.sql` - Conference scheduling
- `20241224000004_create_email_templates.sql` - Email system

## Health Checks

The application provides two health check endpoints:

- **`GET /health`** - Basic application health check
- **`GET /health/db`** - Database connectivity check

Example:
```bash
curl http://localhost:8080/health
curl http://localhost:8080/health/db
```

## Manual Migration Management

If you need to manage migrations manually, you can use the SQLx CLI:

### Install SQLx CLI
```bash
cargo install sqlx-cli --no-default-features --features postgres
```

### Run Migrations
```bash
sqlx migrate run
```

### Create New Migration
```bash
sqlx migrate add <migration_name>
```

### Revert Last Migration
```bash
sqlx migrate revert
```

### Check Migration Status
```bash
sqlx migrate info
```

## Troubleshooting

### Database Connection Errors

If you see connection errors, verify:
1. PostgreSQL is running: `pg_isready`
2. Your credentials are correct in `.env`
3. The PostgreSQL user has database creation privileges

### Migration Errors

If migrations fail:
1. Check the application logs for detailed error messages
2. Verify your database user has the necessary privileges
3. Ensure no manual schema changes conflict with migrations
4. You can manually inspect the `_sqlx_migrations` table to see which migrations have run

### Reset Database

To start fresh (WARNING: This deletes all data):
```bash
dropdb call_for_papers
cargo run  # Will recreate and migrate
```

## Database Schema

For detailed information about the database schema, see [migrations/README.md](migrations/README.md).

## Development Workflow

1. Make sure PostgreSQL is running
2. Configure your `.env` file
3. Run `cargo run` - migrations run automatically
4. Develop your features
5. Create new migrations as needed with `sqlx migrate add`
6. Commit migration files to version control

## Production Deployment

In production:
1. Set `DATABASE_URL` environment variable
2. Ensure the PostgreSQL user has appropriate privileges
3. The application will handle database creation and migrations automatically on first startup
4. Subsequent deployments will only run new migrations

## Security Notes

- Never commit `.env` files to version control
- Use strong passwords for PostgreSQL users
- In production, use connection pooling (already configured with max 5 connections)
- Consider using SSL for database connections in production by modifying the `DATABASE_URL`
