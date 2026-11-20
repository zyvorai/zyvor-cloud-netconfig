# Configuration

cloud-netconfig's config surface is a single YAML file read by `cloud-netconfigd` at startup. Supplementary interfaces, metadata refresh, routing tables, and provider-specific options all live in this file.

## Config file location

| Consumer | Path |
|----------|------|
| Daemon (`cloud-netconfigd`) | `/etc/cloud-network/cloud-network.yaml` |
| CLI validate (`cnctl apply`) | `/etc/cloud-network/cloud-network.yaml` (override with `-c`) |
| Packaged example (after install) | `/etc/cloud-network/config.yaml` |

After `make install`, copy or symlink the packaged example to `cloud-network.yaml` so the daemon loads it.

Annotated reference: `distribution/etc/cloud-network/config.yaml` in the repository. Provider-specific examples live under `distribution/etc/cloud-network/examples/` (`aws.yaml`, `azure.yaml`, `gcp.yaml`, `minimal.yaml`, `multi-interface.yaml`, `production.yaml`).

## Core sections

```yaml
logging:
  level: info          # trace | debug | info | warn | error
  format: text         # text | json

server:
  listen:
    address: 127.0.0.1
    port: 5209

metadata:
  refresh_interval: 300s
  request_timeout: 10s

network:
  interfaces:
    enabled:
      - eth1
      - eth2
  routing:
    table_base: 9999
    policy_routing: true
    manage_default_routes: true

cloud:
  auto_detect: true
  # provider: aws | azure | gcp  (when auto_detect: false)

security:
  user: cloud-network
  capabilities:
    - CAP_NET_ADMIN
```

## Validate before restart

```bash
cnctl apply --dry-run
cnctl apply --dry-run -c /path/to/staging.yaml
```

Dry-run parses YAML, validates durations and port settings, and prints a summary without restarting the daemon. To apply:

```bash
sudo cp my-config.yaml /etc/cloud-network/cloud-network.yaml
sudo systemctl restart cloud-netconfigd
```

## Provider overrides

| Provider | Key fields |
|----------|--------------|
| Azure | `cloud.azure.api_version` |
| AWS | `cloud.aws.imds_version` (1 or 2), `cloud.aws.token_ttl` |
| GCP | `cloud.gcp.recursive` |

Set `cloud.auto_detect: false` and `cloud.provider: aws` when you need a fixed provider (see `examples/aws.yaml` for IMDSv2).

## Troubleshooting

- **Daemon uses defaults despite editing config** — confirm the active file is `cloud-network.yaml`, not only `config.yaml`.
- **Invalid duration errors** — use suffixed values (`300s`, `5m`, `1h`); bare numbers are rejected.
- **Supplementary NIC ignored** — list exact interface names in `network.interfaces.enabled`; patterns in the example file are comments only until wired in your copy.

## Next steps

- [Getting started](getting-started.md)
- [Common workflows](workflows.md)
- [Admin basics](admin-basics.md)

## Operate (CLI)

1. Edit `/etc/cloud-network/cloud-network.yaml` (or stage a file elsewhere).
2. Validate: `cnctl apply --dry-run -c /path/to/file.yaml`.
3. Install: `sudo cp file.yaml /etc/cloud-network/cloud-network.yaml`.
4. **Empty / fail:** Dry-run output shows parse/validation errors; fix YAML before restart.
5. **Success:** `systemctl restart cloud-netconfigd` completes; `cnctl status all` reflects the new listen address and interface list.
