import pytest
from cyl_manager.services.base import BaseService
from cyl_manager.services.plex import PlexService

def test_service_instantiation():
    svc = PlexService()
    assert svc.name == "plex"
    assert svc.port == "32400"
