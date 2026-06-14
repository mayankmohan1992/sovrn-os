# Sovrn OS — Build Error Log

Chronological record of every error and fix.

---

## Error Log

### Error #1: OOBE missing Cargo.toml
- **Phase:** Phase 0 — Pre-build
- **Error:** `cargo build` impossible — no Cargo.toml in `src/oobe/`
- **Fix:** Scaffolded `Cargo.toml` with gtk4/libadwaita/reqwest deps. Also created `src/main.rs` (missing entry point).
- **Status:** Resolved

### Error #2: `pub.sig` typo in sovrn-message-queue
- **Phase:** Phase 0b — Fix compilation errors
- **Error:** `expected identifier, found '.'` at `src/sovrn-message-queue/src/queue.rs:15`
- **Fix:** Changed `pub.sig` to `pub sig`
- **Status:** Resolved

### Error #3: `StaticSecret` not found in x25519-dalek
- **Phase:** Phase 0b
- **Error:** `StaticSecret` was removed/unavailable in x25519-dalek v2.0.1
- **Fix:** Added `static_secrets` feature to workspace Cargo.toml. Changed encryption.rs to use `EphemeralSecret` for ephemeral keys.
- **Status:** Resolved

### Error #4: `from_default_env_or` not found on EnvFilter
- **Phase:** Phase 0b
- **Error:** 4 main.rs files used `EnvFilter::from_default_env_or("info")` which doesn't exist in tracing-subscriber 0.3
- **Fix:** Replaced with `EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))`
- **Status:** Resolved

### Error #5: `SigningKey::generate()` removed in ed25519-dalek v2
- **Phase:** Phase 0b
- **Error:** `generate(&mut csprng)` method does not exist
- **Fix:** Use `OsRng.fill_bytes()` + `SigningKey::from_bytes()` pattern
- **Status:** Resolved

### Error #6: `INTERNAL_ERROR` removed from ed25519-dalek
- **Phase:** Phase 0b
- **Error:** `ed25519_dalek::INTERNAL_ERROR` no longer exists
- **Fix:** Changed error return to use `anyhow::Result` instead of `SignatureError`
- **Status:** Resolved

### Error #7: `DhtService` not Clone + module re-declaration in binary
- **Phase:** Phase 0b
- **Error:** `ygg.rs` tried to call `self.clone()` on non-Clone DhtService. main.rs re-declared modules that lib.rs already declared.
- **Fix:** Changed `listen_yggdrasil` from method to standalone function taking `Arc<DhtService>`. Removed duplicate `mod` declarations from main.rs.
- **Status:** Resolved

### Error #8: `rpc_handles` typo + crate name mismatches
- **Phase:** Phase 0b
- **Error:** `rpc_handles` instead of `rpc_handle` in identity main.rs. `sovrn_mq` instead of `sovrn_message_queue` in mq main.rs.
- **Fix:** Fixed typos and crate name references.
- **Status:** Resolved

### Error #9: Debos overlay destination path — parent dir not created
- **Phase:** Phase 6 — ISO build
- **Error:** Debos `overlay` action with `destination: /var/lib/sovrn/pwa-dist` fails because `os.Mkdir` (not `os.MkdirAll`) cannot create intermediate directories.
- **Fix:** Changed all overlays to use `destination: /` with full filesystem path in source directory (e.g., `overlays/pwa-dist/var/lib/sovrn/pwa-dist/`). Abandoned previous approach of `mkdir -p` run actions before overlays (layering mismatch).
- **Status:** Resolved

### Error #10: Binaries land at `/` not `/usr/bin/`
- **Phase:** Phase 6 — ISO build
- **Error:** `build-iso.sh` copied binaries to `overlays/bin/` (no path prefix), so `destination: /` maps them to `/sovrn-dht` instead of `/usr/bin/sovrn-dht`. All systemd units reference `/usr/bin/sovrn-*`.
- **Fix:** Moved overlay binaries to `overlays/bin/usr/bin/`. Updated `build-iso.sh` copy target to `$OVERLAY_DIR/bin/usr/bin/`.
- **Status:** Resolved

### Error #11: Yggdrasil config path mismatch
- **Phase:** Phase 6 — ISO build
- **Error:** `yggdrasil.service` references `-useconffile /etc/yggdrasil/yggdrasil.conf` but overlay provides config at `/etc/sovrn/yggdrasil.conf`.
- **Fix:** Changed `yggdrasil.service` ExecStart to `/etc/sovrn/yggdrasil.conf` in both source and overlay.
- **Status:** Resolved

