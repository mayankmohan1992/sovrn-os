#!/usr/bin/env bash
# Sovrn OS Hybrid Disk Image Builder
# Creates a bootable hybrid disk image (BIOS + UEFI) from the rootfs tarball.
#
# Usage:
#   sudo ./build-iso-image.sh              # uses build/sovrn-os-rootfs.tar.gz
#   sudo ./build-iso-image.sh path/to/rootfs.tar.gz
#
# Output: build/sovrn-os-hybrid.img (4 GB)
#
# Prerequisites (Linux):
#   apt install parted grub-pc-bin grub-efi-amd64-bin xorriso mtools
#
# The image has GPT partitioning with:
#   - Partition 1: 1 MB BIOS boot partition (bios_grub)
#   - Partition 2: 512 MB EFI System Partition (FAT32, esp)
#   - Partition 3: Remainder as ext4 root filesystem (LABEL=SOVRN_OS)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_DIR/build"
ROOTFS_TAR="${1:-$BUILD_DIR/sovrn-os-rootfs.tar.gz}"
OUTPUT_IMG="$BUILD_DIR/sovrn-os-hybrid.img"
IMG_SIZE_MB=4096
EFI_SIZE_MB=512
ROOT_LABEL="SOVRN_OS"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'
log()  { echo -e "${GREEN}[build-img]${NC} $*"; }
warn() { echo -e "${YELLOW}[build-img WARN]${NC} $*" >&2; }
err()  { echo -e "${RED}[build-img ERROR]${NC} $*" >&2; exit 1; }

# ── Prerequisites ──────────────────────────────────────────────────
check_prereqs() {
    local missing=0
    for cmd in parted losetup grub-install mkfs.ext4 mkfs.fat xorriso mtools; do
        command -v "$cmd" >/dev/null 2>&1 || { warn "$cmd not found"; missing=1; }
    done
    if [ "$missing" -eq 1 ]; then
        err "Install missing tools: apt install parted grub-pc-bin grub-efi-amd64-bin xorriso mtools"
    fi
}

# ── Root check ─────────────────────────────────────────────────────
check_root() {
    [ "$(id -u)" -eq 0 ] || err "This script must be run as root (needs losetup/mount/grub-install). Use sudo."
}

