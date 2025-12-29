#!/bin/bash
# Call for Papers - Database Backup Script

set -euo pipefail

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

# Configuration
APP_NAME="call-for-papers"
DEPLOY_DIR="/opt/${APP_NAME}"
BACKUP_DIR="${DEPLOY_DIR}/backups"
COMPOSE_FILE="compose.prod.yaml"
RETENTION_DAYS=30

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Create backup directory if it doesn't exist
mkdir -p "${BACKUP_DIR}"

# Generate backup filename with timestamp
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="${BACKUP_DIR}/backup_${TIMESTAMP}.sql.gz"

log_info "Creating database backup..."

# Check if running from deployment directory or local
if [ -f "${DEPLOY_DIR}/${COMPOSE_FILE}" ]; then
    COMPOSE_PATH="${DEPLOY_DIR}/${COMPOSE_FILE}"
else
    COMPOSE_PATH="compose.prod.yaml"
fi

# Create backup
if docker-compose -f "${COMPOSE_PATH}" ps | grep -q postgres; then
    # Backup database
    docker-compose -f "${COMPOSE_PATH}" exec -T postgres \
        pg_dump -U postgres --clean --if-exists call_for_papers | gzip > "${BACKUP_FILE}"

    if [ -f "${BACKUP_FILE}" ]; then
        SIZE=$(du -h "${BACKUP_FILE}" | cut -f1)
        log_info "Backup created: ${BACKUP_FILE} (${SIZE})"
    else
        log_error "Backup failed"
        exit 1
    fi
else
    log_error "PostgreSQL container is not running"
    exit 1
fi

# Clean up old backups
log_info "Cleaning up backups older than ${RETENTION_DAYS} days..."
find "${BACKUP_DIR}" -name "backup_*.sql.gz" -type f -mtime +${RETENTION_DAYS} -delete

# List recent backups
log_info "Recent backups:"
ls -lh "${BACKUP_DIR}"/backup_*.sql.gz | tail -n 5

log_info "Backup completed successfully"
