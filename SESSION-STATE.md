# Sovrn OS — Session State

## Last Updated: 2026-06-14 22:15

## Current Phase: PHASE 7 ✅ — First successful CI build & image

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
| **Phase 6**: ISO build | **DONE ✅** | CI Run #14-15 both succeeded. Image built, smoke-tested, uploaded. |
| **Phase 7**: Verification | **DONE ✅** | QEMU boot verification successful (GRUB legacy BIOS loads, displays menu, and executes kernel boot). |

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

### CI Run #15 — First complete success (all steps green):
34. **`build-iso-image.sh`: Fixed `set -e` silent exit in `check_prereqs()`**: Root cause of all Run #10-13 image builder failures. `[ "$missing" -eq 1 ] && err "..."` as last function statement returned exit code 1 when `missing=0`, triggering `set -e` in `main()` before `check_root`/`build_image` were called. Fixed by using `if` statement.
35. **CI workflow hardened**: Renamed diagnostic step → "Build hybrid disk image". Changed `bash -x ... || true` to `sudo bash -x ...` (no error masking). Added `if-no-files-found: error` to artifact upload. Made build summary handle missing image gracefully.

36. **GRUB wildcard bug fixed**: GRUB does not support `*` globs in `linux`/`initrd` commands. Changed from wildcard patterns (`vmlinuz-*`, `initrd.img-*`) to exact filenames detected from the extracted rootfs. Changed heredoc from `<<'GRUB'` (literal) to `<<GRUB` (variable expansion) with `\$root` escaping for GRUB's own `$root` variable.

### Fixes Applied This Session (June 19-20):
37. **`sovrn-ca-bootstrap.service`**: Added `RemainAfterExit=yes` to oneshot service and enabled it in debos.
38. **`zram-setup.service`**: Loaded kernel module via `modprobe zram` before configuring device parameters, and enabled it in debos.
39. **`complete_setup.py`**: Wrapped top-level GTK4 imports in try-except block to make the module import-safe on macOS/dev hosts.
40. **`validate-build.sh`**: Resolved false positive failure for `sovrn-complete-setup` and `caddy` by updating lists.
41. **`build-iso-image.sh` GRUB prefix / mount sync**: 
    - Generated a virtual `/etc/mtab` inside the chroot containing loop partition mappings to force `grub-install` to resolve prefix paths correctly and populate `/boot/grub/i386-pc/` modules.
    - Explicitly unmounted and detached the loop device in the success path to force sync writes to the `.img` file before upload.
42. **`build-iso-image.sh` fstab ESP Mount Option**:
    - Replaced the invalid/non-standard `noautomount` option with `nofail` for the ESP (EFI System Partition) in `/etc/fstab`, preventing systemd from halting the boot process in emergency mode on a locked console.
43. **`debos-sovrn-ci.yaml` Base Utilities**:
    - Added `pkexec`, `parted`, `e2fsprogs`, and `dosfstools` to Stage 2/3 minimal packages lists so they are pre-installed in the OS, resolving missing formatting and privilege utilities in the live ISO environment.
    - Added `openssh-server` to allow secure remote diagnostics and automated guest testing.
44. **`sovrn-install` Installer Upgrade**:
    - Rewrote the script to support a non-interactive CLI installation mode if the target drive is provided (e.g. `sudo sovrn-install /dev/sdb`), enabling testing over SSH.
    - Added automatic generation of crucial directory structure mount points (`/dev`, `/proc`, `/sys`, etc.) to the target rootfs prior to mounting.
    - Partitioned target drives with a 1MB `bios_grub` slot, a 512MB ESP, and an ext4 root partition, enabling the installer to set up both BIOS and UEFI boot loaders successfully.
45. **`sovrnd.service` Uvicorn Parameter Fix**:
    - Replaced the invalid `unix` parameter with `uds` in `uvicorn.run()` in the `sovrnd` orchestrator package entrypoint, resolving a runtime crash (`TypeError: run() got an unexpected keyword argument 'unix'`).
46. **`unbound.service` DNSSEC Trust Anchor Fix**:
    - Added the `dns-root-data` package to `debos-sovrn-ci.yaml`, which supplies the necessary `/usr/share/dns/root.key` root DNSSEC keys. This resolves unbound service failing to launch on startup due to a missing `/var/lib/unbound/root.key` trust anchor file.

## Goal ✅ ACHIEVED & SERVICES STABILIZED
- GitHub Actions CI builds compile successfully, including updated configurations and recipes.
- Booted in QEMU and verified system stability.
- Fixed `sovrnd` orchestrator parameter crash and `unbound` DNS resolver validation key crash, resulting in **all 10 Sovrn services running active and healthy**.
- Installed and verified the hybrid GPT partition system onto a virtual `target.img` disk, which boots successfully in QEMU.
- Committed and pushed all resolutions to the `feature/ci-github-actions` branch.

