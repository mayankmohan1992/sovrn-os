# Sovrn — sovrnd API Specification

## Overview

sovrnd is a Python daemon on `127.0.0.1:54771`, reverse-proxied by Caddy at `https://sovrn.local/api/`. The PWA communicates ONLY through this API and the WebSocket at `/ws`.

**Base URL**: `https://sovrn.local/api/v1`
**Auth**: JWT in httpOnly cookie (`sovrn_session`), except login endpoint
**Content-Type**: `application/json` unless multipart
**Rate Limit**: 100 req/min per JWT (429 with `Retry-After` header)

## Standard Error Envelope

```json
{
  "error": {
    "code": "EVENT_NOT_FOUND",
    "message": "Event a7x3k9m2 does not exist",
    "details": {
      "event_id": "a7x3k9m2"
    }
  },
  "request_id": "req_f8d2a1b3c4e5"
}
```

**Error codes**: `AUTH_REQUIRED`, `AUTH_EXPIRED`, `RATE_LIMITED`, `NOT_FOUND`, `ALREADY_EXISTS`, `INVALID_INPUT`, `FORBIDDEN`, `INTERNAL_ERROR`, `SERVICE_UNAVAILABLE`, `DOMAIN_UNAVAILABLE`, `APP_INSTALL_FAILED`, `INSUFFICIENT_MEMORY`

---

## 1. AUTH

### POST /auth/login
```json
// Request
{ "password": "user_login_password" }

// Response 200
{
  "token": "eyJhbGciOiJFZDI1NTE5In0...",
  "expires_at": 1749086400,
  "user": {
    "id": "a7x3k9m2p8f1q4b5",
    "name": "Alice Sharma",
    "domain": "alice.sovrn"
  }
}
```

### POST /auth/logout
```
// No request body. Clears httpOnly cookie.
// Response 204
```

### POST /auth/refresh
```json
// Auto-called by PWA when JWT < 1h until expiry
// Response 200
{ "token": "eyJ...", "expires_at": 1749172800 }
```

### GET /auth/verify
```json
// Response 200
{ "valid": true, "user_id": "a7x3k9m2p8f1q4b5", "scope": "sovrn:all" }
```

---

## 2. FEED

### GET /feed/timeline?cursor={cursor}&limit=50
```json
// Response 200
{
  "events": [
    {
      "id": "sha256abc",
      "kind": 1,
      "author": "a7x3k9m2p8f1q4b5",
      "author_name": "Alice Sharma",
      "author_domain": "alice.sovrn",
      "created_at": 1749000000,
      "content": "Hello Sovrn!",
      "tags": [],
      "media": [],
      "reaction_count": { "like": 5, "boost": 2 },
      "reply_count": 3
    }
  ],
  "cursor": "MjAyNS0wNi0wNVQxMjowMDowMF9hN3gzazltMg==",
  "has_more": true
}
```

### GET /feed/event/{event_id}
```json
// Response 200
{
  "id": "sha256abc",
  "kind": 1,
  "author": "a7x3k9m2p8f1q4b5",
  "content": "Hello Sovrn!",
  "created_at": 1749000000,
  "tags": [],
  "media": [],
  "sig": "edb0de...hex..."
}
```

### POST /feed/event
```json
// Request — Create micro post (kind 1)
{
  "kind": 1,
  "content": "Hello from Sovrn!",
  "tags": [],
  "media": []
}

// Response 201
{
  "id": "sha256abc",
  "kind": 1,
  "author": "a7x3k9m2p8f1q4b5",
  "created_at": 1749000060,
  "content": "Hello from Sovrn!",
  "sig": "edb0de..."
}

// Request — Create media post (kind 2)
{
  "kind": 2,
  "content": "Sunset over Jaipur",
  "tags": [],
  "media": [
    { "cid": "bafybeig...", "type": "image/jpeg", "size": 245760, "alt": "Sunset" }
  ]
}
```

### DELETE /feed/event/{event_id}
```json
// Creates kind-9 delete event referencing the target
// Response 204
```

