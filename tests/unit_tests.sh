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
source src/lib/core.sh
source src/lib/config.sh
source src/lib/utils.sh
source src/lib/docker.sh
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

# We need to redefine calculate_swap_size to use the sourced version's logic
# But the sourced version calls `free` and `df`, so we mock `free` and `df`.

# Case 1: 4GB RAM, 50GB Free Disk (Standard Case) -> 4GB Swap
free() { echo "Mem: 4096"; }
df() {
    # 50GB Free = 51200 MB
    echo "Filesystem 1M-blocks Used Available Use% Mounted on"
    echo "/dev/sda1 100000 48800 51200 50% /"
}

SWAP=$(calculate_swap_size)
if [ "$SWAP" == "4096" ]; then
    echo "PASS: Standard Swap (4GB RAM -> 4GB Swap)"
else
    echo "FAIL: Standard Swap (Got $SWAP)"
    FAILED=1
fi

# Case 2: 4GB RAM, 6GB Free Disk (Safety Cap 50%) -> 3GB Swap (Max safe = 3072)
# Wait, Safety Cap: Max 50% of free. 6GB Free -> 3GB Swap.
# Small Disk Logic: If Free < 10GB (10240MB), Cap at 1GB IF ideal > 1GB.
# So here: Free = 6144 MB (< 10GB). Ideal = 4096.
# Small Disk Logic triggers: Cap at 1024.
# Safety Cap triggers: 3072.
# Result should be 1024.

free() { echo "Mem: 4096"; }
df() {
    # 6GB Free = 6144 MB
    echo "Filesystem 1M-blocks Used Available Use% Mounted on"
    echo "/dev/sda1 10000 3856 6144 60% /"
}
SWAP=$(calculate_swap_size)
if [ "$SWAP" == "1024" ]; then
    echo "PASS: Small Disk Cap (6GB Free -> 1GB Swap)"
else
    echo "FAIL: Small Disk Cap (Expected 1024, Got $SWAP)"
    FAILED=1
fi

# Case 3: 4GB RAM, 12GB Free Disk (> 10GB but 50% cap check)
# Free = 12288 MB (> 10GB). Small Disk Logic OFF.
# Ideal = 4096.
# Safety Cap (50%) = 6144.
# Result = 4096.
free() { echo "Mem: 4096"; }
df() {
    # 12GB Free = 12288 MB
    echo "Filesystem 1M-blocks Used Available Use% Mounted on"
    echo "/dev/sda1 20000 7712 12288 40% /"
}
SWAP=$(calculate_swap_size)
if [ "$SWAP" == "4096" ]; then
    echo "PASS: Medium Disk (12GB Free -> 4GB Swap)"
else
    echo "FAIL: Medium Disk (Expected 4096, Got $SWAP)"
    FAILED=1
fi

# Case 4: 1GB RAM, 6GB Free Disk
# Ideal = 2GB.
# Free = 6144 MB (< 10GB).
# Small Disk Logic: If Free < 10GB. Ideal (2048) > 1024. Cap at 1024.
free() { echo "Mem: 1024"; }
df() {
    # 6GB Free = 6144 MB
    echo "Filesystem 1M-blocks Used Available Use% Mounted on"
    echo "/dev/sda1 10000 3856 6144 60% /"
}
SWAP=$(calculate_swap_size)
if [ "$SWAP" == "1024" ]; then
    echo "PASS: Small RAM on Small Disk (1GB RAM -> 1GB Swap)"
else
    echo "FAIL: Small RAM on Small Disk (Expected 1024, Got $SWAP)"
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

# Test Duplicate handling
echo "Test 5: save_credential (duplicate)"
save_credential "test_key" "new_value"
RET=$(get_auth_value "test_key")
if [ "$RET" == "new_value" ]; then
    echo "PASS: Retrieved updated credential"
else
    echo "FAIL: Did not retrieve updated credential. Got '$RET'"
    FAILED=1
fi

# Test 6: check_port_conflict
echo "Test 6: check_port_conflict"
ss() {
    # Netid State Recv-Q Send-Q Local_Address:Port Peer_Address:Port Process
    echo "tcp LISTEN 0 128 *:8080 *:* users:((\"java\",pid=123,fd=10))"
}
OUTPUT=$(check_port_conflict "8080" "TestService") || true

if echo "$OUTPUT" | grep -q "FATAL: Aborting installation"; then
    echo "PASS: Detected port conflict and aborted"
else
    echo "FAIL: Did not detect port conflict or abort correctly"
    echo "Output: $OUTPUT"
    FAILED=1
fi

ss() {
    echo ""
}
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
