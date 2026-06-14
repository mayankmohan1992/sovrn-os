# Sovrn — System & UX Fixes

## D-U1: Systemd Boot Sequence & Service Dependencies

### Boot Order

```
TIMING  SERVICE                  DEPENDS ON                MANAGED BY
──────  ───────────────────────  ──────────────            ──────────
0s      systemd                   —                         systemd
0s      dbus                      systemd                   systemd
0.5s    NetworkManager            dbus                      systemd
1s      wpa_supplicant            NetworkManager            systemd
1s      nftables                  NetworkManager            systemd
2s      Yggdrasil                 NetworkManager            systemd (apt)
3s      sovrn-dns-resolver        Yggdrasil                 systemd (apt)
4s      sovrn-dht                 Yggdrasil                 sovrnd
4s      sovrn-identity            —                         sovrnd
4s      sovrn-presence            sovrn-dht                 sovrnd
5s      sovrn-message-queue       sovrn-identity            sovrnd
5s      sovrn-feed                sovrn-message-queue       sovrnd
5s      sovrn-cdn-agent           Yggdrasil                 sovrnd
3s      zram-setup                —                         systemd (apt)
3s      systemd-oomd              —                         systemd (apt)
3s      Caddy                     NetworkManager            systemd (apt)
5s      sovrnd                    Caddy + Yggdrasil         systemd (apt)
8s      sovrn-notify-bridge       —                         systemd (apt)
10s     GNOME Display Manager     Caddy (for PWA theme)     systemd
10s     Podman containers         sovrnd (healthcheck)      sovrnd
```

### Systemd Unit Dependencies

```ini
# sovrn-dht.service
After=network-online.target yggdrasil.service
Wants=yggdrasil.service
Requires=network-online.target

# sovrn-identity.service
After=network-online.target
Requires=network-online.target

# sovrn-presence.service
After=sovrn-dht.service
Requires=sovrn-dht.service

# sovrn-message-queue.service
After=sovrn-identity.service
Requires=sovrn-identity.service

# sovrn-feed.service
After=sovrn-message-queue.service
Requires=sovrn-message-queue.service

# sovrnd.service
After=caddy.service yggdrasil.service
Wants=caddy.service yggdrasil.service
Requires=network-online.target

# Each Podman app container starts AFTER sovrnd is healthy
# sovrnd exposes /health endpoint; systemd checks it before starting containers
```

### sovrnd Responsibilities

| Service | Managed By | Why |
|---------|-----------|-----|
| Yggdrasil | systemd (apt) | Must start before sovrnd, no dependency on sovrnd |
| Caddy | systemd (apt) | Must start before sovrnd so PWA is accessible |
| zram, oomd | systemd (apt) | System-level, started early |
| DHT, Presence, Identity, Feed, Messages, CDN | sovrnd | sovrnd starts and monitors these as child processes or systemd units |
| Podman containers | sovrnd | App lifecycle management is sovrnd's core job |
| Notify bridge | systemd (apt) | Simple bridge, no dependency on sovrnd |

sovrnd monitors all services it manages. If a service crashes, sovrnd restarts it. If sovrnd crashes, systemd restarts sovrnd, which then reconciles service state.

## D-U2: First Boot Desktop Experience

### What the User Sees After OOBE

```
┌──────────────────────────────────────────────────────────────┐
│  GNOME Desktop with Sovrn Customization                       │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │                                                         │  │
│  │   ┌─────────────────────────────────────────────────┐  │  │
│  │   │                                                   │  │  │
│  │   │   Welcome to Sovrn, Alice! 🎉                   │  │  │
│  │   │                                                   │  │  │
│  │   │   Your node is online. You're connected to        │  │  │
│  │   │   47 peers on the Sovrn mesh.                    │  │  │
│  │   │                                                   │  │  │
│  │   │   ┌──────────┐  ┌──────────┐  ┌──────────┐      │  │  │
│  │   │   │ 📡 Feed   │  │ 💬 Messages│  │ 🏠 Homepage│      │  │  │
│  │   │   └──────────┘  └──────────┘  └──────────┘      │  │  │
│  │   │                                                   │  │  │
│  │   │   ┌──────────┐  ┌──────────┐  ┌──────────┐      │  │  │
│  │   │   │ 🔑 Identity│  │ 🔐 Vault  │  │ ⚙️ Settings │      │  │  │
│  │   │   └──────────┘  └──────────┘  └──────────┘      │  │  │
│  │   │                                                   │  │  │
│  │   │              [ Open Sovrn Hub ]                   │  │  │
│  │   │                                                   │  │  │
│  │   └─────────────────────────────────────────────────┘  │  │
│  │                                                         │  │
│  └─────────────────────────────────────────────────────────┘  │
│                                                               │
│  ┌──────┐                                                     │
│  │ Files │  🦊 Firefox  │  📡 Sovrn Hub  │  ⚙ Settings  │    │  ← GNOME Dock
│  └──────┘                                                     │
└──────────────────────────────────────────────────────────────┘
```

