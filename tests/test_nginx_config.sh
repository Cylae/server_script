#!/bin/bash
set -u

# Test Nginx Config Generation

# Mock environment
DOMAIN="example.com"
EMAIL="test@example.com"
source src/lib/core.sh
source src/lib/proxy.sh

# Mock LOG_FILE
LOG_FILE="/tmp/test_log.txt"

# Mock systemctl and certbot
systemctl() { echo "systemctl $@" >&2; }
certbot() { echo "certbot $@" >&2; }
ln() { echo "ln $@" >&2; }

# Mock /etc/nginx directories
mkdir -p /tmp/etc/nginx/sites-available
mkdir -p /tmp/etc/nginx/sites-enabled
# We need to override the paths in the functions or mock the functions to use /tmp
# Since the script uses hardcoded paths /etc/nginx/..., we need to sed the script or use a chroot?
# Easier: Just sed the script into a temporary test script.

cp src/lib/proxy.sh /tmp/test_proxy.sh
sed -i 's|/etc/nginx|/tmp/etc/nginx|g' /tmp/test_proxy.sh

source /tmp/test_proxy.sh

echo "Testing Cloud Subdomain (10G)..."
update_nginx "cloud.example.com" "8080" "proxy"

if grep -q "client_max_body_size 10G;" /tmp/etc/nginx/sites-available/cloud.example.com; then
    echo "PASS: Cloud 10G detected"
else
    echo "FAIL: Cloud 10G NOT detected"
    cat /tmp/etc/nginx/sites-available/cloud.example.com
    exit 1
fi

echo "Testing Normal Subdomain (512M)..."
update_nginx "git.example.com" "3000" "proxy"

if grep -q "client_max_body_size 512M;" /tmp/etc/nginx/sites-available/git.example.com; then
    echo "PASS: Normal 512M detected"
else
    echo "FAIL: Normal 512M NOT detected"
    cat /tmp/etc/nginx/sites-available/git.example.com
    exit 1
fi

echo "All Nginx Tests Passed"
