# Common workflows

| Workflow | Steps |
|----------|-------|
| **Fresh install** | Clone → `make build` → `sudo make install` → symlink/copy config → `systemctl enable --now cloud-netconfigd` |
| **Validate** | `cnctl status all` → `curl http://127.0.0.1:5209/api/status` → `journalctl -u cloud-netconfigd -n 30` |
| **Config change** | Edit YAML → `cnctl apply --dry-run` → copy to `cloud-network.yaml` → `systemctl restart cloud-netconfigd` |
| **Reload signal** | `cnctl reload` (prints systemd guidance) → `sudo systemctl reload cloud-netconfigd` |
| **Upgrade binary** | `systemctl stop cloud-netconfigd` → replace `/usr/bin/cloud-netconfigd` and `/usr/bin/cnctl` → `systemctl start cloud-netconfigd` |
| **Multi-interface rollout** | Start from `examples/multi-interface.yaml` → set `network.interfaces.enabled` → enable `network.routing.policy_routing` |

## cnctl command reference

| Command | Purpose |
|---------|---------|
| `cnctl status [system\|network\|all]` | Daemon, cloud provider, and interface state (default: `all`) |
| `cnctl apply [-c PATH] [--dry-run]` | Validate (and optionally stage) configuration |
| `cnctl reload [--force]` | Reload guidance (systemd reload) |
| `cnctl version` | CLI and license version |

## Local HTTP API

| Endpoint | Response |
|----------|----------|
| `GET /health` | `OK` |
| `GET /api/status` | JSON: `status`, `provider`, `version` |
| `GET /api/cloud/status` | `OK` |

Bind address comes from `server.listen` (default `127.0.0.1:5209`).

## Debug metadata or routing

1. Set `logging.level: debug` in config.
2. `sudo systemctl restart cloud-netconfigd`.
3. `journalctl -u cloud-netconfigd -f`.

Runtime state is written under `/run/cloud-network` (configurable via `state.directory`).

## Operate (CLI)

1. Pick the workflow row that matches your job (install, validate, config change, upgrade).
2. Run the listed commands in order; use `--dry-run` before any restart that changes routing.
3. For mutating changes, schedule a maintenance window — policy routes affect egress paths.
4. **Empty / fail:** Capture `journalctl -u cloud-netconfigd` and `cnctl status network` before retrying.
5. **Success:** `cnctl status all` shows running daemon, expected provider, and addresses on supplementary NICs.
