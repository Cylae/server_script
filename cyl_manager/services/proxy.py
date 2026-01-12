import os
import logging
from ..core import system
from ..core.docker_manager import create_network

logger = logging.getLogger("Proxy")

NGINX_DIR = "/etc/nginx"
SITES_AVAIL = os.path.join(NGINX_DIR, "sites-available")
SITES_ENABLED = os.path.join(NGINX_DIR, "sites-enabled")

def install_nginx():
    """
    Installs Nginx and Certbot.
    """
    system.install_apt_packages(['nginx', 'certbot', 'python3-certbot-nginx'])

    # Ensure directories
    os.makedirs(SITES_AVAIL, exist_ok=True)
    os.makedirs(SITES_ENABLED, exist_ok=True)

    # Clean default
    if os.path.exists(os.path.join(SITES_ENABLED, "default")):
        os.remove(os.path.join(SITES_ENABLED, "default"))

    system.run_command("systemctl enable nginx", check=False)
    system.run_command("systemctl start nginx", check=False)

def update_nginx(domain, port, service_type="standard", proxy_protocol="http"):
    """
    Creates an Nginx configuration for a domain.
    """
    if not os.path.exists(SITES_AVAIL):
        install_nginx()

    conf_path = os.path.join(SITES_AVAIL, domain)

    client_max_body = "10G" if service_type == "cloud" else "512M"

    config = f"""
server {{
    listen 80;
    server_name {domain};

    location / {{
        proxy_pass {proxy_protocol}://127.0.0.1:{port};
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Websockets
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }}

    client_max_body_size {client_max_body};
}}
"""
    with open(conf_path, "w") as f:
        f.write(config)

    # Enable site
    link_path = os.path.join(SITES_ENABLED, domain)
    if not os.path.exists(link_path):
        os.symlink(conf_path, link_path)

    # Test and Reload
    try:
        system.run_command("nginx -t")
        system.run_command("systemctl reload nginx")
    except Exception as e:
        logger.error(f"Nginx configuration failed for {domain}: {e}")
        # Rollback
        if os.path.exists(link_path): os.remove(link_path)
        if os.path.exists(conf_path): os.remove(conf_path)
        raise

def secure_domain(domain, email):
    """
    Runs certbot to secure the domain.
    """
    try:
        cmd = f"certbot --nginx -d {domain} --non-interactive --agree-tos -m {email} --redirect"
        system.run_command(cmd, shell=True)
    except Exception as e:
        logger.error(f"Certbot failed for {domain}: {e}")
