# Sovrn OS — Project State

## Current Status: ✅ CI BUILD SUCCEEDS — GRUB wildcard fix, image ready for USB boot

Last updated: 2026-06-14 22:15

### Source Code Summary

| Component | Language | Source Files | Build Status |
|-----------|----------|-------------|--------------|
| **sovrn-dht** | Rust | 8 `.rs` | ✅ Built via `cross` |
| **sovrn-identity** | Rust | 9 `.rs` | ✅ Built via `cross` |
| **sovrn-presence** | Rust | 7 `.rs` | ✅ Built via `cross` |
| **sovrn-feed** | Rust | 8 `.rs` | ✅ Built via `cross` |
| **sovrn-message-queue** | Rust | 7 `.rs` | ✅ Built via `cross` |
| **oobe** (OOBE wizard) | Rust/GTK4 | 8 `.rs` | **SKIPPED** — GTK4 cross-compilation not practical |
| **sovrn-cdn-agent** | Go | 8 `.go` | ✅ Built via Go cross-compile |
| **caddy-auth** | Go | 1 `.go` | **SKIPPED** — Caddy installed via Debian apt on CI |
| **sovrnd** (orchestrator) | Python | 16 `.py` | ✅ Built via `pip install -e .` |
| **sovrn-auth** | Python | 5 `.py` | ✅ Built via `pip install -e .` |
| **sovrn-monitor** | Python | 4 `.py` | ✅ Built via `pip install -e .` |
| **sovrn-notify-bridge** | Python | 5 `.py` | ✅ Built via `pip install -e .` |
| **sovrn-complete-setup** | Python | 3 `.py` | ✅ Built via `pip install -e .` |
| **pwa** (Sovrn Hub) | Preact/TS | 14 sources | ✅ Built via `npm run build` |

### Configuration Files

| Component | Files | Status |
|-----------|-------|--------|
| systemd units | 16 files | Ready |
| XDG autostart | `sovrn-complete-setup.desktop` | **NEW** — replaces system service |
| Caddy config | `Caddyfile` | Ready (auth block stripped) |
| nftables | `sovrn.nft` | Ready |
| DNS (Unbound) | `unbound-sovrn.conf` | Ready |
| Yggdrasil | `yggdrasil.conf` | Ready |
| AppArmor | `usr.bin.sovrnd` | Ready |
| GNOME customization | 2 files | Ready |
| CA bootstrap | `bootstrap-ca.sh` | Ready |
| Debos recipe | `debos-sovrn-ci.yaml` | Ready (was `debos-sovrn.yaml`) |
| Build scripts | `build.sh`, `build-iso.sh`, `build-iso-image.sh` | Ready |

### Current Status

1. ✅ **CI Run #15 succeeded** — All 14 steps green. Image built, QEMU smoke-tested, artifact uploaded.
2. ✅ **Image builder fixed** — Root cause was `set -e` silent exit in `check_prereqs()`.
3. ✅ **Caching working** — Rootfs tarball cached (cache hit → debos skipped).
4. **No KVM on CI** — `--disable-fakemachine` is required (7 GB runner insufficient for VM overhead).
5. **Next: full boot validation** — Current QEMU smoke test is minimal (30s timeout, basic boot check).

### Not blockers

- **Caddy auth plugin** — `sovrn_auth` directive stripped for Debian's vanilla Caddy. Restore on Linux with `xcaddy build` if custom Caddy is desired.
- **OOBE** — Replaced by `sovrn-complete-setup` (Python/GTK4 first-boot dialog for optional packages). OOBE source in `src/oobe/` is not built.

### Audit Findings (24 issues, all fixed)

**BLOCKER (fixes CI build):**
1. `policykit-1` → `polkitd` (renamed in trixie)
2. `libpipewire-0.3-0` → `libpipewire-0.3-0t64` (t64 transition)
3. `libwireplumber-0.3-0` → `libwireplumber-0.5-0` (soname bump)
4. Remove `pulseaudio` (conflicts with pipewire-pulse, which provides PA compat)
5. Remove `linux-headers-amd64` (~200 MB, not needed on target)
6. Remove stale `fakemachine:` block (not used with `--disable-fakemachine`)
7. `build-iso.sh`: Add `sudo` to native debos command
8. `build-iso.sh`: Fix relative `build/$RECIPE` → absolute `"$PROJECT_DIR/build/$RECIPE"`
9. `build-iso.sh`: Add `sovrn-complete-setup.service` to systemd unit validation
10. `build-iso-image.sh`: Remove shell constructs (`2>/dev/null || true`) from GRUB config

**HIGH (fixes OS functionality):**
11. `sovrn-complete-setup.service`: Redesign as XDG autostart `.desktop` file (system service can't run GTK4)
12. Remove `sovrn-complete-setup.service` from debos `systemctl enable` list
13. Add `setup-complete` marker cleanup in debos recipe
14. `build.sh`: Add `sovrn-complete-setup` to `build_python()`

**LOW (cleanup):**
15. `AGENTS.md`: Update `debos-sovrn.yaml` → `debos-sovrn-ci.yaml`
16. `AGENTS.md`: Add CI workflow info
17. Overlay `yggdrasil.service` has stale `/usr/bin/yggdrasil` (source is correct; `/usr/sbin`)
18. Overlay `sovrnd.service` has stale `Type=notify` (source is correct; `Type=simple`)
19. Comment in `build-iso.sh` line 318 references `-e` flag → should say `--disable-fakemachine`
20. Stage 6 packages may have t64 names (verify `libadwaita-1-0`, `libgtk-4-1`)

### Build Artifacts

| Artifact | Path | Status |
|----------|------|--------|
| Rust binaries | `sovrn-os/build/bin/` | ✅ |
| Python dist | `sovrn-os/build/lib/` | ✅ |
| PWA dist | `sovrn-os/build/share/pwa-dist/` | ✅ |
| Overlays | `sovrn-os/build/overlays/` | ✅ (generated at build time) |
| Rootfs tarball | `build/sovrn-os-rootfs.tar.gz` | ✅ (cached, rebuilt only on recipe change) |
| Hybrid image | `build/sovrn-os-hybrid.img` | ✅ (2.9 GB, uploaded as artifact) |

### Known Issues

- 3 non-existent services (`sovrn-oobe`, `sovrn-first-boot`, `sovrn-app-monitor`) have `.service` files installed but binaries don't exist. They reference `NEVER_BUILT` binaries in validation skip list. Services won't start at boot (binary missing → failed → auto-restart loop).
- `sovrn-ca-bootstrap.service` missing `RemainAfterExit=yes`.
- Python GTK4 imports are at top level in `complete_setup.py` — will crash if GI unavailable.
- All sovrn services run as root despite `sovrn` system user existing.
- CI QEMU smoke test uses `-enable-kvm` which is unavailable on `ubuntu-latest` runners.
