import os
import subprocess
import shutil
import time
from .base import BaseService
from ..core.docker import deploy_service, remove_service, is_installed
from ..core.database import ensure_db, generate_password, save_credential, get_auth_value
from ..core.utils import ask, msg, fatal, warn, success, is_port_open
from ..core.config import get, get_auth_details

class MailService(BaseService):
    name = "mail"
    pretty_name = "Mail Server"
    port = "8081" # Webmail port

    def install(self):
        subdomain = f"mail.{self.domain}"

        # Check for port conflicts (specifically port 25)
        if is_port_open(25):
            warn("Port 25 is currently in use.")
            conflicting_service = None
            for svc in ["postfix", "exim4", "sendmail"]:
                if shutil.which("systemctl"):
                    res = subprocess.run(f"systemctl is-active {svc}", shell=True, stdout=subprocess.DEVNULL)
                    if res.returncode == 0:
                        conflicting_service = svc
                        break

            if conflicting_service:
                warn(f"Detected {conflicting_service} running on port 25.")
                if ask(f"Do you want to stop and disable {conflicting_service} to proceed? [Y/n]", "Y").lower() in ["y", "yes"]:
                    msg(f"Stopping {conflicting_service}...")
                    subprocess.run(f"systemctl stop {conflicting_service}", shell=True, check=True)
                    subprocess.run(f"systemctl disable {conflicting_service}", shell=True, check=True)
                    success(f"{conflicting_service} stopped.")
                else:
                    fatal("Cannot install Mail Server while port 25 is in use.")
            else:
                fatal("Port 25 is in use by an unknown service. Please free up port 25 and try again.")

        # Check profile for ClamAV/SpamAssassin
        enable_clamav = 1
        enable_spamassassin = 1

        if self.profile == "LOW":
            enable_clamav = 0
            enable_spamassassin = 0
            msg(f"Low Hardware Profile Detected ({self.profile}): Disabling ClamAV and SpamAssassin to prevent hang.")

        compose_content = f"""
services:
  mailserver:
    image: mailserver/docker-mailserver:latest
    hostname: mail
    domainname: {self.domain}
    container_name: mailserver
    networks:
      - {self.docker_net}
    ports:
      - "25:25"
      - "143:143"
      - "587:587"
      - "993:993"
    volumes:
      - ./maildata:/var/mail
      - ./mailstate:/var/mail-state
      - ./maillogs:/var/log/mail
      - ./config:/tmp/docker-mailserver
    environment:
      - ENABLE_SPAMASSASSIN={enable_spamassassin}
      - ENABLE_CLAMAV={enable_clamav}
      - ENABLE_FAIL2BAN=1
      - SSL_TYPE=letsencrypt
    cap_add:
      - NET_ADMIN
    restart: always
  roundcube:
    image: roundcube/roundcubemail:latest
    container_name: roundcube
    networks:
      - {self.docker_net}
    restart: always
    volumes:
      - ./roundcube/db:/var/www/db
    ports:
      - "127.0.0.1:8081:80"
    environment:
      - ROUNDCUBEMAIL_DEFAULT_HOST=mailserver
      - ROUNDCUBEMAIL_SMTP_SERVER=mailserver
networks:
  {self.docker_net}:
    external: true
"""
        deploy_service(self.name, self.pretty_name, compose_content.strip(), subdomain, self.port)

        # UFW
        subprocess.run("ufw allow 25,587,465,143,993/tcp", shell=True, stdout=subprocess.DEVNULL)

        msg("Initializing Mail User...")

        # Robust wait loop with timeout
        timeout = 300 # 5 minutes
        start_time = time.time()
        initialized = False

        print("Waiting for mailserver to be ready...", end="", flush=True)
        while True:
            # Check timeout
            if time.time() - start_time > timeout:
                print("")
                fatal("Mail Server initialization timed out.")

            # Check if container crashed
            res_status = subprocess.run("docker inspect -f '{{.State.Running}}' mailserver", shell=True, capture_output=True, text=True)
            if res_status.returncode == 0 and res_status.stdout.strip() == "false":
                print("") # Newline
                warn("Mail Server container is not running.")
                logs = subprocess.run("docker logs mailserver --tail 50", shell=True, capture_output=True, text=True).stdout
                fatal(f"Mail Server crashed during startup.\nLogs:\n{logs}")

            res = subprocess.run("docker exec mailserver setup email list", shell=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            if res.returncode == 0:
                initialized = True
                break
            time.sleep(2)
            # Show simple progress
            print(".", end="", flush=True)
        print("") # Newline

        mail_user = ask("Enter email user", "postmaster")
        full_email = f"{mail_user}@{self.domain}"

        pass_val = ask(f"Enter password for {full_email} (leave empty to generate)")
        if pass_val:
            save_credential("postmaster_pass", pass_val)
        else:
            pass_val = get_auth_value("postmaster_pass")
            if not pass_val:
                pass_val = generate_password()
                save_credential("postmaster_pass", pass_val)

        # Create/Update user
        res = subprocess.run(f"docker exec mailserver setup email list | grep -q '{mail_user}@'", shell=True)
        if res.returncode == 0:
             subprocess.run(f"docker exec mailserver setup email update '{full_email}' '{pass_val}'", shell=True, stdout=subprocess.DEVNULL)
        else:
             subprocess.run(f"docker exec mailserver setup email add '{full_email}' '{pass_val}'", shell=True, stdout=subprocess.DEVNULL)

        subprocess.run("docker exec mailserver setup config dkim", shell=True, stdout=subprocess.DEVNULL)

        msg("Mail Account Credentials:")
        print(f"   User: {full_email}")
        print(f"   Pass: {pass_val}")

    def remove(self):
        subdomain = f"mail.{self.domain}"
        remove_service(self.name, self.pretty_name, subdomain, self.port)
        subprocess.run("ufw delete allow 25,587,465,143,993/tcp", shell=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

class NetdataService(BaseService):
    name = "netdata"
    pretty_name = "Netdata"
    port = "19999"

    def install(self):
        subdomain = f"netdata.{self.domain}"

        compose_content = """
services:
  netdata:
    image: netdata/netdata
    container_name: netdata
    pid: host
    network_mode: host
    restart: unless-stopped
    cap_add:
      - SYS_PTRACE
      - SYS_ADMIN
    security_opt:
      - apparmor:unconfined
    volumes:
      - ./config:/etc/netdata
      - ./lib:/var/lib/netdata
      - ./cache:/var/cache/netdata
      - /etc/passwd:/host/etc/passwd:ro
      - /etc/group:/host/etc/group:ro
      - /proc:/host/proc:ro
      - /sys:/host/sys:ro
      - /etc/os-release:/host/etc/os-release:ro
      - /var/run/docker.sock:/var/run/docker.sock
    environment:
      - NETDATA_CLAIM_TOKEN=
      - NETDATA_CLAIM_URL=
      - NETDATA_CLAIM_ROOMS=
"""
        deploy_service(self.name, self.pretty_name, compose_content.strip(), subdomain, self.port)

    def remove(self):
        subdomain = f"netdata.{self.domain}"
        remove_service(self.name, self.pretty_name, subdomain, self.port)

class FTPService(BaseService):
    name = "ftp"
    pretty_name = "FTP"

    def install(self):
        msg("Installing FTP (vsftpd)...")
        subprocess.run(["apt-get", "install", "-y", "vsftpd"], check=True)
        if os.path.exists("/etc/vsftpd.conf"):
            shutil.copy("/etc/vsftpd.conf", "/etc/vsftpd.conf.bak")

        try:
             pasv_addr = subprocess.check_output("curl -s https://api.ipify.org", shell=True).decode().strip()
        except:
             pasv_addr = "127.0.0.1"

        config = f"""
listen=NO
listen_ipv6=YES
anonymous_enable=NO
local_enable=YES
write_enable=YES
dirmessage_enable=YES
use_localtime=YES
xferlog_enable=YES
connect_from_port_20=YES
chroot_local_user=YES
secure_chroot_dir=/var/run/vsftpd/empty
pam_service_name=vsftpd
rsa_cert_file=/etc/ssl/certs/ssl-cert-snakeoil.pem
rsa_private_key=/etc/ssl/private/ssl-cert-snakeoil.key
ssl_enable=YES
allow_anon_ssl=NO
ssl_tlsv1=YES
pasv_min_port=40000
pasv_max_port=50000
allow_writeable_chroot=YES
pasv_address={pasv_addr}
"""
        with open("/etc/vsftpd.conf", "w") as f:
            f.write(config)

        subprocess.run("ufw allow 20,21,990/tcp", shell=True, stdout=subprocess.DEVNULL)
        subprocess.run("ufw allow 40000:50000/tcp", shell=True, stdout=subprocess.DEVNULL)
        subprocess.run(["systemctl", "restart", "vsftpd"], check=True)

        ftp_user = ask("Enter username for FTP", "cyluser")

        pass_val = ask(f"Enter password for FTP user '{ftp_user}' (leave empty to generate)")
        if pass_val:
            save_credential("ftp_pass", pass_val)
        else:
            pass_val = get_auth_value("ftp_pass")
            if not pass_val:
                pass_val = generate_password()
                save_credential("ftp_pass", pass_val)

        # Check if user exists
        try:
            subprocess.run(["id", ftp_user], check=True, stderr=subprocess.DEVNULL)
        except subprocess.CalledProcessError:
            subprocess.run(["useradd", "-m", "-d", "/var/www", "-s", "/bin/bash", ftp_user], check=True)
            subprocess.run(["usermod", "-a", "-G", "www-data", ftp_user], check=True)
            subprocess.run(["chown", "-R", f"{ftp_user}:www-data", "/var/www"], check=True)
            save_credential("ftp_user", ftp_user)

        # Set password
        subprocess.run(f"echo '{ftp_user}:{pass_val}' | chpasswd", shell=True, check=True)

        msg(f"FTP User: {ftp_user} / {pass_val}")
        success("FTP Installed")

    def remove(self):
        msg("Removing FTP...")
        subprocess.run(["apt-get", "remove", "-y", "vsftpd"], check=True)
        subprocess.run("ufw delete allow 20/tcp", shell=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        success("FTP Removed")

    def is_installed(self):
        return shutil.which("vsftpd") is not None

class GLPIService(BaseService):
    name = "glpi"
    pretty_name = "GLPI"
    port = "8082"

    def install(self):
        subdomain = f"support.{self.domain}"

        pass_val = ask("Enter database password for GLPI (leave empty to generate)")
        if pass_val:
            save_credential("glpi_db_pass", pass_val)
        else:
            pass_val = get_auth_value("glpi_db_pass")
            if not pass_val:
                pass_val = generate_password()
                save_credential("glpi_db_pass", pass_val)

        ensure_db(self.name, self.name, pass_val)

        try:
            res = subprocess.run(f"docker network inspect {self.docker_net} | jq -r '.[0].IPAM.Config[0].Gateway'", shell=True, capture_output=True, text=True)
            host_ip = res.stdout.strip()
        except:
             host_ip = "172.17.0.1"

        compose_content = f"""
services:
  glpi:
    image: diouxx/glpi:latest
    container_name: glpi
    restart: always
    networks:
      - {self.docker_net}
    ports:
      - 127.0.0.1:8082:80
    volumes:
      - ./data:/var/www/html/glpi
    environment:
      - TIMEZONE=Europe/Paris
      - MARIADB_HOST={host_ip}
      - MARIADB_PORT=3306
      - MARIADB_DATABASE={self.name}
      - MARIADB_USER={self.name}
      - MARIADB_PASSWORD={pass_val}
networks:
  {self.docker_net}:
    external: true
"""
        deploy_service(self.name, self.pretty_name, compose_content.strip(), subdomain, self.port)

        msg("GLPI Default Credentials:")
        print(f"   User: glpi")
        print(f"   Pass: glpi")
        warn("Please change the default password immediately!")

    def remove(self):
        subdomain = f"support.{self.domain}"
        remove_service(self.name, self.pretty_name, subdomain, self.port)