### GNOME Customizations

- **Wallpaper**: Sovrn-branded (Sovereign Teal gradient with subtle mesh pattern)
- **Lock screen**: Sovrn logo + user's display name
- **Dock** (dash-to-dock extension, bottom, always visible):
  - Files (Nautilus)
  - Firefox
  - Sovrn Hub (primary PWA interface)
  - Settings
  - Terminal (for power users, in app grid not dock)
- **Default new tab / homepage**: `https://sovrn.local/` (Sovrn Hub)
- **Removed apps**: Cheese, Maps, Contacts, Weather — ships lean
- **Added .desktop entries**: `sovrn-hub.desktop` (opens `https://sovrn.local/` in Firefox as web app)
- **Autostart**: Sovrn Hub opens in Firefox web-app mode on login
- **Top bar**: Sovrn mesh status icon (🟢 Connected / 🔴 Offline) + notification count

### Sovrn Hub PWA

Sovrn Hub is NOT a separate app — it's the main Preact PWA at `https://sovrn.local/`. It's the single interface for:

| Section | What It Shows |
|---------|--------------|
| Feed | Social feed (kind 1/2 events) |
| Messages | DMs (kind 7 events) |
| Homepage | Visual editor for user's `username.sovrn` page |
| Identity | Profile, QR code, aliases, follow list |
| Apps | Self-hosted app management (install, start, stop, health) |
| Settings | Mesh config, backup, domain registration, display preferences |

### First-Login Welcome (3 steps, then dismissed)

1. "Your Sovrn identity is ready. Share your QR code to let others find you."
2. "Sovrn Hub is your control center. Open it anytime from the dock."
3. "Back up your identity recovery words. [Take me there]" → links to Identity > Backup

After step 3, the welcome overlay dismisses and shows the standard Sovrn Hub dashboard.

## D-U3: Seed Phrase Backup UX

### The Name

Never call it "seed phrase" or "recovery key." User-facing terms:

| Technical Term | User-Facing Term |
|---------------|-----------------|
| Seed phrase | **Identity Recovery Words** |
| Ed25519 keypair | **Your Sovrn Identity** |
| Public key hash | **Your unique ID** |
| Encryption key | _(hidden from user)_ |

### OOBE Backup Flow (Enforced)

```
Screen 4: Identity Recovery Words

┌─────────────────────────────────────────────────────┐
│                                                      │
│   🔐 Your Identity Recovery Words                   │
│                                                      │
│   These 24 words are the ONLY way to recover         │
│   your Sovrn identity if your device is lost          │
│ or damaged.                                          │
│                                                      │
│   No company — not even Sovrn — can reset this       │
│   for you. If you lose these words, you lose         │
│   your identity, your domain, and all your data.     │
│                                                      │
│   ┌─────────────────────────────────────────────┐   │
│   │  1. horizon    7. temple    13. river   19. flame │
│   │  2. silver     8. quarter  14. budget  20. craft │
│   │  3. guitar     9. endorse  15. gallery 21. exact │
│   │  4.Verify     10. urban    16. обычай  22. frown │
│   │  5. ________  11. _______  17. _______  23. ____ │
│   │  6. ________  12. _______  18. _______  24. ____ │
│   └─────────────────────────────────────────────────┘ │
│                                                      │
│   □ I have written these down on paper              │
│                                                      │
│          [ Verify 4 Words ]     [ Copy to Clipboard ] │
│                                                      │
└─────────────────────────────────────────────────────┘
```

- Words are revealed in groups of 6 (4 taps to reveal all 24)
- "Copy to Clipboard" copies to clipboard for 30 seconds, then clears
- After revealing all words, user checks "I have written these down on paper"
- **Verification step**: User must type back 4 random words from their phrase
- Cannot proceed until verification succeeds

### Backup Verification (Persistent)

