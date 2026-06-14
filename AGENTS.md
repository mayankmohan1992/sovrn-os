# AGENTS.md — Instructions for AI Coding Agents

This file tells AI agents (like opencode) how to work on the Sovrn OS project.

---

## IMPORTANT: Working Directory

All source files, build artifacts, and configuration files live inside the `sovrn-os/` subdirectory. **All commands must be run from `sovrn-os/`.**

- Source root: `sovrn-os/src/`
- Build output: `sovrn-os/build/`
- Overlays: `sovrn-os/build/overlays/`
- Debos recipe: `sovrn-os/build/debos-sovrn-ci.yaml` (NOT `debos-sovrn.yaml` — that was deleted)

The session tracking files (`SESSION-STATE.md`, `BUILD-ERROR-LOG.md`, `project-state.md`) live one level up in `/Users/mayankmohan/Desktop/Projects/souvrn/` for visibility, but the actual work happens in `sovrn-os/`.

---

## Build Order (CI)

The CI workflow is at `.github/workflows/build-iso.yml`. It runs on `ubuntu-latest` via `scripts/build.sh all`:

1. **Rust**: `cross build --target x86_64-unknown-linux-gnu --release` (inside Docker)
2. **Go**: `GOOS=linux GOARCH=amd64 CGO_ENABLED=0 go build` (CDN agent only)
3. **Python**: `pip install -e .` for all 5 packages (sovrnd, sovrn-auth, sovrn-monitor, sovrn-notify-bridge, sovrn-complete-setup)
4. **PWA**: `npm install && npm run build` in `src/pwa/`
5. **Config assembly**: `build-iso.sh --quick` prepares overlays and runs validation
6. **Debos**: `sudo debos --disable-fakemachine` builds rootfs tarball (no KVM, 7 GB runner)
7. **Verification**: QEMU smoke test (boots image, checks SSH)

**Current focus:** CI builds are succeeding. Next: verify bootable image boots on real hardware (Ventoy or dd).

## Platform Notes

- **CI Host:** GitHub Actions `ubuntu-latest` (x86_64, 7 GB RAM, no KVM)
- **Local:** macOS Apple Silicon (for development only; final build must work on CI)
- **Rust target:** `x86_64-unknown-linux-gnu` (cross-compiled via `cross`)
- **Go target:** Linux amd64 (`GOOS=linux GOARCH=amd64 CGO_ENABLED=0`)
- **ISO build:** Debos with `--disable-fakemachine` (no VM, runs directly on host)

## After Every Edit

1. If applicable, push to `feature/ci-github-actions` and monitor CI run
2. Update `SESSION-STATE.md` with phase progress
3. Log any errors to `BUILD-ERROR-LOG.md`
4. Update `project-state.md` if blockers change

## Port Map (MUST be consistent)

| Service | Port |
|---------|------|
| sovrnd | 54771 |
| PWA Hub | 54772 |
| Notify Bridge | 54773 |
| DHT mesh | 54774 |
| Presence | 54775 |
| Auth | 54776 |
| Monitor | 54777 |
| Yggdrasil | 30550 |

## Common Pitfalls

- OOBE (GTK4) cross-compilation may fail — skip it by commenting out of workspace `Cargo.toml` members if needed
- Python services require venv activation (`~/.venvs/sovrn`)
- **Debos overlay paths**: When using `destination: /` with overlays, the source directory must include the full filesystem path (e.g., `overlays/bin/usr/bin/sovrn-dht` maps to `/usr/bin/sovrn-dht`). Do NOT use `destination` with deep paths — this debos version calls `os.Mkdir` (not `os.MkdirAll`) and fails if parent dirs don't exist.
- **Run `build-iso.sh --quick` instead of calling debos directly** — it prepares overlays and runs pre-flight validation before debos.
- **Binaries must be in `overlays/bin/usr/bin/`**, not directly in `overlays/bin/`. All systemd units reference `/usr/bin/`.
- **CI uses `--disable-fakemachine`** — no KVM available on 7 GB GitHub runner. This also means native `sudo` is required.
- **Ubuntu runners have stale Microsoft apt repos** — CI workflow pre-deletes them before `apt-get update`.
- **Always push to `feature/ci-github-actions` branch** (not `main`). PR #1 is open from this branch to `main`.
- **Run `build-iso.sh` from the repo root** (where `.github/` lives), NOT from `sovrn-os/`. The CI workflow runs from the checkout root.