### GET /feed/event/{event_id}/reactions?cursor={c}&limit=50
```json
// Response 200
{
  "reactions": [
    { "kind": 6, "author": "b3k7m1...", "created_at": 1749000120 },
    { "kind": 6, "author": "c5n2p9...", "created_at": 1749000180 }
  ],
  "cursor": "...",
  "has_more": false
}
```

### POST /feed/event/{event_id}/react
```json
// Request
{ "type": "like" }  // "like" or "boost"

// Response 201
{ "reaction_id": "sha256...", "kind": 6, "target": "sha256abc" }
```

---

## 3. MESSAGES

### GET /messages/conversations?cursor={c}&limit=20
```json
// Response 200
{
  "conversations": [
    {
      "peer_id": "b3k7m1n4q7r2t6w9",
      "peer_name": "Bob Singh",
      "peer_domain": "bob.sovrn",
      "last_message": "See you tomorrow!",
      "last_at": 1748999500,
      "unread_count": 2
    }
  ],
  "cursor": "...",
  "has_more": false
}
```

### GET /messages/{peer_id}?cursor={c}&limit=50
```json
// Response 200
{
  "messages": [
    {
      "id": "msg_sha256_1",
      "from": "a7x3k9m2p8f1q4b5",
      "to": "b3k7m1n4q7r2t6w9",
      "content": "...encrypted...",
      "created_at": 1748999400,
      "read": true
    }
  ],
  "cursor": "...",
  "has_more": true
}
```

### POST /messages/{peer_id}
```json
// Request
{ "content": "...encrypted blob...", "media": [] }

// Response 201
{ "id": "msg_sha256_2", "from": "a7x3k9m2p8f1q4b5", "to": "b3k7m1n4q7r2t6w9", "created_at": 1749000060, "read": false }
```

### POST /messages/{peer_id}/mark_read
```json
// Mark all messages from peer as read
// Response 204
```

### DELETE /messages/{message_id}
```json
// Delete locally only (recipient's copy is already delivered)
// Response 204
```

---

## 4. IDENTITY

### GET /identity/profile
```json
// Response 200
{
  "id": "a7x3k9m2p8f1q4b5",
  "name": "Alice Sharma",
  "domain": "alice.sovrn",
  "about": "Building Sovrn OS",
  "avatar_cid": "bafybeig...",
  "banner_cid": "bafyreig...",
  "follow_count": 42,
  "follower_count": 128,
  "created_at": 1748000000
}
```

### PATCH /identity/profile
```json
// Request (partial update)
{ "name": "Alice S.", "about": "Updated bio" }

// Response 200 — full updated profile
```

### GET /identity/aliases
```json
// Response 200
{ "aliases": [
  { "name": "alice", "domain": "alice.sovrn", "created_at": 1748000000 },
  { "name": "alice-s", "domain": "alice-s.sovrn", "delegated": true, "delegates_to": "a7x3k9m2p8f1q4b5" }
]}
```

### POST /identity/alias
```json
// Request
{ "name": "alice-s" }

// Response 201
{ "name": "alice-s", "domain": "alice-s.sovrn", "available": true, "registered": true }
```

### DELETE /identity/alias/{name}
```json
// Response 204
```

### POST /identity/export
```json
// Request (re-auth required)
{ "password": "user_login_password" }

// Response 200
{ "seed_phrase": "horizon silver guitar...", "master_keypair": "base64...", "aliases": [...], "settings": {...} }
```

### POST /identity/import
```json
// Request (second device setup)
{ "seed_phrase": "horizon silver guitar verify..." }

// Response 200
{ "id": "a7x3k9m2p8f1q4b5", "name": "Alice Sharma", "domain": "alice.sovrn" }
```

---

## 5. MESH

### POST /mesh/register-domain
```json
// Request
{ "name": "alice" }

// Response 201
{ "domain": "alice.sovrn", "public_key": "a7x3k9m2p8f1q4b5", "signature": "sig_hex..." }
```

