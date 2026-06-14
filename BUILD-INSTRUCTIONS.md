# Sovrn OS — Build Instructions for AI Coding Agents

> **READ THIS FILE FIRST.** This is the single entry point for building Sovrn OS from source to a bootable ISO. All other design documents are reference material; this file is the execution plan.

---

## 0. What You're Building

Sovrn OS is a privacy-first, decentralized operating system based on Debian Trixie with:
- **5 Rust mesh services** (DHT, Identity, Presence, Feed, Message Queue)
- **1 Rust GTK4 OOBE wizard** (first-boot setup)
- **4 Python services** (sovrnd orchestrator, Auth, Monitor, Notify Bridge)
- **1 Go CDN agent** + **1 Go Caddy auth plugin**
- **1 Preact PWA** (Sovrn Hub web interface)
- **15 systemd units**, nftables firewall, Unbound DNS, Yggdrasil mesh, AppArmor, CA bootstrap
- **Bootable ISO** via Debos

**Total: 178 source files across 7 languages.**

---

## 1. Prerequisites

### macOS (Apple Silicon M2/M3)

```bash
# Install Homebrew packages
brew install rust go python@3.12 node npm pkg-config dbus

# Install cargo components
rustup target add x86_64-unknown-linux-gnu  # cross-compile target
cargo install cross                            # cross-compilation helper

# Install Docker Desktop (REQUIRED for Debos — Debos only runs on Linux)
# Download from https://docker.com — start Docker Desktop before building

# Install xcaddy for Caddy auth plugin
go install github.com/caddyserver/xcaddy/cmd/xcaddy@latest

# Python venv
python3.12 -m venv ~/.venvs/sovrn
source ~/.venvs/sovrn/bin/activate
pip install fastapi uvicorn pydantic httpx pynacl PyJWT websockets tomli psutil
```

### Linux Mint 22 / Ubuntu 24.04+

```bash
# System packages
sudo apt update && sudo apt install -y \
  build-essential rustc cargo golang-go python3 python3-pip python3-venv \
  nodejs npm pkg-config libssl-dev libgtk-4-dev libadwaita-1-dev \
  debos docker.io yggdrasil unbound nftables caddy \
  qemu-system-x86 ovmf squashfs-tools xorriso isolinux

# Add user to docker group
sudo usermod -aG docker $USER

# Rust (if not installed via apt)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Go (if apt version is old)
# Follow https://go.dev/dl/ for 1.22+

# xcaddy
go install github.com/caddyserver/xcaddy/cmd/xcaddy@latest

# Python deps
python3 -m venv ~/.venvs/sovrn
source ~/.venvs/sovrn/bin/activate
pip install fastapi uvicorn pydantic httpx pynacl PyJWT websockets tomli psutil
```

---

## 2. Project Structure

```
the-os-project/
├── BUILD-INSTRUCTIONS.md          ← YOU ARE HERE
├── BUILD-READINESS-ASSESSMENT.md ← Reference: original readiness doc
├── SESSION-STATE.md               ← Reference: session tracking
├── 23-REPO-STRUCTURE.md ... 34-DEV-ENVIRONMENT-PACKAGING.md  ← Design docs
│
└── sovrn-os/                      ← SOURCE ROOT
    ├── Cargo.toml                 ← Rust workspace (6 crates)
    ├── src/
    │   ├── sovrn-dht/             ← Rust: DHT service
    │   ├── sovrn-identity/        ← Rust: Identity service
    │   ├── sovrn-presence/        ← Rust: Presence service
    │   ├── sovrn-feed/           ← Rust: Feed service
    │   ├── sovrn-message-queue/  ← Rust: Message Queue service
    │   ├── oobe/                 ← Rust: OOBE wizard (GTK4)
    │   ├── sovrn-cdn-agent/      ← Go: CDN agent
    │   ├── caddy-auth/           ← Go: Caddy JWT auth plugin
    │   ├── sovrnd/               ← Python: Central orchestrator (FastAPI)
    │   ├── sovrn-auth/           ← Python: Policy engine (FastAPI)
    │   ├── sovrn-monitor/        ← Python: System monitor (FastAPI)
    │   ├── sovrn-notify-bridge/  ← Python: Notification bridge (FastAPI)
    │   ├── pwa/                  ← Preact: Sovrn Hub web interface
    │   ├── systemd-units/        ← 15 systemd .service/.target files
    │   ├── caddy-config/         ← Caddyfile
    │   ├── nftables/             ← Firewall rules
    │   ├── dns/                  ← Unbound config
    │   ├── yggdrasil/            ← Mesh network config
    │   ├── apparmor/             ← Profiles
    │   └── gnome-customization/  ← dconf overrides
    ├── build/
    │   ├── debos-sovrn.yaml      ← Debos ISO recipe
    │   ├── build-iso.sh           ← ISO build wrapper script
    │   └── calamares/            ← Installer config
    ├── packaging/
    │   └── debian/               ← Debian package control files
    └── scripts/
        ├── build.sh              ← Component build script
        └── bootstrap-ca.sh       ← CA bootstrap
```

