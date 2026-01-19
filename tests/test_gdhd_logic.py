import pytest
from unittest.mock import MagicMock, patch
from cyl_manager.core.system import SystemManager

@pytest.fixture(autouse=True)
def reset_hardware_profile():
    """Reset the hardware profile cache before each test."""
    SystemManager._hardware_profile = None
    yield
    SystemManager._hardware_profile = None

@patch("cyl_manager.core.system.psutil")
def test_gdhd_survival_mode_low_ram(mock_psutil):
    """Test that < 4GB RAM triggers LOW profile."""
    # Mock 3GB RAM, 8 Cores, 2GB Swap (RAM is the bottleneck)
    mock_mem = MagicMock()
    mock_mem.total = 3 * 1024**3
    mock_psutil.virtual_memory.return_value = mock_mem

    mock_psutil.cpu_count.return_value = 8

    mock_swap = MagicMock()
    mock_swap.total = 2 * 1024**3
    mock_psutil.swap_memory.return_value = mock_swap

    assert SystemManager.get_hardware_profile() == "LOW"

@patch("cyl_manager.core.system.psutil")
def test_gdhd_survival_mode_low_cpu(mock_psutil):
    """Test that <= 2 CPUs triggers LOW profile."""
    # Mock 8GB RAM, 2 Cores, 2GB Swap (CPU is the bottleneck)
    mock_mem = MagicMock()
    mock_mem.total = 8 * 1024**3
    mock_psutil.virtual_memory.return_value = mock_mem

    mock_psutil.cpu_count.return_value = 2

    mock_swap = MagicMock()
    mock_swap.total = 2 * 1024**3
    mock_psutil.swap_memory.return_value = mock_swap

    assert SystemManager.get_hardware_profile() == "LOW"

@patch("cyl_manager.core.system.psutil")
def test_gdhd_survival_mode_low_swap(mock_psutil):
    """Test that < 1GB Swap triggers LOW profile."""
    # Mock 8GB RAM, 8 Cores, 512MB Swap (Swap is the bottleneck)
    mock_mem = MagicMock()
    mock_mem.total = 8 * 1024**3
    mock_psutil.virtual_memory.return_value = mock_mem

    mock_psutil.cpu_count.return_value = 8

    mock_swap = MagicMock()
    mock_swap.total = 512 * 1024**2
    mock_psutil.swap_memory.return_value = mock_swap

    assert SystemManager.get_hardware_profile() == "LOW"

@patch("cyl_manager.core.system.psutil")
def test_gdhd_high_performance(mock_psutil):
    """Test that meeting all criteria triggers HIGH profile."""
    # Mock 4GB RAM, 4 Cores, 1GB Swap (Minimal High Spec)
    mock_mem = MagicMock()
    mock_mem.total = 4 * 1024**3
    mock_psutil.virtual_memory.return_value = mock_mem

    mock_psutil.cpu_count.return_value = 4

    mock_swap = MagicMock()
    mock_swap.total = 1 * 1024**3
    mock_psutil.swap_memory.return_value = mock_swap

    assert SystemManager.get_hardware_profile() == "HIGH"

@patch("cyl_manager.core.system.psutil")
def test_gdhd_psutil_failure(mock_psutil):
    """Test that psutil failure defaults to LOW (Safe Mode)."""
    mock_psutil.virtual_memory.side_effect = Exception("Procfs not mounted")

    assert SystemManager.get_hardware_profile() == "LOW"

@patch("cyl_manager.core.system.psutil")
def test_gdhd_cpu_count_none(mock_psutil):
    """Test that if cpu_count returns None (can happen), it defaults to 1 (LOW)."""
    # Mock 8GB RAM, None Cores, 2GB Swap
    mock_mem = MagicMock()
    mock_mem.total = 8 * 1024**3
    mock_psutil.virtual_memory.return_value = mock_mem

    mock_psutil.cpu_count.return_value = None

    mock_swap = MagicMock()
    mock_swap.total = 2 * 1024**3
    mock_psutil.swap_memory.return_value = mock_swap

    assert SystemManager.get_hardware_profile() == "LOW"
