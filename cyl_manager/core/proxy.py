import os
import subprocess
from pathlib import Path
from jinja2 import Template
from .logger import logger
from .config import config

class ProxyManager:
    def __init__(self):
        self.certbot_email = config.EMAIL
        self.domain = config.DOMAIN
        self.nginx_conf_dir = Path("/etc/nginx/sites-available")
        self.nginx_enabled_dir = Path("/etc/nginx/sites-enabled")

    def ensure_infrastructure(self):
        """Installs Nginx and Certbot."""
        if subprocess.run("which nginx", shell=True).returncode != 0:
            logger.info("Installing Nginx...")
            subprocess.run("apt-get update -q && apt-get install -y nginx", shell=True, check=True)

        if subprocess.run("which certbot", shell=True).returncode != 0:
            logger.info("Installing Certbot...")
            subprocess.run("apt-get update -q && apt-get install -y certbot python3-certbot-nginx", shell=True, check=True)

    def generate_config(self, service_name: str, port: str):
        """Generates Nginx configuration for a service."""
        try:
            # Check if we can run apt/install things (skip in test/sandbox if not root or mocked)
            if os.geteuid() == 0:
                self.ensure_infrastructure()
            else:
                logger.warning("Not running as root, skipping Nginx installation/configuration.")
                return
        except Exception as e:
            logger.warning(f"Failed to ensure infrastructure: {e}")
            return

        full_domain = f"{service_name}.{self.domain}" if service_name != "home" else self.domain
        conf_file = self.nginx_conf_dir / full_domain

        template_str = """
server {
    listen 80;
    server_name {{ domain }};

    location / {
        proxy_pass http://127.0.0.1:{{ port }};
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
"""
        template = Template(template_str)
        content = template.render(domain=full_domain, port=port)

        try:
            self.nginx_conf_dir.mkdir(parents=True, exist_ok=True)
            self.nginx_enabled_dir.mkdir(parents=True, exist_ok=True)

            with open(conf_file, "w") as f:
                f.write(content)

            # Enable site
            enabled_link = self.nginx_enabled_dir / full_domain
            if not enabled_link.exists():
                os.symlink(conf_file, enabled_link)

            # Test and reload
            subprocess.run("nginx -t", shell=True, check=True)
            subprocess.run("systemctl reload nginx", shell=True, check=True)
            logger.info(f"Nginx configured for {full_domain}")

            # Obtain SSL
            # self.obtain_ssl(full_domain)
        except Exception as e:
            logger.error(f"Failed to configure Nginx for {full_domain}: {e}")

    def obtain_ssl(self, domain: str):
        if os.geteuid() != 0: return
        cmd = ["certbot", "--nginx", "-d", domain, "--non-interactive", "--agree-tos", "-m", self.certbot_email]
        try:
            subprocess.run(cmd, check=True)
        except subprocess.CalledProcessError as e:
            logger.error(f"Certbot failed for {domain}: {e}")

proxy_manager = ProxyManager()
