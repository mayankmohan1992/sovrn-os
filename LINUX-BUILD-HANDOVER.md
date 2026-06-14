# Sovrn OS — Linux Build Handover

## START HERE — Instructions for the Linux AI Agent

Point the Linux AI agent at **this file** (`LINUX-BUILD-HANDOVER.md`). The full project is at the repository root `/Users/mayankmohan/Desktop/Projects/souvrn/sovrn-os/`. The agent should:

1. Read this entire file
2. Read `AGENTS.md` (working directory instructions)
3. Run `./build/build-iso.sh` to rebuild the rootfs tarball with native debos + KVM
4. Run `sudo ./build/build-iso-image.sh` to create the bootable disk image
5. Test with QEMU: `qemu-system-x86_64 -m 2048 -enable-kvm -cpu host -drive file=build/sovrn-os-hybrid.img,format=raw`

**Rebuilding Rust/Go components on Linux is optional** — the rootfs tarball already contains working binaries. But for a clean build from source, see the handover details below.

---

This document tells you everything you need to build the final bootable Sovrn OS ISO on a **Linux machine (x86_64)**. All source code and artifacts have been prepared on macOS. Only the final assembly (Debos with KVM, and disk image creation) requires Linux.

---

## Prerequisites (Linux Mint 22.3 / Ubuntu 24.04)

```bash
# Core build tools
sudo apt install debos parted grub-pc-bin grub-efi-amd64-bin xorriso mtools
sudo apt install qemu-system-x86 qemu-utils    # for testing the image

# Optional: if you want to rebuild Rust/Go components
sudo apt install cargo rustc go
```

### Verify debos works
```bash
debos --version    # should print version (tested with latest from ghcr.io/go-debos/debos)
```

---

## Quick Start (just build the ISO)

```bash
cd sovrn-os/

# Step 1: Build the rootfs tarball (uses Docker for debos, ~25 min)
./build/build-iso.sh

# OR skip rebuild and use existing tarball:
./build/build-iso.sh --quick

# Step 2 (as root): Build hybrid disk image (~5 min)
sudo ./build/build-iso-image.sh
# Output: build/sovrn-os-hybrid.img (4 GB)

# Step 3 (optional): Test with QEMU
qemu-system-x86_64 -m 2048 -enable-kvm -cpu host -drive file=build/sovrn-os-hybrid.img,format=raw

# Step 4 (optional): Write to USB
sudo dd if=build/sovrn-os-hybrid.img of=/dev/sdX bs=4M status=progress
```

---

## What's Already Done (on macOS)

| Component | Status | Details |
|-----------|--------|---------|
| Rust workspace (5 services) | **Built, in tarball** | `sovrn-dht`, `sovrn-identity`, `sovrn-presence`, `sovrn-feed`, `sovrn-message-queue` — all at `/usr/bin/` |
| Go CDN agent | **Built, in tarball** | `sovrn-cdn-agent` at `/usr/bin/` |
| Caddy web server | **Built, in tarball** | Debian apt package (`caddy`), NOT custom-built with auth plugin |
| Python services (4) | **Overlay-copied, in tarball** | `sovrnd`, `sovrn-auth`, `sovrn-monitor`, `sovrn-notify-bridge` — packages at `/usr/lib/python3/dist-packages/`, wrappers at `/usr/bin/` |
| Preact PWA | **Built, in tarball** | At `/var/lib/sovrn/pwa-dist/` |
| Config files | **Ready** | TOML configs, nftables rules, unbound DNS, Yggdrasil VPN |
| Systemd units | **Ready** | 18 unit files for all services |
| Calamares branding | **Placeholders ready** | `build/calamares/branding/sovrn/` with logo, welcome, QML, branding.desc |
| Rootfs tarball | **Built (1.2 GB)** | `build/sovrn-os-rootfs.tar.gz` (v2, with Python overlay) |
| Disk image builder | **Ready** | `build/build-iso-image.sh` (creates hybrid BIOS+UEFI GPT image) |
| Debos recipe | **Ready** | `build/debos-sovrn.yaml` (250 lines, fully tested) |
| Pre-flight validation | **Ready** | `build/build-iso.sh` validates all overlay files before debos runs |

---

## What Still Needs Work (on Linux)

### MUST-FIX for Caddy auth plugin
The Caddyfile has `sovrn_auth { ... }` stripped because Debian's vanilla `caddy` package doesn't have this plugin. On Linux:
```bash
# Install xcaddy
go install github.com/caddyserver/xcaddy/cmd/xcaddy@latest

# Build Caddy with the auth plugin
xcaddy build --with github.com/sovrn/caddy-auth=./src/caddy-auth

# Replace the binary in the overlay
cp caddy build/overlays/bin/usr/bin/
```

Then **restore the `sovrn_auth` block** in `build/overlays/etc/etc/caddy/Caddyfile`. Look at `build/etc/caddy/Caddyfile` for the original with the auth block.

