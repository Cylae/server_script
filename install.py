import os
import sys
import subprocess
import shutil

def check_docker():
    """Checks if Docker is installed. If not, installs it."""
    if shutil.which("docker") is None:
        print("Docker not found. Installing Docker...")
        try:
            # Install Docker using the convenience script
            subprocess.run("curl -fsSL https://get.docker.com -o get-docker.sh", shell=True, check=True)
            subprocess.run("sh get-docker.sh", shell=True, check=True)
            os.remove("get-docker.sh")
            print("Docker installed successfully.")
        except subprocess.CalledProcessError:
            print("Failed to install Docker. Please install Docker manually.")
            sys.exit(1)
    else:
        print("Docker is already installed.")

def main():
    print("Checking root...")
    if os.geteuid() != 0:
        print("Please run as root")
        sys.exit(1)

    print("Checking system dependencies...")
    # Install pip and venv if missing (redundant if install.sh did its job but safe)
    subprocess.run(["apt-get", "update", "-q"], check=False)
    subprocess.run(["apt-get", "install", "-y", "python3", "python3-pip", "python3-venv", "git", "curl"], check=True)

    check_docker()

    print("Setting up virtual environment...")
    if not os.path.exists(".venv"):
        subprocess.run(["python3", "-m", "venv", ".venv"], check=True)

    print("Installing package...")
    pip_cmd = ".venv/bin/pip"
    subprocess.run([pip_cmd, "install", "-U", "pip", "setuptools", "wheel"], check=True)
    subprocess.run([pip_cmd, "install", "-e", "."], check=True)

    print("Creating symlink...")
    if os.path.exists("/usr/local/bin/cyl-manager"):
        os.remove("/usr/local/bin/cyl-manager")
    os.symlink(os.path.abspath(".venv/bin/cyl-manager"), "/usr/local/bin/cyl-manager")

    print("\nInstallation Complete!")
    print("Run 'cyl-manager menu' to start.")

if __name__ == "__main__":
    main()
