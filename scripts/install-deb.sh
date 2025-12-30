#!/bin/bash
set -e

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
GITHUB_REPO="TXLF/call-for-papers"
VERSION="${1:-latest}"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Call for Papers - Debian Package Installer${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}Error: This script must be run as root (use sudo)${NC}"
   exit 1
fi

# Check if running on Debian/Ubuntu
if ! command -v dpkg &> /dev/null; then
    echo -e "${RED}Error: This script is for Debian/Ubuntu systems only${NC}"
    echo -e "${YELLOW}For other systems, use install-binary.sh${NC}"
    exit 1
fi

# Determine version to install
if [ "$VERSION" = "latest" ]; then
    echo -e "${BLUE}Fetching latest version...${NC}"
    VERSION=$(curl -s "https://api.github.com/repos/$GITHUB_REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"v([^"]+)".*/\1/')

    if [ -z "$VERSION" ]; then
        echo -e "${RED}Error: Could not determine latest version${NC}"
        exit 1
    fi

    echo -e "${GREEN}Latest version: $VERSION${NC}"
else
    # Remove 'v' prefix if present
    VERSION="${VERSION#v}"
fi

# Package details
PACKAGE_NAME="call-for-papers_${VERSION}_amd64.deb"
DOWNLOAD_URL="https://github.com/$GITHUB_REPO/releases/download/v${VERSION}/${PACKAGE_NAME}"

echo ""
echo -e "${BLUE}Downloading package...${NC}"
echo -e "URL: $DOWNLOAD_URL"

# Create temporary directory
TMP_DIR=$(mktemp -d)
cd "$TMP_DIR"

# Download the package
if ! wget -q --show-progress "$DOWNLOAD_URL"; then
    echo -e "${RED}Error: Failed to download package${NC}"
    echo -e "${YELLOW}Please check that version $VERSION exists${NC}"
    rm -rf "$TMP_DIR"
    exit 1
fi

echo ""
echo -e "${BLUE}Installing package...${NC}"

# Install the package
if dpkg -i "$PACKAGE_NAME"; then
    echo -e "${GREEN}Package installed successfully${NC}"
else
    echo -e "${YELLOW}Installing missing dependencies...${NC}"
    apt-get install -f -y
fi

# Cleanup
cd /
rm -rf "$TMP_DIR"

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Installation Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "${BLUE}Next steps:${NC}"
echo ""
echo -e "1. ${YELLOW}Configure PostgreSQL${NC}"
echo -e "   Create a database for the application:"
echo -e "   ${BLUE}sudo -u postgres createdb call_for_papers${NC}"
echo ""
echo -e "2. ${YELLOW}Configure the application${NC}"
echo -e "   Edit the configuration file:"
echo -e "   ${BLUE}sudo nano /etc/call-for-papers/config.toml${NC}"
echo ""
echo -e "   Or set environment variables in:"
echo -e "   ${BLUE}/etc/default/call-for-papers${NC}"
echo ""
echo -e "   Required settings:"
echo -e "   - DATABASE_URL=postgres://user:pass@localhost/call_for_papers"
echo -e "   - JWT_SECRET=\$(openssl rand -base64 64)"
echo ""
echo -e "3. ${YELLOW}Start the service${NC}"
echo -e "   ${BLUE}sudo systemctl enable call-for-papers${NC}"
echo -e "   ${BLUE}sudo systemctl start call-for-papers${NC}"
echo ""
echo -e "4. ${YELLOW}Check status${NC}"
echo -e "   ${BLUE}sudo systemctl status call-for-papers${NC}"
echo -e "   ${BLUE}sudo journalctl -u call-for-papers -f${NC}"
echo ""
echo -e "5. ${YELLOW}Access the application${NC}"
echo -e "   Default: http://localhost:8080"
echo ""
echo -e "${BLUE}Documentation: /usr/share/doc/call-for-papers/${NC}"
echo ""
