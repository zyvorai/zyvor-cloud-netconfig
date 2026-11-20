# Admin basics

| Topic | Guidance |
|-------|----------|
| **Binaries** | `/usr/bin/cloud-netconfigd` (daemon), `/usr/bin/cnctl` (CLI) |
| **Service** | `cloud-netconfigd.service` — `Type=notify`, watchdog enabled, `CAP_NET_ADMIN` |
| **Config dir** | `/etc/cloud-network/` — active daemon config: `cloud-network.yaml` |
| **Runtime state** | `/run/cloud-network/` (metadata cache, per-interface files when enabled) |
| **Logs** | `journalctl -u cloud-netconfigd -f` |
| **User** | Drops to `cloud-network` system user after startup when run as root |
| **Local API** | `127.0.0.1:5209` by default — not exposed beyond localhost unless you change `server.listen` |
| **Examples** | `/usr/share/doc/cloud-netconfig/examples/` after package install |
| **Security** | Follow upstream `SECURITY.md`; restrict config file permissions (`0644`, root-owned) |
| **Support** | [GitHub Issues](https://github.com/zyvorai/zyvor-cloud-netconfig/issues) · [Contact Zyvor](https://zyvor.dev/contact) for Enterprise |

## systemd essentials

```bash
sudo systemctl enable --now cloud-netconfigd
sudo systemctl status cloud-netconfigd
sudo systemctl restart cloud-netconfigd
sudo journalctl -u cloud-netconfigd -f
```

The unit uses `WatchdogSec=60s` and expects the daemon to ping systemd when `security.watchdog.enabled` is true in config.

## Packaging paths

| Artifact | Installed path |
|----------|----------------|
| Daemon + CLI | `/usr/bin/` |
| Default config example | `/etc/cloud-network/config.yaml` |
| systemd unit | `/lib/systemd/system/cloud-netconfigd.service` |
| Shell completions | bash, zsh, fish under `/usr/share/` |

DEB/RPM/Arch packages follow the same layout — see `distribution/README.md` in the repository.

## Privileges

The daemon starts as root, creates state directories, then drops to `cloud-network` while retaining `CAP_NET_ADMIN` for netlink route and address operations.

See also [Getting started](getting-started.md).

## Operate (CLI)

1. Treat `cloud-netconfigd.service` as the source of truth for process health.
2. Use the local API only from the host (`curl` to `127.0.0.1:5209`) — do not expose the port publicly without TLS and auth.
3. After upgrades, run `cnctl version` and compare with `/api/status` JSON.
4. **Empty / fail:** Inspect unit logs and confirm the `cloud-network` user exists.
5. **Success:** Service is `active (running)` and watchdog notifications stay healthy in `journalctl`.
