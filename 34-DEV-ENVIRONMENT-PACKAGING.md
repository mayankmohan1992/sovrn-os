# Sovrn — Dev Environment & Debian Packaging

## Dev Environment Quickstart

### Prerequisites

```bash
# All commands run on Ubuntu/Debian host system

# 1. Install Rust (for mesh services)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup default stable

# 2. Install Python 3.12+
sudo apt install python3.12 python3.12-venv python3.12-dev python3-pip

# 3. Install Node.js 20+ (for PWA)
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install nodejs

# 4. Install system dependencies
sudo apt install \
  build-essential pkg-config libssl-dev \
  libgtk-4-dev libadwaita-1-dev libglib2.0-dev \
  podman caddy yggdrasil \
  libsqlite3-dev \
  borgbackup \
  nftables \
  apparmor apparmor-utils

# 5. Install Preact CLI
npm install -g preact-cli

# 6. Install just (command runner)
curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to $HOME/.local/bin
```

### Clone & Build

```bash
# Clone the repo
git clone https://github.com/sovrn-os/sovrn-os.git
cd sovrn-os

# Run dev setup
just dev-setup

# Build all Rust services
just build-mesh

# Build sovrnd (Python)
just build-sovrnd

# Build PWA
just build-pwa

# Run tests
just test
```

### justfile

File: `justfile`

```makefile
# Sovrn OS — Just command runner

# Default vars
rust-target := "release"
pwa-port := "54772"

# Dev setup — install all prerequisites
dev-setup:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "=== Installing Rust dependencies ==="
    cd src/sovrn-dht && cargo build --{{rust-target}} && cd ../..
    echo "=== Setting up Python venv ==="
    python3.12 -m venv .venv
    . .venv/bin/activate && pip install -e src/sovrnd/
    echo "=== Setting up PWA ==="
    cd src/pwa && npm install && cd ../..
    echo "=== Creating runtime dirs ==="
    mkdir -p /tmp/sovrn-dev/{sockets,data,ca}
    echo "=== Dev environment ready ==="

# Build all mesh services (Rust)
build-mesh:
    cd src && cargo build --{{rust-target}} --workspace

# Build sovrnd (Python)
build-sovrnd:
    . .venv/bin/activate && pip install -e src/sovrnd/

# Build PWA
build-pwa:
    cd src/pwa && npm run build
    cp -r src/pwa/dist/* /tmp/sovrn-dev/data/pwa/

# Build OOBE wizard (GTK4)
build-oobe:
    cd src/oobe && pip install -e .

# Build everything
build-all: build-mesh build-sovrnd build-pwa build-oobe

# Run sovrnd in dev mode
run-sovrnd:
    . .venv/bin/activate && sovrnd --config dev-configs/sovrnd.toml

# Run PWA dev server (with HMR)
run-pwa:
    cd src/pwa && npm run dev -- --port {{pwa-port}}

# Run specific mesh service
run-service service:
    cd src/{{service}} && cargo run --{{rust-target}}

# Run all services locally (dev mode)
run-all:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Starting Yggdrasil..."
    sudo yggdrasil -useconffile dev-configs/yggdrasil-dev.conf &
    echo "Starting sovrn-dht..."
    just run-service sovrn-dht &
    echo "Starting sovrn-identity..."
    just run-service sovrn-identity &
    echo "Starting sovrn-presence..."
    just run-service sovrn-presence &
    echo "Starting sovrn-feed..."
    just run-service sovrn-feed &
    echo "Starting sovrn-mq..."
    just run-service sovrn-message-queue &
    echo "Starting sovrnd..."
    just run-sovrnd &
    echo "Starting PWA dev server..."
    just run-pwa &
    echo "=== All services running ==="
    echo "PWA: http://localhost:{{pwa-port}}"
    echo "API: http://localhost:54771/api/v1"
    echo "Press Ctrl+C to stop all"

# Run tests
test:
    cd src && cargo test --workspace
    . .venv/bin/activate && cd src/sovrnd && pytest
    cd src/pwa && npm test

# Lint all code
lint:
    cd src && cargo clippy --workspace -- -D warnings
    . .venv/bin/activate && cd src/sovrnd && ruff check .
    cd src/pwa && npm run lint

# Clean build artifacts
clean:
    cd src && cargo clean
    rm -rf .venv src/pwa/node_modules src/pwa/dist
    rm -rf /tmp/sovrn-dev

# Build bootable ISO (requires Debos)
build-iso:
    cd packaging && sudo debos recipes/sovrn-os.yaml

# Package .deb files
package-debs:
    cd packaging && ./build-debs.sh

# Generate self-signed certs for dev
dev-certs:
    mkdir -p /tmp/sovrn-dev/ca
    openssl req -x509 -newkey ed25519 -nodes \
        -keyout /tmp/sovrn-dev/ca/sovrn.local.key \
        -out /tmp/sovrn-dev/ca/sovrn.local.pem \
        -days 365 -subj "/CN=sovrn.local"

# Seed mock data for development
seed-data:
    . .venv/bin/activate && python scripts/seed-dev-data.py
```

