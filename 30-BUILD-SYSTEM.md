# Sovrn OS — Build System Documentation

## Overview

Sovrn OS uses a monorepo with `just` as the task runner. The build produces:
1. 5 Rust crates (mesh services) — compiled via Cargo workspace
2. 1 Go binary (CDN agent) — compiled via `go build`
3. 4 Python packages (sovrnd, auth, monitor, notify-bridge) — installed via pip/setuptools
4. 1 PWA (static site) — built via Vite
5. 1 GTK4 app (OOBE) — built via meson
6. 1 bootable ISO — built via Debos

---

## justfile (Task Runner)

```make
# justfile — Sovrn OS task runner
# Install: cargo install just

default:
    @just --list

# ── Development Setup ──────────────────────────────────────

dev-setup:
    ./scripts/dev-setup.sh

# ── Rust Services (Cargo Workspace) ────────────────────────

build-mesh-services:
    cargo build --release

test-mesh-services:
    cargo test --workspace

lint-mesh-services:
    cargo clippy --workspace -- -D warnings

# ── Python Services ─────────────────────────────────────────

build-sovrnd:
    cd src/sovrnd && python -m build

build-app-monitor:
    cd src/sovrn-app-monitor && python -m build

build-notify-bridge:
    cd src/sovrn-notify-bridge && python -m build

build-auth:
    cd src/sovrn-auth && python -m build

test-sovrnd:
    cd src/sovrnd && python -m pytest tests/ -v

lint-sovrnd:
    cd src/sovrnd && ruff check . && mypy sovrnd/

# ── Go CDN Agent ────────────────────────────────────────────

build-cdn-agent:
    cd src/sovrn-cdn-agent && go build -o ../../target/release/sovrn-cdn-agent ./cmd/sovrn-cdn-agent/

test-cdn-agent:
    cd src/sovrn-cdn-agent && go test ./...

# ── PWA ─────────────────────────────────────────────────────

dev-pwa:
    cd src/pwa && pnpm install && pnpm dev

build-pwa:
    cd src/pwa && pnpm install && pnpm build
    mkdir -p src/debos/overlays/rootfs/var/lib/sovrn/pwa
    cp -r src/pwa/dist/* src/debos/overlays/rootfs/var/lib/sovrn/pwa/

test-pwa:
    cd src/pwa && pnpm test

lint-pwa:
    cd src/pwa && pnpm run lint

# ── OOBE ─────────────────────────────────────────────────────

build-oobe:
    cd src/oobe && meson setup build --prefix=/usr && ninja -C build

test-oobe:
    cd src/oobe && python -m pytest tests/ -v

# ── Build All ────────────────────────────────────────────────

build-all: build-mesh-services build-cdn-agent build-sovrnd build-app-monitor build-notify-bridge build-auth build-pwa build-oobe
    @echo "All components built successfully"

# ── Debian Packages ─────────────────────────────────────────

build-debs: build-all
    ./scripts/build-debs.sh

# ── ISO Build ────────────────────────────────────────────────

build-iso: build-debs
    sudo debos src/debos/sovrn-os.yaml

# ── Test All ────────────────────────────────────────────────

test: test-mesh-services test-cdn-agent test-sovrnd test-pwa test-oobe
    @echo "All tests passed"

# ── Lint All ────────────────────────────────────────────────

lint: lint-mesh-services lint-sovrnd lint-pwa
    @echo "All lints passed"

# ── Clean ────────────────────────────────────────────────────

clean:
    cargo clean
    cd src/pwa && rm -rf node_modules dist
    cd src/oobe && rm -rf build
    cd src/sovrnd && rm -rf dist *.egg-info
    rm -rf target/debs
    rm -rf src/debos/sovrn-os.iso
```

---

## Cargo Workspace Layout

Five Rust crates in a single workspace sharing dependencies:

