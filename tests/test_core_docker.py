import unittest
from unittest.mock import patch, MagicMock
from cyl_manager.core.docker import DockerManager
from cyl_manager.core.exceptions import ServiceError

class TestDockerManager(unittest.TestCase):
    def setUp(self):
        # Reset singleton instance for each test
        DockerManager._instance = None
        DockerManager._client = None

    @patch("cyl_manager.core.docker.docker.from_env")
    def test_singleton_initialization(self, mock_from_env):
        mock_client = MagicMock()
        mock_from_env.return_value = mock_client

        manager1 = DockerManager()
        manager2 = DockerManager()

        self.assertIs(manager1, manager2)
        mock_from_env.assert_called_once()
        self.assertIs(manager1.client, mock_client)

    @patch("cyl_manager.core.docker.docker.from_env")
    def test_ensure_network_creates_network(self, mock_from_env):
        mock_client = MagicMock()
        mock_from_env.return_value = mock_client

        # Setup mock behavior: first get raises NotFound, then create happens
        import docker.errors
        mock_client.networks.get.side_effect = docker.errors.NotFound("Network not found")

        manager = DockerManager()
        manager.ensure_network()

        mock_client.networks.get.assert_called_with("server-net")
        mock_client.networks.create.assert_called_with("server-net", driver="bridge")

    @patch("cyl_manager.core.docker.docker.from_env")
    def test_ensure_network_already_exists(self, mock_from_env):
        mock_client = MagicMock()
        mock_from_env.return_value = mock_client

        manager = DockerManager()
        manager.ensure_network()

        mock_client.networks.get.assert_called_with("server-net")
        mock_client.networks.create.assert_not_called()

if __name__ == "__main__":
    unittest.main()
