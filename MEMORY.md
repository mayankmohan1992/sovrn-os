# MEMORY.md — Cross-Session Context

## Project

Sovrn OS — a privacy-first, decentralized operating system based on Debian Trixie with Yggdrasil mesh networking, `.sovrn` TLD, and an integrated PWA hub.

## Architecture (Top 5 Decisions)

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Base distro | Debian Trixie | Stable, Debos support |
| Mesh services | Rust (tokio) | Memory safety, async, small binaries |
| Orchestrator | Python/FastAPI | Rapid prototyping, rich async ecosystem |
| Frontend | Preact | Small bundle, hooks, TypeScript |
| OOBE wizard | GTK4/libadwaita | Native GNOME integration |

## Environment

- **Machine:** Apple Silicon (arm64)
- **Host OS:** macOS
- **Build method:** Cross-compilation for `x86_64-unknown-linux-gnu` via `cross` (Docker)
- **ISO generation:** Debos inside Docker Desktop
- **Python venv:** `~/.venvs/sovrn` (Python 3.12)
- **Go binaries path:** `~/go/bin`

## Key Paths

| Resource | Path |
|----------|------|
| Project root | `/Users/mayankmohan/Desktop/Projects/souvrn` |
| Source root | `/Users/mayankmohan/Desktop/Projects/souvrn/sovrn-os/src` |
| Rust workspace | `/Users/mayankmohan/Desktop/Projects/souvrn/sovrn-os/Cargo.toml` |
| Build output | `/Users/mayankmohan/Desktop/Projects/souvrn/sovrn-os/build/bin` |
| Build instructions | `/Users/mayankmohan/Desktop/Projects/souvrn/BUILD-INSTRUCTIONS.md` |

## Decisions Made During Build

*(To be populated as build progresses)*

## Session Log

- **2026-06-14:** Prerequisites installed. OOBE Cargo.toml missing — needs scaffolding before first build.
