import pytest
from cyl_manager.services.base import BaseService
from cyl_manager.services.plex import PlexService

def test_service_instantiation():
    svc = PlexService()
    assert svc.name == "plex"
    assert svc.port == "32400"

def test_resource_limits(monkeypatch):
    # Mock HIGH profile
    def mock_high_profile():
        return "HIGH"

    monkeypatch.setattr("cyl_manager.services.base.determine_profile", mock_high_profile)

    svc = PlexService()
    # Depending on how the service class initializes profile, we might need to re-init
    svc.profile = "HIGH"
    assert svc.get_resource_limit(default_high="1024M", default_low="512M") == "1024M"

    # Mock LOW profile
    def mock_low_profile():
        return "LOW"

    monkeypatch.setattr("cyl_manager.services.base.determine_profile", mock_low_profile)

    svc.profile = "LOW"
    assert svc.get_resource_limit(default_high="1024M", default_low="512M") == "512M"
