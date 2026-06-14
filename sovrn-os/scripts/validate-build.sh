#!/usr/bin/env bash
# Sovrn OS — Pre-Flight Build Validator
# Runs exhaustive offline checks before debos to prevent 25-min loop failures.
# Usage: ./scripts/validate-build.sh [--fix]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_DIR/build"
OVERLAY_DIR="$BUILD_DIR/overlays"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

PASS=0
WARN=0
FAIL=0
ERRORS=""

pass() { PASS=$((PASS+1)); }
warn() { WARN=$((WARN+1)); echo -e "${YELLOW}[WARN]${NC} $*"; }
fail() { FAIL=$((FAIL+1)); echo -e "${RED}[FAIL]${NC} $*"; ERRORS="$ERRORS\n  - $*"; }

check_file() {
    if [ ! -f "$1" ]; then fail "Missing: $1"; else pass; fi
}
check_dir() {
    if [ ! -d "$1" ]; then fail "Missing directory: $1"; else pass; fi
}
check_not_exists() {
    if [ -e "$1" ]; then fail "Should not exist: $1"; else pass; fi
}

echo "========================================"
echo "  Sovrn OS — Pre-Flight Build Validator"
echo "========================================"
echo ""

# ── 1. Debs recipe overlay actions vs actual source directories ──
echo "--- Overlay sources (debos recipe) ---"
check_dir "$OVERLAY_DIR/bin/usr/bin"
check_dir "$OVERLAY_DIR/etc/etc/sovrn"
check_dir "$OVERLAY_DIR/etc/etc/caddy"
check_dir "$OVERLAY_DIR/etc/etc/nftables"
check_dir "$OVERLAY_DIR/etc/etc/unbound"
check_dir "$OVERLAY_DIR/etc/etc/sovrn/scripts"
check_dir "$OVERLAY_DIR/etc/etc/sovrn/ca"
check_dir "$OVERLAY_DIR/systemd/etc/systemd/system"
check_dir "$OVERLAY_DIR/pwa-dist/var/lib/sovrn/pwa-dist"
check_dir "$OVERLAY_DIR/gnome/etc/dconf/profile"
check_dir "$OVERLAY_DIR/gnome/etc/dconf/db"
echo ""

# ── 2. Binaries ──
echo "--- Binaries (overlays/bin/usr/bin/) ---"
for bin in sovrn-dht sovrn-identity sovrn-presence sovrn-feed sovrn-message-queue sovrn-cdn-agent caddy; do
    f="$OVERLAY_DIR/bin/usr/bin/$bin"
    check_file "$f"
    if [ -f "$f" ]; then
        ftype=$(file "$f" 2>/dev/null)
        if ! echo "$ftype" | grep -q "ELF.*x86-64"; then
            warn "$bin is not an ELF x86-64 binary (type: $ftype)"
        fi
    fi
done
echo ""

# ── 3. Config files ──
echo "--- Config files (overlays/etc/etc/) ---"
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
echo ""

# ── 4. Systemd units ──
echo "--- Systemd units (overlays/systemd/etc/systemd/system/) ---"
SERVICES=(
    sovrn.target sovrnd.service sovrn-dht.service sovrn-identity.service
    sovrn-presence.service sovrn-feed.service sovrn-message-queue.service
    sovrn-auth.service sovrn-monitor.service sovrn-notify-bridge.service
    sovrn-cdn-agent.service yggdrasil.service caddy.service
    sovrn-oobe.service sovrn-ca-bootstrap.service sovrn-first-boot.service
    sovrn-app-monitor.service zram-setup.service
)
for unit in "${SERVICES[@]}"; do
    check_file "$OVERLAY_DIR/systemd/etc/systemd/system/$unit"
done
echo ""

# ── 5. PWA dist ──
echo "--- PWA dist (overlays/pwa-dist/var/lib/sovrn/pwa-dist/) ---"
check_file "$OVERLAY_DIR/pwa-dist/var/lib/sovrn/pwa-dist/index.html"
if [ -d "$OVERLAY_DIR/pwa-dist/var/lib/sovrn/pwa-dist" ]; then
    count=$(find "$OVERLAY_DIR/pwa-dist/var/lib/sovrn/pwa-dist" -type f 2>/dev/null | wc -l | tr -d ' ')
    if [ "$count" -lt 2 ]; then
        fail "PWA dist has only $count files (expected many)"
    else
        pass
    fi
fi

# Check no flat files at pwa-dist root (wrong level)
if [ -f "$OVERLAY_DIR/pwa-dist/index.html" ]; then
    fail "PWA files found at overlays/pwa-dist/ root (should be in var/lib/sovrn/pwa-dist/)"
fi
echo ""

