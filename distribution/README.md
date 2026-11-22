# Distribution Files

This directory contains files for packaging and distribution of cloud-netconfig.

## Structure

```
distribution/
├── etc/
│   └── cloud-network/
│       ├── config.yaml             # Default configuration
│       └── examples/               # Example configurations
│           ├── aws.yaml            # AWS EC2-specific (IMDSv2)
│           ├── azure.yaml          # Azure-specific
│           ├── gcp.yaml            # GCP-specific
│           ├── minimal.yaml        # Minimal setup
│           ├── multi-interface.yaml # Multi-interface setup
│           └── production.yaml     # Production-hardened
├── lib/
│   └── systemd/
│       └── system/
│           └── cloud-netconfigd.service  # Systemd unit file
├── usr/
│   └── share/
│       ├── bash-completion/completions/
│       │   └── cnctl               # Bash completion
│       ├── zsh/site-functions/
│       │   └── _cnctl              # Zsh completion
│       └── fish/vendor_completions.d/
│           └── cnctl.fish          # Fish completion
├── scripts/
│   ├── install.sh                  # Installation script
│   └── uninstall.sh                # Uninstallation script
├── packages/
│   ├── deb/                        # Debian/Ubuntu packaging
│   ├── rpm/                        # Fedora/RHEL/CentOS packaging
│   └── arch/                       # Arch Linux packaging
└── README.md                       # This file
```

## Installation

### Quick Install

Use the provided installation script:

```bash
sudo make build
sudo ./distribution/scripts/install.sh
```

### Manual Installation

#### 1. Install Binaries

```bash
sudo install -m 0755 bin/cloud-netconfigd /usr/bin/
sudo install -m 0755 bin/cnctl /usr/bin/
```

#### 2. Install Configuration

```bash
sudo mkdir -p /etc/cloud-network
sudo install -m 0644 distribution/etc/cloud-network/config.yaml /etc/cloud-network/
```

#### 3. Install Systemd Service

```bash
sudo install -m 0644 distribution/lib/systemd/system/cloud-netconfigd.service /lib/systemd/system/
sudo systemctl daemon-reload
```

#### 4. Install Shell Completions (Optional)

```bash
# Bash
sudo install -Dm644 distribution/usr/share/bash-completion/completions/cnctl /usr/share/bash-completion/completions/cnctl

# Zsh
sudo install -Dm644 distribution/usr/share/zsh/site-functions/_cnctl /usr/share/zsh/site-functions/_cnctl

# Fish
sudo install -Dm644 distribution/usr/share/fish/vendor_completions.d/cnctl.fish /usr/share/fish/vendor_completions.d/cnctl.fish
```

#### 5. Create User and Enable Service

```bash
sudo useradd -r -s /usr/bin/nologin -d /run/cloud-network cloud-network
sudo systemctl enable --now cloud-netconfigd
```

## Package Building

### Debian/Ubuntu

```bash
# See packages/deb/README.md
make build
cp -r distribution/packages/deb/* debian/
dpkg-buildpackage -us -uc -b
```

### Fedora/RHEL/CentOS

```bash
# See packages/rpm/README.md
cp distribution/packages/rpm/cloud-netconfig.spec ~/rpmbuild/SPECS/
rpmbuild -ba ~/rpmbuild/SPECS/cloud-netconfig.spec
```

### Arch Linux

```bash
# See packages/arch/README.md
cd distribution/packages/arch
makepkg -si
```

## Files

### Systemd Service

`lib/systemd/system/cloud-netconfigd.service` - Systemd service unit with:
- Type=notify for proper startup signaling
- Watchdog support (60s)
- Security hardening (capabilities, sandboxing)
- Automatic restart on failure

### Configuration

`etc/cloud-network/config.yaml` - Default configuration file with all options documented.

### Shell Completions

- `usr/share/bash-completion/completions/cnctl` - Bash completion for cnctl
- `usr/share/zsh/site-functions/_cnctl` - Zsh completion for cnctl
- `usr/share/fish/vendor_completions.d/cnctl.fish` - Fish completion for cnctl

## Example Configurations

### minimal.yaml
Absolute minimal configuration. Uses defaults for everything except basic logging.

**Use case**: Quick testing, development

### multi-interface.yaml
Full configuration for instances with multiple network interfaces.

**Use case**:
- AWS EC2 instances with ENIs
- Azure VMs with multiple NICs
- GCP instances with multiple interfaces

### azure.yaml
Azure-optimized configuration with JSON logging for Azure Monitor.

**Features**:
- JSON logging for Azure Monitor integration
- Azure IMDS API version specification
- Proper retry configuration

### aws.yaml
AWS EC2-optimized configuration with IMDSv2 support.

**Features**:
- IMDSv2 for enhanced security
- JSON logging for CloudWatch integration
- Extended retry configuration
- Optional jumbo frame MTU (9001)

### gcp.yaml
Google Cloud Platform-optimized configuration.

**Features**:
- JSON logging for Cloud Logging
- Recursive metadata fetch
- GCP-specific interface naming (ens4, ens5)
- Optional MTU override for VPC default (1460)

### production.yaml
Production-hardened configuration with conservative settings.

**Features**:
- Reduced log verbosity (warn level)
- Longer refresh intervals
- Extended timeouts and retries
- Security-first defaults
- Minimal capability set

## Usage

1. **Choose a configuration**: Pick an example that matches your environment
2. **Copy to /etc/cloud-network/**:
   ```bash
   sudo cp distribution/etc/cloud-network/examples/azure.yaml /etc/cloud-network/config.yaml
   ```
3. **Edit as needed**: Modify interface names, intervals, etc.
4. **Restart daemon**:
   ```bash
   sudo systemctl restart cloud-netconfigd
   ```

## Package Maintainers

Packaging templates are provided in `packages/` for Debian, RPM, and Arch Linux. See the README in each subdirectory for details.

### File Locations

When packaging cloud-netconfig, install files to:

| Source | Destination |
|--------|-------------|
| `bin/cloud-netconfigd` | `/usr/bin/cloud-netconfigd` |
| `bin/cnctl` | `/usr/bin/cnctl` |
| `etc/cloud-network/config.yaml` | `/etc/cloud-network/config.yaml` |
| `etc/cloud-network/examples/` | `/usr/share/doc/cloud-netconfig/examples/` |
| `lib/systemd/system/cloud-netconfigd.service` | `/lib/systemd/system/cloud-netconfigd.service` |
| `usr/share/bash-completion/completions/cnctl` | `/usr/share/bash-completion/completions/cnctl` |
| `usr/share/zsh/site-functions/_cnctl` | `/usr/share/zsh/site-functions/_cnctl` |
| `usr/share/fish/vendor_completions.d/cnctl.fish` | `/usr/share/fish/vendor_completions.d/cnctl.fish` |

### Post-Install Steps

1. Create system user:
   ```bash
   useradd -r -s /usr/bin/nologin -d /run/cloud-network cloud-network
   ```

2. Create state directory:
   ```bash
   mkdir -p /var/lib/cloud-netconfig
   chown cloud-network:cloud-network /var/lib/cloud-netconfig
   ```

3. Reload systemd and optionally enable:
   ```bash
   systemctl daemon-reload
   systemctl enable cloud-netconfigd  # Optional
   ```

## License

All files: Zyvor Production Licence v2.0
