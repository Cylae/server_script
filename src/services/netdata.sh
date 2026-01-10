#!/bin/bash
set -euo pipefail

manage_netdata() {
    local name="netdata"
    local sub="netdata.$DOMAIN"

    if [ "$1" == "install" ]; then
        msg "Installing Netdata..."
        mkdir -p "/opt/$name"
        # Using bind mounts to ensure we can back them up
        mkdir -p "/opt/$name/config" "/opt/$name/lib" "/opt/$name/cache"

        # We run this one manually as it requires many host mounts
        docker run -d --name=netdata --pid=host --network=$DOCKER_NET -p 127.0.0.1:19999:19999 \
          -v "/opt/$name/config:/etc/netdata" \
          -v "/opt/$name/lib:/var/lib/netdata" \
          -v "/opt/$name/cache:/var/cache/netdata" \
          -v /etc/passwd:/host/etc/passwd:ro -v /etc/group:/host/etc/group:ro -v /proc:/host/proc:ro \
          -v /sys:/host/sys:ro -v /etc/os-release:/host/etc/os-release:ro -v /var/run/docker.sock:/var/run/docker.sock \
          --restart always --cap-add SYS_PTRACE --security-opt apparmor=unconfined netdata/netdata

        update_nginx "$sub" "19999" "proxy"
        success "Netdata Installed"
    elif [ "$1" == "remove" ]; then
        docker stop netdata && docker rm netdata

        ask "Do you want to PERMANENTLY DELETE data for Netdata? (y/n):" confirm_delete
        if [[ "$confirm_delete" == "y" ]]; then
            rm -rf "/opt/$name"
        fi

        rm -f "/etc/nginx/sites-enabled/$sub"
        success "Netdata Removed"
    fi
}
