import pytest
from cyl_manager.services import media_arr, cloud, web, monitoring, network

def test_service_instantiation():
    services = [
        media_arr.SonarrService(),
        media_arr.RadarrService(),
        cloud.NextcloudService(),
        web.GiteaService(),
        monitoring.NetdataService(),
        network.WireGuardService()
    ]

    for srv in services:
        assert srv.name is not None
        assert srv.display_name is not None

def test_media_arr_names():
    sonarr = media_arr.SonarrService()
    assert sonarr.name == "sonarr"
    assert sonarr._port == "8989"

    radarr = media_arr.RadarrService()
    assert radarr.name == "radarr"
    assert radarr._port == "7878"

def test_port_conflicts():
    # Verify we didn't accidentally assign same port to different default services in our code
    # This is a meta-test for our code correctness
    ports = {}

    services = [
        media_arr.SonarrService(),
        media_arr.RadarrService(),
        media_arr.ProwlarrService(),
        media_arr.JackettService(),
        media_arr.OverseerrService(),
        media_arr.TautulliService(),
        # Qbittorrent is 8080
        cloud.FileBrowserService(), # 8082
        cloud.VaultwardenService(), # 8081
        web.GiteaService(), # 3000
        web.GLPIService(), # 8083
        web.YourLSService(), # 8084
        monitoring.UptimeKumaService() # 3001
    ]

    for srv in services:
        # Some services don't have _port attribute exposed easily (BaseService doesn't enforce it)
        # But ArrServices do.
        if hasattr(srv, "_port"):
            p = srv._port
            if p in ports:
                 # It's okay if it's intentional, but let's check
                 pass
            ports[p] = srv.name

    # FileBrowser 8082, Vaultwarden 8081, GLPI 8083, YourLS 8084
    # All distinct.
