# Setting Up Pre-commit Hooks

μNet uses `mise` for pre-commit hooks instead of the traditional `pre-commit` framework. Local hooks and CI share the same task definitions, but they now use separate read-only and autofix tasks.

## Quick Setup

```bash
# Install the pre-commit hook (runs automatically when you commit)
mise generate git-pre-commit > .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

## What the Pre-commit Hook Does

When you run `git commit`, it automatically executes `mise run pre-commit`, which depends on `lint-fix` for local autofix behavior:

1. **Fixes typos** with `typos -w`
2. **Formats code** with `cargo fmt`  
3. **Runs clippy** with `--allow-dirty --fix`

CI continues to use the separate read-only `mise run ci-lint` task, which depends on `mise run lint`.

## Manual Tasks

You can run the local autofix path or the read-only CI path directly:

```bash
# Run the same autofix task as the pre-commit hook
mise run pre-commit

# Or run the autofix task directly
mise run lint-fix

# Run the read-only checks used by CI and `status`
mise run lint
```

## Manual Override

If you need to commit without running hooks (emergency situations only):

```bash
git commit --no-verify -m "emergency: bypass pre-commit"
```

## Troubleshooting

**Hook not running?**
```bash
# Check if hook exists and is executable
ls -la .git/hooks/pre-commit

# Regenerate hook if needed
mise generate git-pre-commit > .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

**Want to check what the hook will do?**
```bash
# Test the pre-commit task manually
mise run pre-commit
```
