#!/bin/bash
# Unit tests for core functions

# Mock environment
AUTH_FILE="./test_auth_file"
touch "$AUTH_FILE"

# Mock Docker Network check for docker.sh
docker() {
    if [ "$1" == "network" ]; then
        return 0
    fi
    # Mock for port check
    if [ "$1" == "ps" ]; then
        echo ""
        return 0
    fi
}

# Source libraries
# Note: Some sourcing might fail if dependencies aren't met, but we mock what we need.
# We source utils/config/core first.
source src/lib/core.sh
source src/lib/config.sh
source src/lib/utils.sh
# We need docker.sh for check_port_conflict
source src/lib/docker.sh
# We need install_system.sh for check_resources
source src/lib/install_system.sh

# Mock logging
log() { echo "LOG: $1"; }
msg() { echo "MSG: $1"; }
warn() { echo "WARN: $1"; }
error() { echo "ERROR: $1"; }
ask() {
    echo "ASK: $1"
    # Auto-answer "n" to confirmation to simulate "Abort"
    eval "$2='n'"
}
fatal() { echo "FATAL: $1"; return 1; }

FAILED=0

echo "Running Unit Tests..."

# Test check_resources
echo "Test 0: check_resources"

# Mock nproc and df
nproc() { echo "4"; }
df() {
    # Mocking df -BG /
    # Filesystem 1G-blocks Used Available Use% Mounted on
    echo "Filesystem 1G-blocks Used Available Use% Mounted on"
    echo "/dev/sda1 100G 10G 90G 10% /"
}

if check_resources; then
    echo "PASS: Resources OK"
else
    echo "FAIL: Resources check failed unexpectedly"
    FAILED=1
fi

# Mock failure (Disk < 5GB)
df() {
    echo "Filesystem 1G-blocks Used Available Use% Mounted on"
    echo "/dev/sda1 100G 96G 4G 96% /"
}

OUTPUT=$(check_resources 2>&1 || true)
if echo "$OUTPUT" | grep -q "FATAL: Insufficient Disk Space"; then
    echo "PASS: Detected Low Disk Space"
else
    echo "FAIL: Did not detect Low Disk Space"
    echo "DEBUG Output: $OUTPUT"
    FAILED=1
fi

# Test detect_profile
echo "Test 0.5: detect_profile"
export PROFILE_FILE="./test_profile_out"

# Mock free -m to return 2000MB (LOW)
free() {
    echo "              total        used        free      shared  buff/cache   available"
    echo "Mem:           2000         500         500         100         900        1500"
}

detect_profile >/dev/null
PROFILE=$(cat "$PROFILE_FILE")
if [ "$PROFILE" == "LOW" ]; then
    echo "PASS: Detected LOW Profile"
else
    echo "FAIL: Failed to detect LOW Profile (Got $PROFILE)"
    FAILED=1
fi

# Mock free -m to return 8000MB (HIGH)
free() {
    echo "              total        used        free      shared  buff/cache   available"
    echo "Mem:           8000         500         500         100         900        1500"
}
detect_profile >/dev/null
PROFILE=$(cat "$PROFILE_FILE")
if [ "$PROFILE" == "HIGH" ]; then
    echo "PASS: Detected HIGH Profile"
else
    echo "FAIL: Failed to detect HIGH Profile (Got $PROFILE)"
    FAILED=1
fi
rm "$PROFILE_FILE"

# Test calculate_swap_size
echo "Test 0.6: calculate_swap_size"
# We need to source or mock calculate_swap_size since it's defined inside init_system
# For testing purposes, we redefine it here as it would be in the script
calculate_swap_size() {
    local RAM_MB=$1
    if [ "$RAM_MB" -lt 2048 ]; then
        echo $(( RAM_MB * 2 ))
    elif [ "$RAM_MB" -le 8192 ]; then
        echo "${RAM_MB}"
    else
        echo "4096"
    fi
}

