#!/usr/bin/env bash
# Sovrn ISO Builder
# Builds a bootable ISO image from the project sources.
#
# Prerequisites:
#   macOS:  Docker Desktop + debos Docker image
#   Linux:  debos (apt install debos) or Docker
#
# Usage:
#   ./build-iso.sh                               # Full build (CI recipe)
#   ./build-iso.sh --quick                       # Skip compile, use pre-built binaries
#   ./build-iso.sh --docker                      # Force Docker mode (macOS default)
#   ./build-iso.sh --recipe <yaml>               # Use specific debos recipe

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_DIR/build"
OVERLAY_DIR="$BUILD_DIR/overlays"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() { echo -e "${GREEN}[sovrn-build]${NC} $*"; }
warn() { echo -e "${YELLOW}[sovrn-build WARN]${NC} $*" >&2; }
err() { echo -e "${RED}[sovrn-build ERROR]${NC} $*" >&2; exit 1; }

OS="$(uname -s)"
ARCH="$(uname -m)"
USE_DOCKER=false
QUICK=false
RECIPE="debos-sovrn-ci.yaml"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --docker) USE_DOCKER=true; shift ;;
        --quick) QUICK=true; shift ;;
        --recipe) RECIPE="$2"; shift 2 ;;
        --help) echo "Usage: $0 [--docker] [--quick] [--recipe <yaml>]"; exit 0 ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

if [ "$OS" = "Darwin" ]; then
    USE_DOCKER=true
    log "macOS detected - using Docker for Debos"
fi

# Step 1: Build all components
if [ "$QUICK" = false ]; then
    log "Building all Sovrn OS components..."
    bash "$PROJECT_DIR/scripts/build.sh" all || err "Build failed"
fi

# Step 2: Prepare overlay directories
log "Preparing overlay directories..."
mkdir -p "$OVERLAY_DIR"/{etc,systemd,pwa-dist,gnome,scripts}
mkdir -p "$OVERLAY_DIR/bin/usr/bin"

for bin in sovrn-dht sovrn-identity sovrn-presence sovrn-feed sovrn-message-queue sovrn-cdn-agent; do
    if [ -f "$BUILD_DIR/bin/$bin" ]; then
        cp "$BUILD_DIR/bin/$bin" "$OVERLAY_DIR/bin/usr/bin/"
    else
        warn "Binary not found: $bin"
    fi
done
# caddy is installed via apt inside debos, not from build/bin/