---

## 3. Port Map (MUST be consistent)

All services use these ports. **Do NOT change them without updating ALL references:**

| Service | Port | Protocol | Config Location |
|---------|------|----------|----------------|
| sovrnd | 54771 | HTTP/REST | `sovrnd/configs/sovrnd.toml`, `sovrnd/sovrnd/config.py` |
| PWA Hub | 54772 | HTTP | `caddy-config/Caddyfile` |
| Notify Bridge | 54773 | HTTP/WS | `sovrn-notify-bridge/__main__.py` |
| DHT mesh | 54774 | TCP/UDP | `sovrn-dht/src/lib.rs`, `nftables/sovrn.nft` |
| Presence | 54775 | UDP | `sovrn-presence/src/lib.rs` |
| Auth | 54776 | HTTP | `sovrn-auth/__main__.py` |
| Monitor | 54777 | HTTP | `sovrn-monitor/__main__.py` |
| Yggdrasil | 30550 | TCP/UDP | `yggdrasil/yggdrasil.conf` |

**TLD**: `.sovrn` everywhere. No references to `.os` remain (migrated).

---

## 4. Build Order

### Step 4.1: Rust Workspace (5 mesh services + OOBE)

```bash
cd sovrn-os

# Build all Rust crates
cargo build --release --workspace

# Expected outputs in target/release/:
#   sovrn-dht, sovrn-identity, sovrn-presence, sovrn-feed,
#   sovrn-message-queue, sovrn-oobe

# Copy to build dir
mkdir -p build/bin
cp target/release/sovrn-{dht,identity,presence,feed,message-queue,oobe} build/bin/
```

**Common build errors and fixes:**

| Error | Fix |
|-------|-----|
| `openssl-sys` build fails | `sudo apt install libssl-dev` (Linux) or `brew install openssl` then `export OPENSSL_DIR=/opt/homebrew/opt/openssl` (macOS) |
| `rusqlite` build fails | Ensure `libsqlite3-dev` installed, or use `bundled` feature (already set in Cargo.toml) |
| `gtk4` build fails | `sudo apt install libgtk-4-dev libadwaita-1-dev` (Linux). On macOS: cross-compile or skip OOBE binary |
| Cross-compilation for Linux on macOS | Use `cross build --target x86_64-unknown-linux-gnu --release` with Docker |

### Step 4.2: Go CDN Agent

```bash
cd sovrn-os/src/sovrn-cdn-agent

# Build
CGO_ENABLED=0 go build -o ../../build/bin/sovrn-cdn-agent ./cmd/sovrn-cdn-agent/

# Run tests
go test ./... -v
```

### Step 4.3: Go Caddy Auth Plugin

```bash
cd sovrn-os/src/caddy-auth

# Build custom Caddy binary with auth plugin
xcaddy build --with github.com/sovrn-os/caddy-auth=. \
  --output ../../build/bin/caddy

# If xcaddy not available:
go build -o ../../build/bin/caddy-auth-plugin .  # standalone, not as Caddy module
```

### Step 4.4: Python Services

```bash
# Create venv (if not done)
python3 -m venv .venv
source .venv/bin/activate

# Install each service
cd sovrn-os/src/sovrnd && pip install -e .
cd ../sovrn-auth && pip install -e .
cd ../sovrn-monitor && pip install -e .
cd ../sovrn-notify-bridge && pip install -e .

# Verify
python3 -c "import sovrnd; print(sovrnd.__version__)"
python3 -c "import sovrn_auth; print(sovrn_auth.__version__)"
```

### Step 4.5: Preact PWA

```bash
cd sovrn-os/src/pwa

# Install dependencies
npm install

# Development build
npm run dev     # Starts dev server at localhost:54772

# Production build
npm run build   # Outputs to dist/

# Copy to build dir
mkdir -p ../../build/share/pwa-dist
cp -r dist/ ../../build/share/pwa-dist/
```

