# Sovrn — Round 8 Decisions

## J1: Directory Structure → Accepted
- Service data: `/var/lib/sovrn/`
- User config/data: `/home/user/.sovrn/`
- Details:
  - `/var/lib/sovrn/dht/` — DHT node data
  - `/var/lib/sovrn/identity/` — key management
  - `/var/lib/sovrn/feed/` — social feed SQLite database
  - `/var/lib/sovrn/messages/` — message queue
  - `/var/lib/sovrn/presence/` — presence tracking data
  - `/var/lib/sovrn/cdn/` — CDN agent cache
  - `/home/user/.sovrn/config/` — user settings
  - `/home/user/.sovrn/backup/` — export directory
  - `/home/user/.sovrn/media/` — cached media
  - `/home/user/.sovrn/homepage/` — homepage files

## J2: Local API → REST (HTTP/JSON)
- PWA communicates with mesh services via REST API at sovrn.local/api/ (same origin as PWA, served through Caddy)
- Simpler to develop and debug
- Unix socket option available later if performance becomes an issue
- API schema documented at sovrn.local/api/docs (auto-generated)

## J3: Offline-First PWA → Yes
- All PWA apps work completely offline
- Service worker caches all app assets (HTML, CSS, JS)
- SQLite database (local) is always available
- Offline behavior:
  - Feed: view cached posts, compose drafts (posted when back online)
  - Messages: read old messages, compose drafts
  - Homepage: full edit capability, publish when back online
  - Settings: all settings available offline
- Sync status indicator in PWA header: "Online" / "Offline" / "Syncing..."

## J4: App Store Name → "Software Store"
- Simple, clear, no branding fluff
- Uses GNOME Software as the base application
- Renamed to "Software Store" in Sovrn