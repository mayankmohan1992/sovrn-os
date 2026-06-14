# Sovrn — Systemd Units & System Configuration

## Systemd Unit Files

### sovrnd.service

```ini
[Unit]
Description=Sovrn Orchestrator Daemon
Documentation=https://sovrn.org/docs/sovrnd
After=network-online.target caddy.service yggdrasil.service
Wants=caddy.service yggdrasil.service
Requires=network-online.target
ConditionPathExists=/var/lib/sovrn/sovrnd/sovrnd.db

[Service]
Type=notify
NotifyAccess=main
Environment=SOVRN_HOME=/var/lib/sovrn
Environment=SOVRN_CONFIG=/etc/sovrn/sovrnd.toml
Environment=SOVRN_SOCKETS=/var/lib/sovrn/sockets
Environment=PYTHONPATH=/opt/sovrn/lib/python3.12/site-packages
ExecStart=/opt/sovrn/bin/sovrnd --config /etc/sovrn/sovrnd.toml
ExecReload=/bin/kill -HUP $MAINPID
Restart=on-failure
RestartSec=5
StartLimitBurst=5
StartLimitIntervalSec=60

# Resource limits
LimitNOFILE=65536
MemoryMax=512M
MemoryHigh=400M
CPUQuota=50%

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
PrivateTmp=true
ReadWritePaths=/var/lib/sovrn /run/sovrnd
ReadOnlyPaths=/etc/sovrn
CapabilityBoundingSet=CAP_NET_BIND_SERVICE

[Install]
WantedBy=multi-user.target
```

### sovrn-dht.service

```ini
[Unit]
Description=Sovrn DHT Resolver
After=network-online.target yggdrasil.service
Requires=network-online.target yggdrasil.service

[Service]
Type=notify
Environment=SOVRN_SOCKETS=/var/lib/sovrn/sockets
ExecStart=/usr/local/bin/sovrn-dht --config /etc/sovrn/dht.toml
Restart=on-failure
RestartSec=3
StartLimitBurst=10
StartLimitIntervalSec=30

LimitNOFILE=32768
MemoryMax=128M
CPUQuota=25%

# Socket directory must exist
ExecStartPre=/bin/mkdir -p /var/lib/sovrn/sockets

[Install]
WantedBy=multi-user.target
```

### sovrn-identity.service

```ini
[Unit]
Description=Sovrn Identity Service
After=network-online.target sovrn-dht.service
Requires=network-online.target

[Service]
Type=notify
Environment=SOVRN_SOCKETS=/var/lib/sovrn/sockets
ExecStart=/usr/local/bin/sovrn-identity --config /etc/sovrn/identity.toml
Restart=on-failure
RestartSec=3

LimitNOFILE=8192
MemoryMax=64M
CPUQuota=15%

[Install]
WantedBy=multi-user.target
```

### sovrn-presence.service

```ini
[Unit]
Description=Sovrn Presence Service
After=sovrn-dht.service yggdrasil.service
Requires=sovrn-dht.service

[Service]
Type=notify
Environment=SOVRN_SOCKETS=/var/lib/sovrn/sockets
ExecStart=/usr/local/bin/sovrn-presence --config /etc/sovrn/presence.toml
Restart=on-failure
RestartSec=3

LimitNOFILE=32768
MemoryMax=64M
CPUQuota=15%

[Install]
WantedBy=multi-user.target
```

### sovrn-feed.service

```ini
[Unit]
Description=Sovrn Feed Service
After=sovrn-message-queue.service sovrn-identity.service
Requires=sovrn-identity.service

[Service]
Type=notify
Environment=SOVRN_SOCKETS=/var/lib/sovrn/sockets
ExecStart=/usr/local/bin/sovrn-feed --config /etc/sovrn/feed.toml
Restart=on-failure
RestartSec=3

LimitNOFILE=32768
MemoryMax=256M
MemoryHigh=200M
CPUQuota=30%

[Install]
WantedBy=multi-user.target
```

### sovrn-message-queue.service

```ini
[Unit]
Description=Sovrn Message Queue
After=sovrn-identity.service
Requires=sovrn-identity.service

[Service]
Type=notify
Environment=SOVRN_SOCKETS=/var/lib/sovrn/sockets
ExecStart=/usr/local/bin/sovrn-mq --config /etc/sovrn/mq.toml
Restart=on-failure
RestartSec=3

LimitNOFILE=16384
MemoryMax=128M
CPUQuota=20%

[Install]
WantedBy=multi-user.target
```

### sovrn-cdn-agent.service

