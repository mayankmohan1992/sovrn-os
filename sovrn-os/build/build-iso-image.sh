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
            umount /tmp/sovrn-mnt/boot/efi 2>/dev/null || true
            umount /tmp/sovrn-mnt/dev 2>/dev/null || true
            umount /tmp/sovrn-mnt/proc 2>/dev/null || true
            umount /tmp/sovrn-mnt/sys 2>/dev/null || true
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
    log "Mounting root partition..."
    mkdir -p /tmp/sovrn-mnt
    root_part="${loop_dev}p3"
    efi_part="${loop_dev}p2"
    mount "$root_part" /tmp/sovrn-mnt

    # Extract rootfs tarball first so we can mount into its directories
    log "Extracting rootfs tarball..."
    tar -xzf "$ROOTFS_TAR" -C /tmp/sovrn-mnt
    log "  Rootfs extracted"

    # Mount EFI partition and bind mounts for chroot
    log "Setting up chroot environment..."
    mkdir -p /tmp/sovrn-mnt/boot/efi
    mount "$efi_part" /tmp/sovrn-mnt/boot/efi
    mount --bind /dev /tmp/sovrn-mnt/dev
    mount --bind /proc /tmp/sovrn-mnt/proc
    mount --bind /sys /tmp/sovrn-mnt/sys

    # Detect exact kernel filenames (GRUB does not expand globs)
    KERNEL_NAME=$(ls /tmp/sovrn-mnt/boot/vmlinuz-* 2>/dev/null | head -1 | xargs basename)
    INITRD_NAME=$(ls /tmp/sovrn-mnt/boot/initrd.img-* 2>/dev/null | head -1 | xargs basename)
    if [ -z "$KERNEL_NAME" ] || [ -z "$INITRD_NAME" ]; then
        err "No kernel found in rootfs /boot/ (vmlinuz-* or initrd.img-* missing)"
    fi
    log "  Kernel: $KERNEL_NAME"
    log "  Initrd: $INITRD_NAME"

    # Generate fstab
    log "Generating /etc/fstab..."
    cat > /tmp/sovrn-mnt/etc/fstab <<FSTAB
LABEL=SOVRN_OS  /          ext4  defaults,errors=remount-ro  0  1
LABEL=SOVRN_EFI /boot/efi  vfat  defaults,nofail              0  2
tmpfs           /tmp       tmpfs defaults,nosuid,nodev         0  0
FSTAB

    # Install GRUB (BIOS + UEFI) inside chroot
    log "Installing GRUB bootloader..."

    # Temporarily override /etc/mtab inside chroot for grub-install to resolve loop partition paths correctly
    local mtab_was_symlink=0
    if [ -L /tmp/sovrn-mnt/etc/mtab ]; then
        mtab_was_symlink=1
        rm -f /tmp/sovrn-mnt/etc/mtab
    fi
    cat > /tmp/sovrn-mnt/etc/mtab <<EOF
${loop_dev}p3 / ext4 rw,relatime 0 0
${loop_dev}p2 /boot/efi vfat rw,relatime 0 0
EOF

    # Install BIOS GRUB
    chroot /tmp/sovrn-mnt grub-install --target=i386-pc "$loop_dev" 2>&1 | tail -2

    # Install UEFI GRUB
    mkdir -p /tmp/sovrn-mnt/boot/efi/EFI/BOOT
    chroot /tmp/sovrn-mnt grub-install --target=x86_64-efi --removable \
        --efi-directory=/boot/efi \
        --no-nvram 2>&1 | tail -2

    # Restore /etc/mtab symlink inside chroot
    if [ "$mtab_was_symlink" -eq 1 ]; then
        rm -f /tmp/sovrn-mnt/etc/mtab
        ln -sf ../proc/self/mounts /tmp/sovrn-mnt/etc/mtab
    fi

    # Clean up chroot mounts so they don't lock the filesystems
    log "Cleaning up chroot environment..."
    umount /tmp/sovrn-mnt/boot/efi
    umount /tmp/sovrn-mnt/dev
    umount /tmp/sovrn-mnt/proc
    umount /tmp/sovrn-mnt/sys

    # Write GRUB config
    log "Writing GRUB config..."
    mkdir -p /tmp/sovrn-mnt/boot/grub
    cat > /tmp/sovrn-mnt/boot/grub/grub.cfg <<GRUB
set default=0
set timeout=5

search --set=root --label SOVRN_OS --no-floppy

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

    # Unmount target filesystem and detach loop device manually to ensure everything is flushed/synced
    log "Unmounting target filesystem..."
    umount /tmp/sovrn-mnt
    root_part=""
    efi_part=""

    log "Detaching loop device..."
    losetup -d "$loop_dev"
    loop_dev=""

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
