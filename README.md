# Call for Papers (CFP) System

An open source call for papers management system designed for open source conferences. Built with Rust and Yew.rs, this self-hostable solution provides a complete workflow from talk submission to conference scheduling.

## Overview

This CFP system enables conference organizers to efficiently manage talk submissions, collaborate on selections, and build conference schedules while providing speakers with a streamlined submission experience.

## Key Features

### For Speakers

- **Easy Talk Submission**: Submit proposals with title, summary, and optional detailed description
- **Flexible File Uploads**: Add slides initially or return to upload them later
- **Tag Your Talks**: Select relevant labels and tags for categorization
- **Track Your Status**: View submission status (submitted, pending, accepted, rejected)
- **Respond to Acceptances**: Accept or decline talk acceptances through the interface
- **Multiple Auth Options**: Sign in with username/password or social logins (GitHub, Google, Apple, Facebook)

### For Organizers

- **Review System**: Read and rate all submitted talks
- **Collaborative Rating**: View aggregated ratings from all organizers
- **Label Management**: Add, remove, or modify labels on any talk
- **AI-Powered Tagging**: Export submissions for auto-tagging via OpenAI or Anthropic Claude APIs
- **State Management**: Move talks through workflow states (submitted → pending → accepted/rejected)
- **Schedule Builder**:
  - Define conference tracks (rooms)
  - Create time slots across conference days
  - Assign accepted talks to the schedule
- **Communication Tools**:
  - Send emails to speakers using customizable templates
  - Automated notifications for status changes
  - Bulk email functionality

## Tech Stack

- **Backend**: Rust
- **Frontend**: Yew.rs (Rust WebAssembly framework)
- **Database**: PostgreSQL
- **Deployment**: Self-hosted on EC2 instances
- **License**: MIT

## Architecture

The system is designed as a monolithic application that compiles to a single binary:
- Rust backend serves a RESTful API
- Yew.rs frontend compiled to WebAssembly
- Static assets served directly by the Rust backend
- PostgreSQL for data persistence

## Data Model

### Talk States
1. **Submitted**: Initial state when speaker submits
2. **Pending**: Accepted by organizers, awaiting speaker confirmation
3. **Accepted**: Speaker confirmed they will present
4. **Rejected**: Not selected for the conference

### Core Entities
- **Users**: Speakers and organizers with role-based permissions
- **Talks**: Proposals with title, summary, description, slides, tags, and state
- **Ratings**: Individual organizer ratings for each talk
- **Schedule**: Conference days, time slots, tracks, and talk assignments
- **Email Templates**: Customizable templates for speaker communications

## Configuration

The system uses a configuration file to support different conferences:
- Customize for any conference or organization
- Default template provided for TXLF (Texas Linux Fest)
- Configure email templates
- Customize branding and styling

## Getting Started

### Quick Install (Production)

The easiest way to install Call for Papers is using our automated installers.

#### Debian/Ubuntu (Recommended)

```bash
# Download and run the installer
wget https://raw.githubusercontent.com/TXLF/call-for-papers/main/scripts/install-deb.sh
sudo bash install-deb.sh

# Configure database and settings
sudo nano /etc/call-for-papers/config.toml

# Start the service
sudo systemctl enable call-for-papers
sudo systemctl start call-for-papers
```

The Debian package provides:
- Automatic dependency installation
- Systemd service integration
- Standard package management (`apt remove call-for-papers`)
- Follows Linux Filesystem Hierarchy Standard

#### Other Platforms (Linux, macOS)

```bash
# Download and run the binary installer
wget https://raw.githubusercontent.com/TXLF/call-for-papers/main/scripts/install-binary.sh
sudo bash install-binary.sh

# Configure settings
sudo nano /etc/default/call-for-papers

# Start the service
sudo systemctl enable call-for-papers
sudo systemctl start call-for-papers
```

Supported platforms:
- Linux x86_64 (Intel/AMD 64-bit)
- Linux ARM64 (ARM 64-bit)
- macOS x86_64 (Intel Macs)
- macOS ARM64 (Apple Silicon)

#### Pre-built Binaries

