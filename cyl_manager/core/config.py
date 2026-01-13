import os

# Constants
CONFIG_FILE = "/etc/cyl_manager.conf"
AUTH_FILE = "/root/.auth_details"
LOG_FILE = "/var/log/server_manager.log"

# Default configuration
DEFAULT_CONFIG = {
    "DOMAIN": "example.com",
    "EMAIL": "admin@example.com",
    "DOCKER_NET": "server-net"
}

_config_cache = {}

def load_config():
    """Loads configuration from the config file."""
    global _config_cache
    config = DEFAULT_CONFIG.copy()
    if os.path.exists(CONFIG_FILE):
        with open(CONFIG_FILE, "r") as f:
            for line in f:
                if "=" in line:
                    key, value = line.strip().split("=", 1)
                    config[key] = value
    _config_cache = config
    return config

def get(key):
    """Gets a configuration value."""
    if not _config_cache:
        load_config()
    return _config_cache.get(key)

def set_config(key, value):
    """Sets a configuration value and saves it."""
    if not _config_cache:
        load_config()
    _config_cache[key] = value
    save_config()

def save_config():
    """Saves the current configuration to the config file."""
    # Ensure config file is writable or try to write to a local file if permission denied
    target_file = CONFIG_FILE
    try:
        with open(target_file, "a"):
            pass
    except PermissionError:
        target_file = "cyl_manager.conf"

    with open(target_file, "w") as f:
        for key, value in _config_cache.items():
            f.write(f"{key}={value}\n")

def get_auth_details():
    """Reads auth details from .auth_details file."""
    if os.path.exists(AUTH_FILE):
        with open(AUTH_FILE, "r") as f:
            return f.read()
    return ""