```
Cargo.toml                    # Workspace root (see sovrn-os/Cargo.toml)
src/sovrn-dht/Cargo.toml      # Member crate
src/sovrn-identity/Cargo.toml  # Member crate
src/sovrn-presence/Cargo.toml  # Member crate
src/sovrn-feed/Cargo.toml      # Member crate
src/sovrn-message-queue/Cargo.toml  # Member crate
```

All share workspace dependencies (`tokio`, `serde`, `rusqlite`, etc.) defined in the root `Cargo.toml`. Release profile: `opt-level = "s"`, `lto = true`, `strip = true` for minimal binary size.

Build command: `cargo build --release` from workspace root. Outputs to `target/release/{sovrn-dht, sovrn-identity, sovrn-presence, sovrn-feed, sovrn-message-queue}`.

---

## Python Packaging — sovrnd

```toml
# src/sovrnd/pyproject.toml
[project]
name = "sovrnd"
version = "0.1.0"
requires-python = ">=3.12"
dependencies = [
    "fastapi>=0.111",
    "uvicorn>=0.30",
    "pydantic>=2.7",
    "httpx>=0.27",
    "tomli>=2.0",
    "pynacl>=1.5",
    "podman>=4.0",
]

[project.scripts]
sovrnd = "sovrnd.__main__:main"

[build-system]
requires = ["setuptools>=70.0"]
build-backend = "setuptools.backends._legacy:_Backend"

[tool.ruff]
target-version = "py312"
line-length = 100

[tool.mypy]
python_version = "3.12"
strict = true
```

sovrnd runs as system Python (no venv in production). Dependencies are installed via apt from Debian packages. Development uses `pip install -e .` with a venv.

Other Python packages (sovrn-auth, sovrn-app-monitor, sovrn-notify-bridge) follow the same pattern.

---

## PWA Build — Vite + Preact

```typescript
// src/pwa/vite.config.ts (already in repo)
// - @preact/preset-vite for JSX transform
// - vite-plugin-pwa for service worker generation
// - Dev proxy: /api → localhost:54771, /ws → ws://localhost:54771
// - Production build: static files to dist/
// - Install target: /var/lib/sovrn/pwa/
```

**Build pipeline:**
1. `pnpm install` — install dependencies
2. `pnpm build` — Vite produces optimized JS/CSS/HTML to `dist/`
3. Copy `dist/` contents to `/var/lib/sovrn/pwa/` (in .deb postinst or Debos overlay)
4. Service worker is auto-generated by `vite-plugin-pwa`

**Development:**
```bash
cd src/pwa && pnpm dev  # Starts Vite dev server on :54772 with HMR
```

---

## Debos Recipe for ISO Generation