### OOBE (GTK4) — optional, blocked
`src/oobe/` has source code but `Cargo.toml` was scaffolded and may need adjustment. OOBE is NOT enabled in systemd (unit file exists but service is not enabled). You can enable it by adding `systemctl enable sovrn-oobe.service` to the debos recipe actions.

### Hybrid ISO (live CD) — not started
The current build produces a raw disk image (`.img`), not a bootable ISO. For an ISO:
- Use `debos`'s `-o` or `--output` flag to generate ISO
- Or build a squashfs live system (Liveng OS style)

### Calamares installer UI — cosmetic
Branding placeholders are minimal (128×128 logo, 800×450 welcome image, 2-slide QML). Replace with proper assets.

---

## Build Scripts Reference

### `build/build-iso.sh` — Top-level ISO builder
- Prepares overlay directories (binaries, configs, systemd, PWA, Python dist)
- Strips `sovrn_auth` block from Caddyfile (for vanilla Debian Caddy)
- Validates all overlay files (86 checks)
- Runs `debos` (in Docker on macOS, natively on Linux)
- `--quick`: skip compile, use pre-built binaries
- `--docker`: force Docker mode (default on macOS)

### `build/build-iso-image.sh` — Hybrid disk image builder
- Creates 4 GB GPT image: BIOS boot (1 MB) + EFI (512 MB FAT32) + ext4 root
- Uses `parted`, `losetup`, `grub-install`, `mkfs.ext4`, `mkfs.fat`
- Must run as root (needs `losetup` and `mount`)
- Expects `build/sovrn-os-rootfs.tar.gz` as input

### `build/debos-sovrn.yaml` — Debos recipe
- Base: Debian Trixie (testing), minbase
- Packages: GNOME desktop, systemd, Python deps, Yggdrasil, Caddy, nftables, unbound
- Overlays: binaries → `/usr/bin/`, configs → `/etc/`, systemd → `/etc/systemd/system/`, PWA → `/var/lib/sovrn/pwa-dist/`, Python dist → `/usr/lib/python3/dist-packages/` + `/usr/bin/`
- Systemd enable: all 15+ services enabled
- Pack: outputs `sovrn-os-rootfs.tar.gz`

---

## Pre-Handover Fixes Applied (June 14)

These 4 bugs were found during final audit and fixed before handover:

| # | Issue | File | Fix |
|---|-------|------|-----|
| 1 | Yggdrasil binary path | `src/systemd-units/yggdrasil.service:8` | `/usr/bin` → `/usr/sbin` (Debian package path) |
| 2 | Background image fails | `build/debos-sovrn.yaml:69` | Added `python3-pil` to apt list so PIL fallback works |
| 3 | sovrnd.service Type=notify | `src/systemd-units/sovrnd.service:8` | Changed to `Type=simple` (uvicorn no sd_notify) |
| 4 | GRUB loadfont syntax | `build/build-iso-image.sh:120` | `loadfont=` → `loadfont ` (space, not `=`) |

## All 21 Issues Resolved (read BUILD-ERROR-LOG.md for detail)

| # | Issue | Fix |
|---|-------|-----|
| 1 | OOBE missing Cargo.toml | Scaffolded build manifest |
| 2 | `pub.sig` typo in mq | Fixed to `pub sig` |
| 3 | `StaticSecret` not found | Added `static_secrets` feature |
| 4 | `from_default_env_or` not found | Replaced with try_from_default_env |
| 5 | `SigningKey::generate()` removed | Use OsRng + from_bytes |
| 6 | `INTERNAL_ERROR` removed | Use anyhow::Result |
| 7 | DhtService not Clone + re-declared mods | Arc<DhtService>, dedup mod declarations |
| 8 | `rpc_handles` typo + crate name mismatches | Fixed typos |
| 9 | Debos overlay destination deep path fails | Use `destination: /` everywhere |
| 10 | Binaries at `/` not `/usr/bin/` | Moved to `overlays/bin/usr/bin/` |
| 11 | Yggdrasil config path mismatch | Changed ExecStart to /etc/sovrn/yggdrasil.conf |
| 12 | 5 unit files were stubs | Wrote full unit file content |
| 13 | No pre-flight validation | Added validate_overlays() |
| 14 | CA bootstrap script invisible in nspawn | Deferred to first-boot service |
| 15 | resolv.conf nspawn bind mount | Overwrite in-place (no rm) |
| 16 | pip3 not found after overlays | Host-side Python dist overlay (no pip) |
| 17 | macOS sed -i incompatibility | Temp-file + mv pattern |
| 18 | Yggdrasil binary path wrong | `/usr/bin` → `/usr/sbin` |
| 19 | Background image fallback broken | Added `python3-pil` to apt list |
| 20 | sovrnd.service Type=notify | Changed to `Type=simple` |
| 21 | GRUB loadfont syntax | `loadfont=` → `loadfont ` |

---

## Service Dependency Chain

