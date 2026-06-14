# Sovrn OS — BUILD READINESS ASSESSMENT

**Date:** 2026-06-04
**Scope:** All 25 design documents (00-22, FOLLOW-PERFORMANCE, SESSION-STATE)
**Purpose:** Identify every implementation detail missing that would prevent a developer from compiling and running the OS.

---

## EXECUTIVE SUMMARY

The design documents are comprehensive at the architectural and UX level. Decisions are finalized through Round 10. However, the project is **NOT build-ready**. Critical implementation details are missing across every category — language/runtime choices for half the components, no database schemas, no API endpoint specifications beyond a few sync/push routes, no inter-process communication protocols between services, no build system, no dev environment setup, and no testing strategy. Below is a structured gap analysis for each document.

---

## CATEGORY 1: PROGRAMMING LANGUAGE / RUNTIME

### What's Decided
| Component | Language | Source Doc |
|---|---|---|
| PWA (Feed, Messages, Homepage, Settings) | Preact + preact/signals | 18-K2 |
| sovrnd (orchestration daemon) | Python | 19 (sovrnd section) |
| OOBE wizard | GTK4 (custom) | 02-COMPONENTS |
| Notify bridge | Python (50 LOC) | 18-K1 |
| sovrn-app-monitor | Python (inferred from D-U5 pseudocode) | 22-D-U5 |

### What's MISSING (Gap List)
| Component | Missing Decision | Impact |
|---|---|---|
| sovrn-dht (DHT DNS resolver) | Language not specified | Must pick Rust/Go/Python — performance-critical |
| sovrn-identity (key management service) | Language not specified | Must pick — security-critical |
| sovrn-presence (heartbeat/gossip) | Language not specified | Must pick — network-critical |
| sovrn-feed (social feed service) | Language not specified | 02 says "Rust" for services but sovrnd is Python — unclear if this applies |
| sovrn-message-queue | Language not specified | Must pick |
| sovrn-cdn-agent | 02 says "Rust/Go daemon" but no final choice | Must pick one |
| Caddy configuration generator | Language not specified (sovrnd module? standalone?) | Implementation detail |
| sovrn-auth (JWT auth proxy) | Language not specified | Must pick |
| PhotoDNA/NSFW scanner module | Language not specified | Must pick |
| Calamares/OOBE installer hooks | Language not specified (Python? C++? Vala?) | Must match Calamares framework |
| DHT overlay protocol | Language not specified | Must pick — performance-critical |
| PWA Service Worker | JavaScript (implied) but no framework choice for offline sync | Workbox config details needed |

**Verdict:** ~10 components have no language specified. Rust was recommended for "service development" in doc 02 but sovrnd is Python. This inconsistency must be resolved.

---

## CATEGORY 2: CONFIGURATION FILE SCHEMAS AND PATHS

### What's Decided
- `/var/lib/sovrn/` — service data directory (16-J1, 19 directory structure)
- `/home/user/.sovrn/` — user config/data (16-J1)
- Caddy serves PWA from `/var/lib/sovrn/pwa/` (20-D-N2)
- Self-signed CA at `/var/lib/sovrn/ca/root-ca.pem` (20 SSL section)
- OCI image cache at `/var/lib/sovrn/images/` (22-D-U4)
- App manifests (`sovrn-app.yml`) schema defined (19 manifest section)
- systemd unit dependency order defined (22-D-U1)

### What's MISSING
| Gap | Detail | Impact |
|---|---|---|
| sovrnd config file | Path, format (TOML? YAML?), schema completely undefined | Daemon can't start without config |
| Yggdrasil config | How Sovrn configures Yggdrasil (peers, allowed keys, etc.) | No mesh connectivity |
| Caddy Caddyfile | Full template for `sovrn.local` + `*.sovrn.local` + mesh routing listed in D-N4 migration checklist as TODO | Caddy can't route |
| systemd-resolved config | DNS dispatcher for 3 namespaces listed as TODO in doc 20 | DNS doesn't work |
| nftables rules | Firewall rules for mesh vs internet traffic completely absent | No network isolation |
| AppArmor profiles | Mentioned in 02 and 03 but zero profile definitions | KSPP hardening not specified |
| Podman network config | `10.47.0.0/16` mentioned but no podman network create command or config | Containers can't communicate |
| LUKS2 config | Encryption parameters (cipher, key size, PBKDF) not specified | Can't build installer |
| KSPP kernel config | Mentioned but no actual config file | Can't build hardened kernel |
| zram config | "50% of RAM" stated but no zramtab/config generation details | No zram on boot |
| systemd-oomd config | No threshold values defined (medium/critical percentages) | No OOM protection |
| GNOME customization | "libadwaita recoloring + CSS overrides" mentioned but no actual CSS/override files | Can't build desktop |
| self-signed CA generation | Process for generating root CA + per-app certs on install not scripted | HTTPS broken |
| Seed node config | How to configure the 5 seed nodes (Yggdrasil peers, DHT bootstrap) undefined | No bootstrap |
| Unbound DNS config | Config for DoH/DNSSEC resolver not specified | Regular DNS not configured |

