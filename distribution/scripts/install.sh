#!/bin/bash
# Copyright 2026 Zyvor AI Labs · https://zyvor.dev
# SPDX-License-Identifier: LicenseRef-Zyvor-Production-2.0
# Installation script for cloud-netconfig

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
PREFIX="${PREFIX:-/usr}"
SYSCONFDIR="${SYSCONFDIR:-/etc}"
LIBDIR="${LIBDIR:-/lib}"
BINDIR="${BINDIR:-$PREFIX/bin}"
DATADIR="${DATADIR:-$PREFIX/share}"

# Project name
PROJECT="cloud-netconfig"
DAEMON="cloud-netconfigd"
CTL="cnctl"
USER="cloud-network"

info() {
    echo -e "${GREEN}==>${NC} $1"
}

warn() {
    echo -e "${YELLOW}==>${NC} $1"
}

error() {
    echo -e "${RED}==>${NC} $1" >&2
}

check_root() {
    if [[ $EUID -ne 0 ]]; then
        error "This script must be run as root"
        exit 1
    fi
}

install_binaries() {
    info "Installing binaries..."

    if [[ ! -f "bin/$DAEMON" ]] || [[ ! -f "bin/$CTL" ]]; then
        error "Binaries not found in bin/. Please run 'make build' first."
        exit 1
    fi

    install -v -m 0755 "bin/$DAEMON" "$BINDIR/"
    install -v -m 0755 "bin/$CTL" "$BINDIR/"
}

install_config() {
    info "Installing configuration files..."

    install -v -d -m 0755 "$SYSCONFDIR/cloud-network"

    if [[ ! -f "$SYSCONFDIR/cloud-network/config.yaml" ]]; then
        install -v -m 0644 "distribution/etc/cloud-network/config.yaml" "$SYSCONFDIR/cloud-network/"
    else
        warn "Configuration file already exists at $SYSCONFDIR/cloud-network/config.yaml"
        warn "Not overwriting. New config saved as config.yaml.new"
        install -v -m 0644 "distribution/etc/cloud-network/config.yaml" "$SYSCONFDIR/cloud-network/config.yaml.new"
    fi

    # Install examples
    install -v -d -m 0755 "$DATADIR/doc/$PROJECT/examples"
    install -v -m 0644 distribution/etc/cloud-network/examples/*.yaml "$DATADIR/doc/$PROJECT/examples/"
}

install_systemd() {
    info "Installing systemd service..."

    install -v -m 0644 "distribution/lib/systemd/system/$DAEMON.service" "$LIBDIR/systemd/system/"
    systemctl daemon-reload
}

install_completions() {
    info "Installing shell completions..."

    # Bash
    install -v -d -m 0755 "$DATADIR/bash-completion/completions"
    install -v -m 0644 "distribution/usr/share/bash-completion/completions/$CTL" "$DATADIR/bash-completion/completions/"

    # Zsh
    install -v -d -m 0755 "$DATADIR/zsh/site-functions"
    install -v -m 0644 "distribution/usr/share/zsh/site-functions/_$CTL" "$DATADIR/zsh/site-functions/"

    # Fish
    install -v -d -m 0755 "$DATADIR/fish/vendor_completions.d"
    install -v -m 0644 "distribution/usr/share/fish/vendor_completions.d/$CTL.fish" "$DATADIR/fish/vendor_completions.d/"
}

create_user() {
    info "Creating system user..."

    if id "$USER" &>/dev/null; then
        warn "User $USER already exists"
    else
        useradd -r -s /usr/bin/nologin -d /run/cloud-network -c "Cloud Network Configuration Daemon" "$USER"
        info "Created user: $USER"
    fi

    # Create state directory
    install -v -d -m 0755 -o "$USER" -g "$USER" "/var/lib/$PROJECT"
}

enable_service() {
    info "Enabling and starting service..."

    systemctl enable "$DAEMON.service"

    if systemctl is-active --quiet "$DAEMON.service"; then
        warn "Service is already running. Restarting..."
        systemctl restart "$DAEMON.service"
    else
        systemctl start "$DAEMON.service"
    fi

    sleep 2

    if systemctl is-active --quiet "$DAEMON.service"; then
        info "Service started successfully"
    else
        error "Failed to start service"
        systemctl status "$DAEMON.service"
        exit 1
    fi
}

main() {
    check_root

    info "Installing $PROJECT..."

    install_binaries
    install_config
    install_systemd
    install_completions
    create_user

    info ""
    info "Installation complete!"
    info ""
    info "Configuration: $SYSCONFDIR/cloud-network/config.yaml"
    info "Examples: $DATADIR/doc/$PROJECT/examples/"
    info ""

    read -p "Do you want to enable and start the service now? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        enable_service
    else
        info "To enable the service later, run:"
        info "  systemctl enable --now $DAEMON"
    fi
}

main "$@"
