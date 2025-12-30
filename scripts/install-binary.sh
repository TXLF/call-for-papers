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
INSTALL_DIR="${2:-/opt/call-for-papers}"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Call for Papers - Binary Installer${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}Error: This script must be run as root (use sudo)${NC}"
   exit 1
fi

# Detect platform
detect_platform() {
    local os=$(uname -s)
    local arch=$(uname -m)

    case "$os" in
        Linux)
            case "$arch" in
                x86_64)
                    echo "call-for-papers-linux-amd64.tar.gz"
                    ;;
                aarch64|arm64)
                    echo "call-for-papers-linux-arm64.tar.gz"
                    ;;
                *)
                    echo -e "${RED}Error: Unsupported architecture: $arch${NC}"
                    exit 1
                    ;;
            esac
            ;;
        Darwin)
            case "$arch" in
                x86_64)
                    echo "call-for-papers-macos-amd64.tar.gz"
                    ;;
                arm64)
                    echo "call-for-papers-macos-arm64.tar.gz"
                    ;;
                *)
                    echo -e "${RED}Error: Unsupported architecture: $arch${NC}"
                    exit 1
                    ;;
            esac
            ;;
        *)
            echo -e "${RED}Error: Unsupported operating system: $os${NC}"
            exit 1
            ;;
    esac
}

PLATFORM=$(detect_platform)
echo -e "${GREEN}Detected platform: $PLATFORM${NC}"

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

# Download URLs
BINARY_URL="https://github.com/$GITHUB_REPO/releases/download/v${VERSION}/${PLATFORM}"
FRONTEND_URL="https://github.com/$GITHUB_REPO/releases/download/v${VERSION}/frontend-dist.tar.gz"

echo ""
echo -e "${BLUE}Creating installation directory...${NC}"
mkdir -p "$INSTALL_DIR"
cd "$INSTALL_DIR"

echo ""
echo -e "${BLUE}Downloading backend binary...${NC}"
if ! wget -q --show-progress "$BINARY_URL"; then
    echo -e "${RED}Error: Failed to download binary${NC}"
    exit 1
fi

echo ""
echo -e "${BLUE}Downloading frontend assets...${NC}"
if ! wget -q --show-progress "$FRONTEND_URL"; then
    echo -e "${RED}Error: Failed to download frontend${NC}"
    exit 1
fi

echo ""
echo -e "${BLUE}Extracting files...${NC}"

# Extract binary
tar xzf "$PLATFORM"
chmod +x call-for-papers
rm "$PLATFORM"

# Extract frontend
tar xzf frontend-dist.tar.gz
rm frontend-dist.tar.gz

echo ""
echo -e "${BLUE}Creating directories...${NC}"
mkdir -p "$INSTALL_DIR/uploads"
mkdir -p "$INSTALL_DIR/logs"
mkdir -p /etc/call-for-papers

# Create config file if it doesn't exist
if [ ! -f /etc/call-for-papers/config.toml ]; then
    echo -e "${BLUE}Creating example config...${NC}"
    cat > /etc/call-for-papers/config.example.toml << 'EOF'
# Call for Papers Configuration
# Copy this to config.toml and customize for your conference

[conference]
name = "Your Conference Name"
short_name = "YCN"
year = 2025
location_city = "Your City"
location_state = "Your State"
location_country = "Your Country"

[database]
url = ""  # Set via DATABASE_URL environment variable

[server]
host = "0.0.0.0"
port = 8080

[security]
jwt_secret = ""  # Set via JWT_SECRET environment variable
jwt_expiration_hours = 168  # 7 days
EOF
    echo -e "${YELLOW}Example config created at /etc/call-for-papers/config.example.toml${NC}"
fi

echo ""
echo -e "${BLUE}Creating systemd service...${NC}"

cat > /etc/systemd/system/call-for-papers.service << EOF
[Unit]
Description=Call for Papers Service
After=network.target postgresql.service

[Service]
Type=simple
User=root
WorkingDirectory=$INSTALL_DIR
ExecStart=$INSTALL_DIR/call-for-papers
Restart=on-failure
RestartSec=10

# Environment file (create this with your settings)
EnvironmentFile=-/etc/default/call-for-papers

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=$INSTALL_DIR/uploads $INSTALL_DIR/logs

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=call-for-papers

[Install]
WantedBy=multi-user.target
EOF

# Create environment file template
if [ ! -f /etc/default/call-for-papers ]; then
    cat > /etc/default/call-for-papers << 'EOF'
# Call for Papers Environment Configuration
# Set your environment variables here

# Required
DATABASE_URL=postgres://postgres:postgres@localhost/call_for_papers
JWT_SECRET=CHANGE_THIS_TO_A_RANDOM_SECRET

# Optional
#APP_PORT=8080
#RUST_LOG=info

# OAuth (optional)
#GOOGLE_CLIENT_ID=
#GOOGLE_CLIENT_SECRET=
#GITHUB_CLIENT_ID=
#GITHUB_CLIENT_SECRET=

# AI APIs (optional)
#CLAUDE_API_KEY=
#OPENAI_API_KEY=

# Email (optional)
#SMTP_HOST=
#SMTP_PORT=587
#SMTP_USERNAME=
#SMTP_PASSWORD=
#SMTP_FROM=
EOF
    echo -e "${YELLOW}Environment template created at /etc/default/call-for-papers${NC}"
fi

# Reload systemd
systemctl daemon-reload

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Installation Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "${BLUE}Installation directory: $INSTALL_DIR${NC}"
echo ""
echo -e "${BLUE}Next steps:${NC}"
echo ""
echo -e "1. ${YELLOW}Configure PostgreSQL${NC}"
echo -e "   ${BLUE}sudo -u postgres createdb call_for_papers${NC}"
echo ""
echo -e "2. ${YELLOW}Configure environment variables${NC}"
echo -e "   ${BLUE}sudo nano /etc/default/call-for-papers${NC}"
echo ""
echo -e "   Set at minimum:"
echo -e "   - DATABASE_URL"
echo -e "   - JWT_SECRET (generate with: openssl rand -base64 64)"
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
echo -e "${BLUE}For manual start (without systemd):${NC}"
echo -e "   ${BLUE}cd $INSTALL_DIR${NC}"
echo -e "   ${BLUE}export DATABASE_URL='...'${NC}"
echo -e "   ${BLUE}export JWT_SECRET='...'${NC}"
echo -e "   ${BLUE}./call-for-papers${NC}"
echo ""