# ── 6. Cross-reference ExecStart paths ──
echo "--- Cross-reference ExecStart paths ---"
PYTHON_SERVICES="sovrnd sovrn-auth sovrn-monitor sovrn-notify-bridge"
NEVER_BUILT="sovrn-oobe sovrn-app-monitor sovrn-first-boot"
SYSTEM_PACKAGES="yggdrasil"

for unit in "$OVERLAY_DIR/systemd/etc/systemd/system/"*.service; do
    unit_name=$(basename "$unit")
    while IFS= read -r line; do
        bin_path=$(echo "$line" | sed 's/ExecStart=//' | awk '{print $1}')
        if echo "$bin_path" | grep -q "^/usr/bin/"; then
            bin_name=$(basename "$bin_path")
            skip=0
            for s in $PYTHON_SERVICES $NEVER_BUILT $SYSTEM_PACKAGES; do
                if [ "$bin_name" = "$s" ]; then skip=1; break; fi
            done
            if [ "$skip" -eq 0 ]; then
                check_file "$OVERLAY_DIR/bin/usr/bin/$bin_name"
            fi
        fi

        # Check config file references (--config /etc/sovrn/*.toml)
        cfg_path=$(echo "$bin_path" | grep -o "/etc/sovrn/[^ ]*\.toml" || true)
        if [ -n "$cfg_path" ]; then
            cfg_name=$(basename "$cfg_path")
            check_file "$OVERLAY_DIR/etc/etc/sovrn/$cfg_name"
        fi
    done < <(grep "^ExecStart=" "$unit" 2>/dev/null || true)
done

# Check yggdrasil service specifically for correct config path
if grep -q "/etc/yggdrasil/yggdrasil.conf" "$OVERLAY_DIR/systemd/etc/systemd/system/yggdrasil.service" 2>/dev/null; then
    fail "yggdrasil.service still references /etc/yggdrasil/ (should be /etc/sovrn/)"
fi
echo ""

# ── 7. Debos run action file references ──
echo "--- Debos run action file references ---"
check_file "$OVERLAY_DIR/etc/etc/sovrn/scripts/bootstrap-ca.sh"
check_file "$OVERLAY_DIR/etc/etc/sovrn/scripts/sovrn.nft"
check_file "$OVERLAY_DIR/etc/etc/unbound/sovrn.conf"
echo ""

# ── 8. GNOME customizations ──
echo "--- GNOME customizations ---"
check_file "$OVERLAY_DIR/gnome/etc/dconf/profile/sovrn"
check_file "$OVERLAY_DIR/gnome/etc/dconf/db/sovrn"
echo ""

# ── 9. Check for leftover stale files ──
echo "--- Leftover stale file check ---"
if [ -d "$BUILD_DIR/etc" ]; then
    for f in caddy.service sovrn-ca-bootstrap.service sovrn-first-boot.service sovrn-app-monitor.service zram-setup.service; do
        sf="$BUILD_DIR/etc/$f"
        if [ -f "$sf" ] && head -1 "$sf" 2>/dev/null | grep -q "see 31-SYSTEMD-UNITS"; then
            warn "Stale stub found: $sf (should be deleted)"
        fi
    done
fi
echo ""

# ── 10. Config syntax validation (basic) ──
echo "--- Config syntax validation ---"

# Validate yggdrasil.conf is valid JSON5-ish
if [ -f "$OVERLAY_DIR/etc/etc/sovrn/yggdrasil.conf" ]; then
    # Yggdrasil uses JSON-like format, check for basic structure
    if grep -q "{" "$OVERLAY_DIR/etc/etc/sovrn/yggdrasil.conf" 2>/dev/null; then
        pass
    else
        warn "yggdrasil.conf may not have valid JSON structure"
    fi
fi

# Check bootstrap-ca.sh is executable in overlay (will be set by chmod in debos)
if [ -f "$OVERLAY_DIR/etc/etc/sovrn/scripts/bootstrap-ca.sh" ]; then
    if [ ! -x "$OVERLAY_DIR/etc/etc/sovrn/scripts/bootstrap-ca.sh" ]; then
        warn "bootstrap-ca.sh is not executable (will be fixed by chmod run action)"
    fi
fi
echo ""

# ── Summary ──
echo "========================================"
echo "  Validation Results"
echo "========================================"
echo -e "  ${GREEN}Pass:${NC}  $PASS"
echo -e "  ${YELLOW}Warn:${NC}  $WARN"
echo -e "  ${RED}Fail:${NC}  $FAIL"
echo ""

if [ "$FAIL" -gt 0 ]; then
    echo -e "${RED}Validation FAILED with $FAIL error(s):${NC}$ERRORS"
    echo ""
    echo "Fix errors above before running build-iso.sh"
    exit 1
elif [ "$WARN" -gt 0 ]; then
    echo -e "${YELLOW}Validation passed with $WARN warning(s) — review above${NC}"
    exit 0
else
    echo -e "${GREEN}All checks passed — ready to build!${NC}"
    exit 0
fi
