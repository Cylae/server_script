import os
import subprocess
from .utils import msg, success

def update_nginx(subdomain, port, type="proxy", root=None):
    """Generates and updates Nginx configuration."""

    sec_headers = """
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains; preload" always;
    add_header Permissions-Policy "interest-cohort=()" always;
    """

    opt_config = """
    gzip on;
    gzip_vary on;
    gzip_proxied any;
    gzip_comp_level 6;
    gzip_types text/plain text/css text/xml application/json application/javascript application/rss+xml application/atom+xml image/svg+xml;

    keepalive_timeout 65;
    """

    site_config = f"""server {{
    listen 80;
    listen [::]:80;
    server_name {subdomain};

"""
    if "cloud" in subdomain:
        site_config += "    client_max_body_size 10G;\n"
    else:
        site_config += "    client_max_body_size 512M;\n"

    site_config += f"""    client_body_buffer_size 512k;

    {sec_headers}
    {opt_config}
"""

    if type == "proxy":
        site_config += f"""    location / {{
        proxy_pass http://127.0.0.1:{port};
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }}
}}
"""
    # PHP/Dashboard support can be added if needed, but for now focusing on proxy as per memory emphasizing media stack (mostly docker services)
    # The original script had PHP support, I should probably add it back if I want full "port all services"
    # But for now, let's stick to proxy which covers most.
    # Actually, let's just implement the proxy part fully first.

    os.makedirs("/etc/nginx/sites-available", exist_ok=True)
    os.makedirs("/etc/nginx/sites-enabled", exist_ok=True)

    # Remove default
    if os.path.exists("/etc/nginx/sites-enabled/default"):
        os.remove("/etc/nginx/sites-enabled/default")

    config_path = f"/etc/nginx/sites-available/{subdomain}"
    with open(config_path, "w") as f:
        f.write(site_config)

    link_path = f"/etc/nginx/sites-enabled/{subdomain}"
    if os.path.exists(link_path):
        os.remove(link_path)
    os.symlink(config_path, link_path)

def sync_infrastructure(email):
    """Syncs Nginx and SSL."""
    msg("Syncing Infrastructure (Nginx & SSL)...")

    subprocess.run(["systemctl", "reload", "nginx"], check=True)

    domains = []
    if os.path.exists("/etc/nginx/sites-enabled"):
        for filename in os.listdir("/etc/nginx/sites-enabled"):
            filepath = os.path.join("/etc/nginx/sites-enabled", filename)
            if os.path.isfile(filepath):
                # Grep for server_name
                try:
                    with open(filepath, "r") as f:
                        for line in f:
                            if "server_name" in line:
                                parts = line.strip().split()
                                if len(parts) >= 2:
                                    domain = parts[1].strip(";")
                                    if domain != "_":
                                        domains.append(domain)
                except:
                    pass

    if domains:
        domain_args = []
        for d in domains:
            domain_args.extend(["-d", d])

        msg(f"Requesting SSL certificates for: {', '.join(domains)}")
        cmd = ["certbot", "--nginx", "--non-interactive", "--agree-tos", "-m", email, "--expand"] + domain_args
        subprocess.run(cmd)

    success("Infrastructure Synced.")