---

## Debian Package Structure

### sovrnd (Python daemon) — pyproject.toml

File: `src/sovrnd/pyproject.toml`

```toml
[build-system]
requires = ["setuptools>=68.0", "wheel"]
build-backend = "setuptools.build_meta"

[project]
name = "sovrnd"
version = "0.1.0"
description = "Sovrn OS orchestrator daemon"
requires-python = ">=3.12"
dependencies = [
    "fastapi>=0.110.0",
    "uvicorn[standard]>=0.29.0",
    "pydantic>=2.6",
    "podman>=5.0.0",
    "python-jose[cryptography]>=3.3.0",
    "cryptography>=42.0",
    "httpx>=0.27",
    "aiosqlite>=0.19",
    "websockets>=12.0",
    "tomli>=2.0;python_version<'3.11'",
]

[project.scripts]
sovrnd = "sovrnd.main:main"
sovrn-app-monitor = "sovrnd.app_monitor:main"

[tool.setuptools.packages.find]
where = ["src"]

[tool.ruff]
line-length = 100
target-version = "py312"

[tool.pytest.ini_options]
testpaths = ["tests"]
asyncio_mode = "auto"
```

### Cargo Workspace — src/Cargo.toml

File: `src/Cargo.toml`

```toml
[workspace]
resolver = "2"
members = [
    "sovrn-dht",
    "sovrn-identity",
    "sovrn-presence",
    "sovrn-feed",
    "sovrn-message-queue",
    "sovrn-cdn-agent",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "GPL-3.0"

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.7", features = ["runtime-tokio", "sqlite"] }
ed25519-dalek = { version = "2", features = ["serde"] }
x25519-dalek = { version = "2", features = ["serde"] }
sha2 = "0.10"
rand = "0.8"
tracing = "0.1"
tracing-subscriber = "0.3"
thiserror = "1"
anyhow = "1"
```

### Debian package for sovrnd

Directory: `packaging/debs/sovrnd/`

```
packaging/debs/sovrnd/
├── debian/
│   ├── control
│   ├── rules
│   ├── changelog
│   ├── compat
│   ├── install
│   ├── postinst
│   ├── prerm
│   └── sovrnd.service
├── src/
│   └── (sovrnd Python source)
└── Makefile
```

`packaging/debs/sovrnd/debian/control`:
```
Source: sovrnd
Section: net
Priority: optional
Maintainer: Sovrn Project <dev@sovrn.org>
Build-Depends: debhelper (>= 12), python3 (>= 3.12), python3-pip
Standards-Version: 4.6.0

Package: sovrnd
Architecture: all
Depends: python3 (>= 3.12), python3-venv, caddy, podman, yggdrasil, sqlite3, libsqlite3-0
Recommends: sovrn-dht, sovrn-identity, sovrn-presence, sovrn-feed, sovrn-mq, sovrn-cdn-agent
Suggests: borgbackup, nftables
Description: Sovrn OS orchestrator daemon
 Manages self-hosted apps, mesh services, and system configuration
 for the Sovrn privacy-first operating system.
```

