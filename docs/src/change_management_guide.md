# Change Management Guide

> **Audience:** Network operators coordinating configuration changes.
> **Objective:** Provide a reliable workflow for proposing, approving, and applying changes.

The change management subsystem tracks all configuration updates before they are applied to devices.

## Typical Workflow

1. **Propose a change** using the CLI:

```bash
unet changes propose --file router-update.toml
```

2. **Review and approve** the change:

```bash
unet changes approve <change-id>
```

3. **Apply** after approval:

```bash
unet changes apply <change-id>
```

Change requests are stored in the `configuration_changes` table. The backend enforces that only approved changes are executed.

## See Also

- [Server Backend](06_server_backend.md#103-change-management--secrets)
- [CLI Tool](05_cli_tool.md#change-commands)
- [User Configuration Tutorial](user_config_tutorial.md)
