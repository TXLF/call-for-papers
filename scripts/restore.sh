#!/bin/bash
# Call for Papers - Database Restore Script

set -euo pipefail

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Configuration
APP_NAME="call-for-papers"
DEPLOY_DIR="/opt/${APP_NAME}"
BACKUP_DIR="${DEPLOY_DIR}/backups"
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

# Check if backup file is provided
if [ $# -eq 0 ]; then
    log_error "Usage: $0 <backup_file.sql.gz>"
    log_info "Available backups:"
    ls -lh "${BACKUP_DIR}"/backup_*.sql.gz 2>/dev/null || log_warn "No backups found"
    exit 1
fi

BACKUP_FILE="$1"

# Check if backup file exists
if [ ! -f "${BACKUP_FILE}" ]; then
    log_error "Backup file not found: ${BACKUP_FILE}"
    exit 1
fi

# Confirm restore
log_warn "WARNING: This will replace the current database with the backup!"
log_warn "Backup file: ${BACKUP_FILE}"
read -p "Are you sure you want to continue? (yes/no): " -r
echo

if [ "$REPLY" != "yes" ]; then
    log_info "Restore cancelled"
    exit 0
fi

# Check if running from deployment directory or local
if [ -f "${DEPLOY_DIR}/${COMPOSE_FILE}" ]; then
    COMPOSE_PATH="${DEPLOY_DIR}/${COMPOSE_FILE}"
else
    COMPOSE_PATH="compose.prod.yaml"
fi

# Check if PostgreSQL is running
if ! docker-compose -f "${COMPOSE_PATH}" ps | grep -q postgres; then
    log_error "PostgreSQL container is not running"
    exit 1
fi

# Create a backup of current database before restore
log_info "Creating safety backup of current database..."
SAFETY_BACKUP="${BACKUP_DIR}/pre_restore_$(date +%Y%m%d_%H%M%S).sql.gz"
docker-compose -f "${COMPOSE_PATH}" exec -T postgres \
    pg_dump -U postgres call_for_papers | gzip > "${SAFETY_BACKUP}"
log_info "Safety backup created: ${SAFETY_BACKUP}"

# Restore database
log_info "Restoring database from ${BACKUP_FILE}..."

if [[ "${BACKUP_FILE}" == *.gz ]]; then
    gunzip -c "${BACKUP_FILE}" | docker-compose -f "${COMPOSE_PATH}" exec -T postgres \
        psql -U postgres -d call_for_papers
else
    cat "${BACKUP_FILE}" | docker-compose -f "${COMPOSE_PATH}" exec -T postgres \
        psql -U postgres -d call_for_papers
fi

if [ $? -eq 0 ]; then
    log_info "Database restored successfully"
    log_info "Safety backup is available at: ${SAFETY_BACKUP}"
else
    log_error "Restore failed"
    log_warn "You can restore the safety backup with:"
    log_warn "$0 ${SAFETY_BACKUP}"
    exit 1
fi

# Verify restore
log_info "Verifying database connection..."
if docker-compose -f "${COMPOSE_PATH}" exec -T postgres \
    psql -U postgres -d call_for_papers -c "SELECT COUNT(*) FROM information_schema.tables;" &> /dev/null; then
    log_info "Database verification passed"
else
    log_error "Database verification failed"
    exit 1
fi

log_info "Restore completed successfully"
