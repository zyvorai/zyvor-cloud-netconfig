# Using cnctl (CLI)

cloud-netconfig is operated through **cnctl** and the **cloud-netconfigd** systemd service — there is no first-party web console.

## Essentials

```bash
cnctl status all
cnctl status network
cnctl apply --dry-run
cnctl version
curl -s http://127.0.0.1:5209/api/status
```

## Where to go next

| Job | Doc |
|-----|-----|
| First install | [Getting Started](getting-started.md) |
| Config format | [Configuration](configuration.md) |
| Day-2 jobs | [Workflows](workflows.md) |
| Host / packaging | [Admin Basics](admin-basics.md) |
| Topic index | [PAGE_INDEX.md](PAGE_INDEX.md) |

## Operate (CLI)

1. Run `cnctl status all` after any install or config change.
2. Use `cnctl apply --dry-run` before copying YAML to `/etc/cloud-network/cloud-network.yaml`.
3. **Empty / fail:** Fall back to `journalctl -u cloud-netconfigd` and the `/health` endpoint.
4. **Success:** Commands exit 0 and status output matches the cloud VM's metadata and interfaces.
