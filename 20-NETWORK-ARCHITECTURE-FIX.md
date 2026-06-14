# Sovrn — Network Architecture Fix: DNS, Origin & Serving

## Problem Statement

Three interrelated architectural issues threaten the integrity of the Sovrn OS networking stack:

1. **TLD Naming Conflict** — Multiple docs use `.os` and `.sovrn` interchangeably for the mesh TLD. The decision is `.sovrn`. All `.os` references that mean "mesh TLD" must be replaced.

2. **PWA Serving Architecture (Origin Scoping)** — The PWA is referenced at both `localhost.sovrn:54772` (Preact dev server) AND `sovrn.local:443` (Caddy). Service workers are scoped to their origin — two origins = two separate cache states, two SW registrations, split offline data. This is a hard blocker.

3. **DNS Namespace Collision** — `.sovrn` (mesh) and potential `.sovrn.local` (local apps) can confuse DNS resolution. We need a clean DNS dispatcher design.

## Decisions

### D-N1: Canonical PWA URL — `sovrn.local` Only (HTTPS)

**Users access the PWA exclusively at `https://sovrn.local/`.**

| URL | Role | Users See It? |
|---|---|---|
| `https://sovrn.local/` | **Canonical PWA URL** — production, service worker scope | YES — the only way to use Sovrn |
| `http://sovrn.local/` | Redirect to HTTPS (Caddy auto-redirect) | No — redirects |
| `https://sovrn.local/api/` | Daemon API, same origin | No — PWA fetches internally |
| `http://localhost:54772/` | Preact dev server — development only | NO — developer-only, never user-facing |

**Why single origin:**
- Service workers are scoped to their origin. `https://sovrn.local/` and `http://localhost.sovrn:54772/` are different origins. If users hit both, they get two SW registrations, two caches, two IndexedDB states — data silently diverges.
- PWA install prompt (beforeinstall) only fires on HTTPS origins.
- Push notifications and Web APIs (Clipboard API, etc.) require a secure context.
- CORS is trivially avoided when PWA and API share the same origin.

### D-N2: Preact Dev Server (port 54772) — Development Only

