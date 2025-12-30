# Deployment Scripts

This directory contains scripts and configuration files for deploying and managing the Call for Papers application in production.

## Contents

### Installation Scripts

- **`install-deb.sh`** - Install from Debian/Ubuntu package (recommended for Debian/Ubuntu)
- **`install-binary.sh`** - Install from binary tarball (works on all platforms)

### Deployment Scripts

- **`deploy.sh`** - Main production deployment script
- **`update.sh`** - Rolling update script for upgrading to new versions
- **`backup.sh`** - Database backup script
- **`restore.sh`** - Database restore script
- **`setup-ssl.sh`** - SSL certificate setup using Let's Encrypt

### Configuration Files

- **`envoy.yaml`** - Envoy proxy configuration with SSL, rate limiting, and observability
- **`call-for-papers.service`** - Systemd service for native binary deployment
- **`call-for-papers-docker.service`** - Systemd service for Docker Compose deployment

## Installation Methods

### Method 1: Automated Installation (Recommended)

The easiest way to install Call for Papers is using our installation scripts. These scripts automatically download and set up the latest release.

#### Debian/Ubuntu Package Installation

For Debian and Ubuntu systems, install using the .deb package:

```bash
# Download and run the installer
wget https://raw.githubusercontent.com/TXLF/call-for-papers/main/scripts/install-deb.sh
sudo bash install-deb.sh

# Or install a specific version
sudo bash install-deb.sh v0.1.0
```

**Benefits:**
- Automatic dependency installation
- Systemd service automatically configured
- Standard Debian/Ubuntu package management
- Clean uninstall with `sudo apt remove call-for-papers`
- Follows Filesystem Hierarchy Standard (FHS)

**Locations:**
- Binary: `/usr/bin/call-for-papers`
- Config: `/etc/call-for-papers/`
- Data: `/var/lib/call-for-papers/`
- Logs: `/var/log/call-for-papers/`
- Service: `/lib/systemd/system/call-for-papers.service`

#### Binary Installation (All Platforms)

For other Linux distributions, macOS, or manual installation:

```bash
# Download and run the installer
wget https://raw.githubusercontent.com/TXLF/call-for-papers/main/scripts/install-binary.sh
sudo bash install-binary.sh

# Or install a specific version to a custom location
sudo bash install-binary.sh v0.1.0 /opt/call-for-papers
```

**Supported platforms:**
- Linux x86_64 (Intel/AMD 64-bit)
- Linux ARM64 (ARM 64-bit)
- macOS x86_64 (Intel Macs)
- macOS ARM64 (Apple Silicon)

**Default locations:**
- Installation: `/opt/call-for-papers/`
- Config: `/etc/call-for-papers/`
- Environment: `/etc/default/call-for-papers`
- Service: `/etc/systemd/system/call-for-papers.service`

### Method 2: Manual Download

