# Deployment Scripts

This directory contains scripts and configuration files for deploying and managing the Call for Papers application in production.

## Contents

### Deployment Scripts

- **`deploy.sh`** - Main production deployment script
- **`update.sh`** - Rolling update script for upgrading to new versions
- **`backup.sh`** - Database backup script
- **`restore.sh`** - Database restore script
- **`setup-ssl.sh`** - SSL certificate setup using Let's Encrypt

### Configuration Files

- **`envoy.yaml`** - Envoy proxy configuration with SSL, rate limiting, and observability
- **`nginx.conf`** - Legacy nginx configuration (deprecated, use Envoy instead)
- **`call-for-papers.service`** - Systemd service for native binary deployment
- **`call-for-papers-docker.service`** - Systemd service for Docker Compose deployment

## Quick Start

### Initial Deployment

1. **Prepare the server:**
   ```bash
   # Install Docker/Podman
   sudo apt-get update
   sudo apt-get install -y podman podman-compose

   # Or for Docker:
   sudo apt-get install -y docker.io docker-compose
   ```

2. **Deploy the application:**
   ```bash
   # From your local machine, copy files to server
   scp -r . user@server:/tmp/cfp

   # On the server
   cd /tmp/cfp
   sudo ./scripts/deploy.sh
   ```

3. **Configure Envoy proxy:**
   ```bash
   # Edit envoy.yaml and update the domain name
   nano /opt/call-for-papers/scripts/envoy.yaml
   # Change cfp.example.com to your actual domain
   ```

   **Note:** Envoy runs as a Docker container and is automatically started with the production stack. No separate installation needed!

4. **Setup SSL:**
   ```bash
   sudo ./scripts/setup-ssl.sh cfp.example.com
   ```

### Updates

To update to a new version:

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
