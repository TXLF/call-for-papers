#!/bin/bash
# Call for Papers - SSL Setup Script
# Sets up Let's Encrypt SSL certificates using certbot

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
        apt-get install -y certbot python3-certbot-nginx
    elif command -v yum &> /dev/null; then
        yum install -y certbot python3-certbot-nginx
    else
        log_error "Could not install certbot. Please install it manually."
        exit 1
    fi
fi

# Create webroot directory for ACME challenges
mkdir -p /var/www/certbot

# Check if nginx is installed
if ! command -v nginx &> /dev/null; then
    log_error "Nginx is not installed. Please install it first."
    exit 1
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

    # Test nginx configuration
    log_info "Testing nginx configuration..."
    nginx -t

    if [ $? -eq 0 ]; then
        log_info "Reloading nginx..."
        systemctl reload nginx
        log_info "SSL setup completed successfully!"
        log_info "Certificate location: /etc/letsencrypt/live/${DOMAIN}/"
    else
        log_error "Nginx configuration test failed. Please check your configuration."
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
else
    log_warn "Certbot auto-renewal timer not found"
    log_info "Setting up cron job for renewal..."

    # Add cron job if not exists
    CRON_CMD="0 3 * * * /usr/bin/certbot renew --quiet --post-hook 'systemctl reload nginx'"
    (crontab -l 2>/dev/null | grep -F "${CRON_CMD}") || (crontab -l 2>/dev/null; echo "${CRON_CMD}") | crontab -
fi

log_info "SSL setup complete!"
log_info "Your site should now be accessible at: https://${DOMAIN}"
