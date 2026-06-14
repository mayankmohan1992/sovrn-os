# Sovrn OS — Session State

## Last Updated: 2026-06-14 15:30

## Current Phase: PHASE 7 ✅ — Comprehensive audit complete — 24 issues found

## Critical Note
All source files and build artifacts live inside `sovrn-os/`. Commands must run from `sovrn-os/` or use absolute paths.

---

## Build Environment

| Detail | Value |
|--------|-------|
| Machine | macOS (Apple Silicon M2/M3) |
| Builder | opencode AI agent |
| Build target | GitHub Actions CI (`ubuntu-latest`, no KVM) |
| Docker Desktop | Running (Virtualization Framework enabled) |
| Python venv | `~/.venvs/sovrn` (3.12.13) |

---

## Build Phase Tracker

| Phase | Status | Notes |
|-------|--------|-------|
| **Phase 1**: Rust workspace (Linux x86_64) | **DONE** | 6 ELF binaries in `sovrn-os/build/bin/`, cross-compiled via Docker. OOBE skipped (GTK4 can't cross-compile on macOS). |
| **Phase 2**: Go components | **DONE** | CDN agent in `sovrn-os/build/bin/`. Caddy from Debian apt (no xcaddy on CI). |
| **Phase 3**: Python services | **DONE** | 5 services (incl. sovrn-complete-setup) installed via `pip install -e .` |
| **Phase 4**: Preact PWA | **DONE** | Built with service worker, dist in `sovrn-os/build/share/pwa-dist/` |
| **Phase 5**: Config assembly | **DONE** | All configs, systemd units, overlays populated in `sovrn-os/build/overlays/` |
| **Phase 6**: ISO build | **IN PROGRESS** | 9 CI runs attempted, all failed. Currently fixing all issues for Run #10. |
| **Phase 7**: Verification | **NOT STARTED** | Awaiting first successful CI build. |

---

## Fixes Applied This Session (June 14)

### Previous fixes (local macOS build):
1. **Bin overlay path**: Moved `overlays/bin/*` → `overlays/bin/usr/bin/*`.
2. **Yggdrasil config path**: Fixed service to use `/etc/sovrn/yggdrasil.conf`.
3. **5 stub unit files**: Wrote full content for missing systemd units.
4. **Pre-flight validation**: Added `validate_overlays()` to `build-iso.sh`.
5. **CA bootstrap deferred**: Moved to first boot via systemd service.
6. **resolv.conf nspawn bind mount**: Fixed overwrite approach.
7. **Unbound missing parent dir**: Added `mkdir -p /etc/unbound`.
8. **Python install: pip → host-side overlay**: Switched to `install_python_pkg()`.
9. **Debos recipe simplified**: Removed `python-src` overlay.
10. **Caddyfile auth stripping**: Strips `sovrn_auth` block during overlay prep.
11. **Calamares branding placeholders**: Created placeholder files.
12. **macOS sed compatibility**: Changed to temp-file+mv pattern.
13. **build-iso-image.sh created**: 4 GB hybrid GPT disk image builder.
14. **Yggdrasil binary path**: Fixed `/usr/bin` → `/usr/sbin`.
15. **Background image**: Added `python3-pil` to apt.
16. **sovrnd.service Type**: Fixed `notify` → `simple`.
17. **GRUB loadfont syntax**: Fixed `loadfont=` → `loadfont `.

### CI Build Fixes (Runs #1-9, all failed):
18. **Caddy validation**: Removed caddy from overlay binary validation (now apt-installed).
19. **Debos command**: Switched from `sudo` → `--disable-fakemachine` to avoid OOM.
20. **Python package structure**: Fixed flat layout (no nested `src/`) so `pip install -e .` works.
21. **Microsoft repos**: Pre-delete stale MS repo files before apt-get update (403 fix).
22. **`linux-firmware` → `firmware-linux`**: Ubuntu → Debian package name.
23. **`policykit-1` → `polkitd`**: PolicyKit renamed upstream.
24. **`debos-sovrn-ci.yaml` created**: CI-optimized recipe replacing `debos-sovrn.yaml`.

### Comprehensive Audit Fixes (June 14):
25. **Package name fixes**: `libpipewire-0.3-0` → `libpipewire-0.3-0t64`, `libwireplumber-0.3-0` → `libwireplumber-0.5-0`, removed `pulseaudio` (pipewire-pulse replaces it).
26. **`linux-headers-amd64` removed**: Too large (~200 MB), not needed in minimal image.
27. **fakemachine block removed**: Not used with `--disable-fakemachine`.
28. **`sudo` + absolute path in debos command**: `build-iso.sh` line 319 fixed.
29. **sovrn-complete-setup.service → XDG autostart**: System service cannot run GTK4 (no display). Created `.desktop` file in `/etc/xdg/autostart/`.
30. **GRUB loadfont shell constructs removed**: `2>/dev/null || true` cleaned from GRUB config.
31. **`sovrn-complete-setup` added to `scripts/build.sh`**: Now built in `build_python()`.
32. **`build-iso.sh` validation**: Added `sovrn-complete-setup.service` to systemd unit check.
33. **`AGENTS.md` updated**: `debos-sovrn.yaml` → `debos-sovrn-ci.yaml`, added CI workflow info.

## Goal
- Get the GitHub Actions CI build (`feature/ci-github-actions` → `main` PR #1) to succeed and produce `sovrn-os-hybrid.img`.

## All state files and AGENTS.md updated with comprehensive audit findings