### Error #12: 5 systemd unit files are stubs
- **Phase:** Phase 6 — ISO build
- **Error:** `caddy.service`, `sovrn-ca-bootstrap.service`, `sovrn-first-boot.service`, `sovrn-app-monitor.service`, `zram-setup.service` only contained `# see 31-SYSTEMD-UNITS.md`.
- **Fix:** Wrote full unit file content from design doc `29-SYSTEMD-CONFIGS.md`, adapted to current binary paths.
- **Status:** Resolved

### Error #13: No pre-flight validation before debos
- **Phase:** Phase 6 — ISO build
- **Error:** Each debos run (~25 min) revealed one error at a time. No validation step caught overlay path mismatches early.
- **Fix:** Added `validate_overlays()` function to `build-iso.sh` that checks all files exist and cross-references ExecStart paths.
- **Status:** Resolved

### Error #14: CA bootstrap script not found in nspawn overlay
- **Phase:** Phase 6 — ISO build
- **Error:** `chmod: cannot access '/etc/sovrn/scripts/bootstrap-ca.sh': No such file or directory`. File exists in overlay source but invisible inside nspawn (`--disable-fakemachine`).
- **Fix:** Inlined bootstrap-ca.sh into recipe run action, then removed — CA deferred to first boot via systemd service for per-node unique keys.
- **Status:** Resolved (by deferral)

### Error #15: resolv.conf nspawn bind mount
- **Phase:** Phase 6 — ISO build
- **Error:** `rm: cannot remove '/etc/resolv.conf': Device or resource busy` — systemd-nspawn mounts /etc/resolv.conf.
- **Fix:** Changed to `printf '%s\n' 'nameserver 1.1.1.1' > /etc/resolv.conf` — overwrites in-place without removing the mount.
- **Status:** Resolved

### Error #16: Pip3 not found after overlays in nspawn
- **Phase:** Phase 6 — ISO build
- **Error:** `pip3: not found` when running inside nspawn after overlay actions. Overlay binaries/scripts are visible via overlayfs bind mount, but pip3 (installed via apt) is missing from PATH or broken.
- **Fix:** Removed pip dependency entirely. Python packages are now prepared on the host and placed into `overlays/python-dist/` at final filesystem locations (`/usr/lib/python3/dist-packages/` and `/usr/bin/`). Debos uses a simple overlay copy with no pip execution needed.
- **Status:** Resolved

### Error #17: macOS sed -i incompatibility
- **Phase:** Phase 6 — ISO build
- **Error:** `sed -i` on macOS requires an explicit backup extension (`sed -i ''`) while GNU sed doesn't. `build-iso.sh` uses sed for Caddyfile stripping, which fails on macOS.
- **Fix:** Changed `sed -i` to redirect-to-temp-file + `mv` pattern, which works on both macOS and Linux.
- **Status:** Resolved

### Error #18: Yggdrasil binary path wrong in systemd unit
- **Phase:** Phase 7 — Final audit
- **Error:** `yggdrasil.service` references `/usr/bin/yggdrasil` but Debian apt package installs to `/usr/sbin/yggdrasil`. Service will fail to start at boot.
- **Fix:** Changed `ExecStart` to `/usr/sbin/yggdrasil`.
- **Status:** Resolved

### Error #19: Background image fallback creates invalid file
- **Phase:** Phase 7 — Final audit
- **Error:** `debos-sovrn.yaml` uses `convert` (ImageMagick) and `python3 -c "from PIL import Image"` for background generation, but neither imagemagick nor python3-pil is in the apt package list. Both fallbacks fail, final `touch` creates 0-byte invalid PNG.
- **Fix:** Added `python3-pil` to the runtime apt packages list.
- **Status:** Resolved

### Error #20: sovrnd.service Type=notify incompatible with uvicorn
- **Phase:** Phase 7 — Final audit
- **Error:** `sovrnd.service` uses `Type=notify` but uvicorn (used by sovrnd) does not support sd_notify protocol. systemd waits 30s for READY=1 signal, then kills the service.
- **Fix:** Changed to `Type=simple`. Removed `TimeoutStartSec=30` (not applicable to Type=simple).
- **Status:** Resolved

### Error #21: GRUB loadfont syntax error in build-iso-image.sh
- **Phase:** Phase 7 — Final audit
- **Error:** `loadfont=($root)/boot/grub/fonts/unicode.pf2` uses `=` instead of a space (`loadfont` is a GRUB command, not a variable assignment). Silently ignored due to `2>/dev/null || true`.
- **Fix:** Changed `loadfont=` to `loadfont ` (space instead of `=`).
- **Status:** Resolved