`packaging/debs/sovrnd/debian/install`:
```
opt/sovrn/bin/sovrnd /opt/sovrn/bin/
opt/sovrn/bin/sovrn-app-monitor /opt/sovrn/bin/
opt/sovrn/bin/sovrn-notify-bridge /opt/sovrn/bin/
etc/sovrn/sovrnd.toml /etc/sovrn/
etc/caddy/Caddyfile /etc/caddy/
etc/nftables/sovrn.nft /etc/nftables/
usr/local/bin/sovrn-bootstrap-ca /usr/local/bin/
usr/local/bin/sovrn-dns-dispatcher /usr/local/bin/
usr/local/bin/sovrn-podman-setup.sh /usr/local/bin/
lib/systemd/system/sovrnd.service /lib/systemd/system/
lib/systemd/system/sovrn-app-monitor.service /lib/systemd/system/
lib/systemd/system/sovrn-notify-bridge.service /lib/systemd/system/
lib/systemd/system/sovrn-ca-bootstrap.service /lib/systemd/system/
lib/systemd/system/sovrn-first-boot.service /lib/systemd/system/
lib/systemd/system/sovrn-dns-dispatcher.service /lib/systemd/system/
lib/systemd/system/zram-setup.service /lib/systemd/system/
lib/systemd/system/sovrn-apps.slice /lib/systemd/system/
```

`packaging/debs/sovrnd/debian/postinst`:
```bash
#!/bin/bash
set -e

# Create sovrn user and group
if ! id -u sovrn > /dev/null 2>&1; then
    useradd --system --home-dir /var/lib/sovrn --shell /usr/sbin/nologin sovrn
fi

# Create directories
mkdir -p /var/lib/sovrn/{sockets,data,ca,images,app-data,pwa,apps,logs}
mkdir -p /etc/sovrn
chown -R sovrn:sovrn /var/lib/sovrn

# Set up Podman network
/usr/local/bin/sovrn-podman-setup.sh || true

# Enable linger for sovrn user
loginctl enable-linger sovrn 2>/dev/null || true

# Enable services
systemctl daemon-reload
systemctl enable sovrnd.service
systemctl enable sovrn-app-monitor.service
systemctl enable sovrn-notify-bridge.service
systemctl enable sovrn-ca-bootstrap.service
systemctl enable sovrn-first-boot.service
systemctl enable sovrn-dns-dispatcher.service
systemctl enable zram-setup.service

# Enable Caddy
systemctl enable caddy.service
```

### Debian packages for Rust services

Each Rust service follows the same pattern. Build with `cargo deb`:

```bash
# Build sovrn-dht .deb
cd src/sovrn-dht && cargo deb --target x86_64-unknown-linux-gnu

# Output: target/debian/sovrn-dht_0.1.0_amd64.deb
```

Each service Cargo.toml includes:

```toml
[package.metadata.deb]
maintainer = "Sovrn Project <dev@sovrn.org>"
copyright = "2025, Sovrn Project"
license-file = ["LICENSE", "3"]
depends = "libc6 (>= 2.31)"
section = "net"
priority = "optional"
assets = [
    ["target/release/sovrn-dht", "/usr/local/bin/sovrn-dht", "755"],
    ["config/dht.toml", "/etc/sovrn/dht.toml", "644"],
]
systemd-units = { unit-name = "sovrn-dht", unit-scripts = "debian/" }
```

---

## Debos ISO Build Recipe

File: `packaging/recipes/sovrn-os.yaml`