### GET /mesh/lookup/{domain}
```json
// Response 200
{ "domain": "bob.sovrn", "public_key": "b3k7m1n4q7r2t6w9", "ygg_address": "200:...", "online": true }
```

### GET /mesh/check-availability?name={name}
```json
// Response 200
{ "name": "alice", "available": false, "suggestions": ["alice-2", "alice-s", "alicejaipur"] }
```

### GET /mesh/peers?limit=100
```json
// Response 200
{ "peers": [
  { "public_key": "b3k7m1...", "domain": "bob.sovrn", "ygg_address": "200:...", "latency_ms": 12, "online": true },
  { "public_key": "c5n2p9...", "domain": "carol.sovrn", "ygg_address": "200:...", "latency_ms": 45, "online": true }
], "total": 47 }
```

### GET /mesh/status
```json
// Response 200
{
  "connected": true,
  "ygg_address": "200:abc:...",
  "peers_count": 47,
  "dht_nodes": 12,
  "uptime_seconds": 86400,
  "mesh_version": "0.1.0"
}
```

---

## 6. PRESENCE

### GET /presence/online?limit=50
```json
// Response 200
{ "peers": [
  { "public_key": "b3k7m1...", "domain": "bob.sovrn", "status": "online", "last_heartbeat": 1749000030 },
  { "public_key": "c5n2p9...", "domain": "carol.sovrn", "status": "away", "last_heartbeat": 1748999500 }
]}
```

### PUT /presence/status
```json
// Request
{ "status": "online" }  // "online", "away", "dnd", "offline"

// Response 200
{ "status": "online" }
```

---

## 7. CDN

### POST /cdn/push
```json
// Request
{ "cid": "bafybeig...", "size": 245760, "checksum": "sha256:abc123..." }

// Response 202
{ "push_id": "push_abc", "status": "queued", "providers": ["fastcdn", "meshcdn"] }
```

### GET /cdn/status/{push_id}
```json
// Response 200
{ "push_id": "push_abc", "status": "complete", "providers": [
  { "name": "fastcdn", "status": "uploaded", "url": "https://fastcdn.example/bafybeig..." }
]}
```

### GET /cdn/providers
```json
// Response 200
{ "providers": [
  { "id": "fastcdn", "name": "FastCDN", "price_per_gb": 0.02, "regions": ["asia", "eu", "na"], "rating": 4.5 }
]}
```

### POST /cdn/subscribe
```json
// Request
{ "provider_id": "fastcdn", "plan": "starter", "payment_method": "upi" }

// Response 201
{ "subscription_id": "sub_abc", "provider": "fastcdn", "plan": "starter", "storage_gb": 10, "bandwidth_gb": 100 }
```

---

## 8. HOMEPAGE

### POST /homepage/publish
```json
// Request
{ "content_cid": "bafybeig...", "content_type": "html" }

// Response 201
{ "domain": "alice.sovrn", "content_cid": "bafybeig...", "published_at": 1749000060 }
```

### GET /homepage/{domain}
```json
// Response 200
{ "domain": "alice.sovrn", "content_cid": "bafybeig...", "content_type": "html", "updated_at": 1749000060 }
// Client fetches content from local cache or CDN
```

### DELETE /homepage
```json
// Response 204
```

---

## 9. FOLLOW / BLOCK

### POST /follow/{user_id}
```json
// Response 201
{ "following": true, "user_id": "b3k7m1n4q7r2t6w9" }
```

### DELETE /follow/{user_id}
```json
// Response 204
```

### GET /follow/followers?cursor={c}&limit=50
```json
// Response 200
{ "followers": [
  { "id": "b3k7m1...", "name": "Bob Singh", "domain": "bob.sovrn", "followed_at": 1748900000 }
], "cursor": "...", "has_more": false }
```

### GET /follow/following?cursor={c}&limit=50
```json
// Response 200
{ "following": [
  { "id": "b3k7m1...", "name": "Bob Singh", "domain": "bob.sovrn", "followed_at": 1748900000 }
], "cursor": "...", "has_more": false }
```

