# Sovrn — Database Schemas

## Migration Strategy

All databases use SQLite with versioned migrations. Each service manages its own database file. Migration files are embedded in the service binary (Rust) or shipped alongside (Python).

- **Migration format**: `V{N}_{description}.sql` where N is zero-padded 3-digit number
- **Migration tracking**: Each database has a `_migrations` table tracking applied versions
- **Rollback**: Not supported in v1 — migrations are forward-only

---

## 1. feed.db — `/var/lib/sovrn/feed/feed.db`

```sql
-- V001_initial.sql
CREATE TABLE IF NOT EXISTS _migrations (
    version INTEGER PRIMARY KEY,
    description TEXT NOT NULL,
    applied_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

CREATE TABLE IF NOT EXISTS events (
    id TEXT PRIMARY KEY,              -- SHA-256 of serialized event
    kind INTEGER NOT NULL,            -- Event kind (0-300)
    author TEXT NOT NULL,              -- Public key hash
    content TEXT NOT NULL,            -- Event body
    created_at INTEGER NOT NULL,      -- Unix timestamp
    sig TEXT NOT NULL,                -- Ed25519 signature hex
    raw_json TEXT NOT NULL,           -- Original JSON for verification
    inserted_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),

    CHECK(kind >= 0 AND kind <= 9999)
);

CREATE INDEX idx_events_kind ON events(kind);
CREATE INDEX idx_events_author ON events(author);
CREATE INDEX idx_events_created ON events(created_at DESC);
CREATE INDEX idx_events_kind_author ON events(kind, author);

CREATE TABLE IF NOT EXISTS follows (
    source TEXT NOT NULL,             -- Follower pubkey hash
    target TEXT NOT NULL,             -- Followed pubkey hash
    created_at INTEGER NOT NULL,
    removed_at INTEGER,               -- NULL if still following
    PRIMARY KEY(source, target)
);

CREATE INDEX idx_follows_source ON follows(source);
CREATE INDEX idx_follows_target ON follows(target);

CREATE TABLE IF NOT EXISTS blocks (
    source TEXT NOT NULL,
    target TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    removed_at INTEGER,
    PRIMARY KEY(source, target)
);

CREATE TABLE IF NOT EXISTS reactions (
    id TEXT PRIMARY KEY,              -- SHA-256 of reaction event
    target_event TEXT NOT NULL,       -- Event being reacted to
    kind INTEGER NOT NULL,            -- 6 (like) or specific reaction
    author TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY(target_event) REFERENCES events(id) ON DELETE CASCADE
);

CREATE INDEX idx_reactions_target ON reactions(target_event);
CREATE INDEX idx_reactions_author ON reactions(author);

CREATE TABLE IF NOT EXISTS media_refs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id TEXT NOT NULL,
    cid TEXT NOT NULL,                -- Content-addressed ID
    media_type TEXT NOT NULL,         -- image/jpeg, video/mp4, etc.
    size_bytes INTEGER NOT NULL,
    width INTEGER,
    height INTEGER,
    alt_text TEXT,
    FOREIGN KEY(event_id) REFERENCES events(id) ON DELETE CASCADE
);

CREATE INDEX idx_media_event ON media_refs(event_id);
CREATE INDEX idx_media_cid ON media_refs(cid);

CREATE TABLE IF NOT EXISTS timeline_cache (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,            -- Whose timeline
    event_id TEXT NOT NULL,
    rank REAL NOT NULL,              -- For sorting (chronological or algorithmic)
    seen INTEGER DEFAULT 0,           -- 0=unread, 1=read
    FOREIGN KEY(event_id) REFERENCES events(id) ON DELETE CASCADE
);

CREATE INDEX idx_timeline_user ON timeline_cache(user_id, rank DESC);

CREATE TABLE IF NOT EXISTS content_cache (
    cid TEXT PRIMARY KEY,             -- Content-addressed ID
    local_path TEXT NOT NULL,          -- Path on disk
    size_bytes INTEGER NOT NULL,
    media_type TEXT NOT NULL,
    accessed_at INTEGER NOT NULL,
    expires_at INTEGER                 -- NULL = no expiry
);

CREATE INDEX idx_cache_access ON content_cache(accessed_at);
```

---

## 2. messages.db — `/var/lib/sovrn/messages/messages.db`