| Context | What Runs | URL |
|---|---|---|
| **Production (user's Sovrn OS)** | Preact builds static assets → Caddy serves them | `https://sovrn.local/` |
| **Development (coding)** | Preact dev server with HMR | `http://localhost:54772/` |

In production:
- The Preact app is built (`preact build`) into static HTML/CSS/JS.
- Output is placed in `/var/lib/sovrn/pwa/` (or equivalent static dir).
- Caddy serves these static files directly — no Node.js process in production.
- The daemon API is reverse-proxied by Caddy at `/api/` on the same origin.

In development:
- Developer runs `preact dev` on port 54772 for hot module replacement.
- Caddy in dev mode can proxy `/` to `localhost:54772` for HMR, or developer uses `localhost:54772` directly.
- **The dev server never runs on a user's production Sovrn installation.**

### D-N3: DNS Resolution Flow — Three Namespaces

```
┌──────────────────────────────────────────────────────────────────────┐
│                      DNS RESOLUTION FLOW                            │
│                                                                      │
│  User types URL in browser                                          │
│         │                                                            │
│         ▼                                                            │
│  ┌─────────────┐                                                     │
│  │ DNS Client   │  (systemd-resolved + custom Sovrn dispatcher)     │
│  └──────┬──────┘                                                     │
│         │                                                            │
│         ▼                                                            │
│  ┌──────────────────────────────────────────┐                        │
│  │          NAMESPACE DISPATCHER             │                        │
│  │                                            │                        │
│  │  TLD ends in .sovrn?                      │                        │
│  │    YES → Yggdrasil mesh DNS (DHT)         │                        │
│  │    NO  → continue                         │                        │
│  │                                            │                        │
│  │  TLD is .sovrn.local?                     │                        │
│  │    YES → local service resolution          │                        │
│  │    NO  → continue                         │                        │
│  │                                            │                        │
│  │  Everything else:                          │                        │
│  │    → regular DNS (Unbound resolver)        │                        │
│  └──────────────────────────────────────────┘                        │
│                                                                      │
│  THREE NAMESPACES:                                                   │
│                                                                      │
│  1. .sovrn          → Mesh DHT → Yggdrasil IPv6                    │
│     alice.sovrn      → 200::cafe:... (mesh node)                   │
│     bob.sovrn        → 200::face:... (mesh node)                   │
│                                                                      │
│  2. .sovrn.local     → Local services (via /etc/hosts + mDNS)       │
│     sovrn.local       → 127.0.0.1 (Caddy — PWA + daemon API)      │
│     nextcloud.sovrn.local → 10.47.0.x (Podman container)          │
│     vaultwarden.sovrn.local → 10.47.0.y (Podman container)       │
│                                                                      │
│  3. everything else → Regular DNS (Unbound) → internet             │
│     google.com       → 142.250.x.x (internet)                    │
│     github.com        → 140.82.x.x (internet)                     │
└──────────────────────────────────────────────────────────────────────┘
```

**Implementation:**

The DNS dispatcher is implemented in `systemd-resolved` with a fallback to a custom local DNS resolver:

1. **`/etc/nsswitch.conf`** — standard glibc resolution order
2. **`/etc/hosts`** — contains `127.0.0.1 sovrn.local` (and Podman container IPs for installed apps)
3. **`systemd-resolved`** — configured with:
   - `DNS=.sovrn` → route to Yggdrasil mesh DNS (custom DHT resolver on `127.0.0.1:53535`)
   - `DNS=~.sovrn.local` → handle via `/etc/hosts` + mDNS (Avahi)
   - `DNS=.*` → forward to Unbound (regular internet DNS)
4. **Yggdrasil mesh DNS resolver** — small daemon listening on `127.0.0.1:53535` that queries the DHT for `.sovrn` domains and returns Yggdrasil IPv6 addresses

**Subdomain pattern for self-hosted apps:**

Each installed app gets a subdomain under `.sovrn.local`:

| App | Subdomain | Resolves To |
|---|---|---|
| Sovrn PWA + API | `sovrn.local` | `127.0.0.1` (Caddy) |
| Nextcloud | `nextcloud.sovrn.local` | Caddy reverse proxy → `10.47.0.x` |
| Vaultwarden | `vaultwarden.sovrn.local` | Caddy reverse proxy → `10.47.0.y` |

**Why `.sovrn.local` and not path prefixes:**

Previous design used path prefixes (`/nextcloud/`). Subdomain-based routing is better because:
- Each app gets its own origin (necessary for apps like Nextcloud that expect their own domain)
- No path collision risk (paths belong to the app, not the proxy)
- Service workers for each app are isolated by subdomain
- SSL cert per app is cleaner (Caddy auto-manages via `sovrn.local` wildcard)

However, for simplicity in v1, both path-prefix AND subdomain routing work through Caddy. The canonical approach for self-hosted apps is **subdomain routing** (`nextcloud.sovrn.local`). Path-prefix routing (`/nextcloud/`) remains available as a fallback.

### D-N4: Caddy Configuration Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                     CADDY REVERSE PROXY                             │
│                     sovrn.local:80/443                             │
│                                                                     │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │  HTTPS listener (sovrn.local:443)                             │  │
│  │                                                               │  │
│  │  Auto-TLS: self-signed CA, trusted by OS cert store          │  │
│  │                                                               │  │
│  │  Routes:                                                      │  │
│  │                                                               │  │
│  │  /              → static files at /var/lib/sovrn/pwa/         │  │
│  │                   (Preact build output — JS/CSS/HTML)        │  │
│  │                                                               │  │
│  │  /api/*        → sovrnd REST API                              │  │
│  │                   unix socket or localhost:54771               │  │
│  │                                                               │  │
│  │  /api/notify   → notify bridge on localhost:54773            │  │
│  │                   (POST to push desktop notifications)         │  │
│  │                                                               │  │
│  │  /ws           → WebSocket for real-time feed updates         │  │
│  │                   (sovrnd pushes feed/message events)         │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                                                                     │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │  Per-App Subdomain Routes (added dynamically by sovrnd)      │  │
│  │                                                               │  │
│  │  nextcloud.sovrn.local    → 10.47.0.x:8080                   │  │
│  │  vaultwarden.sovrn.local → 10.47.0.y:80                      │  │
│  │  wallabag.sovrn.local    → 10.47.0.z:8080                    │  │
│  │  ...                                                          │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                                                                     │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │  Mesh Domain Routes (SOVRN TLD .sovrn)                        │  │
│  │                                                               │  │
│  │  *.sovrn (via Yggdrasil)  →  Caddy on ygg0 interface         │  │
│  │    alice.sovrn → Yggdrasil IPv6 of Alice's node              │  │
│  │                                                               │  │
│  │  User's own node listens on:                                  │  │
│  │    theirdomain.sovrn (ygg0 interface, port 80/443)           │  │
│  │    Caddy serves:                                              │  │
│  │      /           → user's homepage                            │  │
│  │      /api/feed/*  → feed API (mesh-visible, read-only)       │  │
│  │      /api/presence → presence API (mesh-visible)             │  │
│  └───────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

**Caddy is the ONLY entry point for user-facing HTTP/HTTPS traffic.**

Key Caddy behaviors:
- **Auto-HTTPS** for `sovrn.local` using a self-signed root CA. The Sovrn OS installer adds this CA to the system trust store and the bundled browsers' trust stores. This ensures HTTPS works without warnings.
- **HTTP → HTTPS redirect** on port 80 for `sovrn.local`.
- **Dynamic route management** via Caddy API — `sovrnd` adds/removes per-app subdomain routes when apps are installed/uninstalled.
- **Service worker scope** — With PWA and API on the same origin (`sovrn.local`), the service worker default scope `/` covers all fetches. No scope issues.

### D-N5: Service Worker Scoping Solution

**Problem:** If the PWA is accessible at two different origins, service workers register independently. Data in IndexedDB, Cache API, and SW registrations are origin-scoped. Two origins = two copies of everything, and they diverge silently.

**Solution: Single origin, single service worker.**

```
https://sovrn.local/                    ← PWA served here
https://sovrn.local/api/*               ← API served here
https://sovrn.local/sw.js               ← Service worker registered here
https://sovrn.local/api/notify          ← Notify bridge here
```

The service worker at `https://sovrn.local/sw.js`:
- Configured with `scope: '/'`
- Intercepts ALL fetches to `/` and `/api/*`
- Cache-first strategy for PWA static assets (offline support)
- Network-first strategy for API calls (fresh data when online)
- Background sync for offline message queuing
- Push notification support via Notify bridge

**What changes from prior docs:**

| Prior | New |
|---|---|
| PWA at `localhost.sovrn:54772` + `sovrn.local:443` | PWA at `sovrn.local:443` ONLY |
| `localhost.sovrn:54772` as user-facing URL | `localhost.sovrn:54772` = development ONLY, never shown to users |
| Preact dev server in production | Preact builds static assets for production; dev server = dev only |
| API on port 54772 alongside PWA | API via Caddy reverse proxy at `sovrn.local/api/` (same origin as PWA) |

### D-N6: How the PWA Communicates with Mesh Services AND Local App Management

```
┌──────────────────────────────────────────────────────────────────────┐
│  PWA (https://sovrn.local/)                                         │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │ Preact UI Components                                           │  │
│  │ Feed | Messages | Homepage | Settings | App Center             │  │
│  └────────────────────────────┬───────────────────────────────────┘  │
│                               │ same-origin fetch()                  │
│  ┌────────────────────────────▼───────────────────────────────────┐  │
│  │ Service Worker (/sw.js)                                        │  │
│  │ - Cache PWA assets (offline support)                            │  │
│  │ - Queue offline mutations                                       │  │
│  │ - Push notification handler                                    │  │
│  └────────────────────────────┬───────────────────────────────────┘  │
│                               │                                      │
│  ┌────────────────────────────▼───────────────────────────────────┐  │
│  │ CADDY (sovrn.local:443)                                        │  │
│  │                                                                │  │
│  │ /              → static PWA files                               │  │
│  │ /api/v1/feed   → sovrnd (localhost:54771)                      │  │
│  │ /api/v1/msg    → sovrnd (localhost:54771)                      │  │
│  │ /api/v1/mesh   → sovrnd (localhost:54771)                      │  │
│  │ /api/v1/apps   → sovrnd (localhost:54771)  ← app management    │  │
│  │ /api/v1/ident  → sovrnd (localhost:54771)                      │  │
│  │ /api/notify    → notify bridge (localhost:54773)               │  │
│  │ /ws            → sovrnd WebSocket (real-time updates)          │  │
│  └────────────────────────────────────────────────────────────────┘  │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────────┐  │
│  │ SOVRN DAEMON (sovrnd)                                          │  │
│  │                                                                │  │
│  │ Communicates with:                                              │  │
│  │ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐         │  │
│  │ │ Yggdrasil│ │ DHT DNS  │ │ Identity │ │ Presence │         │  │
│  │ └──────────┘ └──────────┘ └──────────┘ └──────────┘         │  │
│  │ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐         │  │
│  │ │ Feed/Soc │ │ Messages │ │ CDN Agent│ │App Mgmt  │         │  │
│  │ └──────────┘ └──────────┘ └──────────┘ └──────────┘         │  │
│  │                                                                │  │
│  │ App Management:                                                │  │
│  │ - Install/uninstall containers via Podman SDK                  │  │
│  │ - Add/remove Caddy routes via Caddy API                        │  │
│  │ - Auto-provision app accounts                                  │  │
│  │ - Health monitoring + restart                                  │  │
│  └────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────┘
```

**Key principle: The PWA NEVER makes direct requests to mesh nodes.** All network traffic goes through the local daemon, which handles:

1. **Mesh communication** — The PWA calls `/api/v1/mesh/*` endpoints. `sovrnd` handles Yggdrasil connectivity, DHT lookups, presence tracking, and message routing.

2. **Local app management** — The PWA calls `/api/v1/apps/*` endpoints. `sovrnd` handles Podman container lifecycle, Caddy route updates, and health checks.

3. **Same origin** — Both mesh services and local management are under `https://sovrn.local/api/`. Zero CORS issues.

4. **Offline support** — When the mesh is unreachable, the PWA still works:
   - Local features (settings, app management, cached feed) remain functional
   - Service worker serves cached PWA assets
   - Queued mutations (new posts, messages) sync when connectivity returns

## TLD Replacement Summary

The following replacements were made across all project documents:

| Old | New | Context |
|---|---|---|
| `.os` (as mesh TLD) | `.sovrn` | e.g., `name.os` → `name.sovrn` |
| `home.os` | `home.sovrn` | PWA URL in dev context |
| `localhost.sovrn:54772` (as primary PWA URL) | `sovrn.local` | Production PWA is always through Caddy |
| `the OS has a mesh browser (for .os content)` | `the browser auto-routes .sovrn to mesh` | Descriptive text |

Files modified:
- `01-ARCHITECTURE.md` — `.os` → `.sovrn` in D10
- `03-NETWORK.md` — `.os` → `.sovrn` in DHT/DNS references
- `05-UX-FLOW.md` — `.os` → `.sovrn` in domain claim UI
- `07-OPEN-QUESTIONS.md` — `.os` references → `.sovrn`
- `09-ROUND3-QUESTIONS.md` — `.os` references → `.sovrn`, PWA URL clarified
- `16-ROUND8-DECISIONS.md` — PWA URL clarified to `sovrn.local`
- `19-SELF-HOSTED-APPS.md` — Updated Caddy/origin architecture

## Port Allocation (Updated)

| Port | Service | Notes |
|---|---|---|
| 80 | Caddy HTTP → redirect to HTTPS | User-facing on `sovrn.local` |
| 443 | Caddy HTTPS | **Primary entry point for PWA + API** |
| 54771 | sovrnd API (internal) | Caddy proxies `/api/*` here. Loopback only. |
| 54772 | Preact dev server | **Development only.** Never runs in production. |
| 54773 | Notify bridge | Loopback only. Caddy proxies `/api/notify` here. |
| 53535 | Mesh DNS resolver | Loopback only. Queries DHT for `.sovrn` domains. |

## SSL/TLS for sovrn.local

Since `sovrn.local` is a local domain (no public CA will issue a cert for it):

1. **On install**, Sovrn generates a self-signed root CA (`/var/lib/sovrn/ca/root-ca.pem`).
2. **Caddy** uses this CA to issue a certificate for `sovrn.local` and `*.sovrn.local`.
3. **The root CA** is added to:
   - System trust store (`update-ca-certificates`)
   - Firefox cert store (via `certutil` or auto-config)
   - Chromium cert store (uses system store on Linux)
4. **Result:** All browsers on Sovrn OS trust `https://sovrn.local/` without warnings.
5. **Mesh domains** (`*.sovrn`) use Yggdrasil's built-in TLS or HTTP (mesh is already encrypted at the transport layer — Yggdrasil provides E2E encryption).

This eliminates the "self-signed cert warning" problem that would otherwise block PWA install prompts and service workers.

## Migration Checklist

- [x] Update `01-ARCHITECTURE.md` D10: replace `.os` with `.sovrn`
- [x] Update `03-NETWORK.md`: replace `.os` references with `.sovrn`
- [x] Update `05-UX-FLOW.md`: change domain UI from `.os` to `.sovrn`
- [x] Update `07-OPEN-QUESTIONS.md`: replace `.os` references with `.sovrn`
- [x] Update `09-ROUND3-QUESTIONS.md`: replace `.os` references, clarify PWA URL
- [x] Update `16-ROUND8-DECISIONS.md`: change PWA URL from `localhost.sovrn:54772` to `sovrn.local`
- [x] Update `19-SELF-HOSTED-APPS.md`: update Caddy architecture, origin design, port table
- [ ] Create `sovrnd` API design: endpoints under `/api/v1/*` on same origin
- [ ] Create Caddy config template for `sovrn.local` + `*.sovrn.local`
- [ ] Implement DNS dispatcher in systemd-resolved config
- [ ] Generate self-signed CA on install, add to system + browser trust stores