**Verdict:** 14+ configuration files need complete schemas. The most critical blockers are sovrnd config, Caddy Caddyfile, DNS dispatcher, and Yggdrasil config.

---

## CATEGORY 3: API REQUEST/RESPONSE JSON SCHEMAS

### What's Decided
- Sync API endpoints defined (D-S1): `GET /api/v1/sync/pull`, `POST /api/v1/sync/push`, `GET /api/v1/sync/status`
- Auth flow defined (D-S2): JWT structure, login flow, cookie settings
- Notify bridge: `POST localhost:54773/notify` (18-K1)
- Feed event schema defined (D-S4): complete JSON structure with kinds 0-300
- App manifest format defined (19 sovrn-app.yml)

### What's MISSING
| API Area | Missing Detail | Impact |
|---|---|---|
| Feed API | No endpoints for: GET feed, POST new event, DELETE event, GET single event, GET reactions | PWA can't display or create posts |
| Messages API | No endpoints for: send DM, get DMs, get conversation list, mark read | PWA can't send/receive messages |
| Identity API | No endpoints for: get profile, update profile, generate alias, list aliases, export identity | OOBE and Settings can't work |
| Mesh/DHT API | No endpoints for: register domain, lookup domain, check domain availability | Domain registration impossible |
| Presence API | No endpoints for: get online status, get peer list | Can't show online/offline |
| CDN API | No endpoints for: push content, check status, get CDN providers | CDN integration impossible |
| Homepage API | No endpoints for: publish homepage, get homepage, update homepage | Homepage editor can't work |
| Follow/Block API | No endpoints for: follow, unfollow, block, unblock, get followers, get following | Social features impossible |
| Search API | No endpoints for: search users, search content, discover | Discovery impossible |
| Settings API | No endpoints for: get/set settings, privacy controls, notification prefs | Settings page empty |
| App Management API | No endpoints for: list available apps, install, uninstall, start, stop, health check | App Center doesn't work |
| Backup API | No endpoint to trigger backup, check status, restore | Backup unusable |
| File upload API | No multipart upload spec for media attachments | Can't attach images/video |
| WebSocket API | `sovrnd WebSocket` mentioned at `/ws` but no message schema | Real-time updates don't work |
| Error response format | No standard error JSON envelope defined | Inconsistent error handling |
| Pagination format | Cursor format defined for sync but not for feed/messages/settings APIs | Infinite scroll broken |
| Rate limiting responses | Limits specified (100/min) but no 429 response body format | Clients can't handle rate limits |

**Verdict:** Only 3 endpoint groups have schemas. ~17 API groups have zero specifications. The PWA has no API contract to code against. This is the single largest gap.

---

## CATEGORY 4: DATA MODELS (DATABASE TABLES, SQLITE SCHEMAS)

### What's Decided
- Separate SQLite files per service: feed.db, messages.db, identity.db, presence.db, cdn.db, config.db (K3)
- IndexedDB schema (Dexie) for PWA offline storage (D-S1)
- Event schema (JSON) with kinds 0-300 (D-S4)
- File paths for all databases (16-J1)

