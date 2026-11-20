# Admin Basics

## Purpose

Packaging, systemd unit, local HTTP API, privileges, and ops.

## How to get there

- Topic id: `admin-basics`
- Section: **Admin → Admin Basics**

## Guide

| Topic | Detail |
|-------|--------|
| **Unit** | `cloud-netconfigd.service` — notify type, watchdog, hardened syscall filter |
| **Capabilities** | `CAP_NET_ADMIN` only after privilege drop |
| **API** | `/health`, `/api/status`, `/api/cloud/status` on `server.listen` |
| **State** | `/run/cloud-network` — metadata persistence when `state.persist_metadata: true` |
| **Completions** | bash, zsh, fish installed under `/usr/share/` |

## Logs and service control

```bash
sudo systemctl status cloud-netconfigd
sudo journalctl -u cloud-netconfigd -f
```

## Security notes

- Keep the HTTP listener on localhost unless you add TLS and access controls.
- Restrict `/etc/cloud-network/*.yaml` to root-readable permissions.
- Review `SECURITY.md` before exposing metadata-derived routes on shared tenants.

## Operate (CLI)

1. Monitor via systemd and journald — there is no remote metrics endpoint in cloud-netconfig.
2. After package upgrades, verify `systemctl cat cloud-netconfigd` matches shipped unit file.
3. **Empty / fail:** Check `cloud-network` user exists and `/run/cloud-network` is writable.
4. **Success:** Watchdog stays healthy; API returns `running` status.

## Related pages

- [Getting Started](../../getting-started.md)
- [Page index](../../PAGE_INDEX.md)
