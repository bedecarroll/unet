<!-- SPDX-License-Identifier: MIT -->

# Secrets Management Guide

> **Audience:** Administrators responsible for protecting credentials.
> **Objective:** Safely store and rotate device and service secrets.

μNet stores sensitive data using a libsodium-based key vault. Secrets are encrypted at rest and never written to logs.

## Adding Secrets

Use the CLI to add credentials:

```bash
unet secrets add --name router1 --value "p@ssw0rd"
```

Secrets can also be sourced from external systems such as AWS Secrets Manager or HashiCorp Vault when the relevant feature flags are enabled.

## Best Practices

- Avoid committing secrets to configuration repositories.
- Rotate credentials regularly and revoke unused keys.

## See Also

- [Security and Compliance Guide](security_compliance_guide.md)
- [Server Backend](06_server_backend.md#103-change-management--secrets)
