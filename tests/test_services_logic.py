from cyl_manager.services.base import BaseService
from cyl_manager.core.docker import deploy_service, remove_service
from cyl_manager.core.utils import msg
import os
from unittest.mock import MagicMock, patch

# Renamed to avoid PytestCollectionWarning and inheriting from BaseService for logic testing
class MockService(BaseService):
    name = "test_service"
    pretty_name = "Test Service"
    port = "9999"

    def install(self):
        pass
    def remove(self):
        pass

def test_base_service_resource_limit_low():
    with patch("cyl_manager.services.base.determine_profile") as mock_profile:
        mock_profile.return_value = "LOW"
        service = MockService()
        assert service.get_resource_limit() == "512M"
        assert service.get_resource_limit(default_low="128M") == "128M"

def test_base_service_resource_limit_high():
    with patch("cyl_manager.services.base.determine_profile") as mock_profile:
        mock_profile.return_value = "HIGH"
        service = MockService()
        assert service.get_resource_limit() == "2048M"
        assert service.get_resource_limit(default_high="4096M") == "4096M"

def test_plex_service_install(tmp_path):
    from cyl_manager.services.plex import PlexService
    with patch("cyl_manager.services.plex.deploy_service") as mock_deploy:
        with patch("cyl_manager.services.base.determine_profile") as mock_profile:
            with patch("os.makedirs"): # Mock makedirs to avoid permission error
                mock_profile.return_value = "LOW"
                service = PlexService()
                service.domain = "test.com"
                service.install()

                # Verify memory limit
                args, _ = mock_deploy.call_args
                compose_content = args[2]
                assert "memory: 1024M" in compose_content
                # Now we expect resolved values, not shell variables
                # The exact value depends on the env, but we know it won't be ${SUDO_UID}
                assert "${SUDO_UID" not in compose_content
                assert "PUID=" in compose_content

def test_gitea_service_install():
    from cyl_manager.services.gitea import GiteaService
    with patch("cyl_manager.services.gitea.deploy_service") as mock_deploy:
        with patch("cyl_manager.services.base.determine_profile") as mock_profile:
             with patch("cyl_manager.services.gitea.ask", return_value="pass"):
                with patch("cyl_manager.services.gitea.ensure_db"):
                    with patch("cyl_manager.services.gitea.save_credential"):
                        mock_profile.return_value = "LOW"
                        service = GiteaService()
                        service.domain = "test.com"
                        service.install()

                        args, _ = mock_deploy.call_args
                        compose_content = args[2]
                        assert "memory: 512M" in compose_content
