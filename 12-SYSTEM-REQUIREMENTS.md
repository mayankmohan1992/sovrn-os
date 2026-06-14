# Sovrn — System Requirements & Default Apps

## System Requirements

### Minimum (Functional but constrained)
| Resource | Minimum | Notes |
|---|---|---|
| RAM | 4 GB | GNOME + services idle at ~1.5 GB, leaves 2.5 GB for apps |
| Storage | 32 GB | OS ~10 GB, swap ~4 GB, user data ~18 GB |
| CPU | x86_64, 2 cores | Dual-core Celeron/Pentium level |
| Network | Any internet connection | WiFi or Ethernet, Yggdrasil works over any |
| Boot | UEFI (recommended) or BIOS | UEFI for Secure Boot support |

### Recommended (Comfortable)
| Resource | Recommended | Notes |
|---|---|---|
| RAM | 8 GB | Smooth GNOME, room for browser tabs + apps |
| Storage | 64 GB+ | Room for media cache, CDN staging, user files |
| CPU | x86_64, 4+ cores | Modern i3/Ryzen 3 or better |
| Network | 10+ Mbps | For mesh content and regular internet |

### Idle Resource Usage (Target)
| Resource | Target | Notes |
|---|---|---|
| RAM | ~1.5 GB | GNOME + Yggdrasil + Caddy + mesh services |
| CPU | < 5% | Services mostly idle, polling on heartbeat schedule |
| Disk I/O | Negligible | SQLite writes are minimal during idle |
| Network | ~10 KB/min | Heartbeat gossip + presence updates |

### Why These Numbers
- GNOME Shell: ~800 MB RAM
- Yggdrasil daemon: ~30 MB RAM
- Caddy web server: ~15 MB RAM
- Mesh services (DNS, identity, presence, message-queue): ~100 MB total
- Systemd + journal + networking: ~200 MB RAM
- Remaining RAM available for user apps, browser tabs, etc.

## Default Applications

### Pre-installed (No Games)
| Category | Application | Why |
|---|---|---|
| File manager | GNOME Files (Nautilus) | Standard, simple |
| Text editor | GNOME Text Editor | Lightweight, not VS Code |
| Terminal | GNOME Console | For users who need it, hidden from default dock |
| Calculator | GNOME Calculator | Basic and scientific modes |
| Media player | GNOME Videos (Totem) + GNOME Music | Local audio/video playback |
| Image viewer | GNOME Image Viewer (Loupe) | View photos |
| Screenshot | GNOME Screenshot | Built into GNOME Shell |
| Archive manager | File Roller | ZIP, TAR, etc. |
| Settings | GNOME Settings + Sovrn Settings | OS settings + mesh settings |
| Browser | Firefox + Chromium | Regular internet browsing |
| Sovrn PWA | Feed, Messages, Homepage Editor, Identity | All mesh services |
| Disk utility | GNOME Disks | Partition management |
| System monitor | GNOME System Monitor | Resource usage |

### NOT included (by design)
- No games (not even Solitaire/Mahjong)
- No email client (mesh messaging replaces this in v1)
- No office suite (users can install LibreOffice via flatpak if needed)
- No video editor (too heavy, niche use case)
- No IDE (users can install VS Code if needed)

### Installable via App Center (GNOME Software)
- LibreOffice, GIMP, OBS Studio, VS Code, Steam, etc.
- All via flatpak (sandboxed) or apt (native)
- Users choose what to add