```ini
[Unit]
Description=Sovrn CDN Agent
After=yggdrasil.service sovrnd.service
Wants=yggdrasil.service

[Service]
Type=notify
Environment=SOVRN_SOCKETS=/var/lib/sovrn/sockets
ExecStart=/usr/local/bin/sovrn-cdn-agent --config /etc/sovrn/cdn-agent.toml
Restart=on-failure
RestartSec=5

LimitNOFILE=16384
MemoryMax=256M
CPUQuota=20%

[Install]
WantedBy=multi-user.target
```

### sovrn-app-monitor.service

```ini
[Unit]
Description=Sovrn App Health Monitor
After=sovrnd.service podman.service
Wants=sovrnd.service podman.service

[Service]
Type=simple
Environment=SOVRN_HOME=/var/lib/sovrn
ExecStart=/opt/sovrn/bin/sovrn-app-monitor
Restart=on-failure
RestartSec=5

MemoryMax=64M
CPUQuota=10%

[Install]
WantedBy=multi-user.target
```

### sovrn-notify-bridge.service

```ini
[Unit]
Description=Sovrn Desktop Notification Bridge
After=sovrnd.service

[Service]
Type=simple
ExecStart=/opt/sovrn/bin/sovrn-notify-bridge
Restart=on-failure
RestartSec=3

MemoryMax=32M
CPUQuota=5%

# Only run when a user session is active
ConditionUserEnvironment=XDG_RUNTIME_DIR

[Install]
WantedBy=default.target
```

### caddy.service (override)

```ini
[Unit]
Description=Caddy Web Server (Sovrn)
After=network-online.target
Requires=network-online.target

[Service]
Type=notify
ExecStart=/usr/bin/caddy run --config /etc/caddy/Caddyfile
ExecReload=/usr/bin/caddy reload --config /etc/caddy/Caddyfile
Restart=on-failure
RestartSec=5

LimitNOFILE=65536
MemoryMax=128M
Capabilities=CAP_NET_BIND_SERVICE
AmbientCapabilities=CAP_NET_BIND_SERVICE

[Install]
WantedBy=multi-user.target
```

### yggdrasil.service (override)

```ini
[Unit]
Description=Yggdrasil Mesh Network
After=network-online.target
Requires=network-online.target

[Service]
Type=notify
ExecStart=/usr/bin/yggdrasil -useconffile /etc/yggdrasil/yggdrasil.conf
ExecReload=/bin/kill -HUP $MAINPID
Restart=on-failure
RestartSec=5

LimitNOFILE=65536
MemoryMax=64M
CPUQuota=20%

# TUN device access
DeviceAllow=/dev/net/tun rw
CapabilityBoundingSet=CAP_NET_ADMIN

[Install]
WantedBy=multi-user.target
```

### zram-setup.service

```ini
[Unit]
Description=Setup Zram Swap
After=local-fs.target
DefaultDependencies=no

[Service]
Type=oneshot
RemainAfterExit=yes

# Create zram0 with LZ4 compression, 50% of RAM
ExecStart=/bin/bash -c '\
  echo lz4 > /sys/block/zram0/comp_algorithm && \
  echo $(($(free -m | awk "/^Mem:/{print $2}") / 2))M > /sys/block/zram0/disksize && \
  mkswap /dev/zram0 && \
  swapon -p 100 /dev/zram0'
ExecStop=/bin/bash -c 'swapoff /dev/zram0 && echo 1 > /sys/block/zram0/reset'

[Install]
WantedBy=multi-user.target
```

### sovrn-ca-bootstrap.service

```ini
[Unit]
Description=Sovrn CA Bootstrap (First Boot)
After=local-fs.target
ConditionPathExists=!/var/lib/sovrn/ca/sovrn.local.pem

[Service]
Type=oneshot
ExecStart=/usr/local/bin/sovrn-bootstrap-ca

[Install]
WantedBy=multi-user.target
```

### sovrn-first-boot.service

```ini
[Unit]
Description=Sovrn First Boot Customization
After=sovrnd.service caddy.service gdm.service
ConditionPathExists=!/var/lib/sovrn/.first-boot-done

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=/opt/sovrn/bin/sovrn-first-boot

[Install]
WantedBy=multi-user.target
```

### sovrn-dns-dispatcher.service

```ini
[Unit]
Description=Sovrn DNS Dispatcher
After=network-online.target sovrn-dht.service
Requires=sovrn-dht.service

[Service]
Type=simple
ExecStart=/usr/local/bin/sovrn-dns-dispatcher
Restart=on-failure
RestartSec=3

MemoryMax=32M
CPUQuota=10%

[Install]
WantedBy=multi-user.target
```