# ── Build Image ────────────────────────────────────────────────────
build_image() {
    local loop_dev="" efi_part="" root_part=""

    # Cleanup handler
    cleanup() {
        log "Cleaning up..."
        if [ -n "${root_part:-}" ] && mountpoint -q /tmp/sovrn-mnt 2>/dev/null; then
            umount /tmp/sovrn-mnt 2>/dev/null || true
        fi
        if [ -n "${efi_part:-}" ] && mountpoint -q /tmp/sovrn-efi 2>/dev/null; then
            umount /tmp/sovrn-efi 2>/dev/null || true
        fi
        if [ -n "${loop_dev:-}" ]; then
            losetup -d "$loop_dev" 2>/dev/null || true
        fi
        rm -rf /tmp/sovrn-mnt /tmp/sovrn-efi 2>/dev/null || true
    }
    trap cleanup EXIT

    # Create empty image
    log "Creating ${IMG_SIZE_MB}MB disk image..."
    dd if=/dev/zero of="$OUTPUT_IMG" bs=1M count="$IMG_SIZE_MB" status=progress

    # Partition (GPT: bios_grub + ESP + root)
    log "Partitioning..."
    parted -s "$OUTPUT_IMG" mklabel gpt
    parted -s "$OUTPUT_IMG" mkpart primary 1MiB 2MiB
    parted -s "$OUTPUT_IMG" set 1 bios_grub on
    parted -s "$OUTPUT_IMG" mkpart primary fat32 2MiB "$((2 + EFI_SIZE_MB))MiB"
    parted -s "$OUTPUT_IMG" set 2 esp on
    parted -s "$OUTPUT_IMG" mkpart primary ext4 "$((2 + EFI_SIZE_MB))MiB" 100%

    # Set up loop device
    log "Setting up loop device..."
    loop_dev=$(losetup --partscan --show --find "$OUTPUT_IMG")
    log "  Loop device: $loop_dev"
    sleep 1  # let partscan settle

    # Format partitions
    log "Formatting partitions..."
    mkfs.fat -F32 -n SOVRN_EFI "${loop_dev}p2"
    mkfs.ext4 -F -L "$ROOT_LABEL" "${loop_dev}p3"

    # Mount partitions
    log "Mounting partitions..."
    mkdir -p /tmp/sovrn-mnt /tmp/sovrn-efi
    mount "${loop_dev}p3" /tmp/sovrn-mnt
    mount "${loop_dev}p2" /tmp/sovrn-efi

    # Extract rootfs tarball
    log "Extracting rootfs tarball..."
    tar -xzf "$ROOTFS_TAR" -C /tmp/sovrn-mnt
    log "  Rootfs extracted"

    # Detect exact kernel filenames (GRUB does not expand globs)
    KERNEL_NAME=$(ls /tmp/sovrn-mnt/boot/vmlinuz-* 2>/dev/null | head -1 | xargs basename)
    INITRD_NAME=$(ls /tmp/sovrn-mnt/boot/initrd.img-* 2>/dev/null | head -1 | xargs basename)
    if [ -z "$KERNEL_NAME" ] || [ -z "$INITRD_NAME" ]; then
        err "No kernel found in rootfs /boot/ (vmlinuz-* or initrd.img-* missing)"
    fi
    log "  Kernel: $KERNEL_NAME"
    log "  Initrd: $INITRD_NAME"

    # Install GRUB (BIOS + UEFI)
    log "Installing GRUB bootloader..."
    grub-install --target=i386-pc --boot-directory=/tmp/sovrn-mnt/boot "$loop_dev" 2>&1 | tail -2
    mkdir -p /tmp/sovrn-efi/EFI/BOOT
    grub-install --target=x86_64-efi --removable \
        --efi-directory=/tmp/sovrn-efi \
        --boot-directory=/tmp/sovrn-mnt/boot \
        --no-nvram 2>&1 | tail -2

    # Write GRUB config
    log "Writing GRUB config..."
    mkdir -p /tmp/sovrn-mnt/boot/grub
    cat > /tmp/sovrn-mnt/boot/grub/grub.cfg <<GRUB
set default=0
set timeout=5

loadfont (\$root)/boot/grub/fonts/unicode.pf2
set gfxmode=auto
insmod efi_gop
insmod efi_uga
insmod gfxterm
insmod gfxmenu
terminal_output gfxterm

menuentry "Sovrn OS" {
    linux /boot/${KERNEL_NAME} root=LABEL=SOVRN_OS quiet splash
    initrd /boot/${INITRD_NAME}
}

menuentry "Sovrn OS (safe mode)" {
    linux /boot/${KERNEL_NAME} root=LABEL=SOVRN_OS nomodeset
    initrd /boot/${INITRD_NAME}
}

menuentry "System setup (UEFI)" {
    fwsetup
}
GRUB

    # Sync and verify
    sync

    log "Image built successfully!"
    log "  Output: $OUTPUT_IMG"
    log "  Size:   $(du -h "$OUTPUT_IMG" | cut -f1)"
    log ""
    log "To write to USB (WARNING: destroys all data on target device):"
    log "  sudo dd if=$OUTPUT_IMG of=/dev/sdX bs=4M status=progress"
    log ""
    log "To test with QEMU:"
    log "  qemu-system-x86_64 -m 2048 -enable-kvm -cpu host -drive file=$OUTPUT_IMG,format=raw"
}

# ── Main ───────────────────────────────────────────────────────────
main() {
    echo ""
    log "Sovrn OS Hybrid Image Builder"
    log "=============================="
    log "Rootfs tarball: $ROOTFS_TAR"
    log "Output image:   $OUTPUT_IMG"
    echo ""

    [ -f "$ROOTFS_TAR" ] || err "Rootfs tarball not found: $ROOTFS_TAR"
    check_prereqs
    check_root
    build_image
}

main "$@"
