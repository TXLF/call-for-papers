# Frontend - Call for Papers

This is the frontend application for the Call for Papers system, built with Yew.rs (Rust WebAssembly framework).

## Prerequisites

- Rust toolchain (1.70 or later)
- `wasm32-unknown-unknown` target
- Trunk (build tool for Rust WASM applications)

## Setup

### 1. Install the WASM target

```bash
rustup target add wasm32-unknown-unknown
```

### 2. Install Trunk

```bash
cargo install trunk
```

## Development

### Run the development server

From the `frontend` directory:

```bash
trunk serve
```

This will:
- Build the frontend
- Watch for file changes
- Serve the application at `http://127.0.0.1:8000`
- Auto-reload on changes

### Run with the backend

For full-stack development, you'll need to run both the backend and build the frontend:

1. **Build the frontend for production:**
   ```bash
   cd frontend
   trunk build --release
   ```

2. **Run the backend** (from the project root):
   ```bash
   cargo run
   ```

3. **Access the application** at `http://localhost:8080`

The backend automatically serves the frontend if the `frontend/dist` directory exists.

## Building

### Development build

```bash
trunk build
```

### Production build

```bash
trunk build --release
```

This creates optimized WASM bundles in `frontend/dist/`.

## Project Structure

```
frontend/
├── src/
│   ├── main.rs           # Entry point
│   ├── app.rs            # Main app component with routing
│   ├── components/       # Reusable UI components
│   └── pages/            # Page components
│       ├── home.rs
│       └── not_found.rs
├── public/
│   └── styles.css        # Global styles
├── index.html            # HTML template
├── Trunk.toml            # Trunk configuration
└── Cargo.toml            # Rust dependencies

```

## Technologies

- **Yew** - React-like framework for Rust/WASM
- **Yew Router** - Client-side routing
- **gloo-net** - HTTP client for WASM
- **Trunk** - Build tool and dev server

## API Integration

The frontend communicates with the backend API at `/api/*` endpoints:

- `GET /api/health` - API health check
- `GET /api/health/db` - Database health check

All API requests should be prefixed with `/api` when the frontend is served by the backend.

## Adding New Pages

1. Create a new file in `src/pages/`
2. Add the route to `src/app.rs` in the `Route` enum
3. Add the route to the `switch` function
4. Export the page module in `src/pages/mod.rs`

Example:
```rust
// src/pages/talks.rs
use yew::prelude::*;

#[function_component(Talks)]
pub fn talks() -> Html {
    html! {
        <div class="card">
            <h2>{ "Talks" }</h2>
        </div>
    }
}
```

## Styling

Global styles are in `public/styles.css`. Component-specific styles can be added inline or in separate CSS files.

## Troubleshooting

### Build fails with "wasm32-unknown-unknown not found"
Install the target:
```bash
rustup target add wasm32-unknown-unknown
```

### Trunk command not found
Install Trunk:
```bash
cargo install trunk
```

### Changes not reflecting
- Make sure you're running `trunk serve` for development
- Clear browser cache if needed
- Check the terminal for build errors

### CORS errors when developing
- Use `trunk serve` to run the frontend independently
- Or build with `trunk build` and run through the backend at `http://localhost:8080`

## Production Deployment

1. Build the frontend:
   ```bash
   cd frontend
   trunk build --release
   ```

2. The backend automatically serves the frontend from `frontend/dist/`

3. Deploy the entire application (both backend and frontend dist) together

## License

MIT
