#!/bin/bash
set -euo pipefail

# ==============================================================================
# MEDIA LIBRARY
# Helpers for the media stack
# ==============================================================================

setup_media_directories() {
    msg "Setting up Unified Media Structure..."

    local MEDIA_ROOT="/opt/media"

    # Create directories
    mkdir -p "$MEDIA_ROOT/movies"
    mkdir -p "$MEDIA_ROOT/tv"
    mkdir -p "$MEDIA_ROOT/music"
    mkdir -p "$MEDIA_ROOT/downloads"
    mkdir -p "$MEDIA_ROOT/watch"

    # Create docker group if not exists
    if ! getent group docker >/dev/null; then
        groupadd docker
    fi

    # Set permissions
    # Root owns it, but docker group can write.
    chown -R root:docker "$MEDIA_ROOT"
    chmod -R 775 "$MEDIA_ROOT"

    success "Media directories configured at $MEDIA_ROOT"
}
