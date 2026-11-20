# Arch Linux Package

This directory contains the PKGBUILD for building Arch Linux packages.

## Building

### Prerequisites

```bash
sudo pacman -S base-devel cargo rust
```

### Build Package

```bash
# From the packages/arch directory
cd distribution/packages/arch

# Build
makepkg -si
```

Or build in a clean chroot (recommended):

```bash
# Setup clean chroot
mkdir ~/chroot
mkarchroot ~/chroot/root base-devel

# Build package
makechrootpkg -c -r ~/chroot
```

## Installation

```bash
sudo pacman -U cloud-netconfig-*.pkg.tar.zst
sudo systemctl enable --now cloud-netconfigd
```

## Removal

```bash
sudo pacman -R cloud-netconfig
```

## Files

- `PKGBUILD` - Package build script
- `cloud-netconfig.install` - Install/remove hooks
- `README.md` - This file

## Submitting to AUR

To submit to the Arch User Repository:

```bash
# Clone AUR repository
git clone ssh://aur@aur.archlinux.org/cloud-netconfig.git aur-cloud-netconfig
cd aur-cloud-netconfig

# Copy files
cp ../PKGBUILD .
cp ../cloud-netconfig.install .

# Generate .SRCINFO
makepkg --printsrcinfo > .SRCINFO

# Commit and push
git add PKGBUILD cloud-netconfig.install .SRCINFO
git commit -m "Update to version X.Y.Z"
git push
```
