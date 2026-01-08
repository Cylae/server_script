#!/bin/bash
# test_simulation.sh

# Source the libraries to test functions in isolation
source lib/utils.sh
source lib/core.sh

# Mock dependencies that won't work in sandbox (like systemctl, docker, apt-get)
systemctl() { echo "MOCK: systemctl $@"; }
apt-get() { echo "MOCK: apt-get $@"; }
docker() {
    if [ "$1" == "ps" ]; then
        echo "0.0.0.0:80->80/tcp"
        echo "0.0.0.0:3000->3000/tcp"
    else
        echo "MOCK: docker $@"
    fi
}
certbot() { echo "MOCK: certbot $@"; }
ufw() { echo "MOCK: ufw $@"; }
sysctl() { echo "MOCK: sysctl $@"; }
mount() { echo "MOCK: mount $@"; }
swapon() { echo "MOCK: swapon $@"; }
mkswap() { echo "MOCK: mkswap $@"; }
chmod() { echo "MOCK: chmod $@"; }
chown() { echo "MOCK: chown $@"; }
mysql() { echo "MOCK: mysql $@"; }
useradd() { echo "MOCK: useradd $@"; }
usermod() { echo "MOCK: usermod $@"; }
chpasswd() { echo "MOCK: chpasswd (piped)"; }

# Setup dummy config
CONFIG_FILE="/tmp/cyl_manager.conf"
AUTH_FILE="/tmp/.auth_details"
LOG_FILE="/tmp/server_manager.log"
DOMAIN="example.com"
export CONFIG_FILE AUTH_FILE LOG_FILE DOMAIN
touch $AUTH_FILE $CONFIG_FILE

# Redirect fd 3 to stdout for testing
exec 3>&1

echo "----------------------------------------------------------------"
echo "TEST 1: Credential Generation (ask_credential)"
echo "----------------------------------------------------------------"

# Test Case 1: Empty input (should generate defaults)
# Using Here-Document to avoid subshell pipe issue
ask_credential "TestService" "admin" "test1" <<EOF


EOF

echo "USER: $SET_USER"
echo "PASS: $SET_PASS"

if [ "$SET_USER" == "admin" ]; then
    echo "PASS: Default User correct"
else
    echo "FAIL: Default User incorrect. Expected 'admin', got '$SET_USER'"
fi

if [ -n "$SET_PASS" ]; then
    echo "PASS: Password generated"
else
    echo "FAIL: Password not generated"
fi

# Test Case 2: Custom input
ask_credential "TestService2" "admin" "test2" <<EOF
customuser
custompass
EOF

echo "USER: $SET_USER"
echo "PASS: $SET_PASS"

if [ "$SET_USER" == "customuser" ]; then
    echo "PASS: Custom User correct"
else
    echo "FAIL: Custom User incorrect. Expected 'customuser', got '$SET_USER'"
fi

if [ "$SET_PASS" == "custompass" ]; then
    echo "PASS: Custom Password correct"
else
    echo "FAIL: Custom Password incorrect. Expected 'custompass', got '$SET_PASS'"
fi


echo "----------------------------------------------------------------"
echo "TEST 2: Safety Check"
echo "----------------------------------------------------------------"

# Check port 80 (should fail/warn)
echo "Checking port 80 (should warn)..."
safety_check "80"

# Check port 9000 (should pass)
echo "Checking port 9000 (should pass)..."
safety_check "9000"


echo "----------------------------------------------------------------"
echo "TEST 3: SSL Enable"
echo "----------------------------------------------------------------"
enable_ssl "test.example.com"

echo "----------------------------------------------------------------"
echo "TEST 4: Service Report"
echo "----------------------------------------------------------------"
# Pipe echo to simulate "Press Enter to continue"
echo "" | show_service_report "Test Service" "https://test.example.com" "admin" "password123"

echo "----------------------------------------------------------------"
echo "SIMULATION COMPLETE"
echo "----------------------------------------------------------------"
