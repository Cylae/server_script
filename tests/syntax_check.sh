#!/bin/bash
set -u

# Simple Syntax Check for Bash Scripts
echo "Running syntax check on .sh files..."

find src -name "*.sh" | while read -r file; do
    if bash -n "$file"; then
        echo -e "\033[0;32m[OK]\033[0m $file"
    else
        echo -e "\033[0;31m[FAIL]\033[0m $file"
        exit 1
    fi
done

echo "All scripts passed syntax check."
