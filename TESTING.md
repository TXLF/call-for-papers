# Testing Guide

This document describes the testing strategy, setup, and best practices for the Call for Papers system.

## Table of Contents

- [Overview](#overview)
- [Test Structure](#test-structure)
- [Running Tests](#running-tests)
- [Test Database Setup](#test-database-setup)
- [Writing Tests](#writing-tests)
- [Test Coverage](#test-coverage)
- [Continuous Integration](#continuous-integration)
- [Troubleshooting](#troubleshooting)

## Overview

The project uses a comprehensive multi-layered testing approach:

1. **Unit Tests**: Test individual functions and modules in isolation
2. **Integration Tests**: Test HTTP endpoints and database interactions with full application context
3. **End-to-End Tests**: Test complete user workflows through the browser UI with real application and database

### Test Philosophy

- **Test isolation**: Each test gets a clean database state
- **Real dependencies**: Integration tests use actual PostgreSQL database
- **Comprehensive coverage**: Test happy paths, error cases, and edge cases
- **Clear assertions**: Tests should clearly document expected behavior
- **Fast execution**: Tests should run quickly to enable rapid development

## Test Structure

### Directory Layout

```
call-for-papers/
├── src/                      # Application source code
│   ├── handlers/             # HTTP handlers (with inline unit tests)
│   ├── models/               # Database models (with inline unit tests)
│   └── lib.rs                # Library exports
├── tests/                    # Integration and E2E tests
│   ├── common/               # Shared integration test utilities
│   │   └── mod.rs            # TestContext and helper functions
│   ├── e2e/                  # E2E test infrastructure
│   │   └── mod.rs            # E2eContext and browser automation helpers
│   ├── auth_tests.rs         # Authentication integration tests
│   ├── talk_tests.rs         # Talk management integration tests
│   ├── label_rating_tests.rs # Label and rating integration tests
│   ├── schedule_tests.rs     # Schedule management integration tests
│   ├── e2e_speaker_tests.rs  # Speaker workflow E2E tests
│   └── e2e_organizer_tests.rs # Organizer workflow E2E tests
└── Cargo.toml                # Dependencies including test dependencies
```

### Test Categories

#### Unit Tests
Located inline with source code using `#[cfg(test)]` modules:

```rust
// In src/handlers/auth.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        // Test password hashing logic
    }
}
```

#### Integration Tests
Located in the `tests/` directory, testing complete HTTP request/response cycles:

```rust
// In tests/auth_tests.rs
#[tokio::test]
async fn test_register_success() {
    let ctx = TestContext::new().await;
    // Make HTTP request, assert response
    ctx.cleanup().await;
}
```

#### End-to-End Tests
Located in `tests/e2e_*.rs`, testing complete user workflows through the browser:

```rust
// In tests/e2e_speaker_tests.rs
#[tokio::test]
#[ignore] // Requires WebDriver
async fn test_speaker_submit_talk() {
    let ctx = E2eContext::new().await.expect("Failed to create E2E context");

    // Register and login
    ctx.register("speaker@example.com", "speaker", "password", "Speaker").await.expect("Failed to register");

    // Navigate to submit talk page
    ctx.goto("/talks/new").await.expect("Failed to navigate");

    // Fill form and submit
    ctx.fill_input("#title", "My Talk").await.expect("Failed to fill title");
    ctx.fill_input("#short_summary", "Summary").await.expect("Failed to fill summary");
    ctx.click("button[type='submit']").await.expect("Failed to submit");

    // Verify talk appears
    ctx.goto("/talks/mine").await.expect("Failed to navigate");
    let talks = ctx.find_all(".talk-item").await.expect("Failed to find talks");
    assert_eq!(talks.len(), 1);

    ctx.cleanup().await;
}
```

## Running Tests

### Prerequisites

1. **PostgreSQL Database**: Running PostgreSQL instance
2. **Test Database**: Create a test database:
   ```bash
   createdb call_for_papers_test
   ```

3. **Environment Variables**:
   ```bash
   export DATABASE_URL="postgres://postgres:postgres@localhost/call_for_papers_test"
   export TEST_DATABASE_URL="postgres://postgres:postgres@localhost/call_for_papers_test"
   export JWT_SECRET="test_jwt_secret_for_testing"
   ```

#### Additional Prerequisites for E2E Tests

4. **WebDriver**: Install geckodriver (Firefox) or chromedriver (Chrome):
   ```bash
   # macOS
   brew install geckodriver

   # Linux
   wget https://github.com/mozilla/geckodriver/releases/download/v0.34.0/geckodriver-v0.34.0-linux64.tar.gz
   tar -xzf geckodriver-v0.34.0-linux64.tar.gz
   sudo mv geckodriver /usr/local/bin/

   # Or use the Makefile
   make webdriver-setup
   ```

5. **Built Frontend**: E2E tests require a built frontend:
   ```bash
   cd frontend && trunk build --release
   ```

### Running All Tests

```bash
# Run all tests (unit + integration)
cargo test

# Run with output
cargo test -- --nocapture

# Run with verbose output
cargo test --verbose
```

### Running Specific Test Suites

```bash
# Run only unit tests
cargo test --lib

# Run only integration tests (excludes E2E)
cargo test --test '*' -- --skip e2e
# Or use Makefile
make test-integration

# Run only E2E tests (requires WebDriver)
cargo test --test 'e2e_*' -- --ignored --test-threads=1
# Or use Makefile
make test-e2e

# Run specific integration test file
cargo test --test auth_tests

# Run specific test by name
cargo test test_register_success
```

### Running E2E Tests

E2E tests require additional setup and run with the `--ignored` flag:

```bash
# 1. Start WebDriver (in separate terminal)
make webdriver-setup
# Or manually: geckodriver --port 4444

# 2. Setup test database
make db-setup

# 3. Build frontend
cd frontend && trunk build --release && cd ..

# 4. Run E2E tests
make test-e2e

# Or run manually with cargo
cargo test --test 'e2e_*' -- --ignored --test-threads=1
```

**Note**: E2E tests run sequentially (`--test-threads=1`) because they:
- Start and stop the application server
- Use shared browser instances
- Manage database state

### Running Tests in Parallel

By default, Rust runs tests in parallel. For debugging, run sequentially:

```bash
cargo test -- --test-threads=1
```

## Test Database Setup

### Automatic Setup

Integration tests use `TestContext` which automatically:
1. Connects to the test database
2. Runs migrations to set up schema
3. Creates a test application instance
4. Provides helper methods for requests

### Manual Setup

If needed, manually set up the test database:

```bash
# Create database
createdb call_for_papers_test

# Run migrations
sqlx migrate run --database-url "postgres://postgres:postgres@localhost/call_for_papers_test"
```

### Database Cleanup

Tests automatically clean up after themselves by truncating tables:

```rust
ctx.cleanup().await; // Truncates all tables
```

The cleanup order respects foreign key constraints:
1. `talk_labels` (junction table)
2. `ratings`
3. `schedule_slots`
4. `tracks`
5. `conferences`
6. `email_templates`
7. `labels`
8. `talks`
9. `auth_providers`
10. `sessions`
11. `users` (base table)

## Writing Tests

### Using TestContext

The `TestContext` provides test infrastructure:

```rust
use tests::common::*;

#[tokio::test]
async fn my_integration_test() {
    // 1. Create test context
    let ctx = TestContext::new().await;

    // 2. Set up test data
    let user_id = create_test_user(
        &ctx.db,
        "user@example.com",
        "username",
        "password",
        "Full Name",
        false, // is_organizer
    ).await;

    // 3. Make HTTP request
    let req = Request::builder()
        .method("GET")
        .uri("/api/some-endpoint")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let (status, response) = ctx.request(req).await;

    // 4. Assert results
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["field"], "expected_value");

    // 5. Cleanup
    ctx.cleanup().await;
}
```

### Helper Functions

The `tests/common/mod.rs` module provides helpers:

#### Creating Test Users

```rust
let user_id = create_test_user(
    &ctx.db,
    "user@example.com",
    "username",
    "password",
    "Full Name",
    false, // is_organizer
).await;
```

#### Generating JWT Tokens

```rust
let token = generate_test_token(
    user_id,
    "user@example.com",
    false, // is_organizer
);
```

#### Creating Test Data

```rust
// Create label
let label_id = create_test_label(&ctx.db, "Rust", "#FF6B35").await;

// Create conference
let conference_id = create_test_conference(&ctx.db, "TXLF 2025").await;

// Create track
let track_id = create_test_track(&ctx.db, conference_id, "Main Hall").await;

// Create talk
let talk_id = create_test_talk(
    &ctx.db,
    speaker_id,
    "Talk Title",
    "Short summary"
).await;
```

### Test Patterns

#### Testing Successful Operations

```rust
#[tokio::test]
async fn test_operation_success() {
    let ctx = TestContext::new().await;

    // Setup
    let user_id = create_test_user(...).await;
    let token = generate_test_token(...);

    // Execute
    let req = Request::builder()
        .method("POST")
        .uri("/api/endpoint")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(json!({...}).to_string()))
        .unwrap();

    let (status, response) = ctx.request(req).await;

    // Assert
    assert_eq!(status, StatusCode::CREATED);
    assert!(response["id"].is_string());

    ctx.cleanup().await;
}
```

#### Testing Authentication

```rust
#[tokio::test]
async fn test_requires_authentication() {
    let ctx = TestContext::new().await;

    // Request without token
    let req = Request::builder()
        .method("GET")
        .uri("/api/protected-endpoint")
        .body(Body::empty())
        .unwrap();

    let (status, _) = ctx.request(req).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);

    ctx.cleanup().await;
}
```

#### Testing Authorization

```rust
#[tokio::test]
async fn test_requires_organizer_role() {
    let ctx = TestContext::new().await;

    // Regular user (not organizer)
    let user_id = create_test_user(..., false).await;
    let token = generate_test_token(user_id, ..., false);

    let req = Request::builder()
        .method("GET")
        .uri("/api/organizer-only-endpoint")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let (status, _) = ctx.request(req).await;

    assert_eq!(status, StatusCode::FORBIDDEN);

    ctx.cleanup().await;
}
```

#### Testing Error Cases

```rust
#[tokio::test]
async fn test_validation_error() {
    let ctx = TestContext::new().await;

    let user_id = create_test_user(...).await;
    let token = generate_test_token(...);

    // Missing required field
    let req = Request::builder()
        .method("POST")
        .uri("/api/endpoint")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(json!({"partial": "data"}).to_string()))
        .unwrap();

    let (status, response) = ctx.request(req).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response["error"].is_string());

    ctx.cleanup().await;
}
```

## Test Coverage

### Current Coverage

The integration test suite covers:

#### Authentication (`tests/auth_tests.rs`) - 11 tests
- ✅ User registration (success, duplicate email, invalid email)
- ✅ User login (success, wrong password, nonexistent user)
- ✅ Protected routes (with/without token, invalid token)
- ✅ Role-based access control (organizer vs regular user)

#### Talk Management (`tests/talk_tests.rs`) - 18 tests
- ✅ Create talk (success, without auth, missing fields)
- ✅ List talks (mine, all as organizer, permission check)
- ✅ Get talk by ID
- ✅ Update talk (success, unauthorized update)
- ✅ Delete talk
- ✅ Change talk state (as organizer, permission check)
- ✅ Respond to talk (accept, reject, wrong state)

#### Labels & Ratings (`tests/label_rating_tests.rs`) - 16 tests
- ✅ Label CRUD operations (create, list, update, delete)
- ✅ Label permissions (organizer required)
- ✅ Add/remove labels to talks
- ✅ Rating CRUD operations (create, update, delete)
- ✅ Rating permissions (organizer required, only own ratings)
- ✅ List ratings with average score

#### Schedule Management (`tests/schedule_tests.rs`) - 16 tests
- ✅ Conference CRUD operations
- ✅ Track CRUD operations
- ✅ Schedule slot management
- ✅ Assign/unassign talks to slots
- ✅ Get conference schedule
- ✅ Permission checks for all operations

#### End-to-End Tests - Speaker Workflows (`tests/e2e_speaker_tests.rs`) - 6 tests
- ✅ Registration and login flow
- ✅ Submit new talk through UI
- ✅ Edit existing talk
- ✅ Delete talk
- ✅ View talk status
- ✅ Respond to talk acceptance

#### End-to-End Tests - Organizer Workflows (`tests/e2e_organizer_tests.rs`) - 6 tests
- ✅ Organizer login and dashboard access
- ✅ View all submitted talks
- ✅ Rate talks
- ✅ Add labels to talks
- ✅ Change talk state (accept/reject)
- ✅ Create conference schedule

### Coverage Statistics

```bash
# Generate HTML coverage report locally
make test-coverage
# Opens: target/llvm-cov/html/index.html

# Generate LCOV format for CI
make test-coverage-ci
# Creates: lcov.info
```

## Test Automation

### Makefile Targets

The project includes comprehensive Makefile targets for test automation:

```bash
# Run all tests
make test

# Run only unit tests (fast, no database required)
make test-unit

# Run only integration tests (requires PostgreSQL)
make test-integration

# Run tests in watch mode (auto-rerun on changes)
make test-watch

# Generate test coverage report
make test-coverage

# Setup test database
make db-setup

# Setup pre-commit hooks
make setup-hooks

# Run all CI checks locally
make ci-local
```

### Pre-commit Hooks

Pre-commit hooks automatically run checks before each commit to catch issues early.

#### Setup

```bash
# Install pre-commit hooks
make setup-hooks

# Or manually:
pip install pre-commit
pre-commit install
```

#### What Gets Checked

The pre-commit hooks run:
1. **File checks**: Trailing whitespace, end-of-file, large files, merge conflicts
2. **Format check**: `cargo fmt --check`
3. **Linting**: `cargo clippy --all-targets --all-features`
4. **Unit tests**: `cargo test --lib`
5. **YAML/Markdown linting**: For configuration and documentation files

#### Manual Runs

```bash
# Run hooks on all files
pre-commit run --all-files

# Run specific hook
pre-commit run cargo-fmt --all-files

# Skip hooks for a commit (not recommended)
git commit --no-verify
```

#### Configuration

Hooks are configured in `.pre-commit-config.yaml`. Update versions with:

```bash
pre-commit autoupdate
```

### Continuous Integration

### GitHub Actions Workflows

The project uses multiple CI workflows for comprehensive testing:

#### CI Workflow (`.github/workflows/ci.yml`)

Runs on every push and PR:

1. **Code Quality Checks**:
   - Format checking: `cargo fmt --check`
   - Linting: `cargo clippy --all-targets --all-features`

2. **Database Setup**:
   - PostgreSQL 16 service container
   - Run migrations with `sqlx migrate run`

3. **Test Execution**:
   - Unit tests: `cargo test --lib`
   - Integration tests: `cargo test --test '*'`

4. **Test Artifacts**:
   - Upload test binaries for debugging (7-day retention)

#### Coverage Workflow (`.github/workflows/coverage.yml`)

Generates and reports code coverage:

1. **Setup**:
   - PostgreSQL database with migrations
   - Install `cargo-llvm-cov`

2. **Coverage Generation**:
   - Run all tests with coverage instrumentation
   - Generate LCOV and summary reports

3. **Coverage Reporting**:
   - Upload to Codecov
   - Upload to Coveralls
   - Comment coverage summary on PRs

4. **Artifacts**:
   - Coverage reports (30-day retention)
   - Coverage summary text file

### CI Environment

The CI uses:
- PostgreSQL 16 Alpine
- Database: `call_for_papers_test`
- Credentials: `postgres:postgres`
- Port: `5432`

Environment variables set:
```yaml
DATABASE_URL: postgres://postgres:postgres@localhost:5432/call_for_papers_test
TEST_DATABASE_URL: postgres://postgres:postgres@localhost:5432/call_for_papers_test
JWT_SECRET: test_jwt_secret_for_ci_pipeline_testing_only
```

## Troubleshooting

### Test Database Connection Issues

**Problem**: Tests fail with "connection refused"

**Solution**:
```bash
# Check PostgreSQL is running
pg_isready

# Verify database exists
psql -l | grep call_for_papers_test

# Create if missing
createdb call_for_papers_test
```

### Migration Errors

**Problem**: "relation does not exist" errors

**Solution**:
```bash
# Run migrations manually
sqlx migrate run --database-url "postgres://postgres:postgres@localhost/call_for_papers_test"

# Or drop and recreate database
dropdb call_for_papers_test
createdb call_for_papers_test
sqlx migrate run --database-url "postgres://postgres:postgres@localhost/call_for_papers_test"
```

### SQLX Offline Mode Issues

**Problem**: "cached query data" errors

**Solution**:
```bash
# Prepare query metadata
cargo sqlx prepare --database-url "postgres://postgres:postgres@localhost/call_for_papers"

# For tests, don't use SQLX_OFFLINE mode
unset SQLX_OFFLINE
```

### Tests Hanging

**Problem**: Tests hang indefinitely

**Solution**:
1. Check for deadlocks in database operations
2. Ensure all tests call `ctx.cleanup().await`
3. Run with timeout:
   ```bash
   timeout 60 cargo test
   ```

### Flaky Tests

**Problem**: Tests pass/fail inconsistently

**Solution**:
1. Ensure proper test isolation with `ctx.cleanup()`
2. Check for race conditions
3. Run sequentially to debug:
   ```bash
   cargo test -- --test-threads=1
   ```

### Port Already in Use

**Problem**: "address already in use" errors

**Solution**:
```bash
# Find process using port
lsof -i :8080

# Kill process
kill -9 <PID>
```

## Best Practices

### Do's ✅

- **Always cleanup**: Call `ctx.cleanup().await` at end of tests
- **Test both success and failure**: Cover happy paths and error cases
- **Test permissions**: Verify authentication and authorization
- **Use descriptive names**: `test_create_talk_success` vs `test1`
- **Keep tests focused**: One concept per test
- **Use helpers**: Leverage `create_test_*` functions
- **Assert meaningful values**: Check specific fields, not just status codes

### Don'ts ❌

- **Don't share state**: Tests should be independent
- **Don't test implementation**: Test behavior, not internals
- **Don't skip cleanup**: Always truncate tables after tests
- **Don't use production data**: Always use test database
- **Don't ignore warnings**: Fix `cargo clippy` warnings in tests
- **Don't test external services**: Mock external dependencies

## Future Improvements

- [ ] Add end-to-end tests with frontend
- [ ] Add performance/load tests
- [ ] Add mutation testing
- [ ] Increase code coverage to 80%+
- [ ] Add API contract tests
- [ ] Add security penetration tests
- [ ] Add stress tests for concurrent operations

## Resources

- [Rust Testing Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tokio Testing Guide](https://tokio.rs/tokio/topics/testing)
- [Axum Testing Examples](https://github.com/tokio-rs/axum/tree/main/examples)
- [SQLx Testing Guide](https://github.com/launchbadge/sqlx#testing)
