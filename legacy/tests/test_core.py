import pytest
from cyl_manager.core.config import load_config, get, set_config, CONFIG_FILE
from cyl_manager.core import config as config_module
from cyl_manager.core.utils import check_command
import os

def test_config(tmp_path):
    # Mock config file path
    fake_config = tmp_path / "cyl_manager.conf"

    # Monkeypatch the module constant
    config_module.CONFIG_FILE = str(fake_config)

    # Reset cache
    config_module._config_cache = {}

    load_config()
    assert get("DOMAIN") == "example.com"
    set_config("TEST_KEY", "TEST_VALUE")
    assert get("TEST_KEY") == "TEST_VALUE"

    # Verify file write
    with open(fake_config, "r") as f:
        content = f.read()
    assert "TEST_KEY=TEST_VALUE" in content

def test_utils():
    assert check_command("ls") is True
    assert check_command("nonexistentcommand") is False
