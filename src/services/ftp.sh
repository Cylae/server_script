#!/bin/bash

manage_ftp() {
    if [ "$1" == "install" ]; then
        msg "Installing FTP (vsftpd)..."
        apt-get install -y vsftpd >/dev/null
        cp /etc/vsftpd.conf /etc/vsftpd.conf.bak 2>/dev/null

        local pasv_addr=$(curl -s https://api.ipify.org)
        cat <<EOF > /etc/vsftpd.conf
listen=NO
listen_ipv6=YES
anonymous_enable=NO
local_enable=YES
write_enable=YES
dirmessage_enable=YES
use_localtime=YES
xferlog_enable=YES
connect_from_port_20=YES
chroot_local_user=YES
secure_chroot_dir=/var/run/vsftpd/empty
pam_service_name=vsftpd
rsa_cert_file=/etc/ssl/certs/ssl-cert-snakeoil.pem
rsa_private_key=/etc/ssl/private/ssl-cert-snakeoil.key
ssl_enable=YES
allow_anon_ssl=NO
ssl_tlsv1=YES
pasv_min_port=40000
pasv_max_port=50000
allow_writeable_chroot=YES
pasv_address=$pasv_addr
EOF
        ufw allow 20,21,990/tcp >/dev/null
        ufw allow 40000:50000/tcp >/dev/null
        systemctl restart vsftpd

        # Ensure user
        ask "Enter username for FTP (default: cyluser):" input_user
        local ftp_user="${input_user:-cyluser}"

        ask "Enter password for FTP user '$ftp_user' (leave empty to generate):" input_pass
        if [ -n "$input_pass" ]; then
             if ! validate_password "$input_pass"; then
                 fatal "Password validation failed."
             fi
             ftp_pass="$input_pass"
             save_credential "ftp_pass" "$ftp_pass"
        else
             ftp_pass=$(get_or_create_password "ftp_pass")
        fi

        if ! id "$ftp_user" &>/dev/null; then
            useradd -m -d /var/www -s /bin/bash "$ftp_user"
            usermod -a -G www-data "$ftp_user"
            chown -R "$ftp_user":www-data /var/www
            save_credential "ftp_user" "$ftp_user"
        fi

        # Always set password to ensure consistency
        echo "$ftp_user:$ftp_pass" | chpasswd
        msg "FTP User: $ftp_user / $ftp_pass"
        success "FTP Installed"
    elif [ "$1" == "remove" ]; then
        apt-get remove -y vsftpd >/dev/null
        ufw delete allow 20/tcp >/dev/null 2>&1 || true
        success "FTP Removed"
    fi
}
