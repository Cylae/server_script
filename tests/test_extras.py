import pytest
import os
from unittest.mock import MagicMock, patch
from cyl_manager.services import proxy
from cyl_manager.core import security, system

# Mock Nginx paths for testing
@pytest.fixture
def mock_nginx_paths(tmp_path, monkeypatch):
    sites_avail = tmp_path / "sites-available"
    sites_enabled = tmp_path / "sites-enabled"
    os.makedirs(sites_avail)
    os.makedirs(sites_enabled)

    monkeypatch.setattr(proxy, 'SITES_AVAIL', str(sites_avail))
    monkeypatch.setattr(proxy, 'SITES_ENABLED', str(sites_enabled))
    return sites_avail, sites_enabled

def test_proxy_update_nginx(mock_nginx_paths, monkeypatch):
    avail, enabled = mock_nginx_paths

    # Mock system commands
    def mock_run(cmd, **kwargs):
        if "nginx -t" in str(cmd): return
        if "systemctl" in str(cmd): return
        return MagicMock() # Return a mock object, not None

    monkeypatch.setattr(system, "run_command", mock_run)
    monkeypatch.setattr(system, "install_apt_packages", lambda x: None)

    proxy.update_nginx("test.com", "8080", "standard")

    conf_file = avail / "test.com"
    assert conf_file.exists()

    with open(conf_file, "r") as f:
        content = f.read()
        assert "server_name test.com;" in content
        assert "proxy_pass http://127.0.0.1:8080;" in content
        assert "client_max_body_size 512M;" in content

    # Test Cloud Variant
    proxy.update_nginx("cloud.test.com", "9000", "cloud")
    conf_file_cloud = avail / "cloud.test.com"
    with open(conf_file_cloud, "r") as f:
        content = f.read()
        assert "client_max_body_size 10G;" in content

def test_security_firewall(monkeypatch):
    cmds = []
    def mock_run(cmd, **kwargs):
        # Handle list vs string
        if isinstance(cmd, list):
            cmds.append(" ".join(cmd))
        else:
            cmds.append(str(cmd))
        return MagicMock()

    monkeypatch.setattr(system, "run_command", mock_run)
    monkeypatch.setattr(system, "install_apt_packages", lambda x: None)

    # We must prevent 'ufw' execution from failing in `run_command` logic if we don't mock it perfectly,
    # but since we mocked `run_command` entirely, it shouldn't execute anything.
    # The error "No such file or directory: 'ufw'" suggests `subprocess.run` was called
    # OR `shlex.split` failed? No.
    # It suggests `run_command` WAS NOT MOCKED correctly or `security.py` imports `run_command` in a way that monkeypatch missed?
    # `from .system import run_command` in `security.py`.
    # `monkeypatch.setattr(system, "run_command", mock_run)` should work if `security.py` uses `system.run_command`
    # BUT `security.py` does `from .system import run_command`.
    # So `security.run_command` is a reference to the ORIGINAL function.
    # We need to patch `cyl_manager.core.security.run_command`.

    monkeypatch.setattr(security, "run_command", mock_run)

    security.configure_firewall()

    assert "ufw default deny incoming" in cmds
    assert "ufw allow ssh" in cmds
