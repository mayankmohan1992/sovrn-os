# Sovrn — PWA Framework & Notifications Decision

## K1: NOTIFICATIONS → REST API to notify-send bridge

Architecture:
```
PWA → HTTP POST localhost:54773/notify → notify-send → D-Bus → GNOME Shell
```

- Tiny Python HTTP server (50 LOC) bound to 127.0.0.1:54773
- Calls system `notify-send` which talks to GNOME Shell via D-Bus
- Runs as systemd service with `Restart=always`
- PWA falls back to Web Notification API if bridge is unreachable
- Native GNOME notifications: proper DND support, lock screen, urgency levels
- Zero browser permission issues — no HTTPS requirement, no user gesture needed
- 3 dependencies: Python 3 (pre-installed), libnotify-bin (pre-installed), systemd (pre-installed)

Port: 54773 (adjacent to PWA port 54772, both in IANA private range)

## K2: PWA FRAMEWORK → Preact + preact/signals

Why Preact over alternatives:

| Criterion | Preact | Solid.js | Svelte | Vanilla JS | Lit |
|---|---|---|---|---|---|
| Security (supply chain) | A- | A- | B+ | A+ | B+ |
| Privacy (telemetry) | A | A | B | A+ | A- |
| Speed | A | A+ | A+ | A+ | A |
| Maintainability | A- | B+ | B+ | C+ | B |
| Ecosystem | A- | C+ | B+ | D | C |
| **Overall** | **A** | **A-** | **B+** | **B** | **B** |

Key reasons:
1. 3KB core, 3 packages in dependency tree — minimal attack surface
2. Zero telemetry, MIT licensed, no corporate surveillance incentive
3. React ecosystem compatibility via preact/compat (95% of React libs work)
4. preact/signals for fine-grained reactivity (fast feed updates)
5. Small team can maintain it — if you know React, you know Preact
6. Service worker + Workbox for offline, Dexie.js for IndexedDB

PWA architecture:
```
src/
  components/     # Preact functional components + signals
  state/          # preact/signals stores (user, feed, messages)
  db/             # IndexedDB (Dexie.js) for offline data
  sw/             # Service worker (Workbox)
  utils/          # Sync queue, conflict resolution, notifications
  app.tsx         # Root component, router
```

## K3: DATABASE → Separate SQLite files per service

| Service | Database | Location |
|---|---|---|
| Social feed | feed.db | /var/lib/sovrn/feed/ |
| Messages | messages.db | /var/lib/sovrn/messages/ |
| Identity | identity.db | /var/lib/sovrn/identity/ |
| Presence | presence.db | /var/lib/sovrn/presence/ |
| CDN cache | cdn.db | /var/lib/sovrn/cdn/ |
| User settings | config.db | /home/user/.sovrn/config/ |

Benefits: Corruption in one service doesn't affect others. Easy to back up individual services. Easy to clear cache for one service without wiping everything.

## K4: THEME → Custom Sovrn design system (see 17-DESIGN-SYSTEM.md)