mkdir -p "$OVERLAY_DIR/systemd/etc/systemd/system"
cp "$PROJECT_DIR"/src/systemd-units/*.service "$OVERLAY_DIR/systemd/etc/systemd/system/"
cp "$PROJECT_DIR"/src/systemd-units/*.target "$OVERLAY_DIR/systemd/etc/systemd/system/"

mkdir -p "$OVERLAY_DIR/etc/etc/sovrn"
mkdir -p "$OVERLAY_DIR/etc/etc/caddy"
mkdir -p "$OVERLAY_DIR/etc/etc/nftables"
mkdir -p "$OVERLAY_DIR/etc/etc/unbound"
mkdir -p "$OVERLAY_DIR/etc/etc/sovrn/ca"

cp "$PROJECT_DIR"/src/caddy-config/Caddyfile "$OVERLAY_DIR/etc/etc/caddy/"
# Strip the sovrn_auth block — vanilla Caddy (from Debian) doesn't have the custom auth plugin
# The auth plugin requires xcaddy which is only available on Linux
sed '/    # Auth plugin/,/    }/d' "$OVERLAY_DIR/etc/etc/caddy/Caddyfile" > "$OVERLAY_DIR/etc/etc/caddy/Caddyfile.tmp" && \
    mv "$OVERLAY_DIR/etc/etc/caddy/Caddyfile.tmp" "$OVERLAY_DIR/etc/etc/caddy/Caddyfile"
cp "$PROJECT_DIR"/src/nftables/sovrn.nft "$OVERLAY_DIR/etc/etc/nftables/"
cp "$PROJECT_DIR"/src/dns/unbound-sovrn.conf "$OVERLAY_DIR/etc/etc/unbound/sovrn.conf"
cp "$PROJECT_DIR"/src/yggdrasil/yggdrasil.conf "$OVERLAY_DIR/etc/etc/sovrn/"
cp "$PROJECT_DIR"/src/sovrn-dht/config/dht.toml "$OVERLAY_DIR/etc/etc/sovrn/" 2>/dev/null || true
cp "$PROJECT_DIR"/src/sovrn-identity/config/identity.toml "$OVERLAY_DIR/etc/etc/sovrn/" 2>/dev/null || true
cp "$PROJECT_DIR"/src/sovrn-presence/config/presence.toml "$OVERLAY_DIR/etc/etc/sovrn/" 2>/dev/null || true
cp "$PROJECT_DIR"/src/sovrn-feed/config/feed.toml "$OVERLAY_DIR/etc/etc/sovrn/" 2>/dev/null || true
cp "$PROJECT_DIR"/src/sovrn-message-queue/config/message-queue.toml "$OVERLAY_DIR/etc/etc/sovrn/" 2>/dev/null || true
cp "$PROJECT_DIR"/src/sovrn-cdn-agent/configs/cdn-agent.toml "$OVERLAY_DIR/etc/etc/sovrn/" 2>/dev/null || true

cat > "$OVERLAY_DIR/etc/etc/sovrn/sovrnd.toml" <<'EOF'
[server]
host = "127.0.0.1"
port = 54771
log_level = "info"

[auth]
jwt_algorithm = "HS256"
jwt_expiry_hours = 24

[paths]
sockets_dir = "/var/lib/sovrn/sockets"
data_dir = "/var/lib/sovrn/sovrnd"
EOF

mkdir -p "$OVERLAY_DIR/pwa-dist/var/lib/sovrn/pwa-dist"
if [ -d "$BUILD_DIR/share/pwa-dist" ]; then
    cp -r "$BUILD_DIR/share/pwa-dist/"* "$OVERLAY_DIR/pwa-dist/var/lib/sovrn/pwa-dist/" 2>/dev/null || true
fi

cp "$PROJECT_DIR"/src/gnome-customization/dconf-profile "$OVERLAY_DIR/gnome/etc/dconf/profile/sovrn" 2>/dev/null || \
    mkdir -p "$OVERLAY_DIR/gnome/etc/dconf/profile" && \
    cp "$PROJECT_DIR"/src/gnome-customization/dconf-profile "$OVERLAY_DIR/gnome/etc/dconf/profile/sovrn"
cp "$PROJECT_DIR"/src/gnome-customization/dconf-db-sovrn.ini "$OVERLAY_DIR/gnome/etc/dconf/db/sovrn" 2>/dev/null || \
    mkdir -p "$OVERLAY_DIR/gnome/etc/dconf/db" && \
    cp "$PROJECT_DIR"/src/gnome-customization/dconf-db-sovrn.ini "$OVERLAY_DIR/gnome/etc/dconf/db/sovrn"

mkdir -p "$OVERLAY_DIR/etc/etc/sovrn/scripts"
cp "$PROJECT_DIR"/scripts/bootstrap-ca.sh "$OVERLAY_DIR/etc/etc/sovrn/scripts/"
cp "$PROJECT_DIR"/src/nftables/sovrn.nft "$OVERLAY_DIR/etc/etc/sovrn/scripts/" 2>/dev/null || true
chmod +x "$OVERLAY_DIR/etc/etc/sovrn/scripts/bootstrap-ca.sh"

# Prepare Python dist overlay (final installed locations, no pip needed)
log "Preparing Python dist overlay..."
PYDIST="$OVERLAY_DIR/python-dist"
mkdir -p "$PYDIST/usr/lib/python3/dist-packages"
mkdir -p "$PYDIST/usr/bin"

# Map source dir names → Python module names (hyphen → underscore)
install_python_pkg() {
    local src_dir="$1"
    local pkg_name="$2"
    local py_module="$3"   # Python import name (underscore form)
    local bin_name="$4"    # wrapper script name

    if [ ! -d "$src_dir" ]; then
        warn "Python package source not found: $src_dir"
        return
    fi

    # Copy the Python package directory (e.g. sovrnd/ or sovrn_auth/)
    local pkg_path="$src_dir/$py_module"
    if [ -d "$pkg_path" ]; then
        cp -r "$pkg_path" "$PYDIST/usr/lib/python3/dist-packages/"
    fi

    # Copy egg-info
    for egginfo in "$src_dir"/*.egg-info; do
        [ -d "$egginfo" ] && cp -r "$egginfo" "$PYDIST/usr/lib/python3/dist-packages/"
    done

    # Create wrapper script (shebang written as octal to avoid sed issues)
    printf '\043!/usr/bin/python3\nfrom %s.__main__ import main\nmain()\n' "$py_module" > "$PYDIST/usr/bin/$bin_name"
    chmod 755 "$PYDIST/usr/bin/$bin_name"

    log "  Installed $bin_name -> $py_module.__main__:main"
}

install_python_pkg "$PROJECT_DIR/src/sovrnd"                "sovrnd"                "sovrnd"                "sovrnd"
install_python_pkg "$PROJECT_DIR/src/sovrn-auth"            "sovrn-auth"            "sovrn_auth"            "sovrn-auth"
install_python_pkg "$PROJECT_DIR/src/sovrn-monitor"         "sovrn-monitor"         "sovrn_monitor"         "sovrn-monitor"
install_python_pkg "$PROJECT_DIR/src/sovrn-notify-bridge"   "sovrn-notify-bridge"   "sovrn_notify_bridge"   "sovrn-notify-bridge"
install_python_pkg "$PROJECT_DIR/src/sovrn-complete-setup" "sovrn-complete-setup" "sovrn_complete_setup" "sovrn-complete-setup"

rm -rf "$PYDIST/usr/lib/python3/dist-packages"/__pycache__ 2>/dev/null || true

# Step 3: Validate overlay structure before running debos
log "Validating overlay structure..."
errors=0

check_file() {
    if [ ! -f "$1" ]; then
        err "Missing: $1"
        errors=$((errors+1))
    fi
}
check_dir() {
    if [ ! -d "$1" ]; then
        err "Missing overlay directory: $1"
        errors=$((errors+1))
    fi
}

# Check overlay source directories exist and have content
check_dir "$OVERLAY_DIR/bin/usr/bin"
check_dir "$OVERLAY_DIR/etc/etc/sovrn"
check_dir "$OVERLAY_DIR/etc/etc/caddy"
check_dir "$OVERLAY_DIR/etc/etc/nftables"
check_dir "$OVERLAY_DIR/etc/etc/unbound"
check_dir "$OVERLAY_DIR/etc/etc/sovrn/scripts"
check_dir "$OVERLAY_DIR/systemd/etc/systemd/system"
check_dir "$OVERLAY_DIR/pwa-dist/var/lib/sovrn/pwa-dist"
check_dir "$OVERLAY_DIR/gnome/etc/dconf/profile"
check_dir "$OVERLAY_DIR/gnome/etc/dconf/db"
check_dir "$OVERLAY_DIR/python-dist/usr/lib/python3/dist-packages"
check_dir "$OVERLAY_DIR/python-dist/usr/bin"
for bin in sovrnd sovrn-auth sovrn-monitor sovrn-notify-bridge sovrn-complete-setup; do
    check_file "$OVERLAY_DIR/python-dist/usr/bin/$bin"
done
for pkg in sovrnd sovrn_auth sovrn_monitor sovrn_notify_bridge sovrn_complete_setup; do
    check_dir "$OVERLAY_DIR/python-dist/usr/lib/python3/dist-packages/$pkg"
done

# Check binaries (caddy not included — installed via apt inside debos)
for bin in sovrn-dht sovrn-identity sovrn-presence sovrn-feed sovrn-message-queue sovrn-cdn-agent; do
    check_file "$OVERLAY_DIR/bin/usr/bin/$bin"
done

# Check config files
check_file "$OVERLAY_DIR/etc/etc/sovrn/sovrnd.toml"
check_file "$OVERLAY_DIR/etc/etc/sovrn/yggdrasil.conf"
check_file "$OVERLAY_DIR/etc/etc/sovrn/dht.toml"
check_file "$OVERLAY_DIR/etc/etc/sovrn/identity.toml"
check_file "$OVERLAY_DIR/etc/etc/sovrn/presence.toml"
check_file "$OVERLAY_DIR/etc/etc/sovrn/feed.toml"
check_file "$OVERLAY_DIR/etc/etc/sovrn/message-queue.toml"
check_file "$OVERLAY_DIR/etc/etc/sovrn/cdn-agent.toml"
check_file "$OVERLAY_DIR/etc/etc/caddy/Caddyfile"
check_file "$OVERLAY_DIR/etc/etc/nftables/sovrn.nft"
check_file "$OVERLAY_DIR/etc/etc/unbound/sovrn.conf"
check_file "$OVERLAY_DIR/etc/etc/sovrn/scripts/bootstrap-ca.sh"

# Check systemd units (at least the critical ones)
for unit in sovrn.target sovrnd.service sovrn-dht.service sovrn-identity.service sovrn-presence.service sovrn-feed.service sovrn-message-queue.service sovrn-auth.service sovrn-monitor.service sovrn-notify-bridge.service sovrn-cdn-agent.service yggdrasil.service caddy.service sovrn-ca-bootstrap.service zram-setup.service; do
    check_file "$OVERLAY_DIR/systemd/etc/systemd/system/$unit"
done

# Check PWA dist files
check_file "$OVERLAY_DIR/pwa-dist/var/lib/sovrn/pwa-dist/index.html"

# Cross-reference: every ExecStart path in unit files should have a matching binary
# Skip Python services (installed via pip) and system packages (installed via apt)
PYTHON_SERVICES="sovrnd sovrn-auth sovrn-monitor sovrn-notify-bridge sovrn-complete-setup"
NEVER_BUILT="sovrn-oobe sovrn-app-monitor sovrn-first-boot"
SYSTEM_PACKAGES="yggdrasil caddy"
log "Cross-referencing ExecStart paths with overlay binaries..."
for unit in "$OVERLAY_DIR/systemd/etc/systemd/system/"*.service; do
    while IFS= read -r line; do
        bin_path=$(echo "$line" | sed 's/ExecStart=//' | awk '{print $1}')
        if echo "$bin_path" | grep -q "^/usr/bin/"; then
            bin_name=$(basename "$bin_path")
            # Skip known non-overlay binaries
            skip=0
            for s in $PYTHON_SERVICES $NEVER_BUILT $SYSTEM_PACKAGES; do
                if [ "$bin_name" = "$s" ]; then
                    skip=1
                    break
                fi
            done
            if [ "$skip" -eq 0 ] && [ ! -f "$OVERLAY_DIR/bin/usr/bin/$bin_name" ]; then
                warn "Unit $(basename "$unit") references $bin_path but no binary at overlays/bin/usr/bin/$bin_name"
                errors=$((errors+1))
            fi
        fi
    done < <(grep "^ExecStart=" "$unit" 2>/dev/null || true)
done

# Check specific known path mappings
if grep -q "/etc/yggdrasil/yggdrasil.conf" "$OVERLAY_DIR/systemd/etc/systemd/system/yggdrasil.service" 2>/dev/null; then
    err "yggdrasil.service still references /etc/yggdrasil/yggdrasil.conf — should be /etc/sovrn/yggdrasil.conf"
    errors=$((errors+1))
fi

if [ "$errors" -gt 0 ]; then
    err "Overlay validation failed with $errors error(s). Fix before running debos."
fi
log "Overlay validation passed."

# Step 4: Build rootfs tarball
log "Building rootfs tarball with Debos..."

if [ "$USE_DOCKER" = true ]; then
    if ! command -v docker &>/dev/null; then
        err "Docker not found. Install Docker Desktop: https://docker.com"
    fi
    if ! docker info &>/dev/null; then
        err "Docker daemon not running. Start Docker Desktop."
    fi

    docker pull ghcr.io/go-debos/debos:latest 2>/dev/null || true

    APT_CACHE_DIR="$HOME/.cache/sovrn-apt"
    mkdir -p "$APT_CACHE_DIR"
    docker run --rm --privileged \
        -v "$PROJECT_DIR":/project \
        -v "$BUILD_DIR":/build \
        -v "$APT_CACHE_DIR":/var/cache/apt/archives \
        ghcr.io/go-debos/debos:latest \
        --disable-fakemachine \
        --artifactdir=/build \
        /project/build/$RECIPE || err "Debos rootfs build failed"

    ROOTFS_TAR="$BUILD_DIR/sovrn-os-rootfs.tar.gz"
    if [ -f "$ROOTFS_TAR" ]; then
        log "Rootfs tarball created: $ROOTFS_TAR"
        log "Size: $(du -h "$ROOTFS_TAR" | cut -f1)" || true
        log ""
        log "NOTE: Bootable ISO requires a Linux host with KVM."
        log "To build the ISO on Linux:"
        log "  1. Install: apt install debos xorriso grub-pc-bin"
        log "  2. Run:     debos build/$RECIPE"
        log ""
        log "Or use the rootfs tarball directly to create a custom image."
    else
        err "Rootfs tarball not found at $ROOTFS_TAR"
    fi
else
    if ! command -v debos &>/dev/null; then
        err "debos not found. Install: apt install debos"
    fi
    # Use --disable-fakemachine to avoid VM memory overhead on RAM-constrained CI runners
    # The -e flag ensures the build runs directly on the host (needs sudo for debootstrap)
    debos --disable-fakemachine build/$RECIPE || err "Debos build failed"
fi
