#!/bin/bash
# Copyright 2026 Zyvor AI Labs · https://zyvor.dev
# SPDX-License-Identifier: LicenseRef-Zyvor-Production-1.0
# Uninstallation script for cloud-netconfig

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

stop_service() {
    info "Stopping and disabling service..."

    if systemctl is-active --quiet "$DAEMON.service"; then
        systemctl stop "$DAEMON.service"
    fi

    if systemctl is-enabled --quiet "$DAEMON.service" 2>/dev/null; then
        systemctl disable "$DAEMON.service"
    fi
}

remove_binaries() {
    info "Removing binaries..."

    rm -fv "$BINDIR/$DAEMON"
    rm -fv "$BINDIR/$CTL"
}

remove_systemd() {
    info "Removing systemd service..."

    rm -fv "$LIBDIR/systemd/system/$DAEMON.service"
    systemctl daemon-reload
}

remove_completions() {
    info "Removing shell completions..."

    rm -fv "$DATADIR/bash-completion/completions/$CTL"
    rm -fv "$DATADIR/zsh/site-functions/_$CTL"
    rm -fv "$DATADIR/fish/vendor_completions.d/$CTL.fish"
}

remove_documentation() {
    info "Removing documentation..."

    rm -rfv "$DATADIR/doc/$PROJECT"
}

remove_user() {
    if id "$USER" &>/dev/null; then
        info "Removing system user..."
        userdel "$USER" 2>/dev/null || warn "Could not remove user $USER"
    fi
}

remove_config() {
    warn "Configuration files in $SYSCONFDIR/cloud-network/ will be preserved"
    warn "State data in /var/lib/$PROJECT/ will be preserved"
    echo
    read -p "Do you want to remove configuration and state data? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        info "Removing configuration and state data..."
        rm -rfv "$SYSCONFDIR/cloud-network"
        rm -rfv "/var/lib/$PROJECT"
    else
        info "Configuration and state data preserved"
    fi
}

main() {
    check_root

    info "Uninstalling $PROJECT..."
    echo

    read -p "Are you sure you want to uninstall $PROJECT? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        info "Uninstallation cancelled"
        exit 0
    fi

    stop_service
    remove_binaries
    remove_systemd
    remove_completions
    remove_documentation
    remove_user
    remove_config

    info ""
    info "Uninstallation complete!"
}

main "$@"
