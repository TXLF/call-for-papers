#!/bin/bash
# Call for Papers - SSL Setup Script
# Sets up Let's Encrypt SSL certificates using certbot for Envoy proxy

set -euo pipefail

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    log_error "Please run as root (use sudo)"
    exit 1
fi

# Get domain name
if [ $# -eq 0 ]; then
    read -p "Enter your domain name (e.g., cfp.example.com): " DOMAIN
else
    DOMAIN=$1
fi

# Get email for Let's Encrypt
read -p "Enter your email for Let's Encrypt notifications: " EMAIL

log_info "Setting up SSL for domain: ${DOMAIN}"

# Install certbot if not already installed
if ! command -v certbot &> /dev/null; then
    log_info "Installing certbot..."

    if command -v apt-get &> /dev/null; then
        apt-get update
        apt-get install -y certbot
    elif command -v yum &> /dev/null; then
        yum install -y certbot
    else
        log_error "Could not install certbot. Please install it manually."
        exit 1
    fi
fi

# Deployment directory
DEPLOY_DIR="/opt/call-for-papers"

# Create webroot directory for ACME challenges
mkdir -p /var/www/certbot

# Check if Envoy is running
if ! docker ps | grep -q cfp-envoy; then
    log_warn "Envoy container is not running."
    log_warn "You need to start the stack with: cd ${DEPLOY_DIR} && docker-compose -f compose.prod.yaml up -d"
    log_warn "Or continue to obtain certificate only (you can restart Envoy later)"
    read -p "Continue? (yes/no): " -r
    if [ "$REPLY" != "yes" ]; then
        exit 0
    fi
fi

# Obtain certificate
log_info "Obtaining SSL certificate..."

certbot certonly \
    --webroot \
    --webroot-path=/var/www/certbot \
    --email "${EMAIL}" \
    --agree-tos \
    --no-eff-email \
    -d "${DOMAIN}"

if [ $? -eq 0 ]; then
    log_info "SSL certificate obtained successfully!"
    log_info "Certificate location: /etc/letsencrypt/live/${DOMAIN}/"

    # Update Envoy configuration with the correct domain
    log_info "Updating Envoy configuration with domain: ${DOMAIN}"
    ENVOY_CONFIG="${DEPLOY_DIR}/scripts/envoy.yaml"

    if [ -f "${ENVOY_CONFIG}" ]; then
        # Backup original config
        cp "${ENVOY_CONFIG}" "${ENVOY_CONFIG}.bak"

        # Replace example domain with actual domain
        sed -i "s|cfp.example.com|${DOMAIN}|g" "${ENVOY_CONFIG}"
        log_info "Envoy configuration updated"
    else
        log_warn "Envoy configuration not found at ${ENVOY_CONFIG}"
        log_warn "Please manually update the certificate paths in your Envoy configuration"
    fi

    # Restart Envoy container to use new certificates
    log_info "Restarting Envoy container..."
    cd "${DEPLOY_DIR}"
    if docker-compose -f compose.prod.yaml restart envoy; then
        log_info "Envoy restarted successfully"

        # Wait for Envoy to be healthy
        log_info "Waiting for Envoy to be healthy..."
        sleep 5

        if curl -f -k https://localhost/api/health &> /dev/null; then
            log_info "SSL setup completed successfully!"
        else
            log_warn "Could not verify HTTPS endpoint. Please check Envoy logs:"
            log_warn "docker-compose -f ${DEPLOY_DIR}/compose.prod.yaml logs envoy"
        fi
    else
        log_error "Failed to restart Envoy. Please check the logs."
        exit 1
    fi
else
    log_error "Failed to obtain SSL certificate"
    exit 1
fi

# Setup auto-renewal
log_info "Setting up automatic certificate renewal..."

# Certbot auto-renewal should be enabled by default
if systemctl list-timers | grep -q certbot; then
    log_info "Certbot auto-renewal timer is active"

    # Update renewal hook to restart Envoy instead of nginx
    RENEWAL_HOOK="/etc/letsencrypt/renewal-hooks/deploy/restart-envoy.sh"
    mkdir -p /etc/letsencrypt/renewal-hooks/deploy

    cat > "${RENEWAL_HOOK}" <<'EOF'
#!/bin/bash
cd /opt/call-for-papers
docker-compose -f compose.prod.yaml restart envoy
EOF

    chmod +x "${RENEWAL_HOOK}"
    log_info "Created renewal hook to restart Envoy"
else
    log_warn "Certbot auto-renewal timer not found"
    log_info "Setting up cron job for renewal..."

    # Add cron job if not exists
    CRON_CMD="0 3 * * * /usr/bin/certbot renew --quiet --deploy-hook 'cd /opt/call-for-papers && docker-compose -f compose.prod.yaml restart envoy'"
    (crontab -l 2>/dev/null | grep -F "certbot renew") || (crontab -l 2>/dev/null; echo "${CRON_CMD}") | crontab -
fi

log_info "SSL setup complete!"
log_info "Your site should now be accessible at: https://${DOMAIN}"
log_info ""
log_info "Envoy admin interface available at: http://localhost:9901"
log_info "View Envoy logs with: docker-compose -f ${DEPLOY_DIR}/compose.prod.yaml logs -f envoy"