```yaml
# src/debos/sovrn-os.yaml
{{- $architecture := "amd64" -}}
{{- $mirror := "http://deb.debian.org/debian" -}}
{{- $suite := "trixie" -}}

architecture: {{ $architecture }}

actions:
  # ── Base System ──────────────────────────────────────────
  - action: debootstrap
    description: "Bootstrap Debian {{ $suite }} base system"
    suite: {{ $suite }}
    mirror: {{ $mirror }}
    variant: minbase

  # ── APT Configuration ───────────────────────────────────
  - action: run
    description: "Configure APT sources and preferences"
    chroot: true
    command: |
      cat > /etc/apt/sources.list << EOF
      deb {{ $mirror }} {{ $suite }} main contrib non-free-firmware
      deb {{ $mirror }} {{ $suite }}-updates main contrib non-free-firmware
      deb http://security.debian.org/debian-security {{ $suite }}-security main contrib non-free-firmware
      EOF
      apt-get update

  # ── Install Packages ────────────────────────────────────
  - action: run
    description: "Install base system packages"
    chroot: true
    script: scripts/install-packages.sh

  # ── Remove Unwanted Packages ─────────────────────────────
  - action: run
    description: "Remove unnecessary packages"
    chroot: true
    command: |
      apt-get -y purge \
        cheese evolution evolution-data-server gnome-maps gnome-contacts \
        gnome-weather gnome-todo gnome-chess aisleriot mahjongg \
        transmission-gtk libreoffice-* && \
      apt-get -y autoremove && \
      apt-get -y clean

  # ── Install Sovrn Packages ───────────────────────────────
  - action: run
    description: "Install Sovrn OS .deb packages"
    chroot: true
    command: |
      dpkg -i /tmp/sovrn-packages/*.deb || apt-get -f install -y

  # ── Install PWA Assets ───────────────────────────────────
  - action: run
    description: "Install Sovrn Hub PWA static assets"
    chroot: true
    command: |
      mkdir -p /var/lib/sovrn/pwa
      cp -r /tmp/pwa-dist/* /var/lib/sovrn/pwa/

  # ── System Configuration ────────────────────────────────
  - action: run
    description: "Configure system settings"
    chroot: true
    script: scripts/configure-system.sh

  # ── Overlay Files ─────────────────────────────────────────
  - action: overlay
    description: "Apply Sovrn rootfs overlay"
    source: overlays/rootfs/

  # ── systemd Enablement ───────────────────────────────────
  - action: run
    description: "Enable systemd services"
    chroot: true
    command: |
      systemctl enable sovrnd
      systemctl enable sovrn-dht
      systemctl enable sovrn-identity
      systemctl enable sovrn-presence
      systemctl enable sovrn-feed
      systemctl enable sovrn-message-queue
      systemctl enable sovrn-cdn-agent
      systemctl enable sovrn-app-monitor
      systemctl enable sovrn-notify-bridge
      systemctl enable sovrn-ca-bootstrap
      systemctl enable sovrn-first-boot
      systemctl enable caddy
      systemctl enable yggdrasil
      systemctl enable zram-setup
      systemctl enable NetworkManager
      systemctl enable systemd-oomd

  # ── Bootloader ───────────────────────────────────────────
  - action: run
    description: "Install systemd-boot"
    chroot: true
    command: |
      bootctl install
      echo "timeout 3" > /boot/loader/loader.conf
      echo "default default.conf" >> /boot/loader/loader.conf
      cat > /boot/loader/entries/default.conf << EOF
      title Sovrn OS
      linux /vmlinuz
      initrd /initrd.img
      options root=LABEL=sovrn-root ro quiet splash
      EOF

  # ── Calamares Installer ──────────────────────────────────
  - action: run
    description: "Install Calamares installer"
    chroot: true
    command: |
      apt-get -y install calamares
      cp -r /tmp/calamares-config/* /etc/calamares/

  # ── Compress ──────────────────────────────────────────────
  - action: run
    description: "Clean up and minimize"
    chroot: true
    command: |
      apt-get -y clean
      rm -rf /var/cache/apt/archives/*
      rm -rf /tmp/*
      fstrim -v /

  # ── Create Image ──────────────────────────────────────────
  - action: image-partition
    description: "Create disk image"
    imagename: sovrn-os.img
    imagesize: 16GB
    partitiontype: gpt
    partitions:
      - name: efi
        fs: vfat
        start: 0%
        end: 512MB
        flags: [boot, esp]
      - name: sovrn-root
        fs: ext4
        start: 512MB
        end: 100%
    mountpoints:
      - mountpoint: /boot/efi
        partition: efi
      - mountpoint: /
        partition: sovrn-root

  - action: run
    description: "Generate ISO from image"
    command: |
      xorriso -as mkisofs \
        -o sovrn-os.iso \
        -isohybrid-mbr /usr/lib/ISOLINUX/isohdpfx.bin \
        -c boot/boot.cat \
        -b boot/efi.img \
        -no-emul-boot \
        -boot-load-size 4 \
        -boot-info-table \
        sovrn-os.img
```

