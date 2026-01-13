import os
import shutil
from .base import BaseService
from ..core.docker import deploy_service, remove_service
from ..core.database import ensure_db, generate_password, save_credential, get_auth_value
from ..core.utils import ask, msg
from ..core.config import get

class GiteaService(BaseService):
    name = "gitea"
    pretty_name = "Gitea"
    port = "3000"

    def install(self):
        subdomain = f"git.{self.domain}"

        # Interactive Credentials
        pass_val = ask("Enter database password for Gitea (leave empty to generate)")
        if pass_val:
            save_credential("gitea_db_pass", pass_val)
        else:
            pass_val = get_auth_value("gitea_db_pass")
            if not pass_val:
                pass_val = generate_password()
                save_credential("gitea_db_pass", pass_val)

        ensure_db(self.name, self.name, pass_val)

        mem_limit = self.get_resource_limit(default_high="1024M", default_low="512M")

        env = self.get_common_env()

        # Get Host IP
        # In Python we can get it via socket or just execute ip command
        import subprocess
        try:
             host_ip = subprocess.check_output("ip -4 route get 1 | awk '{print $7}'", shell=True).decode().strip()
        except:
             host_ip = "172.17.0.1"

        # Note: In f-string for docker-compose we need to be careful with double braces
        compose_content = f"""
services:
  server:
    image: gitea/gitea:latest
    container_name: gitea
    networks:
      - {self.docker_net}
    environment:
      - USER_UID={env['PUID']}
      - USER_GID={env['PGID']}
      - GITEA__database__DB_TYPE=mysql
      - GITEA__database__HOST={host_ip}:3306
      - GITEA__database__NAME={self.name}
      - GITEA__database__USER={self.name}
      - GITEA__database__PASSWD={pass_val}
      - GITEA__server__SSH_DOMAIN={subdomain}
      - GITEA__server__SSH_PORT=2222
      - GITEA__server__ROOT_URL=https://{subdomain}/
    restart: always
    volumes:
      - ./data:/data
    ports:
      - "127.0.0.1:3000:3000"
      - "2222:22"
    deploy:
      resources:
        limits:
          memory: {mem_limit}
networks:
  {self.docker_net}:
    external: true
"""
        deploy_service(self.name, self.pretty_name, compose_content.strip(), subdomain, self.port)

        msg("Gitea Database Credentials:")
        print(f"   User: {self.name}")
        print(f"   Pass: {pass_val}")

    def remove(self):
        subdomain = f"git.{self.domain}"
        remove_service(self.name, self.pretty_name, subdomain, self.port)
