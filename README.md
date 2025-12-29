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

### Prerequisites

- Rust (latest stable version)
- PostgreSQL 12+
- Node.js and npm (for WASM tooling)
- wasm-pack

### Installation

```bash
# Clone the repository
git clone https://github.com/TXLF/call-for-papers.git
cd call-for-papers

# Install dependencies
cargo build

# Setup database
# (instructions coming soon)

# Run migrations
# (instructions coming soon)

# Start the server
cargo run
```

### Configuration

Create a configuration file based on the provided template:

```bash
cp config.example.toml config.toml
# Edit config.toml with your settings
```

## Development

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
# Build optimized release
cargo build --release

# The binary will be available at target/release/call-for-papers
```

## Deployment

The system is designed for self-hosting on bare EC2 instances:

1. Deploy PostgreSQL database
2. Copy the compiled binary to your server
3. Set up environment variables or configuration file
4. Run the binary behind a reverse proxy (Envoy recommended)
5. Configure HTTPS with Let's Encrypt

See `scripts/README.md` for detailed deployment instructions.

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