### What's MISSING
| Database | Missing Schema | Impact |
|---|---|---|
| feed.db | No table definitions: events, follows, blocks, media_refs, reactions, cache_tiers | Can't store or query posts |
| messages.db | No table definitions: conversations, messages, message_queue, delivery_status | Can't store or retrieve DMs |
| identity.db | No table definitions: keys, aliases, delegation_records, domain_registrations | Identity system has no persistence |
| presence.db | No table definitions: peer_status, heartbeat_log, peer_discovery | Can't track who's online |
| cdn.db | No table definitions: push_queue, cdn_providers, content_map | CDN agent can't function |
| config.db (user) | No table definitions: settings, notification_prefs, privacy_settings | Settings can't persist |
| DHT node data | No data format for DHT records, no Kadmelia-like routing table schema | DHT can't store or resolve domains |
| Yggdrasil peer list | No schema for persisted peer configuration | Mesh connectivity not persisted |
| App registry | `/var/lib/sovrn/apps/` — "Registry of installed apps (JSON)" but no schema defined | Can't track installed apps |
| Backup metadata | No schema for Borg backup state tracking | Backup system has no state |

**Verdict:** Zero SQLite CREATE TABLE statements exist anywhere. Every service needs database schema before implementation can begin.

---

## CATEGORY 5: INTER-PROCESS COMMUNICATION PROTOCOLS

### What's Decided
- PWA ↔ sovrnd: REST API over HTTPS via Caddy reverse proxy (16-J2)
- PWA ↔ sovrnd: WebSocket at `/ws` for real-time events (20)
- sovrnd ↔ Caddy: Caddy API (JSON config push) for dynamic routes (19)
- sovrnd ↔ Podman: Podman Python SDK + systemd quadlet files (19)
- sovrnd ↔ systemd: sovrnd manages some services as systemd units (22-D-U1)
- Notify bridge: HTTP POST to localhost:54773 → notify-send (18-K1)

### What's MISSING
| IPC Gap | Detail | Impact |
|---|---|---|
| sovrnd ↔ sovrn-dht | How does sovrnd communicate with the DHT service? Unix socket? HTTP? gRPC? | Services can't talk to each other |
| sovrnd ↔ sovrn-identity | Same — no interface spec for identity key operations | Key operations undefined |
| sovrnd ↔ sovrn-presence | No protocol for presence queries or heartbeat forwarding | Presence system disconnected |
| sovrnd ↔ sovrn-feed | How does the PWA's feed request reach the feed service? Through sovrnd proxy? Direct? | Feed API broken |
| sovrnd ↔ sovrn-message-queue | Message delivery protocol between queue and sovrnd undefined | DM delivery impossible |
| sovrnd ↔ sovrn-cdn-agent | CDN push/pull protocol undefined | CDN integration impossible |
| sovrnd ↔ YYgdrasil | Yggdrasil has an admin API but no spec for how sovrnd configures it | Mesh connectivity not automated |
| Service health checks | sovrnd has `/health` endpoint mentioned but no health check response schema | Can't monitor service health |
| sovrn-auth ↔ Caddy | "Caddy + sovrn-auth validates JWT" but no Caddy plugin/auth middleware spec defined | Auth proxy doesn't work |
| DHT peer protocol | No wire protocol for how DHT nodes communicate (Kademlia? custom?) | DHT overlay can't be implemented |
| Feed push protocol | "Their node pushes notification to YOUR node" — but no protocol for node-to-node event push | Real-time feed updates impossible |
| DM delivery protocol | "Encrypted DM delivered to recipient's node" — but no delivery protocol spec | Encrypted messaging impossible |
| Domain registration protocol | "Sign message, broadcast to DHT" — but no message format, DHT update procedure | Can't register .sovrn domains |
| Alias delegation protocol | "Delegation record signed by master key" — but no wire format, no DHT storage scheme | Aliases don't work |

**Verdict:** sovrnd's internal API to the 6 mesh services has zero specification. The PWA→sovrnd REST API has endpoints but sovrnd→service is completely undefined. Similarly, node-to-node protocols (feed push, DM delivery, DHT) have zero wire format specifications.

---

## CATEGORY 6: BUILD SYSTEM AND PACKAGING

### What's Decided
- Base: Debian testing (02-COMPONENTS)
- OS builder: Debos + live-build (02)
- Service packaging: .deb files for apt (09-Round3 E10)
- Self-hosted apps: Podman containers (19-M2)
- PWA: Preact build → static assets → `/var/lib/sovrn/pwa/` (20)
- ISO: Bootable USB with live mode + install option (13-G1)

