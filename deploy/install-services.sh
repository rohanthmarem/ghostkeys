#!/usr/bin/env bash
set -euo pipefail
cd /home/exedev/ghostkeys
mkdir -p /home/exedev/.config/ghostkeys /home/exedev/.config/openbox
chmod 700 /home/exedev/.config/ghostkeys
if [[ ! -f /home/exedev/.config/ghostkeys/service.env ]]; then
  python3 - <<'PY'
import secrets, os
p='/home/exedev/.config/ghostkeys/service.env'
with open(p,'x') as f:
    f.write('GHOSTKEYS_API_TOKEN='+secrets.token_hex(32)+'\nGHOSTKEYS_OWNER=rohanth.marem@gmail.com\nGHOSTKEYS_ORIGIN=https://ghostkeys.exe.xyz\n')
os.chmod(p,0o600)
PY
fi
cat > /home/exedev/.config/openbox/rc.xml <<'XML'
<?xml version="1.0" encoding="UTF-8"?>
<openbox_config xmlns="http://openbox.org/3.4/rc"><focus><focusNew>yes</focusNew><followMouse>no</followMouse></focus><applications><application class="Google-chrome"><position force="yes"><x>420</x><y>0</y></position><size><width>1020</width><height>900</height></size></application><application name="ghostkeys"><position force="yes"><x>0</x><y>100</y></position></application></applications></openbox_config>
XML
cat > /tmp/ghostkeys-display.service <<'UNIT'
[Unit]
Description=Ghostkeys persistent X11 desktop
After=network.target
[Service]
User=exedev
ExecStart=/usr/bin/Xvfb :99 -screen 0 1440x900x24 -nolisten tcp -ac
Restart=always
RestartSec=2
[Install]
WantedBy=multi-user.target
UNIT
for entry in \
  'window-manager|/usr/bin/dbus-run-session -- /usr/bin/openbox --config-file /home/exedev/.config/openbox/rc.xml' \
  'vnc|/usr/bin/x11vnc -display :99 -localhost -rfbport 5900 -forever -shared -nopw -noxdamage' \
  'websocket|/usr/bin/websockify --heartbeat=20 127.0.0.1:6080 127.0.0.1:5900' \
  'chrome|/usr/bin/google-chrome-stable --user-data-dir=/home/exedev/.config/ghostkeys/chrome --remote-debugging-address=127.0.0.1 --remote-debugging-port=9222 --no-first-run --no-default-browser-check --disable-dev-shm-usage --password-store=basic --ozone-platform=x11 --window-position=420,0 --window-size=1020,900 https://www.office.com/launch/word' \
  'app|/home/exedev/ghostkeys/src-tauri/target/release/ghostkeys' \
  'mcp|/home/exedev/.local/node/bin/node /home/exedev/ghostkeys/deploy/server.mjs'; do
  name=${entry%%|*}
  executable=${entry#*|}
  cat > "/tmp/ghostkeys-$name.service" <<UNIT
[Unit]
Description=Ghostkeys $name
Requires=ghostkeys-display.service
After=ghostkeys-display.service
[Service]
User=exedev
WorkingDirectory=/home/exedev/ghostkeys
Environment=DISPLAY=:99
Environment=GDK_BACKEND=x11
Environment=WEBKIT_DISABLE_COMPOSITING_MODE=1
Environment=PATH=/home/exedev/.local/node/bin:/home/exedev/.cargo/bin:/usr/local/bin:/usr/bin:/bin
EnvironmentFile=/home/exedev/.config/ghostkeys/service.env
ExecStartPre=/bin/sh -c 'until /usr/bin/xdpyinfo -display :99 >/dev/null 2>&1; do sleep 0.2; done'
ExecStart=$executable
Restart=always
RestartSec=3
[Install]
WantedBy=multi-user.target
UNIT
done
sudo install -m 644 /tmp/ghostkeys-*.service /etc/systemd/system/
cat > /tmp/ghostkeys-nginx.conf <<'NGINX'
server {
    listen 127.0.0.1:8000;
    server_name ghostkeys.exe.xyz;
    client_max_body_size 128k;
    if ($http_x_exedev_email != "rohanth.marem@gmail.com") { return 403; }
    location /desktop/ {
        if ($http_x_exedev_token_ctx != "") { return 403; }
        alias /usr/share/novnc/;
        index vnc.html;
        add_header Cache-Control no-store;
    }
    location = /desktop/websockify {
        if ($http_x_exedev_token_ctx != "") { return 403; }
        proxy_pass http://127.0.0.1:6080/;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_read_timeout 86400;
    }
    location / {
        proxy_pass http://127.0.0.1:8790;
        proxy_buffering off;
        proxy_read_timeout 120;
    }
}
NGINX
sudo install -m 644 /tmp/ghostkeys-nginx.conf /etc/nginx/sites-available/ghostkeys
sudo ln -sfn /etc/nginx/sites-available/ghostkeys /etc/nginx/sites-enabled/ghostkeys
sudo nginx -t
sudo systemctl daemon-reload
sudo systemctl enable --now ghostkeys-display ghostkeys-window-manager ghostkeys-vnc ghostkeys-websocket ghostkeys-chrome ghostkeys-mcp
sudo systemctl enable --now nginx
sudo systemctl reload nginx
if [[ -x src-tauri/target/release/ghostkeys ]]; then sudo systemctl enable --now ghostkeys-app; fi