Download pre-built binaries from the [releases page](https://github.com/TXLF/call-for-papers/releases):
- Debian/Ubuntu: `.deb` package
- Linux/macOS: `.tar.gz` archives
- Frontend assets: `frontend-dist.tar.gz`

For detailed deployment instructions, see **[scripts/README.md](scripts/README.md)**.

---

### Development Setup

For contributors and developers who want to build from source:

#### Prerequisites

- Rust (latest stable version)
- PostgreSQL 12+
- Trunk (WASM build tool: `cargo install trunk`)
- wasm32-unknown-unknown target (`rustup target add wasm32-unknown-unknown`)

#### Building from Source

```bash
# Clone the repository
git clone https://github.com/TXLF/call-for-papers.git
cd call-for-papers

# Install dependencies
cargo build

# Setup database
createdb call_for_papers

# Migrations run automatically on server start, or run manually:
sqlx migrate run --database-url "postgres://postgres:postgres@localhost/call_for_papers"

# Configure environment (create .env file - see DEVELOPMENT.md for details)
cat > .env << 'EOF'
DATABASE_URL=postgres://postgres:postgres@localhost/call_for_papers
JWT_SECRET=change-this-to-a-random-secret
EOF

# Start the server (migrations will run automatically)
cargo run
```

For detailed development setup including database configuration, environment variables, and troubleshooting, see **[DEVELOPMENT.md](DEVELOPMENT.md)**.

## Documentation

### User Guides

- **[Organizer Guide](docs/organizer-guide.md)** - Complete guide for conference organizers covering talk management, ratings, AI features, schedule building, and communication tools
- **[Speaker Guide](docs/speaker-guide.md)** - Complete guide for speakers covering account creation, talk submission, status tracking, and presentation preparation

### Technical Documentation

- **[Development Guide](DEVELOPMENT.md)** - Setup instructions, development workflow, and troubleshooting
- **[Database Setup](DATABASE_SETUP.md)** - Database configuration and migration instructions
- **[Configuration Reference](config.example.toml)** - All available configuration options

### Configuration

Create a configuration file based on the provided template:

```bash
cp config.example.toml config.toml
# Edit config.toml with your settings
```

## Development

### Quick Start with Make

The project includes a Makefile for convenience:

```bash
# Install dependencies (Trunk, WASM target)
make install

# Build everything
make build

# Run with hot-reload frontend
make dev-frontend

# Run backend
make run

# Run tests
make test

# See all available commands
make help
```

### Running the Development Environment

For frontend development with hot-reload:

```bash
cd frontend
trunk serve
```

This will serve the frontend at `http://127.0.0.1:8000` with auto-reload on changes.

For full-stack development:

```bash
# Terminal 1: Build frontend for production
cd frontend
trunk build --release

# Terminal 2: Run backend (from project root)
cargo run
```

Access the application at `http://localhost:8080`

### Running Tests

```bash
# Run backend tests
cargo test

# Run integration tests
cargo test --test '*'

# Run frontend tests
cd frontend
wasm-pack test --headless --firefox
```

### Building for Production

```bash
# Build the frontend
cd frontend
trunk build --release
cd ..

# Build the backend (which serves the frontend)
cargo build --release

# The binary will be available at target/release/call-for-papers
```

## Deployment

The system is designed for self-hosting on bare EC2 instances or any Linux server.

### Automated Installation

Use our installation scripts for quick deployment:

```bash
# Debian/Ubuntu (recommended)
wget https://raw.githubusercontent.com/TXLF/call-for-papers/main/scripts/install-deb.sh
sudo bash install-deb.sh

# Other Linux distributions or macOS
wget https://raw.githubusercontent.com/TXLF/call-for-papers/main/scripts/install-binary.sh
sudo bash install-binary.sh
```

### Deployment Steps

1. **Install the application** using automated installer
2. **Deploy PostgreSQL database**
   ```bash
   sudo -u postgres createdb call_for_papers
   ```
3. **Configure the application**
   - Edit `/etc/call-for-papers/config.toml` (Debian package)
   - Or set environment variables in `/etc/default/call-for-papers`
4. **Start the service**
   ```bash
   sudo systemctl enable call-for-papers
   sudo systemctl start call-for-papers
   ```
5. **Set up reverse proxy** (Nginx, Apache, or Envoy)
6. **Configure HTTPS** with Let's Encrypt

For detailed deployment instructions including Docker Compose, SSL setup, monitoring, and troubleshooting, see **[scripts/README.md](scripts/README.md)**.

## AI Integration

The system integrates with AI services for automatic talk categorization:

- **OpenAI ChatGPT API**: For GPT-based tagging
- **Anthropic Claude API**: For Claude-based analysis

Configure API keys in your config file to enable these features.

## Contributing

We welcome contributions! This project is open source under the MIT license.

### How to Contribute

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

### Issue Tracking

This project uses **bd (beads)** for issue tracking. To see available work:

```bash
# Install beads
cargo install beads-cli

# View ready work
bd ready

# View all open issues
bd list --status=open

# See project statistics
bd stats
```

For more information on the beads workflow, run `bd prime`.

## Roadmap

Current development priorities (see `bd stats` for live status):

- **P0**: Core infrastructure, authentication, and talk submission
- **P1**: Schedule building, communication system, configuration
- **P2**: AI integration, advanced features

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

Built for the Texas Linux Fest and the broader open source conference community.

## Contact

- Project Repository: https://github.com/TXLF/call-for-papers
- Issue Tracker: Use `bd` commands or GitHub Issues
- Community: [Coming soon]

---

**Status**: Under active development. Not yet ready for production use.