### Step 4.6: Configuration Files

```bash
cd sovrn-os

# Copy systemd units
mkdir -p build/etc
cp src/systemd-units/*.service src/systemd-units/*.target build/etc/

# Copy configs
mkdir -p build/etc/sovrn build/etc/caddy build/etc/nftables build/etc/unbound
cp src/sovrnd/configs/sovrnd.toml build/etc/sovrn/
cp src/caddy-config/Caddyfile build/etc/caddy/
cp src/nftables/sovrn.nft build/etc/nftables/
cp src/dns/unbound-sovrn.conf build/etc/unbound/sovrn.conf
cp src/yggdrasil/yggdrasil.conf build/etc/sovrn/
cp src/apparmor/usr.bin.sovrnd build/etc/ 2>/dev/null || true

# Copy GNOME customization
mkdir -p build/gnome/etc/dconf/{profile,db}
cp src/gnome-customization/dconf-profile build/gnome/etc/dconf/profile/sovrn
cp src/gnome-customization/dconf-db-sovrn.ini build/gnome/etc/dconf/db/sovrn

# Copy scripts
mkdir -p build/scripts
cp scripts/bootstrap-ca.sh build/scripts/
chmod +x build/scripts/bootstrap-ca.sh
```

---

## 5. Build the ISO

### Option A: Linux (Native Debos)

```bash
cd sovrn-os

# Install Debos
sudo apt install debos qemu-system-x86 ovmf squashfs-tools xorriso isolinux

# Build ISO
sudo debos build/debos-sovrn.yaml \
  --architecture amd64 \
  --output build/sovrn-os-0.1.0-amd64.iso
```

### Option B: macOS (Docker)

```bash
cd sovrn-os

# Use the build wrapper (sets up Docker automatically)
chmod +x build/build-iso.sh
./build/build-iso.sh

# Or manually:
docker pull ghcr.io/go-debos/debos:latest

docker run --rm \
  --device /dev/kvm \
  --device /dev/fuse \
  --group-add kvm \
  -v "$(pwd)":/project \
  -v "$(pwd)/build":/build \
  -w /project \
  ghcr.io/go-debos/debos:latest \
  debos build/debos-sovrn.yaml \
  --architecture amd64 \
  --output build/sovrn-os-0.1.0-amd64.iso
```

### Option C: Linux Mint specifically

```bash
# Same as Option A, but ensure kvm group access:
sudo usermod -aG kvm $USER
# Log out and back in, then:
sudo debos build/debos-sovrn.yaml --architecture amd64
```

---

## 6. Flash and Boot

### Write ISO to USB

```bash
# Linux
sudo dd if=build/sovrn-os-0.1.0-amd64.iso of=/dev/sdX bs=4M status=progress && sync

# macOS (use rdisk for speed)
diskutil eraseDisk FAT32 SOVRN /dev/diskN
sudo dd if=build/sovrn-os-0.1.0-amd64.iso of=/dev/rdiskN bs=4m
```

