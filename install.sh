#!/bin/bash
set -euo pipefail

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}Bootstrapping Cylae Server Manager (Refactored)...${NC}"

# Check root
if [ "$EUID" -ne 0 ]; then
  echo -e "${RED}Please run as root${NC}"
  exit 1
fi

# Ensure git is installed to clone/pull
if ! command -v git &> /dev/null; then
    echo "Installing git..."
    apt-get update -q && apt-get install -y git
fi

# Install dependencies for bootstrap
echo "Installing system dependencies..."
apt-get update -q
apt-get install -y python3 python3-pip python3-venv

# Create venv if not exists
if [ ! -d ".venv" ]; then
    echo "Creating virtual environment..."
    python3 -m venv .venv
fi

# Activate venv
set +u # standard venv activate script might have unbound variables
source .venv/bin/activate
set -u

# Upgrade pip
pip install --upgrade pip

# Install python deps
echo "Installing Python dependencies..."
pip install -r requirements.txt

# Install the package in editable mode
pip install -e .

# Create configuration directory
mkdir -p /etc/cylae
if [ ! -f /etc/cylae/.env ]; then
    echo "Creating default configuration at /etc/cylae/.env"
    echo "DOMAIN=example.com" > /etc/cylae/.env
    echo "EMAIL=admin@example.com" >> /etc/cylae/.env
    echo "DOCKER_NET=server-net" >> /etc/cylae/.env
    chmod 600 /etc/cylae/.env
fi

# Symlink to global path for convenience
ln -sf "$(pwd)/.venv/bin/cyl-manager" /usr/local/bin/cyl-manager

echo -e "${GREEN}Installation Complete! You can run 'cyl-manager' anytime.${NC}"

# Run CLI
cyl-manager
