# SPDX-License-Identifier: LicenseRef-Zyvor-Production-1.0

Name:           cloud-netconfig
Version:        0.3.0
Release:        1%{?dist}
Summary:        Cloud instance network configuration daemon

License:        LicenseRef-Zyvor-Production-1.0URL:            https://github.com/ssahani/cloud-netconfig
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust >= 1.70
BuildRequires:  systemd-rpm-macros
BuildRequires:  make

Requires:       systemd
Requires(pre):  shadow-utils
Requires(post): systemd
Requires(preun): systemd
Requires(postun): systemd

%description
Automatically configures network interfaces for cloud instances by fetching
metadata from cloud providers (AWS, Azure, GCP, Oracle Cloud, Alibaba Cloud,
DigitalOcean).

Features:
* Automatic cloud provider detection
* Dynamic network interface configuration
* Support for multiple network interfaces
* Policy-based routing for supplementary interfaces
* Systemd integration with watchdog support
* Configurable refresh intervals
* HTTP API for status monitoring

%prep
%autosetup

%build
cargo build --release

%install
# Binaries
install -D -m 0755 target/release/cloud-netconfigd %{buildroot}%{_bindir}/cloud-netconfigd
install -D -m 0755 target/release/cnctl %{buildroot}%{_bindir}/cnctl

# Configuration
install -D -m 0644 distribution/etc/cloud-network/config.yaml %{buildroot}%{_sysconfdir}/cloud-network/config.yaml

# Examples
install -d %{buildroot}%{_docdir}/%{name}/examples
install -m 0644 distribution/etc/cloud-network/examples/*.yaml %{buildroot}%{_docdir}/%{name}/examples/

# Systemd service
install -D -m 0644 distribution/lib/systemd/system/cloud-netconfigd.service %{buildroot}%{_unitdir}/cloud-netconfigd.service

# Shell completions
install -D -m 0644 distribution/usr/share/bash-completion/completions/cnctl %{buildroot}%{_datadir}/bash-completion/completions/cnctl
install -D -m 0644 distribution/usr/share/zsh/site-functions/_cnctl %{buildroot}%{_datadir}/zsh/site-functions/_cnctl
install -D -m 0644 distribution/usr/share/fish/vendor_completions.d/cnctl.fish %{buildroot}%{_datadir}/fish/vendor_completions.d/cnctl.fish

# State directory
install -d -m 0755 %{buildroot}%{_sharedstatedir}/%{name}

%pre
getent group cloud-network >/dev/null || groupadd -r cloud-network
getent passwd cloud-network >/dev/null || \
    useradd -r -g cloud-network -d /run/cloud-network -s /sbin/nologin \
    -c "Cloud Network Configuration Daemon" cloud-network
exit 0

%post
%systemd_post cloud-netconfigd.service

%preun
%systemd_preun cloud-netconfigd.service

%postun
%systemd_postun_with_restart cloud-netconfigd.service

if [ $1 -eq 0 ]; then
    # Package removal (not upgrade)
    getent passwd cloud-network >/dev/null && userdel cloud-network
fi

%files
%license LICENSE.txt
%doc README.md
%doc %{_docdir}/%{name}/examples/

%{_bindir}/cloud-netconfigd
%{_bindir}/cnctl

%dir %{_sysconfdir}/cloud-network
%config(noreplace) %{_sysconfdir}/cloud-network/config.yaml

%{_unitdir}/cloud-netconfigd.service

%{_datadir}/bash-completion/completions/cnctl
%{_datadir}/zsh/site-functions/_cnctl
%{_datadir}/fish/vendor_completions.d/cnctl.fish

%dir %attr(0755,cloud-network,cloud-network) %{_sharedstatedir}/%{name}

%changelog
* Tue Jan 21 2025 ZyvorAI Labs Private Limited <legal@zyvor.dev> - 0.3.0-1
- Initial RPM release