### What's MISSING
| Gap | Detail | Impact |
|---|---|---|
| Debos/live-build config | No Debian image recipes, no kickstart/preseed files | Can't build ISO |
| Package tree | No list of which Debian packages to include/exclude in the base image | Can't build minimal OS |
| .deb packaging | No debian/control, rules, or install files for any sovrn service | Can't install services via apt |
| sovrnd .deb structure | sovrnd is Python but no Python packaging specs (pip package? system python? venv?) | Can't install sovrnd |
| PWA build pipeline | No Vite/Preact build configuration, no service worker config | Can't build PWA |
| Calamares installer config | No Calamares modules, partition layout, LUKS setup scripts | Can't build installer |
| GNOME customization scripts | No scripts to remove apps, add dock panel, set wallpaper, create .desktop files | Can't customize desktop |
| systemd unit files | Dependency order defined (22) but no actual .service file contents written | Services can't start |
| Self-signed CA generation script | No script to generate root CA + app certs on first boot | HTTPS broken |
| zram-setup service | Mentioned as systemd service but no unit file or setup script | zram doesn't configure |
| nftables rules file | No nftables config file at all | Firewall undefined |
| AppArmor profiles | No profiles defined for any service | Mandatory hardening missing |
| Secure Boot enrollment | How the UEFI Secure Boot signing key is managed | Can't enable secure boot |
| dm-verity setup | How read-only system partitions are created and verified | Supply chain protection missing |
| ISO size budget | ~64 GB target mentioned but no breakdown of image partitions | Can't build installable ISO |
| CI/CD pipeline | "GitHub Actions or local build server" mentioned but no pipeline defined | No automated builds |
| Signing infrastructure | Dual GPG signing, air-gapped machine mentioned but no operational process | Can't sign packages |

**Verdict:** Zero build infrastructure exists. No Dockerfile, no Makefile, no CI pipeline, no Debian packaging files. A developer cannot produce a bootable ISO from these docs.

---

## CATEGORY 7: TESTING STRATEGY

### What's Decided
- QEMU + cloud-init for automated testing (02-COMPONENTS, development stack)
- Staged rollout for updates (08-UPDATE-SYSTEM)

### What's MISSING
| Gap | Detail | Impact |
|---|---|---|
| Unit testing framework | No framework chosen for any language (pytest? Rust's built-in? Go's testing?) | No unit tests |
| Integration testing plan | No plan for testing service interactions | Can't verify service mesh works |
| E2E testing plan | No plan for testing full OS image | Can't verify ISO boots correctly |
| PWA testing | No testing framework for Preact PWA (Jest? Vitest? Playwright?) | No frontend tests |
| API testing | No API contract testing tool (Pact? manual?) | No API verification |
| Security testing | No fuzz testing, penetration testing, or audit plan for mesh services | No security validation |
| DHT testing | No test harness for simulating mesh DHT with multiple nodes | Can't test mesh networking |
| Sovrnd testing | No mocking strategy for sovrnd's dependencies | Can't test orchestration |
| Performance targets | Only idle usage targets defined (12), no load testing benchmarks | Can't verify performance |
| Update testing | No plan for testing secure update pipeline end-to-end | Can't verify 6-layer security |
| OOBE testing | No automated testing for the Calamares/GTK4 setup wizard | Can't test first-boot experience |
| Accessibility testing | Deferred to v2 (13-H4) but no tracking issue | Will be forgotten |

**Verdict:** Zero test infrastructure or strategy defined. Only "QEMU + cloud-init" is mentioned as a tool with no concrete test plan.

---

## CATEGORY 8: DEVELOPMENT ENVIRONMENT SETUP

### What's Decided
- Preact dev server on port 54772 for development (20-D-N2)
- Caddy in dev mode can proxy to Preact dev server (20)

### What's MISSING
| Gap | Detail | Impact |
|---|---|---|
| Dev environment prerequisites | No list of tools to install (Rust? Go? Python? Node? Preact CLI?) | Developer can't set up workstation |
| Repository structure | No monorepo/polyrepo decision. No directory layout. | Where does code go? |
| Dev startup guide | No "how to run the project locally" instructions | Developer can't run anything |
| Dev networking | How to simulate Yggdrasil + DHT locally for development | Can't test mesh features |
| Dev data | No seed data, test fixtures, or mock services | Can't develop against real data |
| Dev container/VM setup | No Vagrant, Docker Compose, or devcontainer for local development | Developer must build full OS |
| Debugging tools | No guidance on debugging sovrnd, PWA, or mesh services | Can't diagnose issues |
| Hot reload | Only Preact has HMR specified — what about sovrnd, mesh services? | Slow development cycle |
| Logging standard | No log format, level conventions, or centralized logging defined | Can't debug production issues |
| Config management | How dev config differs from production config (seed node addresses, ports, etc.) | Dev-prod parity issues |

