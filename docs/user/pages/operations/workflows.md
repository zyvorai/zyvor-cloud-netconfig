# Workflows

## Purpose

Day-2 validate, reload, multi-interface rollout, and upgrade jobs with cnctl.

## How to get there

- Topic id: `workflows`
- Section: **Operations → Workflows**

## Guide

| Job | Commands |
|-----|----------|
| Health check | `cnctl status all` · `curl http://127.0.0.1:5209/health` |
| Config rollout | `cnctl apply --dry-run` → copy YAML → `systemctl restart cloud-netconfigd` |
| Reload | `sudo systemctl reload cloud-netconfigd` (after `cnctl reload` reminder) |
| Debug | Set `logging.level: debug` → restart → `journalctl -u cloud-netconfigd -f` |
| Upgrade | Stop service → replace binaries → start → `cnctl version` |

## Multi-interface rollout

1. Attach secondary NICs in the cloud console.
2. Copy `examples/multi-interface.yaml` and set `network.interfaces.enabled`.
3. Validate with dry-run, restart daemon, confirm routes with `ip rule` and `ip route show table 9999` (or your `table_base`).

## Operate (CLI)

1. Prefer dry-run and journal inspection before restarting on production multi-homed hosts.
2. Use `cnctl status network` to confirm netlink state matches provider metadata.
3. **Empty / fail:** Capture logs and metadata connectivity (IMDS/GCP metadata endpoints).
4. **Success:** Traffic egresses via the intended interface per policy rules.

## Related pages

- [Getting Started](../../getting-started.md)
- [Page index](../../PAGE_INDEX.md)
