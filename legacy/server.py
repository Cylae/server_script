#!/usr/bin/env python3
"""
Cylae Web Management Interface Entry Point
"""
from cyl_manager.web.app import start_server

if __name__ == "__main__":
    print("🚀 Starting Cylae Web Dashboard on http://0.0.0.0:5000")
    start_server()
