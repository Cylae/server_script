import pytest
from unittest.mock import MagicMock, patch
from cyl_manager.core.system import determine_profile, check_disk_space, check_os, get_hardware_specs

def test_determine_profile_low():
    with patch("cyl_manager.core.system.get_hardware_specs") as mock_specs:
        mock_specs.return_value = {"ram_gb": 2, "cpu_cores": 2}
        assert determine_profile() == "LOW"

        mock_specs.return_value = {"ram_gb": 3.9, "cpu_cores": 4}
        assert determine_profile() == "LOW"

        mock_specs.return_value = {"ram_gb": 8, "cpu_cores": 1}
        assert determine_profile() == "LOW"

def test_determine_profile_high():
    with patch("cyl_manager.core.system.get_hardware_specs") as mock_specs:
        mock_specs.return_value = {"ram_gb": 4, "cpu_cores": 4}
        assert determine_profile() == "HIGH"

        mock_specs.return_value = {"ram_gb": 16, "cpu_cores": 8}
        assert determine_profile() == "HIGH"

def test_check_disk_space_fatal():
    with patch("cyl_manager.core.system.get_hardware_specs") as mock_specs:
        mock_specs.return_value = {"disk_free_gb": 4}
        with patch("cyl_manager.core.system.fatal") as mock_fatal:
            check_disk_space()
            mock_fatal.assert_called_once()

def test_check_disk_space_warn():
    with patch("cyl_manager.core.system.get_hardware_specs") as mock_specs:
        mock_specs.return_value = {"disk_free_gb": 10}
        with patch("cyl_manager.core.system.warn") as mock_warn:
            check_disk_space()
            mock_warn.assert_called_once()

def test_check_disk_space_ok():
    with patch("cyl_manager.core.system.get_hardware_specs") as mock_specs:
        mock_specs.return_value = {"disk_free_gb": 20}
        with patch("cyl_manager.core.system.msg") as mock_msg:
            # First call is debug "Disk Free", second is "OK"
            check_disk_space()
            assert mock_msg.call_count == 2
