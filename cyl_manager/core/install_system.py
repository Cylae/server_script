import os
import subprocess
import shutil
import math
import multiprocessing
from .utils import msg, warn, success, check_command, fatal, is_port_open

def check_resources():
    """Checks CPU and Disk resources."""
    cpu_cores = multiprocessing.cpu_count()
    if cpu_cores < 2:
        warn(f"Low CPU Core count detected ({cpu_cores}). Recommended: 2+")

    try:
        total, used, free = shutil.disk_usage("/")
        free_gb = free // (2**30)
        if free_gb < 5:
            warn(f"Low Disk Space. Free: {free_gb}GB. Recommended: 5GB+")
    except:
        warn("Could not detect disk space. Skipping check.")

def calculate_swap_size():
    """Calculates ideal swap size."""
    # Memory in MB
    with open("/proc/meminfo", "r") as f:
        mem_total_kb = int(f.readline().split()[1])
    ram_mb = mem_total_kb // 1024

    total, used, free = shutil.disk_usage("/")
    free_disk_mb = free // (2**20)

    if ram_mb < 2048:
        ideal_swap = ram_mb * 2
    elif ram_mb <= 8192:
        ideal_swap = ram_mb
    else:
        ideal_swap = 4096

    max_safe_swap = free_disk_mb // 2

    if free_disk_mb < 10240:
        if ideal_swap > 1024:
            ideal_swap = 1024

    return min(ideal_swap, max_safe_swap)

