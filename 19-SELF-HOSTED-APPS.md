# Sovrn — Self-Hosted App System (Round 10 Decisions)

## Overview

Sovrn OS will include an Umbrel-like self-hosted app system. Users can install and run privacy-focused self-hosted apps (Nextcloud, Vaultwarden, Wallabag, etc.) directly from their Sovrn device with minimal configuration — just create a username and password to start using each app.

## Design Principles

1. **Ease of use first** — minimal configuration, OOTB experience
2. **Hybrid deployment** — apt for system/lightweight apps, Podman containers for user-facing apps
3. **No port numbers visible to user** — reverse proxy handles all routing
4. **Resource-aware** — memory management, OOM protection, swap optimization
5. **Declarative** — app manifests describe what to install, Sovrn daemon executes

## Architecture

### M1: Container Runtime → Podman (rootless)

Podman chosen over Docker because:
- **Rootless by default** — aligned with Sovrn's privacy/security stance
- **Systemd integration** — containers become systemd services via quadlet files (`.container`)
- **No daemon** — lower attack surface, no Docker daemon running as root
- **Docker Compose compatible** — `podman-compose` translates compose files to Podman
- **Same OCI images** — all Docker Hub images work unchanged

Implementation:
- Podman + podman-compose installed by default
- Each app runs as a Podman pod with its own network namespace
- Apps run as user 1000:1000 where possible (like Umbrel's pattern)
- systemd quadlets for auto-restart and service management

### M2: App Packaging → Hybrid (apt + Podman)

**System apps (apt):**
- Yggdrasil, Caddy/reverse proxy, DNS resolver, firewall, Firmware update daemon
- These are lightweight, well-maintained in Debian, and don't need container isolation
- Managed by apt + systemd

**User-facing apps (Podman containers):**
- Nextcloud, Vaultwarden, Wallabag, FreshRSS, etc.
- Each app is a directory containing:
  - `sovrn-app.yml` — manifest (adapted from Umbrel's format)
  - `docker-compose.yml` — Podman-compatible compose file
  - `data/` — optional template data copied on install
- Stored in Git repo, cloned to device, polled for updates

**Why not all containers like Umbrel?**
- System services (Yggdrasil, DNS, firewall) must start before networking and containers
- apt packages get Debian security updates automatically — no image pull needed
- Lower RAM overhead for system services (no container layer)
- Podman containers for user apps give us isolation + easy install/remove

### M3: Port Allocation → Reverse Proxy (Zero Visible Ports)

**Problem:** Users should never see port numbers. Port conflicts between apps must be impossible. PWA and daemon API must share the same origin to avoid service worker/CORS issues.

**Solution: Caddy reverse proxy + internal Podman network**

```
User browser → https://sovrn.local (Caddy on 80/443)
                    │
                    ├── / → Sovrn PWA (static files from /var/lib/sovrn/pwa/)
                    ├── /api/v1/* → sovrnd daemon (upstream localhost:54771)
                    ├── /api/notify → Notify bridge (upstream localhost:54773)
                    ├── /ws → sovrnd WebSocket (real-time updates)
                    ├── nextcloud.sovrn.local → Nextcloud container (internal net)
                    ├── vaultwarden.sovrn.local → Vaultwarden container (internal net)
                    ├── wallabag.sovrn.local → Wallabag container (internal net)
                    └── *.sovrn.local → <app>.sovrn.local (any installed app)
```

**How it works:**
1. Caddy runs on ports 80/443 on `sovrn.local` (resolved via /etc/hosts + mDNS)
2. Each app container gets an internal Podman IP on a private bridge network (`10.47.0.0/16`)
3. Caddy reverse proxies each app by subdomain → internal IP:port
4. Apps NEVER expose ports to the host network directly
5. Only Caddy (80/443) binds to host. sovrnd (54771) and notify bridge (54773) bind to loopback only.
6. `.sovrn` TLD requests route through Yggdrasil mesh DNS → Caddy → same reverse proxy
7. PWA and API share the same origin (`https://sovrn.local/`), eliminating CORS and service worker scoping issues. Port 54772 (Preact dev server) is development-only and never runs in production.

**Conflict-free by design:**
- No two apps can claim the same subdomain (Caddy validates on install)
- Apps don't know about ports — they listen on their internal Podman IP
- Adding a new app = adding one Caddy subdomain route (automated by Sovrn daemon)
- Port 80/443 on localhost is only used by Caddy (no conflict possible)

### M4: Authentication → App-Level with Auto-Provisioning (No SSO)

**Problem:** Full SSO is technically complex. But creating a username/password per app is friction.

**Solution: Sovrn-managed auto-provisioning**

Each app has its own independent auth. But the Sovrn daemon automates account creation:

1. User installs Nextcloud from the App Center
2. Sovrn daemon:
   - Pulls container images
   - Starts containers
   - Auto-creates a Nextcloud admin account using:
     - Username: same as Sovrn identity username
     - Password: derived from Sovrn seed (deterministic, like Umbrel's pattern)
   - Stores credentials in the Sovrn password vault (encrypted, local)
3. User clicks "Open Nextcloud" → already logged in via Caddy auth proxy
4. First-run: user can change password in the app settings

**Auth proxy layer (adapted from Umbrel):**
- Caddy + `sovrn-auth` service validates session JWT before proxying to app
- User logs into Sovrn dashboard once → gets JWT → transparent access to all installed apps
- Apps still have independent auth as fallback
- This gives SSO-like UX without SSO complexity — if auth proxy fails, user can still log in directly

**Why this over full SSO:**
- No dependency on a central auth server being up for apps to work
- Each app remains fully functional standalone
- If the Sovrn auth service goes down, apps still work with their own credentials
- Simpler to maintain, simpler to debug

### M5: Memory Management → systemd-oomd + zram + cgroups v2

**Linux-native memory management (no custom daemon needed):**

| Component | Purpose |
|-----------|---------|
| **systemd-oomd** | Monitors memory pressure, kills heavies first, per-cgroup monitoring |
| **zram** | Compressed swap in RAM — effectively doubles available memory on 8GB |
| **cgroups v2** | Each app container gets a memory cgroup with limits |
| **Podman resource limits** | `--memory` and `--cpus` flags on each container |
| **earlyoom** | Userspace OOM killer as backup (faster response than kernel OOM) |

**Implementation:**
- Default: zram swap = 50% of RAM (4GB zram on 8GB system)
- systemd-oomd monitors swap and memory pressure levels
- When memory pressure hits "medium" — desktop notification via Sovrn notify bridge
- When memory pressure hits "critical" — systemd-oomd kills the heaviest app container
- Apps declare `recommendedRam` and `minimumRam` in their manifest
- App Center shows "Recommended: 8GB RAM" warnings for heavy apps
- No hard block on installation — user decides

### M6: App Updates → Git Repo Manifests + Container Registry Pulls

**How updates work (following Umbrel's pattern):**

1. Sovrn daemon polls the app store Git repo every 6 hours (not 5 min — too aggressive)
2. When a new commit is found, daemon reads updated manifests
3. For container apps: pulls new image from Docker Hub / ghcr.io / registry
4. For apt apps: standard Debian `unattended-upgrades`
5. Updates are applied during configurable maintenance window (default: 3-5 AM)
6. Podman containers use `--pull always` on restart
7. Rollback: if new image fails health check within 60s, previous image is restored

**We do NOT host container images ourselves.** Images come from upstream registries (Docker Hub, ghcr.io). Manifests come from our Git repo (which is tiny — just YAML files).

### M7: Backup/Restore → Borg

- **Tool:** BorgBackup (deduplicating, compressed, encrypted)
- **Targets:** Local USB drive, external SSD/HDD, network share
- **Scope:** Full system backup including:
  - `/var/lib/sovrn/` — all service data
  - `/home/user/.sovrn/` — user config, media cache, homepage
  - `/var/lib/sovrn/app-data/` — all app container data
  - App manifests — which apps are installed (for one-click restore)
- **Incremental:** Borg only stores diffs — daily backups are fast after first full
- **Encryption:** Borg encrypts by default — backup is useless without the key
- **Sovrn daemon** triggers backups via systemd timer (daily by default)
- **Restore UX:** "Recover from backup" option in OOBE or Settings → installs same apps + restores data

### M8: App Store → Curated Official + Community Repos

- **Official repo:** `github.com/sovrn-os/sovrn-apps` — curated, tested, verified apps
- **Community repos:** Users can add third-party repos (same pattern as Umbrel)
- **App Center UI:** Built into the Sovrn PWA (not a separate app)
- **Categories:** Files, Security, Reading, Media, Communication, Development, Utilities
- **Each app page shows:** Description, screenshots, resource requirements, install size, RAM usage estimate

### M9: Wallabag → Custom App Package

Wallabag is not in the Umbrel store. We create our own `sovrn-app.yml` + `docker-compose.yml` for it, hosted in the official Sovrn apps repo. Readeck is included as a lighter alternative for users who don't need Wallabag's full feature set.

### M10: App Installation UX → Post-OOBE, Not in Wizard

Self-hosted apps are NOT part of the OOBE wizard. The flow is:

1. OOBE: Create Sovrn identity → network setup → done, into dashboard
2. User opens App Center from Sovrn dashboard
3. Browses apps, clicks Install
4. Sovrn daemon: pull images → start containers → auto-provision account → add to Caddy routes → show in dashboard
5. User clicks "Open" → already authenticated via Sovrn auth proxy

Some lightweight apps (Vaultwarden, AdGuard Home) ship pre-installed but NOT started. User enables them via App Center when ready.

## Sovrn App Manifest Format (sovrn-app.yml)

```yaml
manifestVersion: "1.0"
id: nextcloud
name: Nextcloud
tagline: Your private cloud
version: "33.0.4"
category: files
description: >
  Self-hosted productivity platform...
developer: Nextcloud GmbH
website: https://nextcloud.com
repo: https://github.com/nextcloud/server
support: https://help.nextcloud.com
dependencies: []
permissions: []
recommendedRam: 2048       # MB - recommended for smooth operation
minimumRam: 1024           # MB - absolute minimum
installSize: 578000000     # bytes - compressed image size
containers: 4               # number of containers this app runs
path: nextcloud          # Caddy subdomain: nextcloud.sovrn.local
defaultUsername: sovrn       # Auto-provisioned username
deterministicPassword: true # Derived from Sovrn seed
backupIgnore:
  - "data/appdata_*"        # Cache dirs to skip in backup
restartPolicy: unless-stopped
healthCheck:
  interval: 30s
  timeout: 10s
  retries: 3
```

Non-Umbrel additions:
- `recommendedRam` / `minimumRam` — for App Center warnings
- `containers` — so users know resource impact before installing
- `path` — Caddy subdomain prefix (validated on install to prevent conflicts)
- `restartPolicy` — Podman restart policy
- `healthCheck` — for rollback on failed updates

## Directory Structure (Updated)

```
/var/lib/sovrn/
  ├── app-data/              # Per-app container data
  │   ├── nextcloud/
  │   ├── vaultwarden/
  │   └── wallabag/
  ├── app-stores/            # Cloned Git repos (official + community)
  │   └── sovrn-apps/        # Official app store
  ├── apps/                  # Registry of installed apps (JSON)
  ├── dht/                   # DHT node data
  ├── identity/              # Key management
  ├── feed/                  # Social feed SQLite
  ├── messages/              # Message queue
  ├── presence/              # Presence tracking
  ├── cdn/                   # CDN agent cache
  └── borg/                  # Borg backup config & keys

/home/user/.sovrn/
  ├── config/                # User settings
  ├── backup/                # Export directory
  ├── media/                 # Cached media
  ├── homepage/              # Homepage files
  └── app-configs/           # Per-app user configs (mounted into containers)
```

## Sovrn Daemon (sovrnd)

The Sovrn daemon is the orchestrator — equivalent to Umbrel's `umbreld` but Python instead of TypeScript:

**Responsibilities:**
- App lifecycle: install, start, stop, update, uninstall
- Caddy route management: add/remove path prefixes per app
- Auth proxy: validate JWT, inject auth headers
- Health monitoring: check container health, auto-restart, rollback on failure
- Memory monitoring: integrate with systemd-oomd, send warnings via notify bridge
- App store management: clone/pull Git repos, read manifests
- Auto-provisioning: create accounts in newly installed apps
- Backup scheduling: trigger Borg backups via systemd timer

**Stack:**
- Language: Python (systemd integration, Podman SDK, no Docker daemon dependency)
- API: REST on sovrn.local (via Caddy reverse proxy, same origin as PWA)
- IPC with Podman: podman Python SDK + systemd quadlet files
- IPC with Caddy: Caddy API (JSON config push)

## V1 App Catalog (Shipped in Official Store)

**Tier 1 — Pre-installed (not started):**
| App | Category | Purpose |
|-----|----------|---------|
| Vaultwarden | Security | Password manager |
| AdGuard Home | Network | DNS/ad blocking |

**Tier 2 — One-click install:**
| App | Category | RAM |
|-----|----------|-----|
| Nextcloud | Files | 1-2 GB |
| Wallabag | Reading | 200-500 MB |
| Readeck | Reading | 60 MB |
| FreshRSS | News | 60-150 MB |
| Navidrome | Music | 80-200 MB |
| Memos | Notes | 50-100 MB |
| Gitea | Dev | 150-400 MB |
| SearXNG | Search | 80-200 MB |
| WireGuard Easy | VPN | 20-50 MB |
| File Browser | Files | 10-30 MB |

**Tier 3 — Available but resource-warning:**
| App | Category | RAM | Warning |
|-----|----------|-----|---------|
| Jellyfin | Media | 0.5-2 GB | Transcoding needs GPU |
| Immich | Photos | 0.5-2 GB | ML inference heavy |
| Synapse + Element | Chat | 0.5-1.5 GB | Federation grows unbounded |
| Paperless-ngx | Docs | 0.8-1.5 GB | 5 containers |
| Stalwart | Email | 0.15-0.5 GB | DNS config needed |

## Conflicts Identified & Resolved

### Conflict 1: "OOTB username/password" vs "No SSO"
**User wants:** Install app → create username/password → start using immediately
**User also said:** Don't implement SSO if it's technically complex
**Resolution:** Auto-provisioning (M4). Sovrn daemon creates accounts automatically using deterministic passwords. Auth proxy layer (Caddy + JWT) gives SSO-like UX. If auth proxy goes down, apps still work with their own credentials. Best of both worlds.

### Conflict 2: "apt + containers" vs "Ease of use"
**User wants:** Apt for lightweight, containers for user-facing
**Potential issue:** apt packages often need manual configuration
**Resolution:** System apps (Yggdrasil, DNS, Caddy) are pre-configured by Sovrn and never touched by the user. They start automatically on boot. User never configures them. User-facing apps are all containers — zero config needed, just click Install.

### Conflict 3: "Don't host images ourselves" vs "Decentralized vision"
**User wants:** Not hosting container images (pull from Docker Hub)
**Potential issue:** Docker Hub is centralized — contradicts Sovrn's decentralization
**Resolution:** v1 pulls from Docker Hub. v2/v3 should add: (1) Seed nodes seed popular images over Yggdrasil mesh, (2) Community mirrors that apps can pull from. This is a future enhancement, not a v1 blocker. For now, Docker Hub is fine — images are content-addressed and verifiable.

### Conflict 4: "No port conflicts" vs "Container ports"
**Resolution:** M3. Caddy reverse proxy on 80/443. Apps only listen on internal Podman IPs. Zero port conflicts possible by design.

### Conflict 5: "Memory management" vs "User's machine isn't powerful enough"
**User says:** That's the user's problem, but implement best memory management
**Resolution:** M5. systemd-oomd kills heavies under pressure. zram doubles effective RAM. App Center shows RAM estimates. No hard blocks — warnings only.