```
After OOBE, on every boot for the first 5 boots:

┌─────────────────────────────────────────┐
│                                          │
│  ⚠️ Your Identity Recovery Words         │
│     are not backed up yet.               │
│                                          │
│  If you lose your device, you lose       │
│  everything — your identity, domain,     │
│  messages, and files.                    │
│                                          │
│  [ Back Up Now ]  [ Remind Me Later ]   │
│                                          │
└─────────────────────────────────────────┘
```

After 5th dismiss, changes to a yellow shield icon in top bar instead.

### 3 Backup Paths

| Path | Security | UX |
|------|----------|-----|
| Paper | High (physical) | OOBE encourages this as primary |
| USB Drive | Medium (encrypted file) | Sovrn offers to save encrypted `sovrn-recovery.enc` to mounted USB |
| Mesh Shards (v2) | High (Shamir) | Split seed into 3-of-5 shards, store on trusted mesh peers |

### Seed Phrase Storage (Clarification)

| What | Stored? | Where | Encrypted? |
|------|---------|-------|-----------|
| Seed phrase (24 words) | NO | Never stored digitally | N/A |
| Master keypair (derived from seed) | YES | OS keyring (GNOME Keyring / libsecret) | Encrypted with user's login password |
| App passwords (derived from master key) | NO | Derived on-the-fly via HKDF | N/A |

The seed phrase is shown during OOBE, verified, and then discarded from memory. The master keypair persists encrypted in the OS keyring, unlocked by login password. App passwords are never stored — they're deterministically derived.

## D-U4: Pre-Baked Container Images & Local Registry

### Problem

Docker Hub / ghcr.io pulls are slow and unreliable in India (5-20 Mbps, rate-limited). A "1-click install" that takes 45 minutes feels broken.

### Solution: Local OCI Image Registry on ISO

```
Sovrn ISO (64 GB target)
├── OS image (~10 GB)
├── Pre-baked OCI images (~8 GB):
│   ├── nextcloud:33-latest
│   ├── vaultwarden:1.34-latest
│   ├── mariadb:10.6 (Nextcloud dep)
│   ├── redis:6 (Nextcloud dep)
│   ├── freshrss:1.29-latest
│   ├── wallabag:latest
│   ├── adguardhome:latest
│   ├── navidrome:0.61-latest
│   └── sovrn-app-proxy:1.0
├── Swap + zram config (~4 GB effective)
└── User data partition (remaining)
```

### Install Flow with Local Images

```
1. User clicks "Install Nextcloud" in Sovrn Hub
2. sovrnd checks: Is nextcloud:33-latest in local cache?
   YES → podman load < /var/lib/sovrn/images/nextcloud.tar
          (takes 15-30 seconds, no network)
   NO  → podman pull ghcr.io/.../nextcloud:33
          (takes 5-45 minutes depending on connection)
3. sovrnd creates container, starts it, runs health check
4. sovrnd runs provisioning adapter (creates admin account)
5. sovrnd adds Caddy route: nextcloud.sovrn.local → container IP
6. Sovrn Hub shows "Nextcloud is ready! [Open]"
7. User clicks Open → already logged in via auth proxy
```

### UX for Image Pulling

```
┌──────────────────────────────────────────────┐
│                                               │
│  Installing Nextcloud                         │
│                                               │
│  ███████████████████████░░░░░  78%            │
│                                               │
│  Loading from local cache...                  │
│  Estimated time: 15 seconds                   │
│                                               │
│  [ Cancel ]                                   │
│                                               │
└──────────────────────────────────────────────┘

If pulling from network:
┌──────────────────────────────────────────────┐
│                                               │
│  Installing Jellyfin                          │
│                                               │
│  ████████░░░░░░░░░░░░░░░░░░  22%            │
│                                               │
│  Downloading from registry...                 │
│  112 MB of 490 MB                             │
│  Estimated time: 12-18 minutes                 │
│                                               │
│  □ Install in background                      │
│                                               │
│  [ Cancel ]  [ Install Later ]               │
│                                               │
└──────────────────────────────────────────────┘
```

- Real progress bar with estimated time
- "Install in background" checkbox lets user continue using Sovrn Hub
- Failed pulls auto-retry 3 times with exponential backoff
- "Install Later" schedules install for next time device is on WiFi + charging

### India-Aware Registry Strategy

- Primary: ghcr.io (GitHub Container Registry, good CDN)
- Mirror: registry.sovrn.org (seed nodes host popular images, updated weekly)
- Local cache: /var/lib/sovrn/images/ (on-disk, from ISO)
- Fallback: Docker Hub
- Future (v2): IPFS-based image distribution via Yggdrasil mesh