def init_system_resources():
    """Initializes system resources (dependencies, swap, tuning)."""
    check_resources()
    msg("Initializing System Infrastructure...")

    # 1. Basics
    dependencies = [
        "jq", "pigz", "curl", "wget", "git", "unzip", "gnupg2",
        "apt-transport-https", "ca-certificates", "lsb-release",
        "ufw", "sudo", "htop", "apache2-utils", "fail2ban",
        "bc", "iproute2", "ncurses-bin", "unattended-upgrades",
        "nginx", "mariadb-server", "certbot", "python3-certbot-nginx",
        "php-fpm", "php-mysql", "php-curl", "php-gd", "php-mbstring",
        "php-xml", "php-zip", "php-intl", "php-bcmath"
    ]

    # Simple check if any dependency is missing could be slow, so we just run install if anything seems missing
    # or just run update/install always to ensure latest state
    msg("Installing System Dependencies...")
    env = os.environ.copy()
    env["DEBIAN_FRONTEND"] = "noninteractive"
    subprocess.run(["apt-get", "update", "-q"], env=env, check=True)
    subprocess.run(["apt-get", "install", "-y"] + dependencies, env=env, check=True)

    # Configure Unattended Upgrades
    with open("/etc/apt/apt.conf.d/20auto-upgrades", "w") as f:
        f.write('APT::Periodic::Update-Package-Lists "1";\n')
        f.write('APT::Periodic::Unattended-Upgrade "1";\n')

    # Configure MariaDB to listen on all interfaces (important for Docker containers)
    if os.path.exists("/etc/mysql/mariadb.conf.d/50-server.cnf"):
        subprocess.run(r"sed -i 's/^\s*bind-address.*/bind-address = 0.0.0.0/' /etc/mysql/mariadb.conf.d/50-server.cnf", shell=True)
        try:
            subprocess.run(["systemctl", "restart", "mariadb"], check=True)
        except subprocess.CalledProcessError:
            warn("MariaDB restart failed. Attempting to diagnose...")
            if is_port_open(3306):
                warn("Port 3306 is in use. Check for conflicting services (e.g. Docker containers).")

            # Attempt to revert configuration
            warn("Attempting to revert MariaDB configuration to default (127.0.0.1)...")
            subprocess.run(r"sed -i 's/bind-address = 0.0.0.0/bind-address = 127.0.0.1/' /etc/mysql/mariadb.conf.d/50-server.cnf", shell=True)
            try:
                subprocess.run(["systemctl", "restart", "mariadb"], check=True)
                warn("Reverted MariaDB to localhost binding. Docker connectivity to MariaDB will be limited.")
                success("MariaDB recovered.")
            except subprocess.CalledProcessError:
                logs = ""
                try:
                    # Capture more logs and check error log file
                    res = subprocess.run("journalctl -xeu mariadb.service --no-pager | tail -n 50", shell=True, capture_output=True, text=True)
                    logs = res.stdout
                    if os.path.exists("/var/log/mysql/error.log"):
                         res_err = subprocess.run("tail -n 20 /var/log/mysql/error.log", shell=True, capture_output=True, text=True)
                         logs += "\n--- /var/log/mysql/error.log ---\n" + res_err.stdout
                except:
                    logs = "Could not retrieve logs."

                fatal(f"MariaDB failed to restart even after revert.\nLogs:\n{logs}")

    # 2. Swap & BBR
    if not os.path.exists("/swapfile"):
        msg("Creating Swap File (Smart Size)...")
        swap_mb = calculate_swap_size()
        msg(f"Allocating Swap: {swap_mb}MB")

        try:
            subprocess.run(["fallocate", "-l", f"{swap_mb}M", "/swapfile"], check=True)
        except subprocess.CalledProcessError:
             subprocess.run(f"dd if=/dev/zero of=/swapfile bs=1M count={swap_mb} status=progress", shell=True, check=True)

        os.chmod("/swapfile", 0o600)
        subprocess.run(["mkswap", "/swapfile"], check=True)
        subprocess.run(["swapon", "/swapfile"], check=True)
        with open("/etc/fstab", "a") as f:
            f.write("/swapfile none swap sw 0 0\n")

    # Sysctl
    sysctl_conf = """
# ------------------------------------------------------------------------------
# CYLAE OPTIMIZED NETWORK STACK
# ------------------------------------------------------------------------------
# BBR Congestion Control
net.core.default_qdisc=fq
net.ipv4.tcp_congestion_control=bbr

# TCP Stack Tuning
net.ipv4.tcp_fastopen=3
net.ipv4.tcp_slow_start_after_idle=0
net.ipv4.tcp_tw_reuse=1
net.ipv4.tcp_max_syn_backlog=8192
net.core.somaxconn=8192
net.core.netdev_max_backlog=16384
fs.file-max=100000
# ------------------------------------------------------------------------------
"""
    with open("/etc/sysctl.d/99-cylae-network.conf", "w") as f:
        f.write(sysctl_conf)
    subprocess.run(["sysctl", "--system"], stdout=subprocess.DEVNULL)

    # 3. DNS Optimization
    if os.path.exists("/etc/systemd/resolved.conf"):
        # We use a simple replacement strategy
        subprocess.run(r"sed -i 's/^#\?DNS=.*/DNS=8.8.8.8 8.8.4.4 1.1.1.1 1.0.0.1/' /etc/systemd/resolved.conf", shell=True)
        subprocess.run(r"sed -i 's/^#\?FallbackDNS=.*/FallbackDNS=1.1.1.1 1.0.0.1/' /etc/systemd/resolved.conf", shell=True)
        subprocess.run("systemctl restart systemd-resolved", shell=True, stderr=subprocess.DEVNULL)

    # 4. Docker Engine (Official)
    from .docker import init_docker, get as get_config
    init_docker()

    # Allow Docker subnet to access Host MariaDB
    # Need to get subnet
    docker_net = get_config("DOCKER_NET")
    try:
        res = subprocess.run(f"docker network inspect {docker_net} | jq -r '.[0].IPAM.Config[0].Subnet'", shell=True, capture_output=True, text=True)
        subnet = res.stdout.strip()
        if subnet:
            subprocess.run(f"ufw allow from {subnet} to any port 3306", shell=True, stdout=subprocess.DEVNULL)
    except Exception as e:
        warn(f"Could not configure firewall for MariaDB: {e}")

    success("System Initialized")
