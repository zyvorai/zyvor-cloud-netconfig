# RPM Package

This directory contains the RPM spec file for building Fedora/RHEL/CentOS packages.

## Building

### Prerequisites

```bash
# Fedora/RHEL/CentOS
sudo dnf install rpm-build cargo rust systemd-rpm-macros rpmdevtools

# Setup RPM build tree
rpmdev-setuptree
```

### Build Package

```bash
# Create source tarball
VERSION=0.3.0
git archive --format=tar.gz --prefix=cloud-netconfig-${VERSION}/ HEAD > ~/rpmbuild/SOURCES/cloud-netconfig-${VERSION}.tar.gz

# Copy spec file
cp distribution/packages/rpm/cloud-netconfig.spec ~/rpmbuild/SPECS/

# Build RPM
rpmbuild -ba ~/rpmbuild/SPECS/cloud-netconfig.spec
```

## Installation

```bash
sudo dnf install ~/rpmbuild/RPMS/x86_64/cloud-netconfig-*.rpm
sudo systemctl enable --now cloud-netconfigd
```

## Removal

```bash
sudo dnf remove cloud-netconfig
```

## Files

- `cloud-netconfig.spec` - RPM specification file
- `README.md` - This file
