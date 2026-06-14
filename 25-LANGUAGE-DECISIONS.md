# Sovrn OS — Language Decisions

Every component's language choice, justified.

## Rust Services (Mesh Layer)

### sovrn-dht — Rust

**Why Rust:** DHT is performance-critical. Kademlia-style routing requires sub-millisecond lookups across thousands of entries. Garbage collection pauses would degrade mesh DNS resolution. Memory safety is critical for a service that handles untrusted network input continuously.

**Key Dependencies:**
- `tokio` — async runtime for UDP/TCP listeners on Yggdrasil
- `rusqlite` — local DHT record storage
- `serde` / `serde_json` — message serialization
- `ed25519-dalek` — record signing/verification
- `sha2` — content hashing for DHT keys

**Build Tooling:** Cargo (workspace member), cross-compilation via `cross` for x86_64-unknown-linux-gnu, static linking with musl for minimal .deb size (~3-5 MB binary).

### sovrn-identity — Rust

**Why Rust:** Cryptographic key management is the most security-sensitive component in the entire system. Memory safety prevents entire classes of key leakage vulnerabilities. Ed25519/X25519 operations must be side-channel resistant. Zero-copy key material handling prevents keys from lingering in memory.

**Key Dependencies:**
- `ed25519-dalek` — Ed25519 signing/verification (identity + events)
- `x25519-dalek` — X25519 key agreement (DM encryption)
- `hkdf` — HKDF-SHA256 for deterministic app password derivation
- `sha2` — SHA-256 for DID generation (did:mesh:{pubkey_hash})
- `bip39` — BIP-39 seed phrase generation/recovery (via `bip39` crate)
- `rusqlite` — persistent key storage
- `zeroize` — secure memory wiping for key material

**Build Tooling:** Cargo (workspace member), `strip = true` in release profile, LTO enabled.

### sovrn-presence — Rust

**Why Rust:** Presence service runs a high-frequency heartbeat protocol (2-minute intervals across potentially hundreds of peers). Gossip protocol requires managing many concurrent UDP connections. Memory usage must stay under 10 MB at idle. Rust's zero-cost async with tokio handles this without GC pauses.

**Key Dependencies:**
- `tokio` — async networking (UDP multicast on Yggdrasil)
- `rusqlite` — peer status persistence
- `serde` — heartbeat message serialization
- `tracing` — structured logging

**Build Tooling:** Cargo (workspace member).

### sovrn-feed — Rust

**Why Rust:** Feed service is an append-only event processing pipeline. It validates Ed25519 signatures on every incoming event, writes to SQLite, and serves timeline queries. Performance requirements: handle 1000+ events/sec for a popular node. Rust provides the necessary throughput without GC pauses during batch processing.

**Key Dependencies:**
- `tokio` — async I/O
- `rusqlite` — event storage (append-only log pattern)
- `ed25519-dalek` — event signature verification
- `serde_json` — event serialization (Nostr-like schema)
- `sha2` — event ID generation (SHA-256 of serialized content)

**Build Tooling:** Cargo (workspace member).

### sovrn-message-queue — Rust

**Why Rust:** Message queue handles encrypted DMs with delivery guarantees. It must persist messages to disk, manage delivery receipts, and handle X25519 sealed-box encryption/decryption. Write-ahead log integrity is critical — Rust prevents data corruption from memory safety issues.

**Key Dependencies:**
- `tokio` — async message delivery
- `rusqlite` — persistent message queue (WAL mode)
- `x25519-dalek` — sealed box encryption for DMs
- `serde` — message serialization

**Build Tooling:** Cargo (workspace member).

---

## Go Service

### sovrn-cdn-agent — Go

**Why Go (not Rust):** CDN agent makes many outbound HTTP requests to CDN provider APIs, manages upload/download queues, and handles content-addressed storage caching. Go's excellent `net/http` and `io` libraries make it the best fit. The CDN agent is not part of the critical mesh path — if it crashes, only CDN uploads are affected (feed/messages continue working). Go's simpler concurrency model and fast compile times are preferable here. Go also has mature S3/cloud SDKs for CDN integration.

