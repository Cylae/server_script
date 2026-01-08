#!/bin/bash

# Open fd 3
exec 3>&1

# Mock environment vars
export CONFIG_FILE="./test_config.conf"
export AUTH_FILE="./test_auth_details"
export LOG_FILE="./test.log"
export DOMAIN="test.com"
export EMAIL="admin@test.com"
export DB_ROOT_PASS="mockrootpass"

# Clear previous test artifacts
rm -f "$CONFIG_FILE" "$AUTH_FILE" "$LOG_FILE"
rm -rf ./test_opt
rm -rf ./test_nginx
mkdir -p ./test_opt
mkdir -p ./test_nginx/sites-available
mkdir -p ./test_nginx/sites-enabled
touch ./test_auth_details

# Mocks
docker() { echo "[MOCK] docker $*"; return 0; }
systemctl() { echo "[MOCK] systemctl $*"; return 0; }
apt-get() { echo "[MOCK] apt-get $*"; return 0; }
certbot() { echo "[MOCK] certbot $*"; return 0; }
htpasswd() { echo "[MOCK] htpasswd $*"; echo "mockhash"; return 0; }
mysql() { echo "[MOCK] mysql $*"; return 0; }
ufw() { echo "[MOCK] ufw $*"; return 0; }
curl() { echo "1.2.3.4"; }
openssl() { echo "mock_secret_string"; }

# Mock ask function
INPUT_QUEUE=()
ask() {
    local prompt="$1"
    local var_name="$2"
    echo "PROMPT: $prompt"
    if [ ${#INPUT_QUEUE[@]} -gt 0 ]; then
        local val="${INPUT_QUEUE[0]}"
        INPUT_QUEUE=("${INPUT_QUEUE[@]:1}")
        eval "$var_name=\"$val\""
        echo "INPUT: $val"
    else
        eval "$var_name=\"\""
        echo "INPUT: (empty)"
    fi
}

# Process install.sh
sed -n '/^# 8. ENTRY POINT/q;p' ./install.sh > ./temp_functions.sh
# Remove ask
sed -i '/^ask() {/,/^}/d' ./temp_functions.sh
# Fix paths (absolute paths would be safer but let's stick to relative and subshells)
# We will use absolute paths for the mock dirs to be safe
CWD=$(pwd)
sed -i "s|/opt|$CWD/test_opt|g" ./temp_functions.sh
sed -i "s|/etc/nginx|$CWD/test_nginx|g" ./temp_functions.sh
sed -i "s|/root/.auth_details|$CWD/test_auth_details|g" ./temp_functions.sh
sed -i "s|/var/log/server_manager.log|$CWD/test.log|g" ./temp_functions.sh

source ./temp_functions.sh

# Init required vars
AUTH_FILE="$CWD/test_auth_details"

echo "--- TEST 1: ask_credential_pair ---"
INPUT_QUEUE=("myuser" "mypass")
ask_credential_pair "TestService" "default_admin" TEST_USER TEST_PASS "test_svc"
grep "test_svc_user=myuser" "$AUTH_FILE" && echo "PASS: User saved" || echo "FAIL: User not saved"

echo "--- TEST 2: manage_gitea install ---"
INPUT_QUEUE=("gitea_custom" "gitea_pass" "n")
# Run in subshell to avoid PWD change affecting test script
( manage_gitea "install" )

if [ -f "./test_opt/gitea/docker-compose.yml" ]; then
    echo "PASS: Gitea compose created"
else
    echo "FAIL: Gitea compose not created"
fi

echo "--- TEST 4: sync_infrastructure ---"
touch ./test_nginx/sites-enabled/git.test.com
sync_infrastructure