### POST /block/{user_id}
```json
// Response 201
{ "blocked": true, "user_id": "c5n2p9..." }
```

### DELETE /block/{user_id}
```json
// Response 204
```

### GET /block/list
```json
// Response 200
{ "blocked": ["c5n2p9...", "d8m4q2..."] }
```

---

## 10. SEARCH

### GET /search/users?q={query}&limit=20
```json
// Response 200
{ "results": [
  { "id": "b3k7m1...", "name": "Bob Singh", "domain": "bob.sovrn", "about": "..." }
]}
```

### GET /search/content?q={query}&kind={kind}&limit=20
```json
// Response 200
{ "results": [
  { "id": "sha256abc", "kind": 1, "author": "b3k7m1...", "content": "...", "created_at": 1749000000 }
]}
```

### GET /search/discover?limit=20
```json
// Trending/popular content from mesh
// Response 200
{ "trending": [
  { "id": "sha256abc", "kind": 1, "author": "b3k7m1...", "content": "Check out this...", "reaction_count": 42 }
]}
```

---

## 11. SETTINGS

### GET /settings
```json
// Response 200
{
  "theme": "dark-auto",
  "language": "en",
  "mesh_explorer_visible": true,
  "homepage_editor_advanced": false,
  "default_app_order": ["nextcloud", "vaultwarden"],
  "memory_warning_threshold": 85
}
```

### PATCH /settings
```json
// Request (partial update)
{ "theme": "light" }

// Response 200 — full settings object
```

### GET /settings/notifications
```json
// Response 200
{
  "dm_notifications": true,
  "follow_notifications": true,
  "reaction_notifications": true,
  "mesh_status_notifications": true,
  "app_update_notifications": true,
  "quiet_hours": { "enabled": false, "start": "22:00", "end": "08:00" }
}
```

### PATCH /settings/notifications
```json
// Request (partial update)
// Response 200 — full notification settings
```

### GET /settings/privacy
```json
// Response 200
{
  "profile_visibility": "public",  // "public", "followers_only", "private"
  "online_status_visible": true,
  "allow_mesh_indexing": true,
  "cdn_push_enabled": false
}
```

### PATCH /settings/privacy
```json
// Request (partial update)
// Response 200 — full privacy settings
```

---

## 12. APPS

### GET /apps/available?tier={1,2,3}&limit=50
```json
// Response 200
{ "apps": [
  {
    "id": "nextcloud",
    "name": "Nextcloud",
    "description": "Self-hosted file sync and share",
    "tier": 1,
    "icon_cid": "bafybeig...",
    "memory_mb": 1200,
    "disk_gb": 2,
    "version": "33.0.2",
    "local_image": true,
    "installed": false
  }
], "total": 12 }
```

### GET /apps/installed
```json
// Response 200
{ "apps": [
  {
    "id": "nextcloud",
    "name": "Nextcloud",
    "version": "33.0.2",
    "status": "running",  // "running", "stopped", "installing", "failed", "memory_pressure"
    "memory_used_mb": 1180,
    "url": "https://nextcloud.sovrn.local/",
    "installed_at": 1748000000,
    "last_health_check": 1749000030
  }
]}
```

### POST /apps/install
```json
// Request
{ "app_id": "nextcloud" }

// Response 202
{ "install_id": "inst_abc", "app_id": "nextcloud", "status": "installing", "progress": 0 }
// Progress tracked via WebSocket /ws event: app_install_progress
```

### DELETE /apps/{app_id}
```json
// Uninstalls app, removes container, removes Caddy route
// Response 204
```

### POST /apps/{app_id}/start
```json
// Response 200
{ "app_id": "nextcloud", "status": "starting" }
```

### POST /apps/{app_id}/stop
```json
// Response 200
{ "app_id": "nextcloud", "status": "stopping" }
```

### GET /apps/{app_id}/health
```json
// Response 200
{ "app_id": "nextcloud", "status": "running", "healthy": true, "uptime_seconds": 86400, "memory_used_mb": 1180, "restart_count": 0 }
```