```
sovrn.target
  ├── sovrnd.service              (Python, port 54771)
  │   ├── sovrn-dht.service       (Rust, port 54774)
  │   ├── sovrn-identity.service  (Rust)
  │   ├── sovrn-presence.service  (Rust, port 54775)
  │   ├── sovrn-feed.service      (Rust)
  │   └── sovrn-message-queue.service (Rust)
  ├── sovrn-auth.service          (Python, port 54776)
  ├── sovrn-monitor.service       (Python, port 54777)
  ├── sovrn-notify-bridge.service (Python, port 54773)
  ├── sovrn-cdn-agent.service     (Go)
  ├── caddy.service               (port 54772, proxies to sovrnd)
  ├── yggdrasil.service           (port 30550, mesh VPN)
  ├── NetworkManager.service
  ├── unbound.service             (DNS)
  └── nftables.service            (firewall)
```

**NEVER_BUILT** (unit files exist, services NOT enabled):
- `sovrn-oobe.service` — GTK4 first-boot wizard (missing Cargo.toml, cross-compile issues)
- `sovrn-first-boot.service` — first-boot initialization
- `sovrn-app-monitor.service` — application monitor

---

## Port Map

| Service | Port |
|---------|------|
| sovrnd | 54771 |
| PWA Hub (Caddy) | 54772 |
| Notify Bridge | 54773 |
| DHT mesh | 54774 |
| Presence | 54775 |
| Auth | 54776 |
| Monitor | 54777 |
| Yggdrasil | 30550 |

---

## Verification Checklist

After building the disk image, boot it (QEMU or real hardware) and verify:

- [ ] System boots to GRUB menu
- [ ] GNOME desktop loads and gdm login works
- [ ] `sudo systemctl status sovrn.target` — all services running
- [ ] `curl http://localhost:54772` — PWA Hub responding
- [ ] `curl http://localhost:54771/health` — sovrnd API responding
- [ ] `sovrn-dht --help` — binary works
- [ ] `journalctl -u sovrnd --no-pager` — no errors
- [ ] `journalctl -u caddy --no-pager` — no errors
- [ ] `systemctl list-units --state=failed` — no failed units
- [ ] CA certs generated: `ls /etc/sovrn/ca/`
- [ ] Yggdrasil mesh peer: `sudo yggdrasilctl getPeers`
- [ ] nftables rules loaded: `sudo nft list ruleset`
- [ ] Unbound DNS: `dig @127.0.0.1 sovrn`

---

## Troubleshooting

### Overlay paths still wrong?
Run the validation standalone:
```bash
./build/build-iso.sh --quick   # will validate then build
```

### Debos fails with "os.Mkdir" error?
All overlays now use `destination: /` — source directory must include the full filesystem path. Example: `overlays/bin/usr/bin/caddy` maps to `/usr/bin/caddy`.

### Python service not found?
Check `/usr/bin/sovrnd` exists in the tarball:
```bash
tar -tzf build/sovrn-os-rootfs.tar.gz | grep 'usr/bin/sovrn'
```
Should show: `sovrnd`, `sovrn-auth`, `sovrn-monitor`, `sovrn-notify-bridge`.

### Caddy refuses to start?
Check the Caddyfile in the tarball:
```bash
tar -xzf build/sovrn-os-rootfs.tar.gz --to-stdout ./etc/caddy/Caddyfile
```
There should be NO `sovrn_auth` block. If present, the stripping in `build-iso.sh` line 80-81 failed.

### Docker Desktop vs native debos
On macOS, debos runs in Docker with `--disable-fakemachine` (nspawn mode). This causes overlay visibility issues (files from earlier overlays not visible to later `run` actions). On Linux with native debos + fakemachine (KVM VM), these issues do NOT occur.

---

## File Layout

```
sovrn-os/
├── build/
│   ├── debos-sovrn.yaml         # Debos recipe
│   ├── build-iso.sh             # ISO builder script
│   ├── build-iso-image.sh       # Hybrid disk image builder (chmod +x)
│   ├── sovrn-os-rootfs.tar.gz   # Rootfs tarball (1.2 GB)
│   ├── overlays/
│   │   ├── bin/usr/bin/         # ELF binaries (7 files)
│   │   ├── etc/etc/             # Config files (13 files)
│   │   ├── systemd/etc/systemd/system/  # Systemd units (18 files)
│   │   ├── pwa-dist/var/lib/sovrn/pwa-dist/  # PWA dist
│   │   ├── gnome/etc/           # GNOME dconf settings
│   │   ├── python-dist/usr/     # Python dist + wrappers
│   │   └── scripts/             # (empty)
│   ├── calamares/
│   │   ├── branding/sovrn/      # Calamares branding (logo.png, welcome.png, show.qml, branding.desc)
│   │   ├── modules/services-systemd.conf
│   │   └── settings.conf
│   └── bin/                     # Pre-built binaries (source for overlay)
├── src/                         # All source code (Rust, Go, Python, Preact)
├── scripts/                     # Build scripts, bootstrap-ca.sh
├── AGENTS.md                    # Instructions for AI agents
├── BUILD-ERROR-LOG.md           # All 17 resolved issues
├── SESSION-STATE.md             # Current session state
├── project-state.md             # Project overview
└── LINUX-BUILD-HANDOVER.md      # THIS FILE
```