```sql
-- V001_initial.sql
CREATE TABLE IF NOT EXISTS _migrations (
    version INTEGER PRIMARY KEY,
    description TEXT NOT NULL,
    applied_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

CREATE TABLE IF NOT EXISTS conversations (
    peer_id TEXT PRIMARY KEY,         -- Other party's pubkey hash
    peer_name TEXT,                   -- Cached display name
    peer_domain TEXT,                 -- Cached .sovrn domain
    last_message_preview TEXT,        -- Truncated last message
    last_at INTEGER NOT NULL,         -- Timestamp of last message
    unread_count INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

CREATE INDEX idx_conversations_last ON conversations(last_at DESC);

CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,              -- SHA-256 of message
    conversation_id TEXT NOT NULL,     -- Peer's pubkey hash
    from_id TEXT NOT NULL,             -- Sender pubkey hash
    to_id TEXT NOT NULL,              -- Recipient pubkey hash
    content BLOB NOT NULL,            -- Encrypted message blob
    created_at INTEGER NOT NULL,
    read INTEGER NOT NULL DEFAULT 0,
    local INTEGER NOT NULL DEFAULT 0, -- 1=sent by us, 0=received

    FOREIGN KEY(conversation_id) REFERENCES conversations(peer_id) ON DELETE CASCADE
);

CREATE INDEX idx_messages_conv ON messages(conversation_id, created_at DESC);
CREATE INDEX idx_messages_unread ON messages(conversation_id, read) WHERE read = 0;

CREATE TABLE IF NOT EXISTS message_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    message_id TEXT NOT NULL,
    target_peer TEXT NOT NULL,         -- Destination pubkey hash
    content BLOB NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    next_attempt INTEGER NOT NULL,     -- Unix timestamp
    status TEXT NOT NULL DEFAULT 'queued',  -- queued, sending, failed, delivered
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),

    FOREIGN KEY(message_id) REFERENCES messages(id) ON DELETE CASCADE
);

CREATE INDEX idx_queue_status ON message_queue(status, next_attempt);

CREATE TABLE IF NOT EXISTS delivery_receipts (
    message_id TEXT PRIMARY KEY,
    peer_id TEXT NOT NULL,
    status TEXT NOT NULL,               -- sent, delivered, read
    timestamp INTEGER NOT NULL
);

CREATE INDEX idx_receipts_peer ON delivery_receipts(peer_id);
```

---

## 3. identity.db — `/var/lib/sovrn/identity/identity.db`

```sql
-- V001_initial.sql
CREATE TABLE IF NOT EXISTS _migrations (
    version INTEGER PRIMARY KEY,
    description TEXT NOT NULL,
    applied_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

CREATE TABLE IF NOT EXISTS keys (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    key_type TEXT NOT NULL,              -- 'ed25519_master', 'ed25519_signing', 'x25519_encryption'
    public_key TEXT NOT NULL UNIQUE,      -- Base58-encoded public key
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    active INTEGER NOT NULL DEFAULT 1,

    CHECK(key_type IN ('ed25519_master', 'ed25519_signing', 'x25519_encryption'))
);

CREATE TABLE IF NOT EXISTS aliases (
    name TEXT PRIMARY KEY,               -- Alias name (e.g., 'alice')
    domain TEXT NOT NULL UNIQUE,         -- Full domain (e.g., 'alice.sovrn')
    delegates_to TEXT,                   -- Master pubkey hash if delegated
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    active INTEGER NOT NULL DEFAULT 1,

    FOREIGN KEY(delegates_to) REFERENCES keys(id)
);

CREATE INDEX idx_aliases_active ON aliases(active);

CREATE TABLE IF NOT EXISTS delegation_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    alias_name TEXT NOT NULL,
    delegate_key TEXT NOT NULL,          -- Public key being delegated to
    delegator_sig TEXT NOT NULL,         -- Signature proving delegation
    expires_at INTEGER,                 -- NULL = no expiry
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),

    FOREIGN KEY(alias_name) REFERENCES aliases(name)
);

CREATE TABLE IF NOT EXISTS domain_registrations (
    domain TEXT PRIMARY KEY,             -- e.g., 'alice.sovrn'
    public_key TEXT NOT NULL,             -- Owner's signing key
    registration_sig TEXT NOT NULL,       -- Proof of ownership
    dht_hash TEXT,                       -- DHT content hash for verification
    registered_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    expires_at INTEGER,                 -- NULL = perpetual

    FOREIGN KEY(public_key) REFERENCES keys(public_key)
);

CREATE TABLE IF NOT EXISTS device_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    device_id TEXT NOT NULL UNIQUE,       -- Device fingerprint
    device_name TEXT,                     -- 'Alice's Laptop'
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    last_active INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    active INTEGER NOT NULL DEFAULT 1
);

CREATE INDEX idx_sessions_active ON device_sessions(active);
```

