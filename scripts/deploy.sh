#!/bin/bash
# Call for Papers - Production Deployment Script
# This script deploys the application to a production server

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
APP_NAME="call-for-papers"
DEPLOY_DIR="/opt/${APP_NAME}"
BACKUP_DIR="${DEPLOY_DIR}/backups"
COMPOSE_FILE="compose.prod.yaml"

# Functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_prerequisites() {
    log_info "Checking prerequisites..."

    if ! command -v docker &> /dev/null && ! command -v podman &> /dev/null; then
        log_error "Neither docker nor podman is installed"
        exit 1
    fi

    if ! command -v docker-compose &> /dev/null && ! command -v podman-compose &> /dev/null; then
        log_error "Neither docker-compose nor podman-compose is installed"
        exit 1
    fi

    log_info "Prerequisites check passed"
}

create_directories() {
    log_info "Creating deployment directories..."
    sudo mkdir -p "${DEPLOY_DIR}"
    sudo mkdir -p "${BACKUP_DIR}"
    sudo chown -R "$(whoami):$(whoami)" "${DEPLOY_DIR}"
}

copy_files() {
    log_info "Copying deployment files..."
    cp "${COMPOSE_FILE}" "${DEPLOY_DIR}/"

    if [ ! -f "${DEPLOY_DIR}/.env" ]; then
        log_warn ".env file not found in ${DEPLOY_DIR}"
        log_warn "Copying .env.production as template"
        cp .env.production "${DEPLOY_DIR}/.env"
        log_error "Please edit ${DEPLOY_DIR}/.env with production values before continuing"
        exit 1
    fi
}

backup_database() {
    log_info "Creating database backup..."
    if docker-compose -f "${DEPLOY_DIR}/${COMPOSE_FILE}" ps | grep -q postgres; then
        TIMESTAMP=$(date +%Y%m%d_%H%M%S)
        BACKUP_FILE="${BACKUP_DIR}/backup_${TIMESTAMP}.sql"

        docker-compose -f "${DEPLOY_DIR}/${COMPOSE_FILE}" exec -T postgres \
            pg_dump -U postgres call_for_papers > "${BACKUP_FILE}"

        log_info "Database backed up to ${BACKUP_FILE}"
    else
        log_warn "PostgreSQL container not running, skipping backup"
    fi
}

pull_images() {
    log_info "Pulling latest Docker images..."
    cd "${DEPLOY_DIR}"
    docker-compose -f "${COMPOSE_FILE}" pull
}

deploy() {
    log_info "Deploying application..."
    cd "${DEPLOY_DIR}"

    # Stop old containers
    log_info "Stopping old containers..."
    docker-compose -f "${COMPOSE_FILE}" down

    # Start new containers
    log_info "Starting new containers..."
    docker-compose -f "${COMPOSE_FILE}" up -d

    # Wait for health checks
    log_info "Waiting for application to be healthy..."
    sleep 10

    # Check health
    if curl -f http://localhost:8080/api/health &> /dev/null; then
        log_info "Application is healthy!"
    else
        log_error "Application health check failed"
        log_info "Viewing logs..."
        docker-compose -f "${COMPOSE_FILE}" logs --tail=50
        exit 1
    fi
}

show_status() {
    log_info "Deployment status:"
    cd "${DEPLOY_DIR}"
    docker-compose -f "${COMPOSE_FILE}" ps
}

# Main execution
main() {
    log_info "Starting deployment of ${APP_NAME}..."

    check_prerequisites
    create_directories
    copy_files

    # Backup only if upgrading
    if [ "${1:-}" != "--fresh" ]; then
        backup_database
    fi

    pull_images
    deploy
    show_status

    log_info "Deployment completed successfully!"
    log_info "Application is available at http://localhost:8080"
    log_info "View logs with: cd ${DEPLOY_DIR} && docker-compose -f ${COMPOSE_FILE} logs -f"
}

# Run main function
main "$@"