**Verdict:** A developer cloning a hypothetical repo would have zero guidance on how to build, run, or test any component.

---

## CONSOLIDATED CHECKLIST BY PRIORITY

### P0 — BLOCKERS (Can't start coding without these)
1. **sovrnd API specification** — Complete endpoint list for all 17+ API groups with request/response schemas
2. **SQLite database schemas** — CREATE TABLE statements for all 6 databases
3. **sovrnd → service IPC protocol** — How sovrnd communicates with dht/identity/presence/feed/message/cdn
4. **Node-to-node protocols** — Wire format for: DHT overlay, feed event push, DM delivery, heartbeat gossip, domain registration
5. **Build system** — Repository structure, Debian packaging files, Debos config, Makefile/justfile/taskfile
6. **Language decisions** — Pick languages for all 6 mesh services (dht, identity, presence, feed, message-queue, cdn-agent)
7. **sovrnd Python package structure** — How sovrnd is packaged and installed (system python? venv? pipx?)
8. **Caddy Caddyfile template** — Complete reverse proxy config for production

### P1 — CRITICAL (Can't build a working system without these)
9. **Yggdrasil configuration generation** — How sovrnd configures Yggdrasil peering, allowed keys
10. **DNS dispatcher config** — systemd-resolved + custom resolver config for 3 namespaces
11. **nftables ruleset** — Firewall rules for mesh/internet isolation
12. **AppArmor profiles** — For all services
13. **Calamares installer config** — Partition layout, LUKS, first-boot triggers
14. **systemd unit files** — Actual .service files for all 8+ services
15. **LUKS2 + dm-verity setup** — Encryption and verification scripts
16. **Debian package manifest** — Exact list of packages to install/exclude
17. **GNOME customization scripts** — Dock, wallpaper, default apps, removed apps
18. **Self-signed CA bootstrapping** — Script to generate CA + certs + install in trust stores
19. **sovrn-auth Caddy middleware specification** — JWT validation, fallback behavior

### P2 — IMPORTANT (Needed for a functional v1 but can be developed iteratively)
20. **PhotoDNA/NSFW integration design** — How the content scanner is packaged and called
21. **Borg backup configuration** — Backup scope, schedule, encryption key management
22. **Podman network setup** — Bridge creation, DNS for .sovrn.local subdomains
23. **zram-setup script** — zram config file and systemd service
24. **systemd-oomd configuration** — Pressure thresholds and kill policies
25. **PWA offline sync strategy** — Workbox config, background sync, conflict resolution UI
26. **Preact project scaffold** — Vite config, folder structure, routing, state management
27. **OOBE GTK4 application** — Screens, navigation, identity key generation integration
28. **Signing key infrastructure** — Dual GPG key generation, air-gapped signing workstation process
29. **Seed node deployment** — Ansible/scripts for the 5 seed nodes
30. **CI/CD pipeline** — GitHub Actions / GitLab CI for building ISO, running tests
31. **Dev environment setup** — Devcontainer or VM setup, mock Yggdrasil, sample data

### P3 — NICE TO HAVE (Can add during development)
32. **Performance benchmarks** — Target latency/throughput for each service
33. **Security audit checklist** — Formal threat model validation
34. **Accessibility testing plan** — Deferred to v2 but needs tracking
35. **Internationalization framework** — English-only for v1 but needs i18n scaffolding
36. **Mobile companion app API** — v3 but needs API consideration now
37. **CDN provider integration API** — Payment, storage, status checking
38. **Mail server (Maddy) config** — v2 but needs port/network planning now
39. **Community seed node onboarding** — Process for Phase 2 decentralization

---

## DOCUMENT-BY-DOCUMENT GAP SUMMARY

