# Getting Started

## Purpose

Install cloud-netconfigd and verify metadata-driven networking on a cloud VM.

## How to get there

- Topic id: `getting-started`
- Section: **Onboarding → Getting Started**

## Guide

cloud-netconfig runs as a systemd daemon on Azure, AWS EC2, or GCP VMs. It watches instance metadata and netlink events, then configures supplementary interfaces, custom routing tables, and policy rules so multi-homed hosts route correctly.

## Prerequisites

- A cloud VM with secondary network interfaces attached in the provider console.
- Rust toolchain and `make` for source builds, or a packaged release from GitHub.
- sudo/root for install and route management.

## Install and enable

```bash
git clone https://github.com/zyvorai/zyvor-cloud-netconfig.git
cd zyvor-cloud-netconfig
make build && sudo make install
sudo cp /etc/cloud-network/config.yaml /etc/cloud-network/cloud-network.yaml
sudo systemctl enable --now cloud-netconfigd
```

## First verification

```bash
cnctl status all
curl -s http://127.0.0.1:5209/api/status | jq .
```

## Troubleshooting

- **Service fails on bare metal or non-cloud VMs** — auto-detection requires provider metadata; use a supported cloud instance for testing.
- **Config not loaded** — ensure `cloud-network.yaml` exists (see [Configuration](../setup/configuration.md)).

## Next steps

- [Configuration](../setup/configuration.md)
- [Common workflows](../operations/workflows.md)
- [Admin basics](../admin/admin-basics.md)

## Operate (CLI)

1. After install, run `cnctl status all` before changing production routing.
2. Confirm `/api/status` returns `"status":"running"` and the expected provider string.
3. **Empty / fail:** Read `journalctl -u cloud-netconfigd` for metadata or netlink errors.
4. **Success:** Secondary NICs listed in config show addresses and policy routes in `ip rule` / `ip route show table`.

## Related pages

- [Getting Started](../../getting-started.md)
- [Page index](../../PAGE_INDEX.md)