### GET /apps/{app_id}/logs?limit=100&level={info,warn,error}
```json
// Response 200
{ "logs": [
  { "timestamp": "2025-06-04T12:00:00Z", "level": "info", "message": "Nextcloud started" },
  { "timestamp": "2025-06-04T12:00:01Z", "level": "warn", "message": "Memory usage at 80%" }
]}
```

### POST /apps/{app_id}/restart
```json
// Response 200
{ "app_id": "nextcloud", "status": "restarting" }
```

---

## 13. BACKUP

### POST /backup/trigger
```json
// Request
{ "target": "usb" }  // "usb", "network", "mesh"

// Response 202
{ "backup_id": "bak_abc", "status": "running", "target": "usb" }
// Progress tracked via WebSocket
```

### GET /backup/status
```json
// Response 200
{ "last_backup": "2025-06-04T12:00:00Z", "last_status": "success", "target": "usb", "size_gb": 4.2 }
```

### GET /backup/list
```json
// Response 200
{ "backups": [
  { "id": "bak_abc", "date": "2025-06-04T12:00:00Z", "size_gb": 4.2, "target": "usb", "status": "success" }
]}
```

### POST /backup/{backup_id}/restore
```json
// Request (re-auth required)
{ "password": "user_login_password" }

// Response 202
{ "restore_id": "rst_abc", "status": "running" }
```

---

## 14. FILE UPLOAD

### POST /media/upload
```
Content-Type: multipart/form-data

file: <binary>
alt_text: "Sunset over Jaipur"

// Response 201
{
  "cid": "bafybeig...",
  "url": "/api/v1/media/bafybeig...",
  "type": "image/jpeg",
  "size": 245760,
  "width": 1920,
  "height": 1080
}
```

### GET /media/{cid}
```json
// Returns binary file with appropriate Content-Type header
// Or 302 redirect to CDN URL if available
```

### DELETE /media/{cid}
```json
// Deletes local copy. CDN copy persists until TTL expires.
// Response 204
```

---

## 15. WEBSOCKET (/ws)

Connect at `wss://sovrn.local/api/v1/ws` with JWT cookie auth.

### Message Format (client → server)
```json
{ "type": "subscribe", "channels": ["feed", "dm", "presence", "apps", "mesh"] }
{ "type": "unsubscribe", "channels": ["feed"] }
{ "type": "ping" }
```

### Message Format (server → client)
```json
// Feed update
{ "type": "feed_update", "data": { "event": { "id": "sha256abc", "kind": 1, "author": "b3k7m1...", "content": "Hello!", "created_at": 1749000060 } } }

// DM received
{ "type": "dm_received", "data": { "from": "b3k7m1...", "from_name": "Bob", "message_id": "msg_abc", "preview": "..." } }

// Presence change
{ "type": "presence_change", "data": { "peer": "b3k7m1...", "domain": "bob.sovrn", "status": "online" } }

// App status change
{ "type": "app_status", "data": { "app_id": "nextcloud", "status": "running", "healthy": true } }

// App install progress
{ "type": "app_install_progress", "data": { "install_id": "inst_abc", "app_id": "nextcloud", "progress": 45, "status": "pulling_image" } }

// Mesh status
{ "type": "mesh_status", "data": { "connected": true, "peers_count": 47 } }

// Backup progress
{ "type": "backup_progress", "data": { "backup_id": "bak_abc", "progress": 60, "status": "compressing" } }

// Pong (response to ping)
{ "type": "pong" }
```

---

## 16. SYNC (for completeness, defined in D-S1)

### GET /sync/pull?since={cursor}&limit=100
### POST /sync/push
```json
// Request
{ "pending_events": [
  { "kind": 1, "content": "Offline post", "created_at": 1748990000, "tags": [], "media": [] }
]}
// Response 200
{ "confirmed": [...] , "conflicts": [...], "rejected": [...] }
```

### GET /sync/status
```json
// Response 200
{ "last_sync": 1749000060, "pending_local": 0, "pending_remote": 3 }
```