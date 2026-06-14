# Sovrn — Round 4 Decisions

## F1: Multi-Device → One identity, multiple devices, simultaneous
- Same keypair and identity active on multiple devices at once
- All devices see the same feed, messages, content
- Seamless handoff between devices
- Technical implementation: each device runs a full Sovrn node, all share the same Ed25519 keypair
- Devices synchronize state via the mesh (events, messages, follow lists are all content-addressed and signed)
- Conflict resolution: if two devices post simultaneously, timestamps resolve order (or merge if no conflict)

## F2: Blocking → Bidirectional, network-wide
- If A blocks B: B cannot see A's content, A cannot see B's content
- Block event is signed and propagated to the mesh
- Both nodes stop serving content to the other
- Gotchas addressed below (see 11-BLOCKING-DESIGN.md)

## F3: Notifications → Fine-grained control
- Unified notification center in the PWA
- Per-service controls: enable/disable all notifications from Feed, Messages, etc.
- Per-type controls within each service:
  - Feed: new post from followee, reaction on your post, new follower
  - Messages: new DM, group message
  - System: mesh connectivity, update available, CDN status
- Desktop notifications via GNOME notification daemon
- Sound alerts configurable per type
- Do Not Disturb mode

## F4: Desktop → Default GNOME, Sovrn wallpaper, pre-configured
- Stock GNOME with pre-configured settings:
  - Privacy-focused: telemetry off, location off, search off
  - Pre-installed: Firefox, Chromium, Sovrn PWA in dock
  - Sovrn wallpaper and lock screen image
- No custom GNOME Shell theme, no custom icons
- This keeps maintenance burden low and updates easy

## F5: File Sharing → Yes, supported
- DMs: attach files up to 25 MB per message (encrypted point-to-point)
- Homepage: host arbitrary files on your node (PDFs, code, documents)
- Posts: attach files alongside text (like email attachments)
- Large files (>25 MB): CDN hosted, link in message

## F6: Target Audience → General public
- All UI decisions prioritize clarity and simplicity over power-user features
- No terminal required for any core functionality
- Keyboard shortcuts exist but are never required
- Error messages in plain language, not technical jargon
- Setup should be completable by anyone who can install an OS