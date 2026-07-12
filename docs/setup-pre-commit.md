# Setting Up Pre-commit Hooks

μNet uses mise for pre-commit hooks instead of the traditional pre-commit framework. This ensures **perfect consistency** between local development and CI environments.

## Quick Setup

```bash
# Install the pre-commit hook (runs automatically when you commit)
mise generate git-pre-commit > .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

## What the Pre-commit Hook Does

When you run `git commit`, it automatically executes `mise run pre-commit` which runs the **exact same `lint` task** used in CI:

1. **Fixes typos** with `typos -w`
2. **Formats code** with `cargo fmt`  
3. **Runs clippy** with `--allow-dirty --fix` (auto-fixes issues when possible)

## Benefits

✅ **Zero duplication** - same task as CI uses  
✅ **Auto-fixes** code issues when possible  
✅ **Fast feedback** - catch issues before pushing  
✅ **Consistent** - identical behavior locally and in CI

## Manual Tasks

You can also run the same checks manually:

```bash
# Run the same checks as pre-commit hook
mise run pre-commit

# Or run lint directly (same thing)
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