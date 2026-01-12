import subprocess
import os
import sys
import logging
import shutil
import platform
import glob

# Setup logging
logging.basicConfig(
    filename='/var/log/server_manager.log',
    level=logging.DEBUG,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
console_handler = logging.StreamHandler(sys.stdout)
console_handler.setLevel(logging.INFO)
# Only print info+ to console to keep it clean, user interaction handles the rest
logging.getLogger().addHandler(console_handler)

logger = logging.getLogger("System")

def run_command(command, shell=False, check=True, capture_output=True, env=None):
    """
    Runs a shell command.
    """
    logger.debug(f"Executing: {command}")
    try:
        # If shell is False and command is a string, split it (simple splitting)
        if not shell and isinstance(command, str):
            import shlex
            command = shlex.split(command)

        result = subprocess.run(
            command,
            shell=shell,
            check=check,
            stdout=subprocess.PIPE if capture_output else None,
            stderr=subprocess.PIPE if capture_output else None,
            text=True,
            env=env
        )
        return result
    except subprocess.CalledProcessError as e:
        logger.error(f"Command failed: {command}")
        logger.error(f"STDOUT: {e.stdout}")
        logger.error(f"STDERR: {e.stderr}")
        raise

def check_root():
    if os.geteuid() != 0:
        print("This script must be run as root.")
        sys.exit(1)

def install_apt_packages(packages):
    """
    Installs a list of apt packages.
    """
    if isinstance(packages, str):
        packages = [packages]

    # Filter out installed packages to save time?
    # Apt is smart enough, but we can check existence if we want speed.
    # For now, let apt handle it.

    logger.info(f"Installing packages: {', '.join(packages)}")
    try:
        env = os.environ.copy()
        env['DEBIAN_FRONTEND'] = 'noninteractive'
        run_command(['apt-get', 'update', '-q'], env=env, check=False) # Update first
        run_command(['apt-get', 'install', '-y'] + packages, env=env)
    except subprocess.CalledProcessError:
        logger.error("Failed to install packages.")
        raise

def get_ip():
    """
    Gets the primary IP address.
    """
    try:
        # Use ip route get 1
        result = run_command("ip -4 route get 1", shell=True)
        # Output looks like: "1.1.1.1 via 192.168.1.1 dev eth0 src 192.168.1.50 ..."
        # We want the 'src' value.
        output = result.stdout.strip()
        import re
        match = re.search(r'src\s+(\d+\.\d+\.\d+\.\d+)', output)
        if match:
            return match.group(1)
        return "127.0.0.1"
    except Exception as e:
        logger.error(f"Failed to get IP: {e}")
        return "127.0.0.1"

def get_public_ip():
    try:
        import urllib.request
        with urllib.request.urlopen('https://api.ipify.org', timeout=5) as response:
             return response.read().decode('utf-8')
    except:
        return get_ip()

def check_os():
    try:
        import distro
        return distro.id()
    except ImportError:
        # Fallback
        if os.path.exists('/etc/os-release'):
            with open('/etc/os-release') as f:
                content = f.read()
                if 'ID=debian' in content: return 'debian'
                if 'ID=ubuntu' in content: return 'ubuntu'
        return 'unknown'
