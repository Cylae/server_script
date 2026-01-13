import os
import secrets
import string
import subprocess
from .config import get_auth_details
from .utils import fatal

AUTH_FILE = "/root/.auth_details"

def generate_password(length=16):
    """Generates a secure password."""
    alphabet = string.ascii_letters + string.digits
    return ''.join(secrets.choice(alphabet) for i in range(length))

def save_credential(key, value):
    """Saves a credential to the auth file."""
    with open(AUTH_FILE, "a") as f:
        f.write(f"{key}={value}\n")

def get_auth_value(key):
    """Gets a value from the auth file."""
    if os.path.exists(AUTH_FILE):
        with open(AUTH_FILE, "r") as f:
            for line in f:
                if line.startswith(f"{key}="):
                    return line.strip().split("=", 1)[1]
    return None

def init_db_password():
    """Initializes the database root password."""
    db_root_pass = get_auth_value("mysql_root_password")
    if not db_root_pass:
        # Check .mariadb_auth legacy
        if os.path.exists("/root/.mariadb_auth"):
             with open("/root/.mariadb_auth", "r") as f:
                 for line in f:
                     if "mysql_root_password" in line:
                         db_root_pass = line.strip().split("=")[1]

        if not db_root_pass:
            db_root_pass = generate_password()
            save_credential("mysql_root_password", db_root_pass)
            # Apply to MySQL safely
            import tempfile
            with tempfile.NamedTemporaryFile(mode='w', delete=False) as sql_file:
                sql_file.write(f"ALTER USER 'root'@'localhost' IDENTIFIED BY '{db_root_pass}'; FLUSH PRIVILEGES;")
                sql_file_path = sql_file.name

            try:
                subprocess.run(f"mysql < '{sql_file_path}'", shell=True, stderr=subprocess.DEVNULL)
            finally:
                os.remove(sql_file_path)

    return db_root_pass

def ensure_db(db_name, username, password):
    """Ensures a database and user exist."""
    db_root_pass = init_db_password()

    # Create temp config for root access
    import tempfile
    with tempfile.NamedTemporaryFile(mode='w', delete=False) as temp_cnf:
        temp_cnf.write(f"[client]\nuser=root\npassword={db_root_pass}\nhost=localhost\n")
        temp_cnf_path = temp_cnf.name

    os.chmod(temp_cnf_path, 0o600)

    sql = f"""
CREATE DATABASE IF NOT EXISTS `{db_name}`;
CREATE USER IF NOT EXISTS '{username}'@'%' IDENTIFIED BY '{password}';
GRANT ALL PRIVILEGES ON `{db_name}`.* TO '{username}'@'%';
FLUSH PRIVILEGES;
ALTER USER '{username}'@'%' IDENTIFIED BY '{password}';
"""

    with tempfile.NamedTemporaryFile(mode='w', delete=False) as sql_file:
        sql_file.write(sql)
        sql_file_path = sql_file.name

    try:
        subprocess.run(f"mysql --defaults-extra-file='{temp_cnf_path}' < '{sql_file_path}'", shell=True, check=True, capture_output=True, text=True)
    except subprocess.CalledProcessError as e:
        if "No space left on device" in e.stderr:
            fatal(f"Disk full. Cannot create database '{db_name}'. Please free up space.")
        raise Exception(f"Database operation failed for {db_name}: {e.stderr}")
    finally:
        os.remove(temp_cnf_path)
        os.remove(sql_file_path)