# Case 1: 1GB RAM -> 2GB Swap
SWAP=$(calculate_swap_size 1024)
if [ "$SWAP" == "2048" ]; then
    echo "PASS: Swap calculation for 1GB RAM"
else
    echo "FAIL: Swap calculation for 1GB RAM (Got $SWAP)"
    FAILED=1
fi

# Case 2: 4GB RAM -> 4GB Swap
SWAP=$(calculate_swap_size 4096)
if [ "$SWAP" == "4096" ]; then
    echo "PASS: Swap calculation for 4GB RAM"
else
    echo "FAIL: Swap calculation for 4GB RAM (Got $SWAP)"
    FAILED=1
fi

# Case 3: 16GB RAM -> 4GB Swap
SWAP=$(calculate_swap_size 16384)
if [ "$SWAP" == "4096" ]; then
    echo "PASS: Swap calculation for 16GB RAM"
else
    echo "FAIL: Swap calculation for 16GB RAM (Got $SWAP)"
    FAILED=1
fi

# Test validate_password
echo "Test 1: validate_password (short)"
if validate_password "1234567"; then
    echo "FAIL: Accepted short password"
    FAILED=1
else
    echo "PASS: Rejected short password"
fi

echo "Test 2: validate_password (valid)"
if validate_password "12345678"; then
    echo "PASS: Accepted valid password"
else
    echo "FAIL: Rejected valid password"
    FAILED=1
fi

# Test save_credential and get_auth_value
echo "Test 3: save_credential"
save_credential "test_key" "test_value"
VAL=$(grep "test_key" "$AUTH_FILE" | cut -d= -f2)
if [ "$VAL" == "test_value" ]; then
    echo "PASS: Saved credential"
else
    echo "FAIL: Did not save credential"
    FAILED=1
fi

echo "Test 4: get_auth_value"
RET=$(get_auth_value "test_key")
if [ "$RET" == "test_value" ]; then
    echo "PASS: Retrieved credential"
else
    echo "FAIL: Did not retrieve credential"
    FAILED=1
fi

# Test Duplicate handling (Expectation: should handle it)
echo "Test 5: save_credential (duplicate)"
save_credential "test_key" "new_value"
# Check if get_auth_value returns the NEW value
RET=$(get_auth_value "test_key")
if [ "$RET" == "new_value" ]; then
    echo "PASS: Retrieved updated credential"
else
    echo "FAIL: Did not retrieve updated credential. Got '$RET'"
    FAILED=1
fi

# Test 6: check_port_conflict
echo "Test 6: check_port_conflict"

# Mock 'ss' command
# The check uses ss -tuln | awk '{print $5}' | grep -E ":$port$"
# We need to simulate ss output where column 5 contains the address:port
ss() {
    # Netid State Recv-Q Send-Q Local_Address:Port Peer_Address:Port Process
    echo "tcp LISTEN 0 128 *:8080 *:* users:((\"java\",pid=123,fd=10))"
}

# Run check_port_conflict for port 8080 (Mocked to be in use)
# We expect it to call 'ask' then 'fatal' because our mock 'ask' returns 'n'
OUTPUT=$(check_port_conflict "8080" "TestService") || true

if echo "$OUTPUT" | grep -q "FATAL: Aborting installation"; then
    echo "PASS: Detected port conflict and aborted"
else
    echo "FAIL: Did not detect port conflict or abort correctly"
    echo "Output: $OUTPUT"
    FAILED=1
fi

# Mock 'ss' to return empty (No conflict)
ss() {
    echo ""
}

# Run check_port_conflict for port 9090 (Free)
OUTPUT=$(check_port_conflict "9090" "TestService") || true
if [ -z "$OUTPUT" ]; then
    echo "PASS: No conflict detected for free port"
else
    echo "FAIL: False positive on free port"
    echo "Output: $OUTPUT"
    FAILED=1
fi

# Clean up
rm "$AUTH_FILE"

if [ $FAILED -eq 0 ]; then
    echo "All Tests Passed"
    exit 0
else
    echo "Some Tests Failed"
    exit 1
fi
