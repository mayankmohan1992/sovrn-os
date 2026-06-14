# Sovrn OS — Complete Monorepo Directory Layout

Every file path in the Sovrn OS monorepo with a description of its purpose.

```
sovrn-os/
├── Cargo.toml                          # Rust workspace root — defines all crate members
├── cargo-config.toml                   # Shared Cargo config (target dir, registry, build profiles)
├── justfile                            # Task runner recipes (build-all, test, dev-setup, etc.)
├── dev-setup.sh                        # Developer onboarding script (installs deps, seeds mock data)
├── README.md                           # Project overview, quickstart, architecture diagram
├── LICENSE                              # AGPL-3.0 (or chosen license)
├── .github/
│   └── workflows/
│       ├── ci.yml                      # Lint + unit tests on PR
│       ├── build-iso.yml               # Build bootable ISO via Debos on push to main
│       └── release.yml                 # Tagged release: build ISO + publish checksums
│
├── src/
│   ├── sovrnd/                         # Python daemon — orchestration, REST API, app lifecycle
│   │   ├── pyproject.toml              # PEP 621 package metadata, deps, entry point sovrnd
│   │   ├── sovrnd/
│   │   │   ├── __init__.py
│   │   │   ├── __main__.py             # Entry point: python -m sovrnd
│   │   │   ├── api/                    # FastAPI REST API
│   │   │   │   ├── __init__.py
│   │   │   │   ├── app.py             # FastAPI app factory, CORS, middleware
│   │   │   │   ├── routes/
│   │   │   │   │   ├── __init__.py
│   │   │   │   │   ├── feed.py        # GET/POST feed events
│   │   │   │   │   ├── messages.py    # DM send/receive/mark-read
│   │   │   │   │   ├── identity.py    # Profile, aliases, QR code
│   │   │   │   │   ├── sync.py        # Pull/push sync endpoints
│   │   │   │   │   ├── apps.py        # App install/uninstall/start/stop/health
│   │   │   │   │   ├── settings.py    # User settings, mesh config
│   │   │   │   │   ├── cdn.py         # CDN push/pull/status
│   │   │   │   │   └── presence.py    # Online status, peer list
│   │   │   │   ├── middleware/
│   │   │   │   │   ├── __init__.py
│   │   │   │   │   ├── auth.py        # JWT validation middleware
│   │   │   │   │   └── rate_limit.py  # 100 req/min per JWT
│   │   │   │   └── ws/                # WebSocket handlers
│   │   │   │       ├── __init__.py
│   │   │   │       └── events.py      # Real-time event stream
│   │   │   ├── core/
│   │   │   │   ├── __init__.py
│   │   │   │   ├── config.py          # Load TOML config, defaults, validation
│   │   │   │   ├── db.py              # SQLite connection pool, migrations
│   │   │   │   ├── identity.py        # Ed25519 key management, seed phrase, HKDF
│   │   │   │   └── jwt.py             # JWT issue/verify/refresh
│   │   │   ├── services/              # IPC clients for mesh services
│   │   │   │   ├── __init__.py
│   │   │   │   ├── dht_client.py     # Unix socket client for sovrn-dht
│   │   │   │   ├── identity_client.py # Client for sovrn-identity
│   │   │   │   ├── presence_client.py # Client for sovrn-presence
│   │   │   │   ├── feed_client.py     # Client for sovrn-feed
│   │   │   │   ├── mq_client.py      # Client for sovrn-message-queue
│   │   │   │   └── cdn_client.py      # Client for sovrn-cdn-agent
│   │   │   ├── app_manager/           # Podman + quadlet management
│   │   │   │   ├── __init__.py
│   │   │   │   ├── lifecycle.py       # Install/start/stop/update/remove apps
│   │   │   │   ├── provision.py       # Auto-provision accounts in containers
│   │   │   │   ├── caddy_routes.py    # Dynamic Caddy route management via API
│   │   │   │   └── health.py          # Container health monitoring
│   │   │   └── migrations/            # SQLite migration scripts
│   │   │       ├── __init__.py
│   │   │       └── v001_initial.py    # Create initial tables
│   │   ├── tests/
│   │   │   ├── __init__.py
│   │   │   ├── conftest.py            # Fixtures: mock Podman, mock services
│   │   │   ├── test_api.py
│   │   │   ├── test_identity.py
│   │   │   └── test_app_manager.py
│   │   └── config/
│   │       └── sovrnd.toml            # Default config file (installed to /etc/sovrn/)
│   │
│   ├── sovrn-dht/                     # Rust — Distributed Hash Table for .sovrn DNS
│   │   ├── Cargo.toml                 # Crate manifest
│   │   ├── src/
│   │   │   ├── lib.rs                 # Public API: lookup, register, republish
│   │   │   ├── main.rs                # Binary entry, systemd socket activation
│   │   │   ├── node.rs                # DHT node: join, routing, bucket management
│   │   │   ├── protocol.rs            # Wire protocol: message types, serialization
│   │   │   ├── routing.rs             # Kademlia routing table (k-buckets)
│   │   │   ├── store.rs               # Local key-value store (domain→Yggdrasil addr)
│   │   │   ├── rpc.rs                 # Unix socket IPC server for sovrnd
│   │   │   └── ygg.rs                 # Yggdrasil integration: listen on Ygg addr
│   │   ├── tests/
│   │   │   └── integration_test.rs
│   │   └── config/
│   │       └── dht.toml               # Default: bootstrap nodes, k-bucket size, TTL
│   │
│   ├── sovrn-identity/                # Rust — Key management, seed phrases, signing
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs                 # Public API: generate, sign, verify, derive
│   │   │   ├── main.rs                # Binary entry
│   │   │   ├── keys.rs               # Ed25519 + X25519 keypair generation
│   │   │   ├── seed.rs               # BIP-39 seed phrase generation + recovery
│   │   │   ├── derive.rs             # HKDF derivation: app passwords, sub-keys
│   │   │   ├── sign.rs               # Event signing (Nostr-compatible ed25519)
│   │   │   ├── did.rs                # DID generation: did:mesh:{pubkey_hash}
│   │   │   ├── store.rs              # Encrypted key storage via OS keyring
│   │   │   └── rpc.rs                # Unix socket IPC server
│   │   ├── tests/
│   │   │   └── integration_test.rs
│   │   └── config/
│   │       └── identity.toml
│   │
│   ├── sovrn-presence/                # Rust — Heartbeat & peer tracking
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs                 # Public API: announce, heartbeat, list peers
│   │   │   ├── main.rs                # Binary entry
│   │   │   ├── heartbeat.rs           # 2-min heartbeat + explicit offline signal
│   │   │   ├── gossip.rs              # Gossip protocol for presence propagation
│   │   │   ├── peer_store.rs          # In-memory + SQLite peer tracking
│   │   │   ├── ygg.rs                 # Yggdrasil multicast for peer discovery
│   │   │   └── rpc.rs                 # Unix socket IPC server
│   │   ├── tests/
│   │   │   └── integration_test.rs
│   │   └── config/
│   │       └── presence.toml
│   │
│   ├── sovrn-feed/                    # Rust — Social feed event processing
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs                 # Public API: post, fetch, follow, react
│   │   │   ├── main.rs                # Binary entry
│   │   │   ├── event.rs               # Event types (kind 0-300) and serialization
│   │   │   ├── validation.rs          # Signature verification, content validation
│   │   │   ├── storage.rs             # SQLite event storage (append-only log)
│   │   │   ├── timeline.rs            # Feed timeline construction (follows, filters)
│   │   │   ├── push.rs               # Node-to-node event push (gossip)
│   │   │   └── rpc.rs                 # Unix socket IPC server
│   │   ├── tests/
│   │   │   └── integration_test.rs
│   │   └── config/
│   │       └── feed.toml
│   │
│   ├── sovrn-message-queue/           # Rust — Encrypted DM delivery queue
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs                 # Public API: send, receive, mark_read
│   │   │   ├── main.rs                # Binary entry
│   │   │   ├── queue.rs              # Persistent message queue (SQLite)
│   │   │   ├── encryption.rs          # X25519 + sealed box encryption
│   │   │   ├── delivery.rs            # Node-to-node message delivery
│   │   │   ├── conversation.rs        # Conversation threading
│   │   │   └── rpc.rs                 # Unix socket IPC server
│   │   ├── tests/
│   │   │   └── integration_test.rs
│   │   └── config/
│   │       └── message-queue.toml
│   │
│   ├── sovrn-cdn-agent/               # Go — CDN push/pull, bandwidth management
│   │   ├── go.mod                     # Go module definition
│   │   ├── go.sum
│   │   ├── cmd/
│   │   │   └── sovrn-cdn-agent/
│   │   │       └── main.go           # Binary entry
│   │   ├── internal/
│   │   │   ├── api/                   # HTTP API for sovrnd IPC
│   │   │   │   └── handler.go
│   │   │   ├── cdn/                   # CDN provider abstraction
│   │   │   │   ├── interface.go       # Provider interface
│   │   │   │   └── providers.go       # Registry of providers
│   │   │   ├── push/                  # Content push to CDN
│   │   │   │   └── push.go
│   │   │   ├── cache/                 # Local cache management
│   │   │   │   └── cache.go
│   │   │   └── config/
│   │   │       └── config.go          # Configuration loading
│   │   ├── configs/
│   │   │   └── cdn-agent.toml
│   │   └── Makefile                   # Build + install target
│   │
│   ├── sovrn-auth/                    # Python — Caddy auth middleware (JWT validation)
│   │   ├── pyproject.toml
│   │   ├── sovrn_auth/
│   │   │   ├── __init__.py
│   │   │   ├── middleware.py          # Caddy JSON config + auth handler
│   │   │   ├── jwt.py                 # JWT verification (shared with sovrnd)
│   │   │   └── config.py
│   │   └── tests/
│   │       └── test_jwt.py
│   │
│   ├── sovrn-app-monitor/             # Python — Container health monitor
│   │   ├── pyproject.toml
│   │   ├── sovrn_app_monitor/
│   │   │   ├── __init__.py
│   │   │   ├── monitor.py             # Watch Podman containers, restart on crash
│   │   │   ├── notify.py              # Notify bridge integration
│   │   │   └── config.py
│   │   └── tests/
│   │       └── test_monitor.py
│   │
│   ├── sovrn-notify-bridge/           # Python — HTTP→notify-send bridge (50 LOC)
│   │   ├── pyproject.toml
│   │   ├── sovrn_notify_bridge/
│   │   │   ├── __init__.py
│   │   │   └── bridge.py              # HTTP server on 127.0.0.1:54773
│   │   └── tests/
│   │       └── test_bridge.py
│   │
│   ├── pwa/                           # Preact PWA — Sovrn Hub
│   │   ├── package.json               # Dependencies: preact, preact-router, signals, vitest
│   │   ├── pnpm-lock.yaml
│   │   ├── tsconfig.json              # TypeScript config
│   │   ├── vite.config.ts             # Vite build config (PWA plugin, service worker)
│   │   ├── index.html                 # SPA entry point
│   │   ├── public/
│   │   │   ├── manifest.json          # PWA manifest (name, icons, theme)
│   │   │   ├── sw.ts                  # Service worker (Workbox)
│   │   │   ├── icons/
│   │   │   │   ├── icon-192.png
│   │   │   │   └── icon-512.png
│   │   │   └── favicon.svg
│   │   ├── src/
│   │   │   ├── app.tsx                # Root component + router
│   │   │   ├── app.scss               # Global styles, Sovrn design system tokens
│   │   │   ├── components/
│   │   │   │   ├── Feed.tsx           # Social feed (kind 1/2 events)
│   │   │   │   ├── Messages.tsx       # DM view (kind 7 events)
│   │   │   │   ├── Homepage.tsx       # Homepage editor
│   │   │   │   ├── Identity.tsx       # Profile, QR, aliases
│   │   │   │   ├── AppCenter.tsx      # Install/manage self-hosted apps
│   │   │   │   ├── Settings.tsx       # Mesh config, backup, display prefs
│   │   │   │   ├── SyncStatus.tsx     # Online/Offline/Syncing indicator
│   │   │   │   └── Notification.tsx   # Post/notify bridge integration
│   │   │   ├── state/
│   │   │   │   ├── user.ts            # preact/signals: identity, auth state
│   │   │   │   ├── feed.ts            # preact/signals: posts, follows
│   │   │   │   ├── messages.ts        # preact/signals: conversations
│   │   │   │   └── apps.ts            # preact/signals: installed apps, health
│   │   │   ├── db/
│   │   │   │   └── dexie.ts           # IndexedDB via Dexie.js — offline store
│   │   │   ├── utils/
│   │   │   │   ├── sync.ts            # Sync queue, conflict resolution
│   │   │   │   ├── api.ts             # Fetch wrapper for sovrnd API
│   │   │   │   └── crypto.ts          # Client-side encryption helpers
│   │   │   └── types/
│   │   │       └── events.ts          # TypeScript types for feed events
│   │   └── tests/
│   │       ├── feed.test.ts
│   │       └── sync.test.ts
│   │
│   └── oobe/                          # GTK4 OOBE wizard (PyGObject)
│       ├── pyproject.toml
│       ├── src/
│       │   ├── __init__.py
│       │   ├── main.py                # Application entry (GTK.Application)
│       │   ├── window.py              # MainWindow with stack/pages
│       │   ├── pages/
│       │   │   ├── welcome.py         # Language select, welcome screen
│       │   │   ├── keyboard.py        # Keyboard layout selection
│       │   │   ├── network.py         # WiFi/Ethernet connect
│       │   │   ├── identity.py        # Name, seed phrase, verify
│       │   │   ├── mesh.py            # Yggdrasil status, mesh setup
│       │   │   └── done.py            # "You're all set!" with autostart
│       │   ├── backend/
│       │   │   ├── identity.py        # Key generation, seed phrase IO
│       │   │   ├── network.py         # Yggdrasil config generation
│       │   │   └── system.py          # systemd unit enable, user creation
│       │   └── resources/
│       │       ├── style.css          # Sovrn theme overrides for GTK
│       │       └── oobe.gresource.xml # GTK resource bundle
│       ├── data/
│       │   └── org.sovrn.oobe.metainfo.xml
│       └── tests/
│           └── test_identity.py
│
├── src/calamares-config/              # Calamares installer configuration
│   ├── settings.conf                  # Main Calamares settings
│   ├── modules/
│   │   ├── welcome.conf              # Welcome page config
│   │   ├── keyboard.conf             # Keyboard layout
│   │   ├── partition.conf            # Partition layout (LUKS2 + LVM)
│   │   ├── users.conf                # User creation (Sovrn user)
│   │   ├── luks.conf                 # LUKS2 encryption settings
│   │   └── contextualprocess.conf    # Post-install: enable sovrnd, OOBE
│   └── branding/
│       ├── sovrn/                    # Sovrn branding directory
│       │   ├── branding.desc         # Branding descriptor
│       │   ├── slideshow.html        # Slideshow during install
│       │   └── logo.png              # Sovrn logo
│       └── lang/                     # Translations
│
├── src/debos/                         # Debos recipes for ISO generation
│   ├── sovrn-os.yaml                  # Main Debos recipe
│   ├── overlays/
│   │   ├── rootfs/                    # Files copied to root filesystem
│   │   │   ├── etc/
│   │   │   │   ├── sovrn/
│   │   │   │   │   └── sovrnd.toml   # Default sovrnd config
│   │   │   │   ├── caddy/
│   │   │   │   │   └── Caddyfile     # Production Caddy config
│   │   │   │   ├── yggdrasil/
│   │   │   │   │   └── sovrn.conf    # Yggdrasil config (Sovrn peers)
│   │   │   │   ├── nftables/
│   │   │   │   │   └── sovrn.conf    # nftables ruleset
│   │   │   │   ├── dbus-1/
│   │   │   │   │   └── system.d/
│   │   │   │   │       └── org.sovrn.conf  # D-Bus policy
│   │   │   │   └── NetworkManager/
│   │   │   │       └── conf.d/
│   │   │   │           └── sovrn.conf  # NM config for Yggdrasil
│   │   │   └── usr/
│   │   │       └── local/
│   │   │           └── bin/
│   │   │               └── sovrn-first-boot.sh  # First-boot setup script
│   │   └── live/                      # Live system overlays
│   │       └── etc/
│   │           └── calamares/
│   │               └── settings.conf  # Live mode Calamares settings
│   └── scripts/
│       ├── install-packages.sh        # apt install + remove lists
│       ├── configure-system.sh        # System tweaks, sysctl, limits
│       └── build-iso.sh               # Wrapper script for debos
│
├── src/systemd-units/                 # All systemd unit files
│   ├── sovrnd.service                 # Main daemon
│   ├── sovrn-dht.service              # DHT resolver
│   ├── sovrn-identity.service         # Key management
│   ├── sovrn-presence.service         # Heartbeat / peer tracking
│   ├── sovrn-feed.service             # Social feed
│   ├── sovrn-message-queue.service    # DM queue
│   ├── sovrn-cdn-agent.service        # CDN agent
│   ├── sovrn-app-monitor.service      # Container health monitor
│   ├── sovrn-notify-bridge.service   # HTTP→notify-send bridge
│   ├── caddy.service                  # Caddy with Sovrn config
│   ├── yggdrasil.service              # Mesh VPN (with Sovrn config path)
│   ├── zram-setup.service             # Compressed swap setup
│   ├── sovrn-ca-bootstrap.service    # First-boot CA generation
│   ├── sovrn-first-boot.service       # GNOME customization, dock setup
│   └── sovrn-ca-bootstrap.timer       # Timer for CA cert rotation
│
├── src/caddy/                         # Caddy reverse proxy config
│   ├── Caddyfile                     # Production config: sovrn.local + *.sovrn.local
│   ├── Caddyfile.dev                 # Dev mode: proxy to Preact dev server
│   └── sovrn-auth-plugin/             # Caddy auth middleware module
│       ├── go.mod
│       │── main.go                    # Caddy module: JWT validate + inject headers
│       └── caddy.json                 # JSON config template for dynamic routes
│
├── src/dns-dispatcher/                # DNS resolver dispatch (mesh vs internet)
│   ├── unbound.conf                  # Unbound resolver config
│   └── resolv.conf                   # /etc/resolv.conf pointing to local resolver
│
├── src/nftables/                      # Firewall rules
│   └── sovrn.conf                     # nftables ruleset (mesh isolate, app isolate)
│
├── src/apparmor/                      # AppArmor confinement profiles
│   ├── usr.bin.sovrnd                 # Profile for sovrnd
│   ├── usr.bin.sovrn-dht             # Profile for DHT service
│   ├── usr.bin.sovrn-identity        # Profile for identity service
│   ├── usr.bin.sovrn-presence        # Profile for presence service
│   ├── usr.bin.sovrn-feed            # Profile for feed service
│   ├── usr.bin.sovrn-message-queue   # Profile for message queue
│   ├── usr.bin.sovrn-cdn-agent       # Profile for CDN agent
│   ├── usr.bin.caddy                 # Profile for Caddy
│   └── usr.bin.yggdrasil             # Profile for Yggdrasil
│
├── src/gnome-customization/           # GNOME desktop customization
│   ├── configure-desktop.py           # Python: dock, wallpaper, default apps
│   ├── sovrn-hub.desktop             # .desktop for Sovrn Hub web app
│   ├── sovrn-app-template.desktop    # Template .desktop for self-hosted apps
│   ├── sovrn-hub-autostart.desktop   # Auto-start Sovrn Hub on login
│   ├── firefox-policies.json          # Homepage, new tab, extensions policies
│   ├── oobe-launcher.sh               # GTK4 OOBE launcher script
│   ├── gvfs-mounthelper.override      # Hide unnecessary mount entries
│   ├── sovrn-extension/               # GNOME Shell extension for mesh status
│   │   ├── metadata.json
│   │   ├── extension.js
│   │   ├── prefs.js
│   │   └── stylesheet.css
│   └── wallpaper/
│       └── sovrn-teal.png             # Sovrn branded wallpaper
│
├── src/ca-bootstrap/                  # Self-signed CA for first boot
│   ├── generate-ca.sh                 # Generate root CA + app certs
│   ├── install-ca.sh                  # Install CA into system trust stores
│   └── sovrn-ca-bootstrap.py          # Python: CA generation + cert rotation
│
├── src/scripts/                       # Shared utility scripts
│   ├── dev-setup.sh                   # Developer onboarding: install deps, configure
│   ├── build-debs.sh                  # Build all .deb packages
│   ├── run-local.sh                   # Start all services locally for development
│   ├── mock-yggdrasil.sh              # Configure Yggdrasil for local dev network
│   ├── seed-data.sh                   # Seed mock data for development
│   └── check-health.sh               # Verify all services are running
│
├── packaging/                         # Debian package definitions
│   ├── sovrnd/
│   │   ├── debian/
│   │   │   ├── control                # Package metadata, deps
│   │   │   ├── rules                  # Build rules
│   │   │   ├── install                # File install paths
│   │   │   ├── postinst               # Post-install: enable service, create user
│   │   │   └── postrm                 # Post-remove: cleanup
│   │   └── Makefile
│   ├── sovrn-dht/
│   │   └── debian/
│   │       ├── control
│   │       ├── rules
│   │       └── install
│   ├── sovrn-identity/
│   │   └── debian/
│   │       ├── control
│   │       ├── rules
│   │       └── install
│   ├── sovrn-presence/
│   │   └── debian/
│   │       ├── control
│   │       ├── rules
│   │       └── install
│   ├── sovrn-feed/
│   │   └── debian/
│   │       ├── control
│   │       ├── rules
│   │       └── install
│   ├── sovrn-message-queue/
│   │   └── debian/
│   │       ├── control
│   │       ├── rules
│   │       └── install
│   ├── sovrn-cdn-agent/
│   │   └── debian/
│   │       ├── control
│   │       ├── rules
│   │       └── install
│   ├── sovrn-pwa/                     # PWA static build → /var/lib/sovrn/pwa/
│   │   └── debian/
│   │       ├── control
│   │       ├── rules
│   │       └── install
│   ├── sovrn-oobe/                    # OOBE GTK4 app
│   │   └── debian/
│   │       ├── control
│   │       ├── rules
│   │       └── install
│   └── sovrn-os-meta/                 # Meta package depending on all above
│       └── debian/
│           ├── control                # Depends: sovrnd, sovrn-dht, ... sovrn-pwa
│           └── rules
│
├── tests/
│   ├── integration/                   # Integration tests (QEMU + cloud-init)
│   │   ├── test-boot.py               # Boot ISO, check services
│   │   ├── test-oobe.py               # Run OOBE, verify identity created
│   │   ├── test-mesh.py               # Simulate mesh network
│   │   └── cloud-init/
│   │       └── seed.yaml              # cloud-init config for test VMs
│   └── e2e/
│       └── test-pwa.js                # Playwright E2E tests for PWA
│
└── docs/
    ├── architecture.md                 # High-level architecture diagram
    ├── api-spec.md                     # REST API specification (all endpoints)
    ├── database-schemas.md             # SQLite CREATE TABLE statements
    ├── ipc-protocols.md                # Service IPC protocol docs
    ├── node-protocols.md               # DHT, feed push, DM delivery wire formats
    └── dev-quickstart.md               # How to set up, build, and run locally
```