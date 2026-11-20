# Security

Report vulnerabilities privately to **info@zyvor.dev** or via [zyvor.dev/contact](https://zyvor.dev/contact). Do not open public issues for undisclosed security problems.

`cloud-netconfigd` runs as an unprivileged user with `CAP_NET_ADMIN` only and fetches metadata from cloud provider link-local endpoints.
