from .base import BaseService
from ..core import docker_manager, config, system
import time
import logging
import shlex

logger = logging.getLogger("Database")

class MariaDBService(BaseService):
    @property
    def name(self):
        return "mariadb"

    @property
    def display_name(self):
        return "MariaDB"

    def install(self):
        print(f"Installing {self.display_name}...")
        docker_manager.create_network("server-net")

        root_pass = config.get_or_create_password("DB_ROOT_PASS")

        compose = f"""
services:
  mariadb:
    image: linuxserver/mariadb:latest
    container_name: mariadb
    environment:
      - PUID=1000
      - PGID=1000
      - MYSQL_ROOT_PASSWORD={root_pass}
      - TZ=Etc/UTC
    volumes:
      - /opt/mariadb/config:/config
      - /opt/mariadb/data:/var/lib/mysql
    ports:
      - "3306:3306"
    networks:
      - server-net
    restart: unless-stopped
networks:
  server-net:
    external: true
"""
        docker_manager.deploy_service(self.name, "/opt/mariadb", compose)
        print("MariaDB installed.")

    def uninstall(self):
        docker_manager.remove_service(self.name)

    def is_installed(self):
        return docker_manager.is_running("mariadb")

    def ensure_db(self, db_name, db_user, db_pass):
        """
        Ensures a database and user exist.
        """
        if not self.is_installed():
            self.install()
            print("Waiting for MariaDB to be ready...")
            # Ideally loop check
            time.sleep(15)

        root_pass = config.get_auth_value("DB_ROOT_PASS")

        # We construct a command that passes password via environment variable if possible
        # or uses 'mysql -p' but hides it from ps aux?
        # Actually, `docker exec` env vars are visible via `docker inspect`.
        # The safest way is to pipe the password or config.
        # But for `mysql` client inside container, we can use MYSQL_PWD env var?
        # `docker exec -e MYSQL_PWD=...` is visible in process list of host (the docker client process).

        # Best approach: Create a temporary cnf file inside the container?
        # Or pipe the sql commands.
        # `docker exec -i mariadb mysql -uroot -p...`
        # If we run without shell=True, the arguments are still in /proc.

        # However, we can use `mysql --login-path` or config file.
        # Simpler: Use `docker exec` but try to minimize exposure time? No.
        # Actually, if we use `subprocess.run` without `shell=True`, the command line IS visible in `ps`.

        # Mitigation:
        # Create a temporary file on HOST with credentials, copy to container, run, delete.
        # Or create a SQL file on host.

        import tempfile

        # Commands
        sqls = [
            f"CREATE DATABASE IF NOT EXISTS `{db_name}`;",
            f"CREATE USER IF NOT EXISTS '{db_user}'@'%' IDENTIFIED BY '{db_pass}';",
            f"GRANT ALL PRIVILEGES ON `{db_name}`.* TO '{db_user}'@'%';",
            "FLUSH PRIVILEGES;"
        ]

        full_sql = "\n".join(sqls)

        # Write SQL to temp file
        with tempfile.NamedTemporaryFile(mode='w', delete=False) as tmp:
            tmp.write(full_sql)
            tmp_path = tmp.name

        try:
            # Copy to container
            system.run_command(f"docker cp {tmp_path} mariadb:/tmp/init.sql", shell=True)

            # Execute
            # We still need root password.
            # We can use `-p{root_pass}` but it exposes it.
            # Is there a way to use the env var set in docker-compose?
            # The container has `MYSQL_ROOT_PASSWORD` set in env.
            # Does `mysql` client automatically pick it up? No.
            # But the linuxserver image creates `/config`?
            # We can try to use `mysql -uroot -p$MYSQL_ROOT_PASSWORD` INSIDE the container shell.
            # Because `MYSQL_ROOT_PASSWORD` is an env var of the container (PID 1),
            # `docker exec` doesn't inherit it unless we set it, BUT if we run a shell `sh -c '...'`,
            # we can source it or access it? No, exec starts new process.

            # Use `docker exec -i mariadb sh -c 'mysql -uroot -p"$MYSQL_ROOT_PASSWORD" < /tmp/init.sql'`
            # This works if MYSQL_ROOT_PASSWORD is in the environment of the execed shell.
            # Docker exec does NOT automatically inject container env vars into the exec session?
            # Actually, `docker exec` DOES NOT inherit container env vars by default?
            # Wait, usually it does not.

            # Fallback: Just accept the risk for now or use the temp file approach for password too.
            # Let's write a `.my.cnf` to temp and copy it.

            with tempfile.NamedTemporaryFile(mode='w', delete=False) as cnf:
                cnf.write(f"[client]\nuser=root\npassword={root_pass}\n")
                cnf_path = cnf.name

            system.run_command(f"docker cp {cnf_path} mariadb:/tmp/.my.cnf", shell=True)

            # Run mysql using defaults-extra-file
            cmd = "docker exec -i mariadb mysql --defaults-extra-file=/tmp/.my.cnf < /tmp/init.sql"
            # Using shell=True here to handle redirection `<`?
            # But wait, `docker exec ... < file` works on host.
            # The command above tries to redirect INSIDE container? No, `<` is host shell.
            # But `/tmp/init.sql` is inside container.
            # So: `docker exec -i mariadb mysql ... -e "source /tmp/init.sql"`

            cmd = ["docker", "exec", "mariadb", "mysql", "--defaults-extra-file=/tmp/.my.cnf", "-e", "source /tmp/init.sql"]
            system.run_command(cmd)

            # Cleanup inside container
            system.run_command("docker exec mariadb rm /tmp/init.sql /tmp/.my.cnf", shell=True)

        finally:
            os.remove(tmp_path)
            if 'cnf_path' in locals(): os.remove(cnf_path)
