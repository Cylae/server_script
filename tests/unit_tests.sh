#!/bin/bash
# Unit tests for core functions

# Mock environment
AUTH_FILE="./test_auth_file"
touch "$AUTH_FILE"
source src/lib/core.sh
source src/lib/config.sh
source src/lib/utils.sh

# Mock logging
log() { echo "LOG: $1"; }
msg() { echo "MSG: $1"; }
warn() { echo "WARN: $1"; }
error() { echo "ERROR: $1"; }

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

# Test Duplicate handling (Expectation: should handle it, but currently it appends)
echo "Test 5: save_credential (duplicate)"
save_credential "test_key" "new_value"
# Check if get_auth_value returns the NEW value (tail -n 1)
RET=$(get_auth_value "test_key")
if [ "$RET" == "new_value" ]; then
    echo "PASS: Retrieved updated credential"
else
    echo "FAIL: Did not retrieve updated credential. Got '$RET'"
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
