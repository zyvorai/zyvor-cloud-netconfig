# cloud-netconfig — User Documentation

Automatic network configuration for cloud VMs — secondary IPs, routing tables, and policy-based routing from Azure, AWS EC2, and GCP instance metadata.

| You want to… | Open |
|--------------|------|
| Install and verify | [Getting Started](getting-started.md) |
| Configure the daemon | [Configuration](configuration.md) |
| Run common jobs | [Workflows](workflows.md) |
| Deploy / systemd / API | [Admin basics](admin-basics.md) |
| Full topic index | [PAGE_INDEX.md](PAGE_INDEX.md) |

## Printable PDFs

```bash
node scripts/user-docs/build-user-pdfs.mjs
```

Output lands in [`pdf/`](pdf/):

- `cloud-netconfig-User-README.pdf`
- `cloud-netconfig-Getting-Started.pdf`
- `cloud-netconfig-Page-by-Page.pdf`
- `cloud-netconfig-Admin-Basics.pdf`

Also available: [Using cnctl (CLI)](using-the-dashboard.md).

**→ Product:** https://zyvor.dev/cloud-netconfig · **GitHub:** https://github.com/zyvorai/zyvor-cloud-netconfig