---

## 4. presence.db — `/var/lib/sovrn/presence/presence.db`

```sql
-- V001_initial.sql
CREATE TABLE IF NOT EXISTS _migrations (
    version INTEGER PRIMARY KEY,
    description TEXT NOT NULL,
    applied_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

CREATE TABLE IF NOT EXISTS peer_status (
    public_key TEXT PRIMARY KEY,         -- Peer pubkey hash
    domain TEXT,                         -- .sovrn domain if known
    ygg_address TEXT,                    -- Yggdrasil IPv6 address
    status TEXT NOT NULL DEFAULT 'offline',  -- online, away, dnd, offline
    last_heartbeat INTEGER NOT NULL,     -- Last heartbeat timestamp
    latency_ms INTEGER,                  -- Round-trip time in ms
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),

    CHECK(status IN ('online', 'away', 'dnd', 'offline'))
);

CREATE INDEX idx_peer_status ON peer_status(status);
CREATE INDEX idx_peer_heartbeat ON peer_status(last_heartbeat DESC);

CREATE TABLE IF NOT EXISTS heartbeat_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_key TEXT NOT NULL,
    received_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    source TEXT NOT NULL DEFAULT 'direct',  -- direct, gossip, dht

    FOREIGN KEY(public_key) REFERENCES peer_status(public_key) ON DELETE CASCADE
);

CREATE INDEX idx_heartbeat_time ON heartbeat_log(received_at);

CREATE TABLE IF NOT EXISTS peer_discovery (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_key TEXT NOT NULL,
    discovered_via TEXT NOT NULL,        -- direct, gossip, dht, peer_list
    discovered_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    verified INTEGER NOT NULL DEFAULT 0,

    FOREIGN KEY(public_key) REFERENCES peer_status(public_key) ON DELETE CASCADE
);

CREATE INDEX idx_discovery_verified ON peer_discovery(verified);
```

---

## 5. cdn.db — `/var/lib/sovrn/cdn/cdn.db`

```sql
-- V001_initial.sql
CREATE TABLE IF NOT EXISTS _migrations (
    version INTEGER PRIMARY KEY,
    description TEXT NOT NULL,
    applied_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

CREATE TABLE IF NOT EXISTS push_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cid TEXT NOT NULL UNIQUE,            -- Content-addressed ID
    local_path TEXT NOT NULL,             -- Path to local file
    size_bytes INTEGER NOT NULL,
    checksum TEXT NOT NULL,               -- SHA-256 of file
    media_type TEXT,
    status TEXT NOT NULL DEFAULT 'queued',  -- queued, uploading, complete, failed
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    next_attempt INTEGER,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),

    CHECK(status IN ('queued', 'uploading', 'complete', 'failed'))
);

CREATE INDEX idx_push_status ON push_queue(status, next_attempt);

CREATE TABLE IF NOT EXISTS cdn_providers (
    id TEXT PRIMARY KEY,                 -- e.g., 'fastcdn'
    name TEXT NOT NULL,
    endpoint_url TEXT NOT NULL,
    price_per_gb REAL NOT NULL,
    regions TEXT NOT NULL,               -- JSON array: ["asia","eu","na"]
    api_key_encrypted TEXT,              -- Encrypted provider API key
    active INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS content_map (
    cid TEXT PRIMARY KEY,
    local_path TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    media_type TEXT,
    local_only INTEGER NOT NULL DEFAULT 1,  -- 1=not on CDN, 0=on CDN
    cdn_url TEXT,                            -- CDN URL if pushed
    cdn_provider TEXT,                        -- Provider ID
    pushed_at INTEGER,
    expires_at INTEGER,                       -- TTL or NULL

    FOREIGN KEY(cdn_provider) REFERENCES cdn_providers(id)
);

CREATE INDEX idx_content_local ON content_map(local_only);

CREATE TABLE IF NOT EXISTS subscription_status (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    provider_id TEXT NOT NULL,
    plan TEXT NOT NULL,
    storage_used_mb REAL NOT NULL DEFAULT 0,
    bandwidth_used_mb REAL NOT NULL DEFAULT 0,
    storage_limit_mb REAL NOT NULL,
    bandwidth_limit_mb REAL NOT NULL,
    active INTEGER NOT NULL DEFAULT 1,
    subscribed_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    expires_at INTEGER,

    FOREIGN KEY(provider_id) REFERENCES cdn_providers(id)
);
```

