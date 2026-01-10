#!/bin/bash
set -euo pipefail

manage_portainer() {
    local name="portainer"
    local sub="portainer.$DOMAIN"

    if [ "$1" == "install" ]; then
        msg "Installing Portainer..."
        mkdir -p "/opt/$name/data"

        docker run -d -p 127.0.0.1:9000:9000 --name=portainer --network $DOCKER_NET --restart=always \
        -v /var/run/docker.sock:/var/run/docker.sock -v "/opt/$name/data:/data" portainer/portainer-ce

        update_nginx "$sub" "9000" "proxy"
        success "Portainer Installed"
    elif [ "$1" == "remove" ]; then
        docker stop portainer && docker rm portainer

        ask "Do you want to PERMANENTLY DELETE data for Portainer? (y/n):" confirm_delete
        if [[ "$confirm_delete" == "y" ]]; then
            rm -rf "/opt/$name"
        fi

        rm -f "/etc/nginx/sites-enabled/$sub"
        success "Portainer Removed"
    fi
}
