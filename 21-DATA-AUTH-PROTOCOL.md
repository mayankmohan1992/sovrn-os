# Sovrn — Data Layer, Auth & Protocol Fixes

## D-S1: SQLite ↔ IndexedDB Sync Protocol

### Architecture

```
┌─────────────────────────────────────────────────┐
│                    PWA (Preact)                  │
│           IndexedDB (Dexie.js) ← PWA reads/writes│
│                    ↕ Sync API                     │
│            sovrnd REST API (:54771)               │
│                    ↕                               │
│           SQLite (server-side)                    │
│      /var/lib/sovrn/feed/, messages/, etc.       │
└─────────────────────────────────────────────────┘
```

**Rule: PWA NEVER reads SQLite directly. PWA NEVER writes to SQLite directly. All data access goes through sovrnd API.**

### Data Flow

1. **Offline write**: PWA writes to IndexedDB immediately (instant UI response)
2. **Sync API**: When online, PWA sends pending writes to `sovrnd /api/v1/sync`
3. **Server processes**: sovrnd writes to SQLite, assigns server timestamp, returns confirmed state
4. **PWA updates**: PWA replaces IndexedDB entry with server-confirmed version

### API Endpoints

```
GET  /api/v1/sync/pull?since=<cursor>&limit=100
     → Returns events newer than cursor, plus new cursor

POST /api/v1/sync/push
     Body: { pending_events: [...] }
     → Returns { confirmed: [...], conflicts: [...], rejected: [...] }

GET  /api/v1/sync/status
     → Returns { last_sync: <timestamp>, pending_local: N, pending_remote: N }
```

### Sync Protocol (Pull-based)

| Phase | Action |
|-------|--------|
| PWA startup | GET `/api/v1/sync/pull?since=0` → bulk load into IndexedDB |
| PWA online | POST `/api/v1/sync/push` → send pending IndexedDB writes |
| PWA online | GET `/api/v1/sync/pull?since=<last_cursor>` → get new remote data |
| PWA offline | IndexedDB reads/writes continue locally |
| PWA back online | POST all pending writes → resolve conflicts → pull new data |

### Conflict Resolution

| Event Type | Conflict Strategy |
|-----------|-------------------|
| Posts | Last-writer-wins (server timestamp) |
| Follows/Unfollows | Set operation (no conflict possible) |
| Blocks | Set operation (no conflict possible) |
| DMs | No conflict (unique event ID per message) |
| Profile edits | Last-writer-wins with field-level merge |
| Homepage edits | Last-writer-wins (homepage blob is atomic) |
| Settings | Field-level merge (changed fields win) |

### Cursor Format

Cursors are opaque strings encoding `(timestamp, event_id)`. Example: `MjAyNS0wNi0wNVQxMjowMDowMF9hN3gzazltMg==`

- Client stores last cursor after each successful pull
- On reconnect, sends cursor since last successful sync
- Server returns all events after that cursor

### IndexedDB Schema (Dexie)

```javascript
const db = new Dexie('sovrn');
db.version(1).stores({
  events: '++id, kind, author, created_at, [kind+author]',  // mirror of server events
  pending: '++id, kind, created_at, status',                // offline writes not yet synced
  contacts: '++id, pubkey, name, domain, last_seen',        // follow list cache
  settings: 'key',                                            // user settings
  media_cache: 'url, blob, expires',                         // cached media blobs
});
```

## D-S2: Auth Architecture (Honest SSO + Fallback)

### Honestly: This IS SSO-like behavior

The auth proxy provides SSO-like UX. When it works, user logs in once to Sovrn Hub and transparently accesses all installed apps. When it fails, each app has independent fallback auth.

### Auth Flow

```
┌──────────────────────────────────────────────────────────────┐
│                     USER LOGIN FLOW                          │
│                                                               │
│  1. User opens https://sovrn.local/                          │
│  2. Sovrn Hub PWA shows login screen (if no session)         │
│  3. User enters Sovrn identity password (OS login password)   │
│  4. sovrnd validates password against OS keyring              │
│  5. sovrnd issues JWT (24h expiry, signed with Sovrn key)     │
│  6. PWA stores JWT in httpOnly cookie (secure, same-origin)   │
│  7. PWA is now "logged in" — JWT sent with every request     │
│                                                               │
│  ACCESSING SELF-HOSTED APPS:                                 │
│                                                               │
│  8. User clicks "Open Nextcloud" in Sovrn Hub                │
│  9. PWA navigates to https://nextcloud.sovrn.local/          │
│ 10. sovrn-auth (Caddy middleware) validates JWT               │
│ 11. sovrn-auth injects auth headers → Nextcloud container    │
│ 12. User sees Nextcloud — already logged in                  │
│                                                               │
│  AUTH PROXY DOWN FALLBACK:                                    │
│                                                               │
│  13. sovrn-auth process crashes or is restarting              │
│  14. Caddy detects auth middleware unavailable                │
│  15. Caddy falls back to direct proxy (no auth injection)     │
│  16. App shows its own native login screen                    │
│  17. User logs in with Sovrn-provisioned credentials         │
│  18. Credentials stored in Sovrn password vault (Dexie)       │
│  19. When auth proxy recovers, user is auto-logged-in again  │
└──────────────────────────────────────────────────────────────┘
```

