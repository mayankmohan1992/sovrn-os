# Sovrn OS

A privacy-first, decentralized operating system built on Linux, Rust, Python, Go, and Preact.

## Architecture

- **Linux** (Debian Trixie base) - kernel, systemd, GNOME desktop
- **Rust** mesh services - DHT, Identity, Presence, Feed, Message Queue, OOBE
- **Python** services - sovrnd (orchestrator), Auth, Monitor, Notify Bridge
- **Go** CDN agent - content delivery to Sia/IPFS/Filecoin
- **Preact PWA** - Sovrn Hub web interface
- **GTK4/libadwaita** - OOBE first-boot wizard

## Quick Start

```bash
# Build all components
./scripts/build.sh all

# Build ISO (requires Docker on macOS)
./build/build-iso.sh

# Or use Debos directly (Linux only)
debos build/debos-sovrn.yaml --architecture amd64
```

## Ports

| Service | Port | Protocol |
|---------|------|----------|
| sovrnd | 54771 | HTTP/REST |
| PWA Hub | 54772 | HTTP |
| Notify Bridge | 54773 | HTTP |
| Auth | 54776 | HTTP |
| Monitor | 54777 | HTTP |
| DHT mesh | 54774 | TCP/UDP |
| Presence | 54775 | UDP |
| Yggdrasil | 30550 | TCP/UDP |

## Directory Structure

```
sovrn-os/
  src/
    sovrn-dht/          # Rust - DHT service
    sovrn-identity/     # Rust - Identity service
    sovrn-presence/     # Rust - Presence service
    sovrn-feed/         # Rust - Feed service
    sovrn-message-queue/ # Rust - Message Queue service
    sovrn-cdn-agent/   # Go - CDN agent
    sovrnd/             # Python - Central orchestrator
    sovrn-auth/         # Python - Policy engine
    sovrn-monitor/      # Python - System monitor
    sovrn-notify-bridge/ # Python - Notification bridge
    oobe/               # Rust/GTK4 - First-boot wizard
    pwa/                # Preact - Web hub
    caddy-auth/         # Go - Caddy auth plugin
    systemd-units/      # Systemd service files
    caddy-config/       # Caddy reverse proxy config
    nftables/           # Firewall rules
    dns/                # Unbound DNS config
    yggdrasil/          # Yggdrasil mesh config
    apparmor/           # AppArmor profiles
    gnome-customization/ # dconf/gschema overrides
  build/                # Build output and ISO recipe
  packaging/            # Debian packaging
  scripts/              # Build and utility scripts
```

## License

AGPL-3.0-or-later
