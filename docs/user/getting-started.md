# Getting started

cloud-netconfig applies secondary IPs and policy routing from Azure, AWS EC2, and GCP instance metadata — event-driven via netlink, so changes propagate without manual route edits. This page gets you from install to a running daemon.

## Prerequisites

- Root/sudo access on the host (installs a systemd unit and modifies routes).
- The host must run on a supported cloud (Azure, AWS EC2, or GCP) so instance metadata is reachable.
- Rust toolchain and `make` to build from source (see the upstream README for exact versions).

## 1. Build and install

```bash
git clone https://github.com/zyvorai/zyvor-cloud-netconfig.git
cd zyvor-cloud-netconfig
make build
sudo make install
sudo useradd -M -s /usr/bin/nologin cloud-network 2>/dev/null || true
```

This builds and installs `cloud-netconfigd` (daemon) and `cnctl` (CLI).

## 2. Activate configuration

Packaging installs the annotated example as `/etc/cloud-network/config.yaml`. The daemon reads `/etc/cloud-network/cloud-network.yaml` — copy or symlink before first start:

```bash
sudo cp /etc/cloud-network/config.yaml /etc/cloud-network/cloud-network.yaml
# or: sudo ln -sf /etc/cloud-network/config.yaml /etc/cloud-network/cloud-network.yaml
```

Edit supplementary interfaces (`network.interfaces.enabled`) for your VM's secondary NICs.

## 3. Start the daemon

```bash
sudo systemctl enable --now cloud-netconfigd
```

## 4. Verify

```bash
cnctl status all
curl -s http://127.0.0.1:5209/health
curl -s http://127.0.0.1:5209/api/status
```

`cnctl status all` prints daemon status, detected cloud provider, and local interface addresses. The local HTTP API defaults to `127.0.0.1:5209`.

## Troubleshooting

- **`cnctl status` shows "Daemon Status: not running"** — confirm `systemctl status cloud-netconfigd` and that port `5209` is listening on localhost.
- **Daemon exits immediately** — cloud auto-detection failed; the host must be on Azure, AWS, or GCP. Check `journalctl -u cloud-netconfigd -n 50`.
- **No secondary routes** — confirm `network.interfaces.enabled` lists the correct NIC names and metadata assigns those interfaces.

## Next steps

- [Configuration](configuration.md)
- [Common workflows](workflows.md)
- [Admin basics](admin-basics.md)

## Operate (CLI)

1. Confirm the unit is active: `systemctl is-active cloud-netconfigd`.
2. Read daemon health: `curl -s http://127.0.0.1:5209/api/status`.
3. Inspect host networking: `cnctl status network`.
4. **Empty / fail:** Check `journalctl -u cloud-netconfigd -f`, cloud metadata reachability, and config path (`cloud-network.yaml`).
5. **Success:** Status shows `running`, a provider name, and configured interfaces with addresses.
