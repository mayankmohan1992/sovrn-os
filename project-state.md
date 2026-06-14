# Sovrn OS — Project State

## Current Status: ROOTFS TARBALL READY — Pre-handover audit complete

Last updated: 2026-06-14 10:15

### Source Code Summary

| Component | Language | Source Files | Build Status |
|-----------|----------|-------------|--------------|
| **sovrn-dht** | Rust | 8 `.rs` | ✅ Built, in rootfs |
| **sovrn-identity** | Rust | 9 `.rs` | ✅ Built, in rootfs |
| **sovrn-presence** | Rust | 7 `.rs` | ✅ Built, in rootfs |
| **sovrn-feed** | Rust | 8 `.rs` | ✅ Built, in rootfs |
| **sovrn-message-queue** | Rust | 7 `.rs` | ✅ Built, in rootfs |
| **oobe** (OOBE wizard) | Rust/GTK4 | 8 `.rs` | **BLOCKED** — missing Cargo.toml |
| **sovrn-cdn-agent** | Go | 8 `.go` | ✅ Built, in rootfs |
| **caddy-auth** | Go | 1 `.go` | **SKIPPED** — xcaddy not available on macOS |
| **sovrnd** (orchestrator) | Python | 16 `.py` | ✅ Overlay-copied to /usr/lib/python3/dist-packages/ |
| **sovrn-auth** | Python | 5 `.py` | ✅ Overlay-copied to /usr/lib/python3/dist-packages/ |
| **sovrn-monitor** | Python | 4 `.py` | ✅ Overlay-copied to /usr/lib/python3/dist-packages/ |
| **sovrn-notify-bridge** | Python | 5 `.py` | ✅ Overlay-copied to /usr/lib/python3/dist-packages/ |
| **pwa** (Sovrn Hub) | Preact/TS | 14 sources | ✅ Built, in rootfs (/var/lib/sovrn/pwa-dist/) |

### Configuration Files

| Component | Files | Status |
|-----------|-------|--------|
| systemd units | 18 files | Ready |
| Caddy config | `Caddyfile` | Ready |
| nftables | `sovrn.nft` | Ready |
| DNS (Unbound) | `unbound-sovrn.conf` | Ready |
| Yggdrasil | `yggdrasil.conf` | Ready |
| AppArmor | `usr.bin.sovrnd` | Ready |
| GNOME customization | 2 files + 1 empty dir | Partial (extension/ empty) |
| CA bootstrap | `bootstrap-ca.sh` | Ready |
| Debos recipe | `debos-sovrn.yaml` | Ready |
| Build script | `build.sh`, `build-iso.sh` | Ready |

### Current Blocker

1. **Bootable ISO requires Linux host** — Debos ISO builder needs KVM; macOS Docker doesn't support it. Handoff to Linux agent via `LINUX-BUILD-HANDOVER.md`.

### Not blockers (noted for Linux agent)

- **Caddy auth plugin** — `sovrn_auth` directive stripped for Debian's vanilla Caddy. Restore on Linux with `xcaddy build`.
- **OOBE** — `src/oobe/` has source code but cross-compiling GTK4 on macOS is impractical. Build natively on Linux.

### Build Artifacts Produced

| Artifact | Path | Size | Status |
|----------|------|------|--------|
| Rootfs tarball | `build/sovrn-os-rootfs.tar.gz` | 1.2 GB | ✅ (v2 with Python overlay) |

### Known Issues

- Cross-compiling OOBE (GTK4/libadwaita) on macOS requires Docker-based `cross`
- CA certs not pre-generated in rootfs (intentional — generated per-node at first boot via `sovrn-ca-bootstrap.service`)
- Python services prepared via host-side overlay (no pip inside container) — means package metadata (egg-info) is preserved but editable install (`pip install -e`) not replicated
- Caddyfile `sovrn_auth` block stripped — must be restored when custom Caddy is built on Linux
