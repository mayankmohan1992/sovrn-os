# Sovrn OS — Session State

## Last Updated: 2026-06-14 10:15

## Current Phase: PHASE 7 ✅ — Pre-handover audit complete — 4 bugs fixed

## Critical Note
All source files and build artifacts live inside `sovrn-os/`. Commands must run from `sovrn-os/` or use absolute paths.

---

## Build Environment

| Detail | Value |
|--------|-------|
| Machine | macOS (Apple Silicon M2/M3) |
| Builder | opencode AI agent |
| Docker Desktop | Running (Virtualization Framework enabled) |
| Python venv | `~/.venvs/sovrn` (3.12.13) |

---

## Build Phase Tracker

| Phase | Status | Notes |
|-------|--------|-------|
| **Phase 1**: Rust workspace (Linux x86_64) | **DONE** | 5 ELF binaries in `sovrn-os/build/bin/`, cross-compiled via Docker. OOBE skipped (GTK4 can't cross-compile on macOS). |
| **Phase 2**: Go components | **DONE** | CDN agent + Caddy auth plugin in `sovrn-os/build/bin/`. Caddy built via xcaddy with auth plugin. |
| **Phase 3**: Python services | **DONE** | 4 services installed in `~/.venvs/sovrn` via `pip install -e .` |
| **Phase 4**: Preact PWA | **DONE** | Built with service worker, dist in `sovrn-os/build/share/pwa-dist/` |
| **Phase 5**: Config assembly | **DONE** | All configs, systemd units, overlays populated in `sovrn-os/build/overlays/` |
| **Phase 6**: ISO build | **DONE** | Rootfs tarball rebuilt (v2) with Python services overlay. All 4 Python console scripts present in tarball. |
| **Phase 7**: Verification | **DONE** | Tarball verified: Python binaries at `/usr/bin/sovrnd`, `sovrn-auth`, `sovrn-monitor`, `sovrn-notify-bridge`. Caddyfile clean (no `sovrn_auth` block). ISO image builder ready. |

---

## Fixes Applied This Session (June 14)

1. **Bin overlay path**: Moved `overlays/bin/*` → `overlays/bin/usr/bin/*` so `destination: /` maps to `/usr/bin/`. Fixed `build-iso.sh` to use correct path.
2. **Yggdrasil config path**: Changed `yggdrasil.service` ExecStart from `/etc/yggdrasil/yggdrasil.conf` → `/etc/sovrn/yggdrasil.conf` (matches where overlay places it).
3. **5 stub unit files**: Wrote full content for `caddy.service`, `sovrn-ca-bootstrap.service`, `sovrn-first-boot.service`, `sovrn-app-monitor.service`, `zram-setup.service` (were just `# see 31-SYSTEMD-UNITS.md`).
4. **Pre-flight validation**: Added `validate_overlays()` step to `build-iso.sh` that checks all files exist and cross-references ExecStart paths.
5. **CA bootstrap inlined (then removed)**: Failed because overlay `/etc/sovrn/scripts/bootstrap-ca.sh` was invisible inside nspawn. Inlined script, then removed — CA deferred to first boot for per-node unique keys.
6. **resolv.conf nspawn bind mount**: `rm -f /etc/resolv.conf` → "Device or resource busy" (nspawn mounts it). Changed to `printf 'nameserver 1.1.1.1\n' > /etc/resolv.conf`.
7. **Unbound missing parent dir**: `/etc/unbound/` dir didn't exist before appending config. Added `mkdir -p /etc/unbound`.
8. **Python install: pip → host-side overlay**: `pip3` not found inside nspawn after overlays. Replaced with `install_python_pkg()` in `build-iso.sh`.
9. **Debos recipe simplified**: Removed `python-src` overlay + pip install `run` action; replaced with `overlays/python-dist → /` overlay action.
10. **Caddyfile auth stripping**: `build-iso.sh` strips `sovrn_auth { ... }` block from Caddyfile during overlay prep.
11. **Calamares branding placeholders**: Created `logo.png`, `welcome.png`, `show.qml` in `build/calamares/branding/sovrn/`.
12. **macOS sed compatibility**: Changed `sed -i` to redirect-to-temp+`mv` pattern.
13. **build-iso-image.sh created**: 4 GB hybrid GPT disk image builder.
14. **Yggdrasil binary path**: Changed `ExecStart` from `/usr/bin/yggdrasil` → `/usr/sbin/yggdrasil` (Debian package install location).
15. **Background image**: Added `python3-pil` to debos apt list so PIL fallback works for background generation.
16. **sovrnd.service Type**: Changed `Type=notify` → `Type=simple` (uvicorn doesn't support sd_notify). Removed `TimeoutStartSec=30`.
17. **GRUB loadfont syntax**: Fixed `loadfont=` → `loadfont ` (space, not `=`) in `build-iso-image.sh`.

## Rootfs Tarball (v2 — rebuilt with Python overlay)

- **Path**: `build/sovrn-os-rootfs.tar.gz`
- **Size**: 1.2 GB
- **Contents**: 6 Rust ELF binaries, 4 Python console scripts, 8 config TOML files, 15 systemd units, 14 PWA dist files, CA bootstrap script, nftables rules, unbound config, GNOME dconf
- **Caddyfile**: Clean (no `sovrn_auth` block) — works with vanilla Debian Caddy
