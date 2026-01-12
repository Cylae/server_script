import os
import secrets
import string
import logging

logger = logging.getLogger("Config")

AUTH_FILE = "/root/.auth_details"
CONFIG_FILE = "/etc/cyl_manager.conf"

def generate_password(length=16):
    alphabet = string.ascii_letters + string.digits
    while True:
        password = ''.join(secrets.choice(alphabet) for i in range(length))
        if (any(c.islower() for c in password)
                and any(c.isupper() for c in password)
                and sum(c.isdigit() for c in password) >= 1):
            return password

def get_auth_value(key, default=None):
    if not os.path.exists(AUTH_FILE):
        return default

    with open(AUTH_FILE, 'r') as f:
        for line in f:
            if line.startswith(f"{key}="):
                return line.strip().split('=', 1)[1]
    return default

def save_auth_value(key, value):
    lines = []
    found = False
    if os.path.exists(AUTH_FILE):
        with open(AUTH_FILE, 'r') as f:
            lines = f.readlines()

    with open(AUTH_FILE, 'w') as f:
        os.chmod(AUTH_FILE, 0o600)
        for line in lines:
            if line.startswith(f"{key}="):
                f.write(f"{key}={value}\n")
                found = True
            else:
                f.write(line)
        if not found:
            f.write(f"{key}={value}\n")

def get_or_create_password(key):
    val = get_auth_value(key)
    if not val:
        val = generate_password()
        save_auth_value(key, val)
    return val

def load_config():
    config = {}
    if os.path.exists(CONFIG_FILE):
        with open(CONFIG_FILE, 'r') as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith('#') and '=' in line:
                    k, v = line.split('=', 1)
                    config[k.strip()] = v.strip()
    return config

def save_config(config):
    with open(CONFIG_FILE, 'w') as f:
        for k, v in config.items():
            f.write(f"{k}={v}\n")
