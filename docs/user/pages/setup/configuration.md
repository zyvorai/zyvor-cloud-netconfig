# Configuration

## Purpose

YAML daemon config: logging, metadata refresh, supplementary interfaces, and routing.

## How to get there

- Topic id: `configuration`
- Section: **Setup → Configuration**

## Guide

All daemon behaviour is driven by `/etc/cloud-network/cloud-network.yaml`. Key knobs:

- **`network.interfaces.enabled`** — supplementary NICs to configure from metadata.
- **`network.routing`** — custom table base (`table_base`), policy routing, default route management.
- **`metadata.refresh_interval`** — how often metadata is re-fetched (default `300s`).
- **`cloud.*`** — auto-detect or pin provider; tune Azure API version, AWS IMDSv1/v2, GCP recursive fetch.

Copy a provider example from `distribution/etc/cloud-network/examples/` as a starting point.

## Validate

```bash
cnctl apply --dry-run
cnctl apply --dry-run -c distribution/etc/cloud-network/examples/aws.yaml
```

## Apply

```bash
sudo cp my.yaml /etc/cloud-network/cloud-network.yaml
sudo systemctl restart cloud-netconfigd
```

## Troubleshooting

- **MTU override ignored** — the schema field is `mtu.override_value` (see annotated `config.yaml`).
- **Daemon ignores edits** — wrong filename; active path is `cloud-network.yaml`, not only the packaged `config.yaml`.

## Next steps

- [Getting started](../onboarding/getting-started.md)
- [Common workflows](../operations/workflows.md)
- [Admin basics](../admin/admin-basics.md)

## Operate (CLI)

1. Stage changes in a non-production path; always `--dry-run` first.
2. Diff against examples for your cloud (`aws.yaml`, `azure.yaml`, `gcp.yaml`).
3. **Empty / fail:** Dry-run stderr shows YAML parse or duration validation errors.
4. **Success:** Restart completes; `cnctl status network` reflects new interfaces.

## Related pages

- [Getting Started](../../getting-started.md)
- [Page index](../../PAGE_INDEX.md)
