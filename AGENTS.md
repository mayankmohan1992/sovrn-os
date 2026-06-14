# AGENTS.md — Instructions for AI Coding Agents

This file tells AI agents (like opencode) how to work on the Sovrn OS project.

---

## IMPORTANT: Working Directory

All source files, build artifacts, and configuration files live inside the `sovrn-os/` subdirectory. **All commands must be run from `sovrn-os/`.**

- Source root: `sovrn-os/src/`
- Build output: `sovrn-os/build/`
- Overlays: `sovrn-os/build/overlays/`
- Debos recipe: `sovrn-os/build/debos-sovrn.yaml`

The session tracking files (`SESSION-STATE.md`, `BUILD-ERROR-LOG.md`, `project-state.md`) live one level up in `/Users/mayankmohan/Desktop/Projects/souvrn/` for visibility, but the actual work happens in `sovrn-os/`.

---

## Build Order (DO NOT SKIP)

Follow `BUILD-INSTRUCTIONS.md` strictly. The build order is:

1. **Phase 0**: Fix blockers (missing manifests, etc.)
2. **Phase 1**: Rust workspace (`cross build --target x86_64-unknown-linux-gnu --release`)
3. **Phase 2**: Go components (CDN agent + Caddy auth plugin)
4. **Phase 3**: Python services (`pip install -e .`)
5. **Phase 4**: Preact PWA (`npm install && npm run build`)
6. **Phase 5**: Assemble config files into `build/`
7. **Phase 6**: Build ISO (Debos via Docker)
8. **Phase 7**: Verification checklist

**Phases 1–5 are complete.** Current work is Phase 6 (Debos ISO build).

## Platform Notes

- **Host:** macOS Apple Silicon
- **Rust target:** `x86_64-unknown-linux-gnu` (cross-compiled via `cross`)
- **Go target:** Linux amd64 (native cross-compilation with `GOOS=linux GOARCH=amd64 CGO_ENABLED=0`)
- **ISO build:** Runs inside Docker Desktop (`ghcr.io/go-debos/debos:latest`)

## After Every Edit

1. Run the relevant build command to verify it compiles
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
- Docker Desktop must have "Use Virtualization Framework" enabled for Debos KVM
- Python services require venv activation (`~/.venvs/sovrn`)
- **Debos overlay paths**: When using `destination: /` with overlays, the source directory must include the full filesystem path (e.g., `overlays/bin/usr/bin/sovrn-dht` maps to `/usr/bin/sovrn-dht`). Do NOT use `destination` with deep paths — this debos version calls `os.Mkdir` (not `os.MkdirAll`) and fails if parent dirs don't exist.
- **Run `build-iso.sh --quick` instead of calling debos directly** — it prepares overlays and runs pre-flight validation before debos.
- **Binaries must be in `overlays/bin/usr/bin/`**, not directly in `overlays/bin/`. All systemd units reference `/usr/bin/`.
