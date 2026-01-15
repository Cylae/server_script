import pytest
from unittest.mock import MagicMock, patch
from cyl_manager.core.system import SystemManager
from cyl_manager.services.media import PlexService, SonarrService
from cyl_manager.services.more_services import MailService

@pytest.fixture
def mock_docker():
    """Mocks the DockerManager to prevent real Docker connection attempts."""
    with patch("cyl_manager.services.base.DockerManager") as mock:
        yield mock

def test_low_spec_optimizations(mock_docker):
    """
    Verifies that services generate optimized configuration for low-spec hardware.
    """
    # Mock SystemManager.get_hardware_profile to return LOW
    with patch("cyl_manager.core.system.SystemManager.get_hardware_profile", return_value=SystemManager.PROFILE_LOW):

        # Test PlexService
        plex = PlexService()
        plex_compose = plex.generate_compose()
        plex_env = plex_compose["services"]["plex"]["environment"]

        assert plex_env.get("PLEX_MEDIA_SERVER_MAX_PLUGIN_PROCS") == "2", \
            "Plex should limit plugin procs to 2 on low spec."
        assert "/tmp" not in plex_compose["services"]["plex"]["volumes"][-1], \
            "Plex should NOT use /tmp for transcoding on low spec."

        # Test SonarrService (ArrService)
        sonarr = SonarrService()
        sonarr_compose = sonarr.generate_compose()
        sonarr_env = sonarr_compose["services"]["sonarr"]["environment"]

        assert sonarr_env.get("DOTNET_GCServer") == "0", \
            "Sonarr should use Workstation GC (DOTNET_GCServer=0) on low spec."

        # Test MailService
        mail = MailService()
        mail_compose = mail.generate_compose()
        mail_env = mail_compose["services"]["mailserver"]["environment"]

        assert mail_env.get("ENABLE_CLAMAV") == "0", \
            "MailService should disable ClamAV on low spec."
        assert mail_env.get("ENABLE_SPAMASSASSIN") == "0", \
            "MailService should disable SpamAssassin on low spec."

def test_high_spec_optimizations(mock_docker):
    """
    Verifies that services generate standard/high-performance configuration for high-spec hardware.
    """
    # Mock SystemManager.get_hardware_profile to return HIGH
    with patch("cyl_manager.core.system.SystemManager.get_hardware_profile", return_value=SystemManager.PROFILE_HIGH):

        # Test PlexService
        plex = PlexService()
        plex_compose = plex.generate_compose()
        plex_env = plex_compose["services"]["plex"]["environment"]

        assert plex_env.get("PLEX_MEDIA_SERVER_MAX_PLUGIN_PROCS") == "6", \
            "Plex should allow 6 plugin procs on high spec."
        assert "/tmp:/transcode" in plex_compose["services"]["plex"]["volumes"], \
            "Plex SHOULD use /tmp for transcoding on high spec."

        # Test SonarrService (ArrService)
        sonarr = SonarrService()
        sonarr_compose = sonarr.generate_compose()
        sonarr_env = sonarr_compose["services"]["sonarr"]["environment"]

        assert "DOTNET_GCServer" not in sonarr_env, \
            "Sonarr should use default GC (Server) on high spec."

        # Test MailService
        mail = MailService()
        mail_compose = mail.generate_compose()
        mail_env = mail_compose["services"]["mailserver"]["environment"]

        assert mail_env.get("ENABLE_CLAMAV") == "1", \
            "MailService should enable ClamAV on high spec."