**scripts/install-packages.sh:**
```bash
#!/bin/bash
set -e
apt-get -y install --no-install-recommends \
  # ── Core ──
  linux-image-amd64 systemd-boot network-manager \
  # ── Desktop ──
  gnome-shell gnome-settings-daemon gnome-control-center \
  gdm3 nautilus gnome-console gnome-text-editor \
  gnome-clocks gnome-calculator gnome-screenshot gnome-system-monitor \
  file-roller loupe totem gnome-music \
  # ── Browser ──
  firefox-esr \
  # ── Mesh ──
  yggdrasil yggdrasil-go \
  # ── Web ──
  caddy \
  # ── Containers ──
  podman podman-compose \
  # ── DNS ──
  unbound \
  # ── Security ──
  apparmor apparmor-utils libapparmor1 \
  # ── Python runtime ──
  python3 python3-pip python3-venv \
  # ── Build tools (for in-place updates) ──
  gcc libc6-dev \
  # ── Utilities ──
  borgbackup zram-tools earlyoom dbus-python3 \
  libnotify-bin xdg-utils \
  # ── OOBE deps ──
  python3-gi gir1.2-gtk-4.0 gir1.2-libadwaita-1 \
  gir1.2-gdkpixbuf-2.0 gir1.2-pango-1.0 \
  # ── Installer ──
  calamares \
  # ── Firmware ──
  firmware-linux-nonfree intel-microcode amd64-microcode \
  # ── Misc ──
  polkitd udisks2 upower accountsservice
```

---

## Debian Package Structure

Each .deb package follows standard Debian packaging:

### sovrnd (Python daemon)

```
packaging/sovrnd/
├── debian/
│   ├── control
│   ├── rules
│   ├── install
│   ├── postinst
│   └── postrm
└── Makefile
```

**debian/control:**
```
Source: sovrnd
Section: net
Priority: optional
Maintainer: Sovrn OS Team <team@sovrn.org>
Build-Depends: debhelper (>= 13), python3 (>= 3.12), python3-pip, python3-setuptools
Standards-Version: 4.7.0

Package: sovrnd
Architecture: all
Depends: python3 (>= 3.12), python3-fastapi, python3-uvicorn, python3-pydantic,
         python3-nacl, python3-tomli, python3-httpx,
         ${shlibs:Depends}, ${misc:Depends}
Description: Sovrn OS orchestration daemon
 Main daemon for Sovrn OS. Manages mesh services, self-hosted apps,
 Caddy routes, and provides the REST API for the PWA.
```

**debian/rules:**
```makefile
#!/usr/bin/make -f
export PYBUILD_NAME=sovrnd

%:
	dh $@ --with python3 --buildsystem=pybuild

override_dh_auto_install:
	dh_auto_install
	install -d debian/sovrnd/etc/sovrn
	install -m 644 src/sovrnd/config/sovrnd.toml debian/sovrnd/etc/sovrn/
	install -d debian/sovrnd/lib/systemd/system
	install -m 644 src/systemd-units/sovrnd.service debian/sovrnd/lib/systemd/system/
```

**debian/install:**
```
src/sovrnd/sovrnd/*.py usr/lib/python3/dist-packages/sovrnd/
src/systemd-units/sovrnd.service lib/systemd/system/
src/sovrnd/config/sovrnd.toml etc/sovrn/
```

**debian/postinst:**
```bash
#!/bin/sh
set -e
if [ "$1" = "configure" ]; then
    systemctl daemon-reload
    useradd --system --no-create-home --home /var/lib/sovrn sovrn 2>/dev/null || true
    mkdir -p /var/lib/sovrn /var/log/sovrn /run/sovrn
    chown -R sovrn:sovrn /var/lib/sovrn /var/log/sovrn /run/sovrn
fi
```

### Rust service packages (sovrn-dht, etc.)

Each Rust service .deb follows a similar pattern. Key differences:

**debian/control (sovrn-dht):**
```
Package: sovrn-dht
Architecture: amd64
Depends: libc6 (>= 2.35), ${shlibs:Depends}, ${misc:Depends}, sovrnd (>= 0.1)
Description: Sovrn DHT resolver
 Distributed hash table for .sovrn mesh DNS resolution.
```

