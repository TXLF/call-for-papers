#!/bin/bash
# Call for Papers - Update Script
# Pulls latest images and performs rolling update

set -euo pipefail

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Configuration
APP_NAME="call-for-papers"
DEPLOY_DIR="/opt/${APP_NAME}"
COMPOSE_FILE="compose.prod.yaml"

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

cd "${DEPLOY_DIR}" || exit 1

# Check current status
log_info "Checking current application status..."
docker-compose -f "${COMPOSE_FILE}" ps

# Create backup
log_info "Creating database backup before update..."
if [ -f "./scripts/backup.sh" ]; then
    ./scripts/backup.sh
else
    log_warn "Backup script not found, skipping backup"
fi

# Pull latest images
log_info "Pulling latest Docker images..."
docker-compose -f "${COMPOSE_FILE}" pull

# Check if image was updated
IMAGE_ID_BEFORE=$(docker-compose -f "${COMPOSE_FILE}" images -q backend)
IMAGE_ID_AFTER=$(docker images -q ghcr.io/txlf/call-for-papers:latest)

if [ "${IMAGE_ID_BEFORE}" == "${IMAGE_ID_AFTER}" ]; then
    log_info "Already running the latest version"
    exit 0
fi

log_info "New version detected, performing rolling update..."

# Restart backend with new image
log_info "Restarting backend service..."
docker-compose -f "${COMPOSE_FILE}" up -d --no-deps backend

# Wait for health check
log_info "Waiting for application to be healthy..."
RETRIES=30
COUNT=0

while [ $COUNT -lt $RETRIES ]; do
    if curl -f http://localhost:8080/api/health &> /dev/null; then
        log_info "Application is healthy!"
        break
    fi

    COUNT=$((COUNT + 1))
    if [ $COUNT -eq $RETRIES ]; then
        log_error "Application failed to become healthy after update"
        log_info "Rolling back to previous version..."

        docker-compose -f "${COMPOSE_FILE}" down
        # The previous containers are still available
        docker-compose -f "${COMPOSE_FILE}" up -d

        log_error "Rollback initiated. Please check logs."
        exit 1
    fi

    sleep 2
done

# Show updated status
log_info "Update completed successfully!"
log_info "Current status:"
docker-compose -f "${COMPOSE_FILE}" ps

log_info "View logs with: docker-compose -f ${COMPOSE_FILE} logs -f backend"
