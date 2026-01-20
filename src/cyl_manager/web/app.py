import os
import psutil
from flask import Flask, jsonify, render_template, request
from cyl_manager.core.system import SystemManager
from cyl_manager.core.docker import DockerManager

app = Flask(__name__)

from typing import Any, Dict

@app.route("/")
def index() -> str:
    return "<h1>Cylae Manager</h1><p>Welcome to the dashboard. <a href='/status'>Status</a></p>"

@app.route("/status")
def status() -> Any:
    """
    Returns JSON with current hardware stats and selected profile.
    """
    profile = SystemManager.get_hardware_profile()

    # Get real-time stats (similar to how SystemManager detects them, but live)
    mem = psutil.virtual_memory()
    swap = psutil.swap_memory()
    cpu_count = psutil.cpu_count() or 1
    cpu_percent = psutil.cpu_percent(interval=None) # Non-blocking

    data = {
        "profile": profile,
        "hardware": {
            "ram_total_gb": round(mem.total / (1024**3), 2),
            "ram_used_gb": round(mem.used / (1024**3), 2),
            "ram_percent": mem.percent,
            "swap_total_gb": round(swap.total / (1024**3), 2),
            "swap_used_gb": round(swap.used / (1024**3), 2),
            "cpu_cores": cpu_count,
            "cpu_usage_percent": cpu_percent
        },
        "services": {} # Placeholder for container status
    }

    # Try to fetch container status if Docker is available
    try:
         # Note: DockerManager instance is usually managed via CLI, but we can try to use it here.
         # For now just basic hardware stats as requested by the test scenario.
         pass
    except Exception:
         pass

    return jsonify(data)

def start_server(host: str = "0.0.0.0", port: int = 5000, debug: bool = False) -> None:
    app.run(host=host, port=port, debug=debug)
