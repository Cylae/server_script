#!/bin/bash
set -e

# Colors
GREEN='\033[0;32m'
NC='\033[0m'

echo -e "${GREEN}Bootstrapping Cylae Server Manager...${NC}"

# Check root
if [ "$EUID" -ne 0 ]; then
  echo "Please run as root"
  exit 1
fi

# Ensure Python is installed
if ! command -v python3 &> /dev/null; then
    echo "Python3 not found. Installing..."
    apt-get update -q
    apt-get install -y python3 python3-pip python3-venv git
fi

# Run the Python installer
python3 install.py