Download pre-built binaries from the [releases page](https://github.com/TXLF/call-for-papers/releases):

1. Download the appropriate file for your platform:
   - Debian/Ubuntu: `call-for-papers_VERSION_amd64.deb`
   - Linux x86_64: `call-for-papers-linux-amd64.tar.gz`
   - Linux ARM64: `call-for-papers-linux-arm64.tar.gz`
   - macOS Intel: `call-for-papers-macos-amd64.tar.gz`
   - macOS ARM: `call-for-papers-macos-arm64.tar.gz`
   - Frontend: `frontend-dist.tar.gz`

2. Install the package or extract the binary
3. Download and extract frontend assets
4. Configure and run

### Method 3: Docker Compose

See [Docker Compose section](#option-1-docker-compose-recommended) below.

### Method 4: Build from Source

See the main [README.md](../README.md) and [DEVELOPMENT.md](../DEVELOPMENT.md) for build instructions.

---

## Quick Start

### After Installation

Once installed using the automated installers or deb package:

1. **Configure Database:**
   ```bash
   # Create PostgreSQL database
   sudo -u postgres createdb call_for_papers

   # Set database URL (in config or environment file)
   # For deb: /etc/call-for-papers/config.toml or /etc/default/call-for-papers
   # For binary: /etc/default/call-for-papers
   ```

2. **Configure Application:**
   ```bash
   # Edit configuration (deb package)
   sudo nano /etc/call-for-papers/config.toml

   # Or set environment variables (both methods)
   sudo nano /etc/default/call-for-papers
   ```

   Required settings:
   - `DATABASE_URL`: PostgreSQL connection string
   - `JWT_SECRET`: Random secret key (generate with `openssl rand -base64 64`)

3. **Start the Service:**
   ```bash
   sudo systemctl enable call-for-papers
   sudo systemctl start call-for-papers
   sudo systemctl status call-for-papers
   ```

4. **Verify Installation:**
   ```bash
   # Check logs
   sudo journalctl -u call-for-papers -f

   # Test health endpoint
   curl http://localhost:8080/api/health
   ```

5. **Setup Reverse Proxy (Production):**

   For production, set up a reverse proxy (Nginx, Apache, or Envoy) with SSL:

   ```bash
   # Example with Envoy (Docker-based)
   # Edit envoy.yaml and update the domain name
   nano /opt/call-for-papers/scripts/envoy.yaml

   # Setup SSL certificates
   sudo ./scripts/setup-ssl.sh cfp.example.com
   ```

### Updates

#### Updating Debian/Ubuntu Package

```bash
# Download and run the installer again with the new version
wget https://raw.githubusercontent.com/TXLF/call-for-papers/main/scripts/install-deb.sh
sudo bash install-deb.sh v0.2.0

# Or let it auto-detect the latest
sudo bash install-deb.sh latest

# Restart the service
sudo systemctl restart call-for-papers
```

#### Updating Binary Installation

```bash
# Download and run the binary installer again
wget https://raw.githubusercontent.com/TXLF/call-for-papers/main/scripts/install-binary.sh
sudo bash install-binary.sh v0.2.0

# Restart the service
sudo systemctl restart call-for-papers
```

#### Updating Docker Compose Deployment

```bash
cd /opt/call-for-papers
./scripts/update.sh
```

This will:
- Create a database backup
- Pull the latest Docker image
- Perform a rolling update
- Verify the application is healthy
- Rollback if the update fails

### Backups

**Manual backup:**
```bash
cd /opt/call-for-papers
./scripts/backup.sh
```

**Automated backups:**
The production Docker Compose configuration includes an automated backup service that runs daily.

**Restore from backup:**
```bash
cd /opt/call-for-papers
./scripts/restore.sh /path/to/backup.sql.gz
```

## Deployment Options

### Option 1: Docker Compose (Recommended)

The easiest way to deploy is using Docker Compose:

```bash
# Production deployment
cd /opt/call-for-papers
docker-compose -f compose.prod.yaml up -d
```

**Systemd service:**
```bash
sudo cp scripts/call-for-papers-docker.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable call-for-papers-docker
sudo systemctl start call-for-papers-docker
```

### Option 2: Native Binary

For maximum performance, deploy the native binary:

1. **Build the application:**
   ```bash
   # Build frontend
   cd frontend && trunk build --release && cd ..

   # Build backend
   cargo build --release
   ```

2. **Copy to server:**
   ```bash
   scp target/release/call-for-papers user@server:/opt/call-for-papers/
   scp -r frontend/dist user@server:/opt/call-for-papers/frontend/
   scp -r migrations user@server:/opt/call-for-papers/
   ```

3. **Setup systemd service:**
   ```bash
   sudo cp scripts/call-for-papers.service /etc/systemd/system/
   sudo systemctl daemon-reload
   sudo systemctl enable call-for-papers
   sudo systemctl start call-for-papers
   ```

## Environment Configuration

Create a `.env` file in `/opt/call-for-papers/`:

```bash
# Copy production template
cp .env.production .env

# Edit with your values
nano .env
```

**Required values:**
- `POSTGRES_PASSWORD` - Strong database password
- `JWT_SECRET` - Generate with: `openssl rand -base64 64`

**Optional values:**
- `APP_PORT` - Default: 8080
- OAuth credentials (GitHub, Google, Apple, Facebook)
- SMTP settings for email notifications

## Monitoring

### Check application status

```bash
# Docker Compose
cd /opt/call-for-papers
docker-compose -f compose.prod.yaml ps

# Systemd
sudo systemctl status call-for-papers
```

### View logs

```bash
# Docker Compose
docker-compose -f compose.prod.yaml logs -f backend

# Systemd
sudo journalctl -u call-for-papers -f
```

### Health checks

```bash
# Basic health (through Envoy)
curl https://your-domain.com/api/health

# Or directly to backend
curl http://localhost:8080/api/health

# Database health
curl http://localhost:8080/api/health/db

# Envoy health
curl http://localhost:9901/ready

# Envoy stats and metrics
curl http://localhost:9901/stats
```

### Envoy Admin Interface

Envoy provides a powerful admin interface on port 9901:

```bash
# Access admin interface
http://localhost:9901

# Available endpoints:
# /stats - Detailed statistics and metrics
# /stats/prometheus - Prometheus-formatted metrics
# /clusters - Upstream cluster information
# /config_dump - Current configuration
# /ready - Readiness check
# /server_info - Server information
# /logging - Dynamic log level control
```

**Note:** In production, restrict access to port 9901 to localhost only (already configured in compose.prod.yaml).

## Security Considerations

1. **Firewall Configuration:**
   ```bash
   # Allow only necessary ports
   sudo ufw allow 80/tcp
   sudo ufw allow 443/tcp
   sudo ufw allow 22/tcp
   sudo ufw enable
   ```

2. **Database Security:**
   - Use strong passwords
   - Don't expose PostgreSQL port (5432) to the internet
   - Enable SSL for database connections in production

3. **SSL/TLS:**
   - Always use HTTPS in production
   - Keep certificates up to date (certbot handles this automatically)
   - Use strong cipher suites (configured in envoy.yaml)

4. **Application Security:**
   - Generate a strong JWT secret (minimum 32 characters)
   - Regularly update Docker images
   - Review security audit results: `cargo audit`

## Troubleshooting

### Application won't start

```bash
# Check logs
docker-compose -f compose.prod.yaml logs backend

# Check database connection
docker-compose -f compose.prod.yaml exec postgres pg_isready
```

### Database connection errors

```bash
# Verify PostgreSQL is running
docker-compose -f compose.prod.yaml ps postgres

# Check DATABASE_URL in .env
cat /opt/call-for-papers/.env | grep DATABASE_URL
```

### SSL certificate issues

```bash
# Check certificate status
sudo certbot certificates

# Renew certificates manually
sudo certbot renew

# Restart Envoy to load new certificates
cd /opt/call-for-papers
docker-compose -f compose.prod.yaml restart envoy
```

## Maintenance

### Update Docker images

```bash
cd /opt/call-for-papers
./scripts/update.sh
```

### Database maintenance

```bash
# Connect to PostgreSQL
docker-compose -f compose.prod.yaml exec postgres psql -U postgres -d call_for_papers

# Vacuum database
VACUUM ANALYZE;

# Check database size
SELECT pg_size_pretty(pg_database_size('call_for_papers'));
```

### Log rotation

Logs are automatically rotated by Docker/systemd. To manually clear logs:

```bash
# Docker logs
docker-compose -f compose.prod.yaml logs --tail=0

# Systemd logs
sudo journalctl --vacuum-time=7d
```

## Backup Strategy

1. **Automated daily backups** (included in compose.prod.yaml)
2. **Pre-update backups** (automatic with update.sh)
3. **Manual backups** before major changes

**Backup retention:**
- Daily backups: 7 days
- Weekly backups: 4 weeks
- Monthly backups: 6 months

## Support

For issues or questions:
- Project repository: https://github.com/TXLF/call-for-papers
- Issue tracking: Use `bd` commands
- Documentation: See main README.md and DEVELOPMENT.md