**Key Dependencies:**
- `net/http` — CDN provider API calls
- `github.com/minio/minio-go` — S3-compatible object storage
- `github.com/docker/go-connections` — Podman API (for local CDN cache management)
- `gorm.io/gorm` — SQLite ORM for CDN metadata (optional, may use raw `database/sql`)
- `github.com/spf13/cobra` — CLI and config

**Build Tooling:** Go modules, `go build` produces static binary, Makefile for `build` and `install` targets. Cross-compile with `GOOS=linux GOARCH=amd64`.

---

## Python Services (Orchestration Layer)

### sovrnd — Python 3.12+

**Why Python (already decided):** sovrnd is the orchestrator daemon. It talks to systemd, Podman, Caddy, and D-Bus — all of which have excellent Python bindings. FastAPI provides the REST API with automatic OpenAPI docs. The daemon's performance bottleneck is I/O (waiting for Podman, Caddy, SQLite), not CPU. Python's rich ecosystem for system administration tasks (subprocess, dbus-python, Podman SDK) makes it the right choice.

**Key Dependencies:**
- `fastapi` + `uvicorn` — REST API server
- `pydantic` — request/response validation
- `httpx` — async HTTP client (service IPC, Caddy API)
- `pynacl` — Ed25519 signing (shared logic with identity service)
- `podman` — Podman Python SDK for container lifecycle
- `tomli` — TOML config parsing

**Build Tooling:** `pip install -e .` for development, `python -m build` for sdist/wheel, `dpkg-deb` for .deb packaging. `pyproject.toml` with setuptools backend.

### sovrn-auth — Python (part of sovrnd)

**Why Python:** The auth middleware is a thin JWT validation layer. It runs as a Caddy plugin (via JSON config) or as a sidecar process. ~200 LOC, no performance concern. Shares `pynacl` + `pyjwt` with sovrnd for JWT verification.

**Key Dependencies:**
- `pynacl` — Ed25519 signature verification for JWT
- `pyjwt` — JWT parsing and validation

**Build Tooling:** Installed as part of sovrnd package or separate pip package.

### sovrn-app-monitor — Python

**Why Python:** Container health monitoring is fundamentally a systemd/D-Bus + subprocess task. Python has native D-Bus bindings via PyGObject and `podman` SDK. Monitoring logic is simple (poll health, restart on failure, notify), not performance-critical. The existing pseudocode in 22-D-U5 is Python.

**Key Dependencies:**
- `podman` — container health checks
- `httpx` — notify-bridge HTTP calls

**Build Tooling:** Same as sovrnd — pyproject.toml, pip, dpkg-deb.

### sovrn-notify-bridge — Python (50 LOC)

**Why Python (already decided):** 50 lines of code. An HTTP server that calls `notify-send`. Python's `http.server` makes this trivial. Already decided in 18-K1.

**Key Dependencies:**
- stdlib only — `http.server`, `subprocess`, `json`
- System package: `libnotify-bin` (for `notify-send`)

**Build Tooling:** Installed as a single Python script. Minimal .deb with systemd unit.

---

## Frontend & Desktop

### PWA — Preact + TypeScript + Vite

**Why Preact (already decided in 18-K2):** 3KB core, `preact/signals` for fine-grained reactivity, zero telemetry, MIT license. 95% React ecosystem compatibility via `preact/compat`. Service worker + Workbox for offline. Dexie.js for IndexedDB.

**Key Dependencies:**
- `preact` + `@preact/signals` — UI framework + state
- `preact-router` — client-side routing
- `dexie` — IndexedDB wrapper for offline storage
- `workbox-build` / `vite-plugin-pwa` — service worker generation
- `typescript` — type safety

