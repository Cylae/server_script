#!/bin/bash
set -euo pipefail

# ==============================================================================
# MEDIA LIBRARY MODULE
# Helper for managing shared media directories
# ==============================================================================

setup_media_directories() {
    local MEDIA_ROOT="/opt/media"

    # Only create if not exists to avoid permission spam, but ensure structure
    if [ ! -d "$MEDIA_ROOT" ]; then
        msg "Configuring Media Directories at $MEDIA_ROOT..."
        mkdir -p "$MEDIA_ROOT"
        mkdir -p "$MEDIA_ROOT/movies"
        mkdir -p "$MEDIA_ROOT/tv"
        mkdir -p "$MEDIA_ROOT/music"
        mkdir -p "$MEDIA_ROOT/downloads/completed"
        mkdir -p "$MEDIA_ROOT/downloads/incomplete"
        mkdir -p "$MEDIA_ROOT/downloads/torrents" # For .torrent files
        mkdir -p "$MEDIA_ROOT/watch"

        # Set permissions
        # We allow root and potentially users to write.
        # Ideally, we should set ownership to the user running the containers (usually 1000 or 0)
        # For now, 775 is a safe bet for a single-tenant server.
        chmod -R 775 "$MEDIA_ROOT"
    fi
}
