import urllib.request
import logging

logger = logging.getLogger(__name__)

def get_public_ip():
    """Gets public IP."""
    try:
        return urllib.request.urlopen("https://api.ipify.org").read().decode('utf8')
    except Exception as e:
        logger.warning(f"Could not get public IP: {e}")
        return "127.0.0.1"
