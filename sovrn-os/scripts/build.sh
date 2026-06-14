#!/usr/bin/env bash
# Sovrn OS Build Script
# Builds all Rust services, Go CDN agent, Python packages, and PWA

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_DIR/build"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() { echo -e "${GREEN}[build]${NC} $*"; }
warn() { echo -e "${YELLOW}[build WARN]${NC} $*" >&2; }
err() { echo -e "${RED}[build ERROR]${NC} $*" >&2; exit 1; }

mkdir -p "$BUILD_DIR"/{bin,lib,etc,share}

# ── Check Prerequisites ────────────────────────────────────────
check_prereqs() {
    local missing=0
    
    command -v cargo >/dev/null 2>&1 || { warn "cargo not found"; missing=1; }
    command -v go >/dev/null 2>&1 || { warn "go not found"; missing=1; }
    command -v python3 >/dev/null 2>&1 || { warn "python3 not found"; missing=1; }
    command -v npm >/dev/null 2>&1 || { warn "npm not found"; missing=1; }
    command -v pip3 >/dev/null 2>&1 || { warn "pip3 not found"; missing=1; }
    
    if [ "$missing" -eq 1 ]; then
        err "Missing prerequisites. Install: cargo, go, python3, npm, pip3"
    fi
    log "All prerequisites found"
}

# ── Build Rust Services ─────────────────────────────────────────
build_rust() {
    log "Building Rust mesh services..."
    cd "$PROJECT_DIR"
    
    cargo build --release --workspace 2>&1 | tail -5
    
    # Copy binaries
    for bin in sovrn-dht sovrn-identity sovrn-presence sovrn-feed sovrn-message-queue; do
        if [ -f "target/release/$bin" ]; then
            cp "target/release/$bin" "$BUILD_DIR/bin/"
            log "  Built $bin"
        else
            warn "  $bin not found in target/release"
        fi
    done
}

# ── Build Go CDN Agent ──────────────────────────────────────────
build_go() {
    log "Building Go CDN agent..."
    cd "$PROJECT_DIR/src/sovrn-cdn-agent"
    
    CGO_ENABLED=0 go build -o "$BUILD_DIR/bin/sovrn-cdn-agent" ./cmd/sovrn-cdn-agent/
    log "  Built sovrn-cdn-agent"
}

# ── Build Python Services ───────────────────────────────────────
build_python() {
    log "Building Python services..."
    
    # sovrnd
    cd "$PROJECT_DIR/src/sovrnd"
    pip3 install --target "$BUILD_DIR/lib/sovrnd" . 2>&1 | tail -3 || warn "sovrnd pip install failed"
    
    # sovrn-auth
    cd "$PROJECT_DIR/src/sovrn-auth"
    pip3 install --target "$BUILD_DIR/lib/sovrn-auth" . 2>&1 | tail -3 || warn "sovrn-auth pip install failed"
    
    # sovrn-monitor
    cd "$PROJECT_DIR/src/sovrn-monitor"
    pip3 install --target "$BUILD_DIR/lib/sovrn-monitor" . 2>&1 | tail -3 || warn "sovrn-monitor pip install failed"
    
    # sovrn-notify-bridge
    cd "$PROJECT_DIR/src/sovrn-notify-bridge"
    pip3 install --target "$BUILD_DIR/lib/sovrn-notify-bridge" . 2>&1 | tail -3 || warn "sovrn-notify-bridge pip install failed"
    
    # sovrn-complete-setup
    cd "$PROJECT_DIR/src/sovrn-complete-setup"
    pip3 install --target "$BUILD_DIR/lib/sovrn-complete-setup" . 2>&1 | tail -3 || warn "sovrn-complete-setup pip install failed"
}

# ── Build PWA ───────────────────────────────────────────────────
build_pwa() {
    log "Building PWA (sovrn-hub)..."
    cd "$PROJECT_DIR/src/pwa"
    
    npm install 2>&1 | tail -3
    npm run build 2>&1 | tail -5
    
    if [ -d "dist" ]; then
        cp -r dist/ "$BUILD_DIR/share/pwa-dist/"
        log "  PWA built and copied to $BUILD_DIR/share/pwa-dist/"
    else
        warn "  PWA dist directory not found"
    fi
}

# ── Build Caddy Auth Plugin ──────────────────────────────────────
build_caddy() {
    log "Building Caddy with auth plugin..."
    cd "$PROJECT_DIR/src/caddy-auth"
    
    # Build as a Caddy module (requires xcaddy)
    if command -v xcaddy >/dev/null 2>&1; then
        xcaddy build --with github.com/sovrn-os/caddy-auth="$PROJECT_DIR/src/caddy-auth" \
            --output "$BUILD_DIR/bin/caddy" 2>&1 | tail -5
        log "  Built custom Caddy binary"
    else
        warn "  xcaddy not found, skipping custom Caddy build"
        warn "  Install: go install github.com/caddyserver/xcaddy/cmd/xcaddy@latest"
    fi
}

# ── Copy Config ──────────────────────────────────────────────────
copy_configs() {
    log "Copying configuration files..."
    
    # Systemd units
    cp "$PROJECT_DIR"/src/systemd-units/*.service "$BUILD_DIR/etc/"
    cp "$PROJECT_DIR"/src/systemd-units/*.target "$BUILD_DIR/etc/"
    
    # Config TOML files
    mkdir -p "$BUILD_DIR/etc/sovrn"
    cp "$PROJECT_DIR"/src/sovrnd/configs/sovrnd.toml "$BUILD_DIR/etc/sovrn/" 2>/dev/null ||         echo "# Default sovrnd config" > "$BUILD_DIR/etc/sovrn/sovrnd.toml"
    
    # Caddy
    mkdir -p "$BUILD_DIR/etc/caddy"
    cp "$PROJECT_DIR"/src/caddy-config/Caddyfile "$BUILD_DIR/etc/caddy/"
    
    # nftables
    mkdir -p "$BUILD_DIR/etc/nftables"
    cp "$PROJECT_DIR"/src/nftables/sovrn.nft "$BUILD_DIR/etc/nftables/"
    
    # DNS
    mkdir -p "$BUILD_DIR/etc/unbound"
    cp "$PROJECT_DIR"/src/dns/unbound-sovrn.conf "$BUILD_DIR/etc/unbound/"
    
    log "  Configs copied"
}

# ── Main ─────────────────────────────────────────────────────────
main() {
    local step="${1:-all}"
    
    log "Sovrn OS Build System"
    log "Project dir: $PROJECT_DIR"
    log "Build dir:   $BUILD_DIR"
    log "Target:      $step"
    echo ""
    
    case "$step" in
        all)
            check_prereqs
            build_rust
            build_go
            build_python
            build_pwa
            build_caddy
            copy_configs
            log "Build complete!"
            ;;
        rust)    build_rust ;;
        go)      build_go ;;
        python)  build_python ;;
        pwa)     build_pwa ;;
        caddy)   build_caddy ;;
        config)  copy_configs ;;
        check)   check_prereqs ;;
        *)
            echo "Usage: $0 {all|rust|go|python|pwa|caddy|config|check}"
            exit 1
            ;;
    esac
}

main "$@"
