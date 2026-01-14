import pytest
import threading
from pathlib import Path
from cyl_manager.core.config import Settings, save_settings

def test_config_defaults():
    settings = Settings()
    assert settings.DOMAIN == "example.com"
    assert settings.DOCKER_NET == "server-net"

def test_save_settings_concurrency(tmp_path):
    env_file = tmp_path / ".env"

    # Run multiple threads to save settings
    threads = []

    def worker(i):
        save_settings(f"KEY_{i}", f"VAL_{i}", env_path=env_file)

    for i in range(10):
        t = threading.Thread(target=worker, args=(i,))
        threads.append(t)
        t.start()

    for t in threads:
        t.join()

    content = env_file.read_text()
    for i in range(10):
        assert f"KEY_{i}=VAL_{i}" in content
