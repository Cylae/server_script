import os
import shutil
import subprocess
import tarfile
from datetime import datetime
from .utils import msg, success, warn, fatal, ask

BACKUP_DIR = "/root/backups"

def manage_backup():
    """Manages system backups."""
    os.makedirs(BACKUP_DIR, exist_ok=True)

    msg("Starting Backup...")

    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    backup_file = os.path.join(BACKUP_DIR, f"backup_{timestamp}.tar.gz")

    # 1. Backup /opt (Service Data)
    msg("Backing up /opt data...")
    # Excluding media which can be huge
    exclude_args = ["--exclude=opt/media"]

    # 2. Backup SQL Databases
    msg("Backing up Databases...")
    sql_dump_path = "/tmp/all_databases.sql"

    from .database import init_db_password
    db_root_pass = init_db_password()

    import tempfile
    with tempfile.NamedTemporaryFile(mode='w', delete=False) as temp_cnf:
        temp_cnf.write(f"[client]\nuser=root\npassword={db_root_pass}\nhost=localhost\n")
        temp_cnf_path = temp_cnf.name
    os.chmod(temp_cnf_path, 0o600)

    try:
        subprocess.run(f"mysqldump --defaults-extra-file='{temp_cnf_path}' --all-databases > {sql_dump_path}", shell=True, check=True)
    except subprocess.CalledProcessError:
        warn("Database backup failed!")
    finally:
        os.remove(temp_cnf_path)

    # Create Tarball
    msg("Compressing backup...")
    try:
        with tarfile.open(backup_file, "w:gz") as tar:
            tar.add("/opt", arcname="opt", filter=lambda x: None if "media" in x.name else x)
            if os.path.exists(sql_dump_path):
                tar.add(sql_dump_path, arcname="database_dump.sql")
            if os.path.exists("/etc/nginx/sites-available"):
                tar.add("/etc/nginx/sites-available", arcname="nginx_config")
            if os.path.exists("/root/.auth_details"):
                tar.add("/root/.auth_details", arcname="auth_details")

        success(f"Backup created: {backup_file}")
    except Exception as e:
        fatal(f"Backup failed: {e}")
    finally:
        if os.path.exists(sql_dump_path):
            os.remove(sql_dump_path)

def manage_restore():
    """Restores from a backup."""
    if not os.path.exists(BACKUP_DIR):
        fatal("No backup directory found.")

    backups = sorted([f for f in os.listdir(BACKUP_DIR) if f.endswith(".tar.gz")])
    if not backups:
        fatal("No backups found.")

    print("Available Backups:")
    for i, b in enumerate(backups):
        print(f"{i+1}. {b}")

    choice = ask("Select backup to restore")
    try:
        idx = int(choice) - 1
        if idx < 0 or idx >= len(backups):
            raise ValueError
        selected_backup = os.path.join(BACKUP_DIR, backups[idx])
    except ValueError:
        fatal("Invalid selection.")

    confirm = ask("WARNING: This will overwrite existing data. Continue? (y/n)")
    if confirm.lower() != "y":
        return

    msg("Restoring Backup...")

    # Extract to tmp
    tmp_extract = "/tmp/restore_extract"
    if os.path.exists(tmp_extract):
        shutil.rmtree(tmp_extract)
    os.makedirs(tmp_extract)

    try:
        with tarfile.open(selected_backup, "r:gz") as tar:
            tar.extractall(path=tmp_extract)

        # Restore /opt
        if os.path.exists(f"{tmp_extract}/opt"):
            msg("Restoring /opt...")
            subprocess.run(f"cp -r {tmp_extract}/opt/* /opt/", shell=True)

        # Restore DB
        if os.path.exists(f"{tmp_extract}/database_dump.sql"):
            msg("Restoring Databases...")
            from .database import init_db_password
            db_root_pass = init_db_password()

            import tempfile
            with tempfile.NamedTemporaryFile(mode='w', delete=False) as temp_cnf:
                temp_cnf.write(f"[client]\nuser=root\npassword={db_root_pass}\nhost=localhost\n")
                temp_cnf_path = temp_cnf.name
            os.chmod(temp_cnf_path, 0o600)

            try:
                subprocess.run(f"mysql --defaults-extra-file='{temp_cnf_path}' < {tmp_extract}/database_dump.sql", shell=True, check=True)
            except:
                warn("Database restore failed.")
            finally:
                os.remove(temp_cnf_path)

        # Restore Nginx
        if os.path.exists(f"{tmp_extract}/nginx_config"):
            msg("Restoring Nginx Config...")
            subprocess.run(f"cp -r {tmp_extract}/nginx_config/* /etc/nginx/sites-available/", shell=True)

        # Restore Auth
        if os.path.exists(f"{tmp_extract}/auth_details"):
             shutil.copy(f"{tmp_extract}/auth_details", "/root/.auth_details")

        success("Restore Complete. Please restart services.")

    except Exception as e:
        fatal(f"Restore failed: {e}")
    finally:
        if os.path.exists(tmp_extract):
            shutil.rmtree(tmp_extract)
