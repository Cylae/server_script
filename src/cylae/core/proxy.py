import os
import glob
import logging
from .shell import run_command
from .config import config

logger = logging.getLogger(__name__)

SEC_HEADERS = """
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains; preload" always;
    add_header Permissions-Policy "interest-cohort=()" always;
"""

OPT_CONFIG = """
    gzip on;
    gzip_vary on;
    gzip_proxied any;
    gzip_comp_level 6;
    gzip_types text/plain text/css text/xml application/json application/javascript application/rss+xml application/atom+xml image/svg+xml;

    keepalive_timeout 65;
"""

def update_nginx(subdomain, port, type="proxy", root=None):
    """
    Generates an Nginx configuration file.
    """
    logger.info(f"Updating Nginx config for {subdomain}...")

    # Clean default
    if os.path.exists("/etc/nginx/sites-enabled/default"):
        os.remove("/etc/nginx/sites-enabled/default")

    config_path = f"/etc/nginx/sites-available/{subdomain}"

    # Body size logic
    body_size = "10G" if "cloud" in subdomain else "512M"

    with open(config_path, "w") as f:
        f.write(f"server {{\n")
        f.write(f"    listen 80;\n")
        f.write(f"    listen [::]:80;\n")
        f.write(f"    server_name {subdomain};\n\n")
        f.write(f"    client_max_body_size {body_size};\n")
        f.write(f"    client_body_buffer_size 512k;\n\n")
        f.write(SEC_HEADERS)
        f.write(OPT_CONFIG)
        f.write("\n")

        if type == "proxy":
            f.write("    location / {\n")
            f.write(f"        proxy_pass http://127.0.0.1:{port};\n")
            f.write("        proxy_set_header Host $host;\n")
            f.write("        proxy_set_header X-Real-IP $remote_addr;\n")
            f.write("        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;\n")
            f.write("        proxy_set_header X-Forwarded-Proto $scheme;\n")
            f.write("        proxy_set_header Upgrade $http_upgrade;\n")
            f.write('        proxy_set_header Connection "upgrade";\n')
            f.write("    }\n")
        elif type == "php" or type == "dashboard":
            # Find PHP sock - this is a bit rough in python without scanning,
            # but we can assume standard paths or use glob.
            php_socks = glob.glob("/run/php/php*-fpm.sock")
            php_sock = php_socks[0] if php_socks else "/run/php/php8.1-fpm.sock" # Default fallback

            f.write(f"    root {root};\n")
            f.write("    index index.php index.html;\n")

            if type == "dashboard":
                 f.write('    location / { try_files $uri $uri/ =404; auth_basic "Restricted"; auth_basic_user_file /etc/nginx/.htpasswd; }\n')
            else:
                 f.write("    location / { try_files $uri $uri/ =404; }\n")

            f.write(r"    location ~ \.php$ {" + "\n")
            f.write("        include snippets/fastcgi-php.conf;\n")
            f.write(f"        fastcgi_pass unix:{php_sock};\n")
            f.write("    }\n")

        f.write("}\n")

    # Link
    enabled_path = f"/etc/nginx/sites-enabled/{subdomain}"
    if not os.path.exists(enabled_path):
        os.symlink(config_path, enabled_path)

    # Reload (optional here, usually done in sync)
    # run_command("systemctl reload nginx")

def sync_infrastructure():
    """
    Reloads Nginx and runs Certbot.
    """
    logger.info("Syncing infrastructure...")
    run_command("systemctl reload nginx")

    # Find domains
    domains = []
    files = glob.glob("/etc/nginx/sites-enabled/*")
    for filepath in files:
        with open(filepath, 'r') as f:
            for line in f:
                if "server_name" in line:
                    parts = line.strip().split()
                    if len(parts) >= 2:
                        domain = parts[1].strip(';')
                        if domain != "_":
                            domains.append(domain)

    if domains:
        email = config.get("EMAIL", "admin@example.com") # Should ask user in real scenario
        domain_args = []
        for d in domains:
            domain_args.extend(["-d", d])

        cmd = ["certbot", "--nginx", "--non-interactive", "--agree-tos", "-m", email, "--expand"] + domain_args
        logger.info(f"Requesting certs for: {', '.join(domains)}")
        try:
            run_command(cmd)
        except Exception as e:
            logger.error(f"Certbot failed: {e}")

    logger.info("Infrastructure synced.")