Or use [Ventoy](https://ventoy.net) or [Balena Etcher](https://etcher.balena.io/) for a GUI approach.

### Boot

1. Insert USB into target machine
2. Boot from USB (F12/F2/Esc depending on vendor)
3. Select "Sovrn OS" from GRUB menu
4. On first boot, the OOBE wizard launches automatically
5. Follow: Language → Identity Creation → Domain Registration → Network Setup → Complete

---

## 7. Known Issues and Workarounds

### Rust cross-compilation on macOS

The OOBE binary requires GTK4/libadwaita Linux headers. Options:
1. **Cross-compile** using `cross` (Docker-based): `cross build --target x86_64-unknown-linux-gnu --release`
2. **Skip OOBE** for testing: Comment out `"src/oobe"` from `Cargo.toml` workspace members
3. **Build OOBE separately** in a Linux container or VM

### Debos requires KVM

On macOS, Docker Desktop must have "Use Virtualization Framework" enabled in Settings → General. The `--device /dev/kvm` flag may not work; use the `--privileged` flag instead:

```bash
docker run --rm --privileged \
  -v "$(pwd)":/project \
  ghcr.io/go-debos/debos:latest \
  debos build/debos-sovrn.yaml
```

### PWA build requires Node

If `npm install` fails on M2 Air (8GB RAM), use:
```bash
NODE_OPTIONS=--max_old_space_size=4096 npm run build
```

### Python service dependencies

If `pip install -e .` fails, ensure the venv has the right Python version:
```bash
python3 --version  # Must be >= 3.12
pip install --upgrade pip setuptools wheel
```

---

## 8. Verification Checklist

Before declaring the build successful, verify:

- [ ] `cargo build --release --workspace` succeeds with no errors
- [ ] `go build ./cmd/sovrn-cdn-agent/` succeeds in `src/sovrn-cdn-agent/`
- [ ] `npm run build` succeeds in `src/pwa/`
- [ ] All 15 systemd units reference existing binary paths
- [ ] Port 54771 appears in: `sovrnd.toml`, `config.py`, `app.py`, `__init__.py`, `nft`, `unbound`
- [ ] Port 30550 appears in: `yggdrasil.conf`, `nft`
- [ ] `.sovrn` TLD used consistently (no `.os` references)
- [ ] Caddyfile references `sovrnd.sock` and port `54772`
- [ ] All Rust `Cargo.toml` files have correct workspace path
- [ ] Python `pyproject.toml` files list correct dependencies
- [ ] Generated ISO boots in QEMU: `qemu-system-x86_64 -m 4G -cdrom sovrn-os-0.1.0-amd64.iso`

---

## 9. Quick Reference: Service Binary Names

| Binary | Language | Service Unit | Source Path |
|--------|----------|-------------|-------------|
| `sovrn-dht` | Rust | `sovrn-dht.service` | `src/sovrn-dht/` |
| `sovrn-identity` | Rust | `sovrn-identity.service` | `src/sovrn-identity/` |
| `sovrn-presence` | Rust | `sovrn-presence.service` | `src/sovrn-presence/` |
| `sovrn-feed` | Rust | `sovrn-feed.service` | `src/sovrn-feed/` |
| `sovrn-message-queue` | Rust | `sovrn-message-queue.service` | `src/sovrn-message-queue/` |
| `sovrn-oobe` | Rust/GTK4 | `sovrn-oobe.service` | `src/oobe/` |
| `sovrn-cdn-agent` | Go | `sovrn-cdn-agent.service` | `src/sovrn-cdn-agent/` |
| `sovrnd` | Python | `sovrnd.service` | `src/sovrnd/` |
| `sovrn-auth` | Python | `sovrn-auth.service` | `src/sovrn-auth/` |
| `sovrn-monitor` | Python | `sovrn-monitor.service` | `src/sovrn-monitor/` |
| `sovrn-notify-bridge` | Python | `sovrn-notify-bridge.service` | `src/sovrn-notify-bridge/` |
| `caddy` | Go+Plugin | `caddy.service` (system) | `src/caddy-auth/` |

---

## 10. File Count Summary

- **52 Rust source files** (.rs) across 6 crates
- **30 Python source files** (.py) across 4 services
- **9 Go source files** across 2 packages
- **9 TSX + 3 TS** Preact PWA pages/components
- **18 TOML** configuration files
- **17 systemd units** (.service + .target)
- **4 conf** (Caddy, Unbound, Yggdrasil, nftables)
- **178 total files**, 812 KB source code

---

## 11. Architecture Decision Records

| Decision | Choice | Reason |
|----------|--------|--------|
| Base distro | Debian Trixie | Most packages, stable, Debos support |
| Mesh networking | Yggdrasil | Battle-tested, IPv6 mesh, pure userspace |
| Mesh services | Rust | Memory safety, async (tokio), small binaries |
| Orchestrator | Python/FastAPI | Rapid prototyping, rich async ecosystem |
| CDN agent | Go | Concurrency model, Sia/IPFS SDKs |
| Frontend | Preact | Small bundle, hooks, TypeScript |
| OOBE | GTK4/libadwaita | Native GNOME integration, system-level access |
| DHT | Kademlia | Standard, well-understood, proven in IPFS |
| Encryption | X25519 + AES-256-GCM | NaCl box pattern, modern, audited |
| DNS | Unbound with .sovrn forward | Existing resolver, programmatic .sovrn resolution |
| Firewall | nftables | Kernel-level, Debian default |
| Auth | JWT (HS256) | Simple, stateless, FastAPI integration |
| DB | SQLite (via rusqlite) | Embedded, no server dependency, compact |
| Packaging | .deb + Debos ISO | Native Debian, reproducible builds |

---

*End of build instructions. For design details, read docs 23–34 in the project root.*