```yaml
architecture: amd64

actions:
  - action: debootstrap
    suite: testing
    components:
      - main
      - contrib
      - non-free-firmware
    mirror: http://deb.debian.org/debian

  - action: apt
    description: Install base system packages
    packages:
      - systemd
      - systemd-sysv
      - linux-image-amd64
      - grub-efi-amd64
      - firmware-linux-nonfree
      - sudo
      - dbus
      - network-manager
      - wpasupplicant
      - nftables
      - apparmor
      - apparmor-utils
      - auditd
      - gnome-shell
      - gdm3
      - gnome-control-center
      - gnome-terminal
      - nautilus
      - firefox-esr
      - podman
      - caddy
      - yggdrasil
      - borgbackup
      - sqlite3
      - python3.12
      - python3.12-venv
      - python3-pip
      - git
      - curl
      - libgtk-4-dev
      - libadwaita-1-dev
      - libglib2.0-dev

  - action: apt
    description: Remove unnecessary GNOME apps
    packages:
      - cheese
      - gnome-maps
      - gnome-contacts
      - gnome-weather
      - gnome-music
      - totem
      - simple-scan
      - evince
    remove: true

  - action: run
    description: Create sovrn user
    command: |
      useradd --system --home-dir /var/lib/sovrn --shell /usr/sbin/nologin sovrn
      mkdir -p /var/lib/sovrn/{sockets,data,ca,images,app-data,pwa,apps,logs}
      chown -R sovrn:sovrn /var/lib/sovrn

  - action: run
    description: Install Sovrn packages
    command: |
      dpkg -i /sovrn-packages/sovrnd_0.1.0_amd64.deb
      dpkg -i /sovrn-packages/sovrn-dht_0.1.0_amd64.deb
      dpkg -i /sovrn-packages/sovrn-identity_0.1.0_amd64.deb
      dpkg -i /sovrn-packages/sovrn-presence_0.1.0_amd64.deb
      dpkg -i /sovrn-packages/sovrn-feed_0.1.0_amd64.deb
      dpkg -i /sovrn-packages/sovrn-mq_0.1.0_amd64.deb
      dpkg -i /sovrn-packages/sovrn-cdn-agent_0.1.0_amd64.deb

  - action: run
    description: Copy PWA static assets
    command: |
      mkdir -p /var/lib/sovrn/pwa
      cp -r /pwa-build/dist/* /var/lib/sovrn/pwa/
      chown -R sovrn:sovrn /var/lib/sovrn/pwa

  - action: run
    description: Copy pre-baked container images
    command: |
      mkdir -p /var/lib/sovrn/images
      for img in /container-images/*.tar; do
        podman load -i "$img"
      done

  - action: run
    description: Configure system
    command: |
      # Enable services
      systemctl enable sovrnd.service
      systemctl enable sovrn-app-monitor.service
      systemctl enable sovrn-notify-bridge.service
      systemctl enable sovrn-ca-bootstrap.service
      systemctl enable sovrn-first-boot.service
      systemctl enable sovrn-dns-dispatcher.service
      systemctl enable zram-setup.service
      systemctl enable caddy.service
      systemctl enable yggdrasil.service

      # Configure GRUB
      sed -i 's/GRUB_CMDLINE_LINUX_DEFAULT=.*/GRUB_CMDLINE_LINUX_DEFAULT="quiet splash"/' /etc/default/grub
      update-grub

      # Set hostname
      echo "sovrn" > /etc/hostname

  - action: run
    description: Install Calamares installer
    packages:
      - calamares
      - calamares-settings-debian

  - action: run
    description: Copy OOBE wizard
    command: |
      cp -r /oobe-build/* /usr/lib/sovrn/oobe/
      cp /usr/lib/sovrn/oobe/sovrn-oobe.desktop /etc/xdg/autostart/

  - action: run
    description: Generate squashfs and create ISO
    command: |
      # This is handled by Debos automatically
      # The resulting image will be at /output/sovrn-os-amd64.iso
```

---

## Dev Configuration Files

### dev-configs/sovrnd.toml (development)

```toml
[general]
hostname = "sovrn-dev"
data_dir = "/tmp/sovrn-dev/data"
sockets_dir = "/tmp/sovrn-dev/sockets"
log_level = "debug"

[api]
host = "127.0.0.1"
port = 54771
cors_origins = ["http://localhost:54772", "https://sovrn.local"]
max_upload_mb = 25

[auth]
jwt_expiry_hours = 168  # 7 days for dev
jwt_refresh_hours = 24

[dns]
local_tld = "sovrn.local"
mesh_tld = "sovrn"
dns_port = 53535

[yggdrasil]
socket_path = "/var/run/yggdrasil/yggdrasil.sock"
config_path = "dev-configs/yggdrasil-dev.conf"

[podman]
network_name = "sovrn-apps"
network_subnet = "10.47.0.0/16"
image_cache_dir = "/tmp/sovrn-dev/images"
app_data_dir = "/tmp/sovrn-dev/app-data"

[defaults]
preinstalled_apps = []
auto_start_apps = false
```

### dev-configs/yggdrasil-dev.conf (development)

```yaml
IfName: ygg0
AdminListen: unix:///tmp/sovrn-dev/yggdrasil.sock
Peers: []  # No real peers in dev mode
MulticastInterfaces:
  - Regex: .*
    Beacon: false
    Listen: false
    Port: 0
SessionFirewall:
  Enable: false
LogLevel: debug
```

