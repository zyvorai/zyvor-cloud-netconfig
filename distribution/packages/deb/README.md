# Debian Package

This directory contains files for building Debian/Ubuntu packages.

## Building

### Prerequisites

```bash
sudo apt install debhelper cargo rustc build-essential
```

### Build Package

```bash
# Build the project
cargo build --release

# Create debian package directory structure
mkdir -p debian
cp -r distribution/packages/deb/* debian/

# Build package
dpkg-buildpackage -us -uc -b
```

## Files

- `control` - Package metadata and dependencies
- `cloud-netconfig.install` - File installation mappings
- `cloud-netconfig.postinst` - Post-installation script
- `cloud-netconfig.prerm` - Pre-removal script
- `cloud-netconfig.postrm` - Post-removal script

## Installation

```bash
sudo dpkg -i ../cloud-netconfig_*.deb
sudo systemctl enable --now cloud-netconfigd
```

## Removal

```bash
# Remove package (keep configuration)
sudo apt remove cloud-netconfig

# Purge package (remove everything)
sudo apt purge cloud-netconfig
```