## D-U5: App Crash Recovery & Health Dashboard

### sovrn-app-monitor Service

A systemd service that watches all Sovrn-managed Podman containers:

```ini
# sovrn-app-monitor.service
[Unit]
Description=Sovrn App Health Monitor
After=sovrnd.service podman.service

[Service]
Type=simple
ExecStart=/usr/bin/sovrn-app-monitor
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

### Health Monitoring Logic

```python
# Simplified sovrn-app-monitor logic
for app in installed_apps:
    container = podman.get_container(app.id)
    
    if container.is_healthy():
        app.status = "running"
    elif container.is_restarting():
        app.status = "restarting"
        notify(f"{app.name} is restarting...", severity="warning")
    elif container.is_oom_killed():
        app.status = "memory_pressure"
        notify(f"{app.name} was paused to save memory. Close other apps and restart it.", 
               severity="warning", action="restart")
    elif container.is_crashed():
        retry_count = get_retry_count(app.id)
        if retry_count < 3:
            podman.restart(app.id)
            increment_retry_count(app.id)
        else:
            app.status = "failed"
            notify(f"{app.name} has stopped unexpectedly. [Restart] [View Details]",
                   severity="error")
```

### User-Facing Health Dashboard

```
┌──────────────────────────────────────────────────────┐
│  Sovrn Hub → Apps                                    │
│                                                       │
│  ┌──────────────────────────────────────────────┐    │
│  │  🟢 Nextcloud          Running  1.2 GB RAM   │    │
│  │  🟢 Vaultwarden        Running  80 MB RAM    │    │
│  │  🟡 FreshRSS            Restarting...         │    │
│  │  ⚫ Jellyfin            Stopped  [▶ Start]   │    │
│  └──────────────────────────────────────────────┘    │
│                                                       │
│  💡 Memory: 4.2 GB used of 8 GB (52%)               │
│  💡 Tip: Stop unused apps to free memory             │
│                                                       │
│  ┌──────────────────────────────────────────────┐    │
│  │  ⚠️ FreshRSS has restarted 2 times today     │    │
│  │     Reason: Out of memory                     │    │
│  │                                               │    │
│  │  [ View Logs ]  [ Restart ]  [ Uninstall ]   │    │
│  └──────────────────────────────────────────────┘    │
│                                                       │
│  [ + Install More Apps ]                             │
│                                                       │
└──────────────────────────────────────────────────────┘
```

### Desktop Notifications via Notify Bridge

- Container crashed → `sovrn-notify "Jellyfin has stopped. Tap to restart."`
- Memory pressure (85%+) → `sovrn-notify "Memory is running low. Some apps may be paused."`
- App installed → `sovrn-notify "Nextcloud is ready! Tap to open."`
- Update available → `sovrn-notify "Nextcloud 34 is available. Update now?"`
- All notifications route through the same `localhost:54773/notify` bridge

## D-U6: Contradiction Resolutions

### CDN Revenue Model (resolved)

**Document 01 says "no cut by default", Document 06 says "5-10% commission".**

Resolution: These are NOT contradictory. The correct reading is:

- CDN providers set their own prices and users pay them directly
- When a user pays through Sovrn's built-in payment system (UPI/card/crypto), Sovrn takes a 5-10% facilitation fee
- If a user pays a CDN provider directly (outside Sovrn), Sovrn takes 0%
- Free tier content hosting: Sovrn takes nothing

This is the same model as app stores — developers charge what they want, the platform takes a cut of transactions it processes. Document 01's "no cut" refers to Sovrn not charging for basic mesh services (DNS, messaging, feed).

### Seed Phrase vs Key Derivation (resolved)

**Document 04 says "seed phrase NEVER stored digitally", Document 19 says "derived from Sovrn seed."**

Resolution:

- The **seed phrase** (24 words) is NEVER stored in plaintext — this is correct
- The **master keypair** (Ed25519 private key) IS stored encrypted in the OS keyring, unlocked by login password
- App passwords are **derived** from the master key via HKDF, never stored
- If the OS keyring is lost (device failure), the seed phrase is entered to regenerate the master key
- HKDF derivation ensures: same seed → same master key → same app passwords every time

This is consistent: the 24-word phrase is mnemonic-only (shown once in OOBE, then discarded from memory). The derived key lives encrypted. No contradiction.

### Self-Hosted Apps on Mesh (resolved)

Self-hosted apps are **local-only by default**. User can opt-in to share specific apps on the mesh via Sovrn Hub settings. See doc 21 (D-S5) for full details.