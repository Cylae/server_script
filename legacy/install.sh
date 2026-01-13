#!/bin/bash
set -e

# Colors
GREEN='\033[0;32m'
NC='\033[0m'

echo -e "${GREEN}Bootstrapping Cylae Server Manager (Python Edition)...${NC}"

# Check root
if [ "$EUID" -ne 0 ]; then
  echo "Please run as root"
  exit 1
fi

# Install dependencies for bootstrap
apt-get update -q
apt-get install -y python3 python3-pip python3-venv git

# Create venv
if [ ! -d ".venv" ]; then
    python3 -m venv .venv
fi

# Activate venv
source .venv/bin/activate

# Install python deps
pip install -r requirements.txt

# Install the package in editable mode
pip install -e .

# Symlink to global path for convenience
ln -sf $(pwd)/.venv/bin/cyl-manager /usr/local/bin/cyl-manager

echo -e "${GREEN}Installation Complete! You can run 'cyl-manager' anytime.${NC}"

# Run CLI
cyl-manager
