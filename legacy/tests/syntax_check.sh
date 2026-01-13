#!/bin/bash
# Syntax checker for all shell scripts

echo "Checking syntax..."
FAIL=0
while read -r file; do
    if ! bash -n "$file"; then
        echo "FAIL: $file"
        FAIL=1
    else
        echo "OK: $file"
    fi
done < <(find . -type f -name "*.sh")

exit $FAIL
