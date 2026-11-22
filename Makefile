# Copyright 2026 Zyvor AI Labs · https://zyvor.dev
# SPDX-License-Identifier: LicenseRef-Zyvor-Production-2.0
HASH := $(shell git rev-parse --short HEAD)
COMMIT_DATE := $(shell git show -s --format=%ci ${HASH})
BUILD_DATE := $(shell date '+%Y-%m-%d %H:%M:%S')
VERSION := ${HASH} (${COMMIT_DATE})

BUILDDIR ?= .
SRCDIR ?= .

.PHONY: help
help:
	@echo "make [TARGETS...]"
	@echo
	@echo "This is the maintenance makefile of cloud-network-setup. The following"
	@echo "targets are available:"
	@echo
	@echo "    help:               Print this usage information."
	@echo "    build:              Builds project"
	@echo "    install:            Installs binary, configuration and unit files"
	@echo "    clean:              Cleans the build"

$(BUILDDIR)/:
	mkdir -p "$@"

$(BUILDDIR)/%/:
	mkdir -p "$@"

.PHONY: build
build:
	- mkdir -p bin
	cargo build --release
	cp target/release/cloud-netconfigd bin/
	cp target/release/cnctl bin/

.PHONY: install
install:
	# Binaries
	install -v -m 0755 bin/cloud-netconfigd /usr/bin/
	install -v -m 0755 bin/cnctl /usr/bin/

	# Configuration
	install -v -d -m 0755 /etc/cloud-network
	install -v -m 0644 distribution/etc/cloud-network/config.yaml /etc/cloud-network/

	# Systemd service
	install -v -m 0644 distribution/lib/systemd/system/cloud-netconfigd.service /lib/systemd/system/
	systemctl daemon-reload

	# Shell completions
	install -v -d -m 0755 /usr/share/bash-completion/completions
	install -v -m 0644 distribution/usr/share/bash-completion/completions/cnctl /usr/share/bash-completion/completions/
	install -v -d -m 0755 /usr/share/zsh/site-functions
	install -v -m 0644 distribution/usr/share/zsh/site-functions/_cnctl /usr/share/zsh/site-functions/
	install -v -d -m 0755 /usr/share/fish/vendor_completions.d
	install -v -m 0644 distribution/usr/share/fish/vendor_completions.d/cnctl.fish /usr/share/fish/vendor_completions.d/

PHONY: clean
clean:
	cargo clean
	rm -rf bin
