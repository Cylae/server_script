import sys
import shutil
import logging
import time

# ANSI Color Codes
GREEN = "\033[0;32m"
RED = "\033[0;31m"
YELLOW = "\033[1;33m"
CYAN = "\033[0;36m"
NC = "\033[0m" # No Color

logging.basicConfig(
    filename="/var/log/server_manager.log",
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)

def log_info(message):
    logging.info(message)

def log_error(message):
    logging.error(message)

def msg(message):
    print(f"{GREEN}[INFO]{NC} {message}")
    log_info(message)

def success(message):
    print(f"{GREEN}[SUCCESS]{NC} {message}")
    log_info(f"SUCCESS: {message}")

def warn(message):
    print(f"{YELLOW}[WARN]{NC} {message}")
    log_info(f"WARNING: {message}")

def error(message):
    print(f"{RED}[ERROR]{NC} {message}")
    log_error(message)

def fatal(message):
    print(f"{RED}[FATAL]{NC} {message}")
    log_error(f"FATAL: {message}")
    sys.exit(1)

def ask(question, default=None):
    """Asks a question to the user."""
    prompt = f"{question}"
    if default:
        prompt += f" [{default}]"
    prompt += ": "

    try:
        response = input(prompt)
        if not response and default:
            return default
        return response
    except KeyboardInterrupt:
        print("\nOperation cancelled.")
        sys.exit(0)

def check_command(command):
    """Checks if a command exists in the path."""
    return shutil.which(command) is not None

def header_art():
    print(f"""{CYAN}
   ______      __
  / ____/_  __/ /___ ______
 / /   / / / / / __ `/ ___/
/ /___/ /_/ / / /_/ / /__
\____/\__, /_/\__,_/\___/
     /____/
    {NC}""")