---

## 6. config.db — `/var/lib/sovrn/config/config.db`

```sql
-- V001_initial.sql
CREATE TABLE IF NOT EXISTS _migrations (
    version INTEGER PRIMARY KEY,
    description TEXT NOT NULL,
    applied_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,                -- JSON-encoded value
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

-- Default values inserted on first boot:
-- theme: "dark-auto", language: "en", mesh_explorer_visible: true,
-- homepage_editor_advanced: false, default_app_order: ["nextcloud","vaultwarden"],
-- memory_warning_threshold: 85

CREATE TABLE IF NOT EXISTS notification_prefs (
    key TEXT PRIMARY KEY,
    enabled INTEGER NOT NULL DEFAULT 1,
    quiet_hours_start TEXT,             -- "22:00" or NULL
    quiet_hours_end TEXT,              -- "08:00" or NULL
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

-- Keys: dm_notifications, follow_notifications, reaction_notifications,
--        mesh_status_notifications, app_update_notifications, backup_notifications

CREATE TABLE IF NOT EXISTS privacy_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

-- Keys: profile_visibility (public/followers_only/private),
--        online_status_visible (true/false),
--        allow_mesh_indexing (true/false),
--        cdn_push_enabled (true/false)

CREATE TABLE IF NOT EXISTS app_registry (
    app_id TEXT PRIMARY KEY,            -- e.g., 'nextcloud'
    name TEXT NOT NULL,
    tier INTEGER NOT NULL DEFAULT 1,    -- 1=curated, 2=community, 3=advanced
    version TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'available',  -- available, installing, installed, failed
    container_id TEXT,                  -- Podman container ID
    memory_limit_mb INTEGER,            -- cgroup memory limit
    install_date INTEGER,
    last_health_check INTEGER,
    auto_start INTEGER NOT NULL DEFAULT 1,
    port INTEGER,                       -- Internal port
    subdomain TEXT,                      -- e.g., 'nextcloud' for nextcloud.sovrn.local
    local_image INTEGER NOT NULL DEFAULT 0, -- 1=pre-baked on ISO

    CHECK(tier IN (1, 2, 3)),
    CHECK(status IN ('available', 'installing', 'installed', 'running', 'stopped', 'failed', 'memory_pressure'))
);

CREATE INDEX idx_app_status ON app_registry(status);

CREATE TABLE IF NOT EXISTS backup_metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    target_type TEXT NOT NULL,          -- usb, network, mesh
    target_path TEXT,                    -- Mount point or URL
    status TEXT NOT NULL DEFAULT 'pending',  -- pending, running, success, failed
    size_bytes INTEGER,
    duration_seconds INTEGER,
    started_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    completed_at INTEGER,

    CHECK(target_type IN ('usb', 'network', 'mesh', 'cloud'))
);
```

---

## 7. sovrnd.db — `/var/lib/sovrn/sovrnd/sovrnd.db`