---

## sovrnd Configuration File

File: `/etc/sovrn/sovrnd.toml`

```toml
# Sovrn OS — sovrnd configuration
# This file is auto-generated by OOBE and should not be edited manually.

[general]
hostname = "sovrn"  # Updated by OOBE
data_dir = "/var/lib/sovrn"
sockets_dir = "/var/lib/sovrn/sockets"
log_level = "info"  # debug, info, warn, error

[api]
host = "127.0.0.1"
port = 54771
cors_origins = ["https://sovrn.local"]
max_upload_mb = 25

[auth]
jwt_expiry_hours = 24
jwt_refresh_hours = 1  # Refresh if < 1h until expiry
hkdf_salt_prefix = "sovrn-app-"
keyring_service = "sovrn"
keyring_key = "sovrn-master-keypair"

[websocket]
max_connections = 100
ping_interval_seconds = 30

[dns]
local_tld = "sovrn.local"
mesh_tld = "sovrn"
dns_port = 53535

[yggdrasil]
socket_path = "/var/run/yggdrasil/yggdrasil.sock"
config_path = "/etc/yggdrasil/yggdrasil.conf"

[podman]
network_name = "sovrn-apps"
network_subnet = "10.47.0.0/16"
image_cache_dir = "/var/lib/sovrn/images"
app_data_dir = "/var/lib/sovrn/app-data"

[backup]
target = "usb"  # usb, network, mesh, cloud
borg_repo_path = "/media/sovrn-backup"
borg_encryption = "repokey"
schedule = "daily"
schedule_time = "03:00"

[defaults]
preinstalled_apps = ["nextcloud", "vaultwarden", "freshrss"]
auto_start_apps = true
```

---

## Podman Network Configuration

```bash
# sovrn-podman-setup.sh — run on first boot
#!/bin/bash
set -euo pipefail

# Create the Sovrn apps network
podman network exists sovrn-apps 2>/dev/null || \
  podman network create \
    --driver bridge \
    --subnet 10.47.0.0/16 \
    --gateway 10.47.0.1 \
    --internal \
    sovrn-apps

# Enable podman socket for systemd integration
systemctl --user enable podman.socket

# Enable lingering for sovrn user (required for rootless podman + systemd)
loginctl enable-linger sovrn
```

---

## Systemd OOM Configuration

File: `/etc/systemd/oomd.conf`

```ini
[OOM]
# Enable systemd-oomd
DefaultMemoryPressureDurationSec=20
DefaultMemoryPressureLimit=10%

# Per-slice memory limits
# (configured via drop-in files)
```

File: `/etc/systemd/system/sovrn-apps.slice`

```ini
[Unit]
Description=Sovrn Self-Hosted Apps Slice

[Slice]
MemoryMax=4G
MemoryHigh=3.5G
CPUQuota=200%
```

All Podman containers run under this slice via systemd quadlets.

---

## sovrn-app-monitor Pseudocode

File: `/opt/sovrn/bin/sovrn-app-monitor`

