import pytest
from unittest.mock import MagicMock, patch
from cyl_manager.services.media import PlexService
from cyl_manager.services.more_services import MailService
from cyl_manager.services.gitea import GiteaService

@pytest.fixture
def mock_system():
    # Only patch methods, not constants
    with patch("cyl_manager.services.base.DockerManager"), \
         patch("cyl_manager.services.base.SystemManager.get_hardware_profile") as mock_profile, \
         patch("cyl_manager.services.base.SystemManager.get_uid_gid") as mock_uid, \
         patch("cyl_manager.services.base.SystemManager.get_timezone") as mock_tz, \
         patch("cyl_manager.services.media.settings") as mock_settings:

        mock_uid.return_value = ("1000", "1000")
        mock_tz.return_value = "UTC"
        mock_settings.DATA_DIR = "/opt/cylae"
        mock_settings.MEDIA_ROOT = "/opt/media"
        mock_settings.DOCKER_NET = "server-net"
        yield mock_profile

def test_plex_transcode_volume_high(mock_system):
    mock_system.return_value = "HIGH"
    svc = PlexService()

    config = svc.generate_compose()
    volumes = config["services"]["plex"]["volumes"]
    assert "/tmp:/transcode" in volumes

def test_plex_transcode_volume_low(mock_system):
    mock_system.return_value = "LOW"
    svc = PlexService()

    config = svc.generate_compose()
    volumes = config["services"]["plex"]["volumes"]
    assert "/tmp:/transcode" not in volumes
    assert any("transcode" in v for v in volumes)

def test_mailserver_optimization(mock_system):
    mock_system.return_value = "LOW"
    svc = MailService()

    config = svc.generate_compose()
    env = config["services"]["mailserver"]["environment"]
    assert env["ENABLE_CLAMAV"] == "0"
    assert env["ENABLE_SPAMASSASSIN"] == "0"

def test_gitea_config(mock_system):
    svc = GiteaService()
    config = svc.generate_compose()
    env = config["services"]["gitea"]["environment"]
    assert env["GITEA__database__DB_TYPE"] == "sqlite3"