| Doc | Title | Key Missing Implementation Details |
|-----|-------|-----------------------------------|
| 00 | VISION | None — vision only |
| 01 | ARCHITECTURE | No spec for how resolved decisions are implemented (D1-D12 are decisions, not specs) |
| 02 | COMPONENTS | Rust recommended but sovrnd is Python — inconsistency unresolved; no version pins |
| 03 | NETWORK | No DHT wire protocol, no Yggdrasil config spec, no nftables rules, no CDN agent API |
| 04 | IDENTITY | No key storage format (keyring schema), no HKDF parameter details, no alias delegation wire format |
| 05 | UX-FLOW | No OOBE GTK4 implementation spec, no Calamares integration hooks |
| 06 | REVENUE | No payment integration API, no CDN marketplace API, no UPI/crypto SDK chosen |
| 07 | OPEN-QUESTIONS | All resolved — good |
| 08 | UPDATE-SYSTEM | No APT repo structure, no signing key management, no build reproducibility scripts |
| 09 | ROUND3-QUESTIONS | All resolved — good |
| 10 | ROUND3-DECISIONS | F1 multi-device sync protocol undefined; notification schema undefined |
| 11 | BLOCKING-DESIGN | Block event propagation mechanism undefined beyond JSON sketch |
| 12 | SYSTEM-REQUIREMENTS | No actual hardware compatibility testing matrix |
| 13 | ROUND5-6 | Guest mode persistence undefined; PhotoDNA integration unspecified |
| 14 | MODERATION | PhotoDNA integration: how packaged, how called, legal entity status undefined |
| 15 | SEED-NODES | No deployment scripts, no monitoring, no uptime SLA definition |
| 16 | ROUND8 | J2 REST API — only sync endpoints defined, 17+ other API groups missing schemas |
| 17 | DESIGN-SYSTEM | No actual CSS/component files, no icon library import, no GNOME theme .css file |
| 18 | ROUND9 | Preact scaffold undefined; Dexie schema specified but no migration strategy |
| 19 | SELF-HOSTED-APPS | sovrnd service lifecycle management undefined; Podman SDK calls unspecified; auth proxy middleware unspecified |
| 20 | NETWORK-FIX | Caddy config template listed as TODO; DNS dispatcher listed as TODO; port 53535 DNS resolver not implemented |
| 21 | DATA-AUTH | sovrnd API design listed as TODO; event schema defined but no API routes to create/fetch events |
| 22 | SYSTEM-UX-FIXES | No Calamares config, no .desktop file templates, no GNOME extension configs |
| FOLLOW-PERFORMANCE | Architecture handles unlimited follows but SQLite queries/indexes not defined |
| SESSION-STATE | Tracking document — no implementation gaps itself |

---

## QUANTITATIVE SUMMARY

| Category | Decisions Made | Implementation Specs Missing | % Complete |
|---|---|---|---|
| Language/Runtime | 5 decided | 10 undecided | 33% |
| Config Schemas | ~6 decided | ~14 undefined | 30% |
| API Endpoints | ~3 endpoint groups | ~17 endpoint groups | 15% |
| Database Schemas | 1 (IndexedDB/Dexie) | 6 (SQLite) | 14% |
| IPC Protocols | 5 defined | 14 undefined | 26% |
| Build/Packaging | ~6 conceptual decisions | ~17 concrete specs | 26% |
| Testing Strategy | 1 tool mentioned | ~11 areas with no plan | 8% |
| Dev Environment | 1 (Preact dev server) | ~9 areas undefined | 10% |

**Overall Build Readiness: ~20%**

The design documents provide excellent architectural vision and UX flows. They are suitable for securing buy-in and guiding implementation. However, they are approximately 80% away from being build-ready. A senior developer could not sit down and start writing code today — they would need to make ~100+ implementation decisions that the docs defer or never address.

---

## RECOMMENDED NEXT STEPS

1. **Write the sovrnd API specification** — This is the biggest single gap. Define every endpoint, every request/response body, every error code. This unlocks both backend and frontend development.

2. **Write SQLite schemas** — 6 database schemas would unblock all service implementation.

3. **Create a repository structure** — Decide monorepo vs. polyrepo, create directory layout, add empty package scaffolds.

4. **Specify service IPC** — Choose gRPC/Unix sockets/HTTP for sovrnd↔service communication. This determines service architecture.

5. **Write node-to-node protocol specs** — DHT, feed push, DM delivery, heartbeat. These are the network layer of the mesh.

6. **Pin language choices** — Decide Rust vs. Go vs. Python for each of the 6 mesh services.

7. **Create a Dev Quickstart guide** — How to clone, build, and run locally. This is the onboarding for any contributor.

---

*Assessment produced by analyzing all 25 design documents against 8 implementation categories.*