```python
#!/usr/bin/env python3
"""Sovrn App Health Monitor — watches Podman containers and reports status to sovrnd."""

import json
import subprocess
import time
import urllib.request

SOPIND_API = "http://127.0.0.1:54771/api/v1/apps/{app_id}/health"
NOTIFY_BRIDGE = "http://127.0.0.1:54773/notify"
CHECK_INTERVAL = 30  # seconds
MAX_RESTART_ATTEMPTS = 3
RESTART_WINDOW = 3600  # 1 hour

def get_container_status(app_id: str) -> dict:
    """Get Podman container status."""
    result = subprocess.run(
        ["podman", "inspect", "--format", "{{.State.Status}}", f"sovrn-{app_id}"],
        capture_output=True, text=True
    )
    if result.returncode != 0:
        return {"status": "not_found", "healthy": False}

    status = result.stdout.strip()
    # Check for OOM kill
    oom_result = subprocess.run(
        ["podman", "inspect", "--format", "{{.State.OOMKilled}}", f"sovrn-{app_id}"],
        capture_output=True, text=True
    )
    is_oom = oom_result.stdout.strip() == "true"

    return {
        "status": "memory_pressure" if is_oom else status,
        "healthy": status == "running" and not is_oom,
        "oom_killed": is_oom
    }

def restart_container(app_id: str) -> bool:
    """Attempt to restart a container."""
    result = subprocess.run(
        ["podman", "restart", f"sovrn-{app_id}"],
        capture_output=True, text=True
    )
    return result.returncode == 0

def notify(title: str, body: str, severity: str = "info"):
    """Send desktop notification via notify bridge."""
    data = json.dumps({
        "title": title,
        "body": body,
        "severity": severity
    }).encode()
    req = urllib.request.Request(
        NOTIFY_BRIDGE,
        data=data,
        headers={"Content-Type": "application/json"}
    )
    try:
        urllib.request.urlopen(req, timeout=5)
    except Exception:
        pass  # Non-critical

def monitor():
    """Main monitoring loop."""
    # Get list of installed apps from sovrnd
    while True:
        try:
            response = urllib.request.urlopen(
                "http://127.0.0.1:54771/api/v1/apps/installed", timeout=10
            )
            apps = json.loads(response.read()).get("apps", [])

            for app in apps:
                app_id = app["id"]
                status = get_container_status(app_id)

                if status["status"] == "not_found":
                    continue

                if status.get("oom_killed"):
                    notify(
                        f"{app['name']} Paused",
                        f"{app['name']} was paused to save memory. Close other apps and restart it.",
                        "warning"
                    )
                elif status["status"] == "running" and not status["healthy"]:
                    # Health check failing — try restart
                    if restart_container(app_id):
                        notify(f"Restarting {app['name']}", f"{app['name']} is being restarted.", "info")

                # Report status to sovrnd
                try:
                    health_url = f"http://127.0.0.1:54771/api/v1/apps/{app_id}/health"
                    urllib.request.urlopen(urllib.request.Request(
                        health_url, method="PUT",
                        data=json.dumps(status).encode(),
                        headers={"Content-Type": "application/json"}
                    ), timeout=5)
                except Exception:
                    pass

        except Exception as e:
            # sovrnd not ready yet, retry
            pass

        time.sleep(CHECK_INTERVAL)

if __name__ == "__main__":
    monitor()
```

---

## sovrn-first-boot Script

File: `/opt/sovrn/bin/sovrn-first-boot`

```python
#!/usr/bin/env python3
"""Sovrn first-boot customization — sets up GNOME desktop after OOBE."""
import subprocess
import os
import json

def run(cmd: str):
    subprocess.run(cmd, shell=True, check=True)

def setup_gnome():
    """Configure GNOME for Sovrn experience."""
    # Set default wallpaper
    run("gsettings set org.gnome.desktop.background picture-uri "
        "'file:///usr/share/backgrounds/sovrn/sovrn-teal.png'")

    # Set dark theme
    run("gsettings set org.gnome.desktop.interface color-scheme 'prefer-dark'")

    # Configure dock (dash-to-dock)
    run("gsettings set org.gnome.shell favorite-apps "
        "'[\"org.gnome.Nautilus.desktop\", \"firefox.desktop\", "
        "\"sovrn-hub.desktop\", \"org.gnome.Settings.desktop\"]'")

    # Set Firefox homepage to Sovrn Hub
    run("gsettings set org.gnome.desktop.screensaver lock-enabled true")

    # Enable night light
    run("gsettings set org.gnome.settings-daemon.plugins.color night-light-enabled true")

    # Remove unnecessary GNOME apps
    apps_to_remove = ["cheese", "gnome-maps", "gnome-contacts", "gnome-weather",
                       "gnome-music", "totem", "evince", "simple-scan"]
    for app in apps_to_remove:
        run(f"dpkg --remove {app} 2>/dev/null || true")

    # Create Sovrn Hub .desktop file
    os.makedirs("/usr/share/applications", exist_ok=True)
    with open("/usr/share/applications/sovrn-hub.desktop", "w") as f:
        f.write("""[Desktop Entry]
Name=Sovrn Hub
Comment=Sovrn Control Center
Exec=firefox --ssb https://sovrn.local/
Icon=sovrn-hub
Type=Application
Categories=Network;System;
StartupNotify=true
""")

    # Set as autostart
    os.makedirs("/etc/xdg/autostart", exist_ok=True)
    with open("/etc/xdg/autostart/sovrn-hub.desktop", "w") as f:
        f.write("""[Desktop Entry]
Name=Sovrn Hub
Comment=Open Sovrn Hub on login
Exec=firefox --ssb https://sovrn.local/
Icon=sovrn-hub
Type=Application
X-GNOME-Autostart-enabled=true
""")

    # Mark first boot as done
    with open("/var/lib/sovrn/.first-boot-done", "w") as f:
        f.write("done")

if __name__ == "__main__":
    setup_gnome()
```