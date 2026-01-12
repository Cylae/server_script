import pytest
from cyl_manager.core import config, system

def test_config_generation():
    pw = config.generate_password()
    assert len(pw) == 16
    assert any(c.islower() for c in pw)
    assert any(c.isupper() for c in pw)
    assert any(c.isdigit() for c in pw)

def test_auth_details(tmp_path):
    # Mock AUTH_FILE
    f = tmp_path / ".auth_details"
    config.AUTH_FILE = str(f)

    config.save_auth_value("TEST_KEY", "12345")
    val = config.get_auth_value("TEST_KEY")
    assert val == "12345"

    val2 = config.get_or_create_password("NEW_KEY")
    assert len(val2) == 16
    assert config.get_auth_value("NEW_KEY") == val2

def test_system_ip_fallback(monkeypatch):
    # Mock run_command to fail or return garbage
    def mock_run(*args, **kwargs):
        raise Exception("Fail")

    monkeypatch.setattr(system, "run_command", mock_run)
    assert system.get_ip() == "127.0.0.1"