```sql
-- V001_initial.sql
CREATE TABLE IF NOT EXISTS _migrations (
    version INTEGER PRIMARY KEY,
    description TEXT NOT NULL,
    applied_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

CREATE TABLE IF NOT EXISTS installed_apps (
    app_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    container_name TEXT NOT NULL,        -- Podman container name
    image TEXT NOT NULL,                 -- Container image reference
    port INTEGER NOT NULL,
    subdomain TEXT NOT NULL,
    memory_limit_mb INTEGER NOT NULL DEFAULT 512,
    auto_start INTEGER NOT NULL DEFAULT 1,
    status TEXT NOT NULL DEFAULT 'installing',
    installed_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    last_started_at INTEGER,
    restart_count INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_installed_status ON installed_apps(status);

CREATE TABLE IF NOT EXISTS app_health_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    app_id TEXT NOT NULL,
    status TEXT NOT NULL,                -- healthy, unhealthy, crashed, oom_killed
    memory_used_mb REAL,
    cpu_percent REAL,
    message TEXT,
    logged_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),

    FOREIGN KEY(app_id) REFERENCES installed_apps(app_id) ON DELETE CASCADE
);

CREATE INDEX idx_health_app_time ON app_health_log(app_id, logged_at DESC);

CREATE TABLE IF NOT EXISTS caddy_routes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    subdomain TEXT NOT NULL UNIQUE,      -- e.g., 'nextcloud' for nextcloud.sovrn.local
    upstream TEXT NOT NULL,               -- e.g., 'http://10.47.0.2:8080'
    app_id TEXT NOT NULL,
    tls INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    active INTEGER NOT NULL DEFAULT 1,

    FOREIGN KEY(app_id) REFERENCES installed_apps(app_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS service_state (
    service_name TEXT PRIMARY KEY,        -- e.g., 'sovrn-dht', 'sovrn-presence'
    status TEXT NOT NULL DEFAULT 'stopped',  -- running, stopped, failed, restarting
    pid INTEGER,
    started_at INTEGER,
    health_check_url TEXT,
    last_health_check INTEGER,
    restart_count INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_service_status ON service_state(status);
```

---

## 8. audit.db — `/var/lib/sovrn/audit/audit.db`

```sql
-- V001_initial.sql
CREATE TABLE IF NOT EXISTS _migrations (
    version INTEGER PRIMARY KEY,
    description TEXT NOT NULL,
    applied_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

CREATE TABLE IF NOT EXISTS audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    action TEXT NOT NULL,                 -- app_install, app_uninstall, app_start, app_stop,
                                         -- domain_register, settings_change, backup_trigger,
                                         -- identity_export, auth_login, auth_logout
    actor TEXT NOT NULL,                  -- 'user', 'sovrnd', 'system'
    target TEXT,                          -- What was acted upon (app_id, domain, etc.)
    details TEXT,                         -- JSON details
    ip_address TEXT,                      -- Always '127.0.0.1' for localhost
    timestamp INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

CREATE INDEX idx_audit_action ON audit_log(action);
CREATE INDEX idx_audit_time ON audit_log(timestamp DESC);

CREATE TABLE IF NOT EXISTS auth_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL,             -- login, logout, refresh, failed_attempt
    user_id TEXT,
    session_id TEXT,
    ip_address TEXT,
    user_agent TEXT,
    timestamp INTEGER NOT NULL DEFAULT (strftime('%s','now')),

    CHECK(event_type IN ('login', 'logout', 'refresh', 'failed_attempt'))
);

CREATE INDEX idx_auth_time ON auth_events(timestamp DESC);
CREATE INDEX idx_auth_user ON auth_events(user_id);

CREATE TABLE IF NOT EXISTS rate_limits (
    key TEXT PRIMARY KEY,                 -- 'ip:127.0.0.1' or 'user:a7x3k9...'
    request_count INTEGER NOT NULL DEFAULT 0,
    window_start INTEGER NOT NULL,
    blocked_until INTEGER
);

-- TTL cleanup: entries older than 2 minutes are pruned by sovrnd every 60 seconds
```

---

## Common Queries

### Feed: Get timeline for user (chronological, paginated)
```sql
SELECT e.* FROM events e
JOIN timeline_cache tc ON e.id = tc.event_id
WHERE tc.user_id = ? AND tc.rank <= ?
ORDER BY tc.rank DESC
LIMIT 50;
```

### Messages: Get unread count per conversation
```sql
SELECT peer_id, unread_count FROM conversations
WHERE unread_count > 0
ORDER BY last_at DESC;
```

### Identity: Get all active aliases for user
```sql
SELECT name, domain, delegates_to FROM aliases
WHERE active = 1;
```

### Apps: Get apps needing health check
```sql
SELECT app_id, name, status, last_started_at
FROM installed_apps
WHERE status IN ('running', 'installing')
  AND (last_started_at IS NULL OR last_started_at > ?);
```

### Audit: Get recent admin actions
```sql
SELECT action, target, details, timestamp FROM audit_log
WHERE action LIKE 'app_%' OR action = 'domain_register'
ORDER BY timestamp DESC
LIMIT 20;
```

### CDN: Get items pending push
```sql
SELECT cid, local_path, size_bytes, checksum, attempts
FROM push_queue
WHERE status = 'queued'
ORDER BY created_at ASC
LIMIT 10;
```