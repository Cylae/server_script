#!/bin/bash
set -e

# Bootstrap Python environment
echo "Bootstrapping Cylae Manager (Python Rewrite)..."

if [ "$EUID" -ne 0 ]; then
    echo "Please run as root."
    exit 1
fi

# Install python3 and venv if missing
if ! command -v python3 >/dev/null 2>&1; then
    apt-get update && apt-get install -y python3 python3-pip python3-venv python3-distro
fi

# Ensure distro is installed if not present in venv (if we use system packages)
# But we are using venv. So we should install distro inside venv.

# Create venv
if [ ! -d ".venv" ]; then
    python3 -m venv .venv
fi

# Activate
source .venv/bin/activate

# Install requirements
if [ -f "requirements.txt" ]; then
    pip install -r requirements.txt
else
    pip install distro
fi

# Run
export PYTHONPATH=.
python3 -m cyl_manager
