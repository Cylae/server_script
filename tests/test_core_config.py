import os
import pytest
from unittest.mock import MagicMock, patch
from pathlib import Path
from cyl_manager.core.config import Settings, save_settings, ENV_FILE_PATH, settings, reload_settings

# Use a temporary file for config testing
@pytest.fixture
def temp_env_file(tmp_path):
    env_file = tmp_path / ".env"
    env_file.write_text("DOMAIN=old.com\nEMAIL=old@test.com", encoding="utf-8")
    return env_file

def test_settings_defaults():
    # Test defaults
    s = Settings(_env_file=None)
    assert s.DOMAIN == "example.com"
    assert s.DOCKER_NET == "server-net"

def test_settings_load_from_env(temp_env_file):
    # Patch the global ENV_FILE_PATH logic implicitly by instantiating Settings with env_file arg
    # However, SettingsConfigDict uses the module level constant or args.
    # We can pass _env_file to override.
    s = Settings(_env_file=temp_env_file)
    assert s.DOMAIN == "old.com"
    assert s.EMAIL == "old@test.com"

def test_save_settings(temp_env_file):
    with patch("cyl_manager.core.config.ENV_FILE_PATH", temp_env_file):
        # We also need to patch the global settings object or reload it
        save_settings("DOMAIN", "new.com", env_path=temp_env_file)

        content = temp_env_file.read_text()
        assert "DOMAIN=new.com" in content
        assert "EMAIL=old@test.com" in content # Should preserve other keys

        # Verify it updated the global settings (mocking reload behavior is tricky in integration)
        # But we can check if the file was written correctly.

def test_save_settings_new_key(temp_env_file):
    with patch("cyl_manager.core.config.ENV_FILE_PATH", temp_env_file):
        save_settings("NEW_KEY", "new_value", env_path=temp_env_file)
        content = temp_env_file.read_text()
        assert "NEW_KEY=new_value" in content