**Build Tooling:** Vite (dev server with HMR on port 54772, production build to static files). `pnpm` for dependency management. Output goes to `/var/lib/sovrn/pwa/` in production.

### OOBE — GTK4 + Python (PyGObject)

**Why Python + GTK4 (already decided in 02-COMPONENTS):** GTK4 is the modern GNOME toolkit with native Wayland support. Python via PyGObject provides the fastest development path for a GTK4 wizard app (5 screens, no complex rendering). Avoids Vala/C complexity for a one-time setup wizard.

**Key Dependencies:**
- `PyGObject` (gi) — Python GTK4 bindings
- `libadwaita` — GNOME HIG-compliant widgets
- `bip39` (Python) — seed phrase generation (or call sovrn-identity via IPC)

**Build Tooling:** `meson` build system (standard for GNOME apps). `python -m build` for wheel. Installed as system package.

---

## Language Decision Summary

| Component | Language | Binary Size (est.) | RAM (idle est.) | Build Tool |
|-----------|----------|--------------------|-----------------|------------|
| sovrn-dht | Rust | ~4 MB | ~15 MB | Cargo |
| sovrn-identity | Rust | ~3 MB | ~10 MB | Cargo |
| sovrn-presence | Rust | ~3 MB | ~10 MB | Cargo |
| sovrn-feed | Rust | ~4 MB | ~20 MB | Cargo |
| sovrn-message-queue | Rust | ~3 MB | ~15 MB | Cargo |
| sovrn-cdn-agent | Go | ~8 MB | ~25 MB | go build |
| sovrnd | Python 3.12 | ~2 MB (source) | ~50 MB | pip/setuptools |
| sovrn-auth | Python | included in sovrnd | included | pip |
| sovrn-app-monitor | Python | ~0.5 MB | ~20 MB | pip |
| sovrn-notify-bridge | Python | ~5 KB | ~5 MB | script |
| PWA | Preact/TS | ~500 KB (gzipped) | browser | Vite/pnpm |
| OOBE | Python/GTK4 | ~1 MB | ~30 MB (GTK) | meson |

**Total service RAM target: ~150 MB** (well within budget for 8 GB system with GNOME at ~800 MB).

---

## IPC Protocol Between Services

All Rust services communicate with sovrnd over **Unix domain sockets** in `/run/sovrn/`:

```
/run/sovrn/
├── sovrn-dht.sock         # DHT lookup/register/republish
├── sovrn-identity.sock    # Key signing, verification, derivation
├── sovrn-presence.sock    # Heartbeat, peer list, status queries
├── sovrn-feed.sock        # Post, fetch, follow, react
├── sovrn-mq.sock          # Send/receive DMs
└── sovrn-cdn.sock         # CDN push/pull/status (also HTTP on port)
```

**Protocol:** JSON-RPC 2.0 over Unix sockets. Why JSON-RPC:
- Simple — no protobuf/gRPC compilation step needed
- Debuggable — `socat - UNIX-CONNECT:/run/sovrn/sovrn-dht.sock` for manual testing
- Language-agnostic — Python sovrnd can call Rust services trivially
- Async — tokio for Rust, asyncio for Python both handle it well

Go CDN agent also exposes HTTP on `127.0.0.1:54774` for sovrnd (in addition to the Unix socket) because Go's `net/http` is more natural than custom Unix socket framing.

---

## Build Matrix

| Component | Compiler/Version | Cross-compile Target | Static Linking |
|-----------|-----------------|---------------------|----------------|
| Rust services | rustc 1.78+ | x86_64-unknown-linux-musl | Yes (musl) |
| Go CDN agent | go 1.22+ | GOOS=linux GOARCH=amd64 | Yes |
| Python services | python 3.12+ | N/A (system Python) | No |
| PWA | node 20+, pnpm 9+ | N/A (static build) | N/A |
| OOBE | python 3.12+ + GTK4 | N/A (system packages) | No |