**debian/rules (sovrn-dht):**
```makefile
#!/usr/bin/make -f

%:
	dh $@

override_dh_auto_build:
	cargo build --release -p sovrn-dht --target x86_64-unknown-linux-gnu

override_dh_auto_install:
	install -d debian/sovrn-dht/usr/bin
	install -m 755 target/x86_64-unknown-linux-gnu/release/sovrn-dht debian/sovrn-dht/usr/bin/
	install -d debian/sovrn-dht/lib/systemd/system
	install -m 644 src/systemd-units/sovrn-dht.service debian/sovrn-dht/lib/systemd/system/
	install -d debian/sovrn-dht/etc/sovrn
	install -m 644 src/sovrn-dht/config/dht.toml debian/sovrn-dht/etc/sovrn/
```

### Meta package (sovrn-os-meta)

```
Package: sovrn-os-meta
Architecture: all
Depends: sovrnd, sovrn-dht, sovrn-identity, sovrn-presence, sovrn-feed,
         sovrn-message-queue, sovrn-cdn-agent, sovrn-app-monitor,
         sovrn-notify-bridge, sovrn-auth, sovrn-pwa, sovrn-oobe,
         caddy, yggdrasil-go, podman, unbound, borgbackup
Description: Sovrn OS — complete system meta-package
 Installs all Sovrn OS components. This is the base system.
```

---

## CI/CD Pipeline (GitHub Actions)

```yaml
# .github/workflows/build-iso.yml
name: Build Sovrn OS ISO

on:
  push:
    branches: [main]
    tags: ['v*']
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install Go
        uses: actions/setup-go@v5
        with:
          go-version: '1.22'

      - name: Install Python
        uses: actions/setup-python@v5
        with:
          python-version: '3.12'

      - name: Install Node
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install pnpm
        run: npm install -g pnpm

      - name: Test Rust services
        run: cargo test --workspace

      - name: Test Go CDN agent
        run: cd src/sovrn-cdn-agent && go test ./...

      - name: Test Python services
        run: |
          pip install -e src/sovrnd
          pip install pytest && cd src/sovrnd && pytest tests/

      - name: Test PWA
        run: cd src/pwa && pnpm install && pnpm test

      - name: Lint Rust
        run: cargo clippy --workspace -- -D warnings

      - name: Lint Python
        run: pip install ruff && ruff check src/sovrnd src/sovrn-auth

  build-iso:
    needs: test
    runs-on: ubuntu-latest
    if: startsWith(github.ref, 'refs/tags/v')
    steps:
      - uses: actions/checkout@v4

      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y debootstrap qemu-user-static binfmt-support debos xorriso

      - name: Build all components
        run: just build-all

      - name: Build .deb packages
        run: just build-debs

      - name: Build ISO
        run: sudo debos src/debos/sovrn-os.yaml

      - name: Upload ISO
        uses: actions/upload-artifact@v4
        with:
          name: sovrn-os-${{ github.ref_name }}
          path: sovrn-os.iso

      - name: Generate checksums
        run: |
          sha256sum sovrn-os.iso > sovrn-os.iso.sha256
          gpg --default-key ${{ secrets.GPG_KEY_ID }} --detach-sign sovrn-os.iso.sha256
```

---

## Dev Environment Setup Script