### scripts/seed-dev-data.py

```python
#!/usr/bin/env python3
"""Seed development database with mock data."""
import asyncio
import aiosqlite
import json
import os

DATA_DIR = os.environ.get("SOVRN_DATA_DIR", "/tmp/sovrn-dev/data")

async def seed_feed():
    async with aiosqlite.connect(f"{DATA_DIR}/feed/feed.db") as db:
        with open("src/sovrnd/migrations/V001_initial.sql") as f:
            await db.executescript(f.read())
        # Insert mock events
        await db.executemany(
            "INSERT INTO events (id, kind, author, content, created_at, sig, raw_json) VALUES (?, 1, ?, ?, ?, '', '{}')",
            [
                ("mock1", "alice", "Hello from dev mode!", 1749000000),
                ("mock2", "bob", "Testing Sovrn in development 🚀", 1749000060),
                ("mock3", "alice", "Self-hosting is the future!", 1749000120),
            ]
        )
        await db.commit()

async def seed_identity():
    async with aiosqlite.connect(f"{DATA_DIR}/identity/identity.db") as db:
        with open("src/sovrnd/migrations/V001_initial.sql") as f:
            await db.executescript(f.read())
        # Insert mock identity
        await db.execute(
            "INSERT INTO keys (key_type, public_key) VALUES (?, ?)",
            ("ed25519_master", "alice_dev_public_key")
        )
        await db.execute(
            "INSERT INTO aliases (name, domain) VALUES (?, ?)",
            ("alice-dev", "alice-dev.sovrn")
        )
        await db.commit()

async def seed_apps():
    async with aiosqlite.connect(f"{DATA_DIR}/config/config.db") as db:
        with open("src/sovrnd/migrations/V001_initial.sql") as f:
            await db.executescript(f.read())
        # Insert available apps
        await db.executemany(
            "INSERT INTO app_registry (app_id, name, tier, version, local_image) VALUES (?, ?, ?, ?, ?)",
            [
                ("nextcloud", "Nextcloud", 1, "33.0.2", 1),
                ("vaultwarden", "Vaultwarden", 1, "1.34.0", 1),
                ("freshrss", "FreshRSS", 1, "1.29.0", 1),
                ("wallabag", "Wallabag", 2, "2.6.0", 0),
                ("navidrome", "Navidrome", 2, "0.61.0", 0),
            ]
        )
        await db.commit()

async def main():
    os.makedirs(f"{DATA_DIR}/feed", exist_ok=True)
    os.makedirs(f"{DATA_DIR}/identity", exist_ok=True)
    os.makedirs(f"{DATA_DIR}/config", exist_ok=True)
    os.makedirs(f"{DATA_DIR}/messages", exist_ok=True)
    os.makedirs(f"{DATA_DIR}/presence", exist_ok=True)
    os.makedirs(f"{DATA_DIR}/cdn", exist_ok=True)

    print("Seeding feed database...")
    await seed_feed()
    print("Seeding identity database...")
    await seed_identity()
    print("Seeding app registry...")
    await seed_apps()
    print("Done! Mock data seeded.")

if __name__ == "__main__":
    asyncio.run(main())
```

---

## Summary: What Now Exists

| File | What It Contains |
|------|------------------|
| 26-API-SPECIFICATION.md | Complete REST API for all 16 endpoint groups with JSON schemas |
| 27-DATABASE-SCHEMAS.md | CREATE TABLE statements for all 8 SQLite databases |
| 28-IPC-NETWORKING-CONFIGS.md | JSON-RPC IPC protocol, Caddy Caddyfile, DNS dispatcher, nftables, AppArmor, CA bootstrap, Yggdrasil config |
| 29-SYSTEMD-CONFIGS.md | All systemd unit files, sovrnd.toml, Podman network setup, OOM config, app monitor script, first-boot script |
| 25-LANGUAGE-DECISIONS.md | Language selection for every component |
| 23-REPO-STRUCTURE.md | Full repo directory layout |
| 30-BUILD-SYSTEM.md | justfile, Cargo workspace, Python packaging, Debos ISO recipe |

A developer can now: clone the repo, run `just dev-setup`, `just build-all`, `just run-all`, and have all services running with mock data.