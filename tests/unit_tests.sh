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
ss() {
    echo "LISTEN 0 128 *:8080 *:* users:((\"java\",pid=123,fd=10))"
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