```bash
#!/usr/bin/env bash
# scripts/dev-setup.sh — Sovrn OS developer environment setup
set -euo pipefail

GREEN='\033[0;32m'
NC='\033[0m'

echo -e "${GREEN}=== Sovrn OS Developer Setup ===${NC}"

# ── Check prerequisites ──────────────────────────────────────
check_cmd() {
    if ! command -v "$1" &>/dev/null; then
        echo "Missing: $1. Installing..."
        shift
        "$@"
    fi
}

# ── Rust ─────────────────────────────────────────────────────
if ! command -v rustc &>/dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi
echo -e "${GREEN}✓${NC} Rust $(rustc --version)"

# ── Go ────────────────────────────────────────────────────────
if ! command -v go &>/dev/null; then
    echo "Installing Go..."
    sudo apt-get install -y golang-go
fi
echo -e "${GREEN}✓${NC} Go $(go version)"

# ── Python 3.12 ──────────────────────────────────────────────
PYTHON_VERSION=$(python3 --version 2>/dev/null | grep -oP '\d+\.\d+' || echo "0.0")
if [[ "$(echo "$PYTHON_VERSION >= 3.12" | bc -l)" != "1" ]]; then
    echo "Installing Python 3.12..."
    sudo add-apt-repository -y ppa:deadsnakes/ppa
    sudo apt-get install -y python3.12 python3.12-venv python3.12-dev
fi
echo -e "${GREEN}✓${NC} Python $(python3 --version)"

# ── Node 20 + pnpm ───────────────────────────────────────────
if ! command -v node &>/dev/null || [[ "$(node --version)" != v20* ]]; then
    curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
    sudo apt-get install -y nodejs
fi
if ! command -v pnpm &>/dev/null; then
    npm install -g pnpm
fi
echo -e "${GREEN}✓${NC} Node $(node --version), pnpm $(pnpm --version)"

# ── just ──────────────────────────────────────────────────────
if ! command -v just &>/dev/null; then
    cargo install just
fi
echo -e "${GREEN}✓${NC} just $(just --version)"

# ── System deps ───────────────────────────────────────────────
sudo apt-get install -y \
    libsqlite3-dev libssl-dev pkg-config \
    libglib2.0-dev libgtk-4-dev libadwaita-1-dev \
    libnotify-dev podman caddy yggdrasil \
    unbound nftables apparmor borgbackup \
    meson ninja-build dbus-x11

# ── Python virtual environment ────────────────────────────────
python3 -m venv .venv
source .venv/bin/activate
pip install --upgrade pip
pip install -e src/sovrnd
pip install -e src/sovrn-auth
pip install -e src/sovrn-app-monitor
pip install -e src/sovrn-notify-bridge
pip install pytest ruff mypy httpx
deactivate

# ── PWA deps ──────────────────────────────────────────────────
cd src/pwa && pnpm install && cd ../..

# ── GTK4 OOBE build ───────────────────────────────────────────
cd src/oobe && meson setup build --prefix=/usr/local && cd ../..

# ── Rust deps ─────────────────────────────────────────────────
cargo build --workspace

# ── Mock Yggdrasil ────────────────────────────────────────────
echo ""
echo -e "${GREEN}=== Setting up mock Yggdrasil ===${NC}"
cat > /tmp/yggdrasil-dev.conf << 'YGBCONF'
{
  "IfName": "ygg0",
  "Listen": ["tcp://127.0.0.1:54321"],
  "Peers": [],
  "InterfacePeers": {},
  "AllowedEncryptionPublicKeys": [],
  "EncryptionPublicKey": "",
  "EncryptionPrivateKey": "",
  "SigningPublicKey": "",
  "SigningPrivateKey": "",
  "MulticastInterfaces": [{
    "Regex": ".*",
    "Beacon": true,
    "Listen": true,
    "Port": 0,
    "Priority": 0
  }]
}
YGBCONF
echo "Mock Yggdrasil config written to /tmp/yggdrasil-dev.conf"
echo "Run: sudo yggdrasil -useconffile /tmp/yggdrasil-dev.conf"

# ── Seed mock data ────────────────────────────────────────────
echo ""
echo -e "${GREEN}=== Seeding mock data ===${NC}"
python3 scripts/seed-data.sh 2>/dev/null || echo "Mock data script not found yet — create scripts/seed-data.sh"

echo ""
echo -e "${GREEN}=== Setup complete! ===${NC}"
echo "Run 'just dev-setup' any time to re-verify dependencies."
echo "Start dev services with: just run-local"