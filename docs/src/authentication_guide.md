<!-- SPDX-License-Identifier: MIT -->

# Authentication Guide

> **Audience:** Administrators securing μNet deployments.
> **Objective:** Understand authentication modes and configuration options.

μNet supports three authentication modes for the API server:

| Mode   | Description                               |
| ------ | ----------------------------------------- |
| `none` | No authentication, suitable for testing.  |
| `basic`| HTTP Basic using the `users` table.        |
| `jwt`  | Bearer JWT tokens with role management.   |

## Configuring Authentication

Set the desired mode in the server configuration:

```toml
[server]
auth = { mode = "jwt", secret = "/etc/unet/jwt-secret.pem" }
```

Refer to the [Server Backend](06_server_backend.md#10--security--auth) for all available fields.

## Managing Users

Use the CLI to manage user accounts and roles:

```bash
unet users add --username alice --role admin
```

See the [CLI Tool](05_cli_tool.md) for full command details.

## See Also

- [Security and Compliance Guide](security_compliance_guide.md)
- [Server Backend](06_server_backend.md)