### JWT Structure

```json
{
  "sub": "a7x3k9m2p8f1q4b5",
  "name": "Alice Sharma",
  "domain": "alice.sovrn",
  "iat": 1749000000,
  "exp": 1749086400,
  "scope": "sovrn:all"
}
```

- Signed with Ed25519 (Sovrn's identity key)
- Stored in httpOnly, Secure, SameSite=Strict cookie
- Refreshed silently 1 hour before expiry
- 24-hour max lifetime, then re-login required

### App Auto-Provisioning (Per-App Adapters)

Every self-hosted app needs a provisioning adapter — a Python script that sovrnd calls after starting the container.

```yaml
# sovrn-app.yml provisioning section
provisioning:
  type: "nextcloud"          # adapter name
  admin_user_script: "/opt/sovrn/provision/nextcloud.sh"
  health_check_url: "http://nextcloud.sovrn.local/status"
  health_check_interval: 10s
  health_check_timeout: 60s   # max wait for app to become healthy
```

**Adapter list (v1):**

| App | Adapter Type | How It Creates Admin |
|-----|-------------|---------------------|
| Nextcloud | env_vars | `NEXTCLOUD_ADMIN_USER` + `NEXTCLOUD_ADMIN_PASSWORD` env vars at first run |
| Vaultwarden | env_vars | `ADMIN_TOKEN` env var, first admin invite |
| Wallabag | env_vars | `WALLABAG_ADMIN_USER/PASSWORD` env vars |
| FreshRSS | env_vars | `FRESHRSS_ADMIN_USER/PASSWORD` env vars |
| Navidrome | env_vars | `ND_ADMIN_USER/PASSWORD` env vars |
| AdGuard Home | first_run_api | POST to `/control/status` with auth after first launch |
| Gitea | first_run_api | POST to `/api/v1/admin/users` with admin token |
| Memos | env_vars | `MEMO_ADMIN` env var |
| SearXNG | no_auth | No auth needed (search engine, not data store) |
| WireGuard Easy | env_vars | `PASSWORD_HASH` env var |

### Password Derivation (Not Storage)

App passwords are **derived, never stored**:

```
app_password = HKDF(
  ikm = Sovrn_master_key,           # Ed25519 private key (encrypted in OS keyring)
  salt = "sovrn-app-" + app_id,     # e.g. "sovrn-app-nextcloud"
  info = "v1-password",
  length = 24
)
→ Base64 encoded → 32-character password
```

- Same seed always produces same password for same app
- Password shown to user once during install, stored in browser vault
- If keyring is lost, re-derivation from seed phrase reproduces all passwords
- No password database to compromise

## D-S3: Local API Authentication

### Threat Model

Any local process (malicious flatpak, browser tab, another app) can hit `sovrn.local/api/`. We must prevent unauthorized access.

### Solution: JWT + Origin Validation + Rate Limiting

```
┌────────────────────────────────────────────────────┐
│              API AUTH LAYERS                        │
│                                                     │
│  Layer 1: TLS + Same-Origin                        │
│    - https://sovrn.local/ only                     │
│    - Caddy rejects cross-origin requests           │
│    - HSTS header enforces HTTPS                    │
│                                                     │
│  Layer 2: JWT Bearer Token                         │
│    - All /api/v1/ endpoints require valid JWT       │
│    - JWT issued on login (see D-S2)                │
│    - Verified by sovrnd on every request            │
│                                                     │
│  Layer 3: Scope-Based Access Control               │
│    - JWT contains user identity and scope           │
│    - mesh:read, mesh:write, apps:read, apps:write   │
│    - PWA gets full scope; each app gets limited     │
│                                                     │
│  Layer 4: Rate Limiting                            │
│    - 100 requests/minute per JWT                   │
│    - 1000 requests/minute for internal sovrnd calls │
│    - 429 Too Many Requests if exceeded              │
│                                                     │
│  Layer 5: Audit Logging                            │
│    - All admin actions logged to /var/log/sovrn/   │
│    - App install/uninstall events logged            │
│    - Failed auth attempts logged                    │
└────────────────────────────────────────────────────┘
```

### Why Not Unix Socket?

Unix sockets with file permissions would provide better isolation but:
- PWA runs in a browser → can't access Unix sockets
- We need the PWA to talk to sovrnd → must be network-accessible
- Origin + JWT provides equivalent protection for browser clients

### Additional Hardening

- sovrnd binds to `127.0.0.1:54771` only (not externally accessible)
- Caddy reverse-proxies `/api/` → `127.0.0.1:54771` on the same origin
- CSRF protection: SameSite=Strict cookies
- No CORS headers needed (same origin)
- Sensitive operations (app install, key export) require re-authentication (password confirmation)

## D-S4: Feed Event Schema (v1)

### Event Structure

Every event in Sovrn follows this JSON structure:

```json
{
  "id": "sha256(event_content)",
  "kind": 1,
  "author": "a7x3k9m2p8f1q4b5",
  "created_at": 1749000000,
  "content": "Hello Sovrn!",
  "tags": [],
  "sig": "edb0de...hex...",
  "media": [],
  "ttl": null
}
```

### Field Definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | string | yes | SHA-256 of serialized event content (deterministic, dedup) |
| kind | integer | yes | Event type (see kinds below) |
| author | string | yes | Ed25519 public key hash (user ID) |
| created_at | integer | yes | Unix timestamp (seconds) |
| content | string | yes | Event body (text, JSON, URL, etc.) |
| tags | array | no | Key-value pairs for metadata |
| sig | string | yes | Ed25519 signature of serialized event |
| media | array | no | Media attachments (CID, type, size) |
| ttl | integer | no | Time-to-live in seconds (ephemeral events) |

### Event Kinds (v1)

| Kind | Name | Content | Description |
|------|------|---------|-------------|
| 0 | metadata | JSON object | Profile: `{name, about, domain, avatar_cid, banner_cid}` |
| 1 | micro | plain text | Short post (≤2000 chars, like a tweet) |
| 2 | media | JSON | Image/video post: `{urls: [{cid, type, w, h, alt}], caption}` |
| 3 | follow | pubkey | Follow: author follows pubkey in content |
| 4 | unfollow | pubkey | Unfollow: author unfollows pubkey in content |
| 5 | block | pubkey | Block: author blocks pubkey in content (bidirectional) |
| 6 | reaction | event_id | Reaction: like/boost for event_id in content |
| 7 | dm | encrypted blob | Encrypted direct message |
| 8 | channel | JSON | Channel/group definition (v1.1, not v1) |
| 9 | delete | event_id | Delete: author requests deletion of event_id |
| 10 | update_profile | JSON | Profile update (replaces kind 0) |
| 100 | heartbeat | null | I'm alive signal (every 2 min) |
| 101 | presence_offline | null | Going offline signal |
| 200 | homepage_update | CID | Homepage content blob CID |
| 300 | media_cdn_push | JSON | Push content to CDN: `{cid, size, checksum}` |

### Serialization Order (for ID and signature)

Fields MUST be serialized in this exact order for deterministic hashing:
`id, kind, author, created_at, content, tags`

Signature covers: `id + kind + author + created_at + content + tags` (concatenated, newline-separated)

### Media Attachments

```json
{
  "media": [
    {
      "cid": "bafybeig...content-addressed-hash",
      "type": "image/jpeg",
      "size": 245760,
      "width": 1920,
      "height": 1080,
      "alt": "Sunset over Jaipur"
    }
  ]
}
```

- Images: max 25 MB per attachment, max 4 per post
- Videos: max 500 MB, CDN only (not served from home node)
- Thumbnails: auto-generated by sovrnd, stored as additional CID

### Ephemeral Events

Events with `ttl` are NOT stored permanently. They expire after `ttl` seconds.
- Heartbeats (kind 100): `ttl = 180` (3 minutes)
- Presence offline (kind 101): `ttl = 3600` (1 hour, for late receivers)

## D-S5: Self-Hosted App → Mesh Network Access

### Decision: Self-hosted apps are local-only by default, with opt-in mesh access

| Access Mode | URL | Who Can Access | Opt-In? |
|-------------|-----|----------------|---------|
| Local only (default) | `https://appname.sovrn.local/` | User on this device only | No (default) |
| Mesh: Private link | `https://appname.username.sovrn/` | User + explicitly invited peers | Yes, per-app |
| Mesh: Public | `https://appname.username.sovrn/` | Anyone on Sovrn mesh | Yes, per-app |

### Mesh Access Implementation

1. User opens Sovrn Hub → App Settings → toggles "Share on Sovrn Mesh"
2. sovrnd creates a Caddy route: `appname.username.sovrn` → internal Podman IP
3. sovrnd registers the subdomain in mesh DHT
4. Auth proxy enforces: private = invite-only, public = mesh-wide
5. Yggdrasil handles routing to the user's node

### Security Considerations

- Self-hosted apps were NOT designed for public internet exposure
- Mesh access requires auth proxy enforcement even for "public" mode
- Rate limiting per-app per-visitor is essential
- The auth proxy injects mesh identity headers so apps can identify visitors
- Recommended: Only expose apps that support authentication (Nextcloud yes, Memos needs auth proxy enforcement)