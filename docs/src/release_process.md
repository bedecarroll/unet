# Release Process

This document describes the complete release process for μNet, including automation tools and manual procedures.

## Overview

μNet uses a comprehensive release automation pipeline that includes:

- Automated version management
- Changelog generation
- Release testing
- Multi-platform binary builds
- Package distribution
- Documentation updates
- Release announcements

## Release Types

### Stable Releases

- Version format: `X.Y.Z` (e.g., `1.0.0`, `0.2.1`)
- Thoroughly tested and ready for production use
- Published to all distribution channels

### Pre-releases

- Version format: `X.Y.Z-suffix` (e.g., `1.0.0-beta.1`, `0.2.0-rc.1`)
- Testing releases with new features
- Limited distribution for testing purposes

## Automated Release Process

### Method 1: GitHub Actions (Recommended)

1. **Prepare Release**

   ```bash
   # Go to GitHub Actions → Prepare Release
   # Enter version (e.g., 0.2.0)
   # Choose auto-publish or manual review
   ```

2. **Review Changes** (if manual review selected)
   - Review the generated pull request
   - Update CHANGELOG.md with specific features
   - Merge PR when ready

3. **Publish Release**
   - Tag is automatically created (auto-publish)
   - Or manually create tag after PR merge
   - Release workflow triggers automatically

### Method 2: Command Line Scripts

1. **Full Release (Automated)**

   ```bash
   ./scripts/release.sh full 0.2.0
   ```

2. **Step-by-Step Release**

   ```bash
   # Prepare release
   ./scripts/release.sh prepare 0.2.0
   
   # Review changes
   git diff
   
   # Commit and publish
   git add . && git commit -m "chore: prepare release 0.2.0"
   ./scripts/release.sh publish 0.2.0
   ```

## Manual Release Process

### Prerequisites

1. **Required Tools**

   ```bash
   # Install required tools
   cargo install cargo-audit
   rustup component add rustfmt clippy
   ```

2. **Clean Repository**

   ```bash
   # Ensure clean working directory
   git status
   git stash  # if needed
   git checkout main
   git pull origin main
   ```

### Step 1: Version Preparation

1. **Update Version Numbers**

   ```bash
   # Use version bump script
   ./scripts/version-bump.sh 0.2.0
   
   # Or manually update all Cargo.toml files
   sed -i 's/version = "0.1.0"/version = "0.2.0"/' Cargo.toml
   sed -i 's/version = "0.1.0"/version = "0.2.0"/' crates/*/Cargo.toml
   sed -i 's/version = "0.1.0"/version = "0.2.0"/' migrations/Cargo.toml
   ```

2. **Update CHANGELOG.md**

   ```markdown
   ## [0.2.0] - 2025-07-01
   
   ### Added
   - New feature X
   - Enhancement Y
   
   ### Changed
   - Improved Z
   
   ### Fixed
   - Bug fix A
   ```

### Step 2: Release Testing

1. **Run Comprehensive Tests**

   ```bash
   # Use release testing script
   ./scripts/test-release.sh
   
   # Or run individual test suites
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   cargo build --workspace --release
   ```

2. **Test Documentation**

   ```bash
   # Build and test documentation
   cargo doc --workspace
   mdbook build docs
   mdbook test docs
   ```

### Step 3: Create Release

1. **Commit Changes**

   ```bash
   git add .
   git commit -m "chore: prepare release 0.2.0"
   ```

2. **Create and Push Tag**

   ```bash
   git tag -a v0.2.0 -m "Release 0.2.0"
   git push origin v0.2.0
   ```

3. **Monitor Release Workflow**
   - Check GitHub Actions for release progress
   - Verify builds complete successfully
   - Confirm assets are uploaded

### Step 4: Post-Release

1. **Generate Announcements**

   ```bash
   ./scripts/release.sh announce 0.2.0
   ```

2. **Publish Announcements**
   - Update GitHub release description
   - Send email announcements
   - Post to social media
   - Update documentation sites

## Release Workflow Details

### GitHub Actions Workflows

1. **prepare-release.yml**
   - Triggered manually via GitHub UI
   - Validates version format
   - Updates version numbers
   - Updates changelog
   - Creates PR or publishes directly

2. **release.yml**
   - Triggered by version tags (v*)
   - Runs comprehensive tests
   - Builds multi-platform binaries
   - Creates GitHub release
   - Uploads release assets
   - Publishes documentation

### Release Assets

Each release includes:

- **Linux (x86_64)**: `unet-linux-x86_64.tar.gz`
- **Linux (musl)**: `unet-linux-x86_64-musl.tar.gz`
- **macOS (Intel)**: `unet-macos-x86_64.tar.gz`
- **macOS (Apple Silicon)**: `unet-macos-aarch64.tar.gz`
- **Windows**: `unet-windows-x86_64.zip`

Each archive contains:

- `unet-server` and `unet-cli` binaries
- `README.md` and `CHANGELOG.md`
- Installation script (`install.sh`)
- Documentation (`docs/`)

### Package Distribution

After GitHub release:

1. **Homebrew** (automated via tap)

   ```bash
   brew install unet
   ```

2. **Debian/Ubuntu** (manual upload)

   ```bash
   wget https://github.com/example/unet/releases/download/v0.2.0/unet_0.2.0_amd64.deb
   sudo dpkg -i unet_0.2.0_amd64.deb
   ```

3. **RPM** (manual upload)

   ```bash
   wget https://github.com/example/unet/releases/download/v0.2.0/unet-0.2.0-1.x86_64.rpm
   sudo rpm -i unet-0.2.0-1.x86_64.rpm
   ```

4. **Docker** (automated)

   ```bash
   docker pull ghcr.io/example/unet:0.2.0
   ```

## Release Schedule

- **Major releases** (X.0.0): Quarterly
- **Minor releases** (X.Y.0): Monthly
- **Patch releases** (X.Y.Z): As needed for critical fixes
- **Pre-releases**: Before major/minor releases

## Quality Gates

All releases must pass:

1. ✅ **Code Quality**
   - Formatting check (`cargo fmt`)
   - Linting (`cargo clippy`)
   - Security audit (`cargo audit`)

2. ✅ **Testing**
   - Unit tests
   - Integration tests
   - Documentation tests

3. ✅ **Build Verification**
   - Multi-platform builds
   - Binary execution tests
   - Docker image builds

4. ✅ **Documentation**
   - API documentation builds
   - mdBook documentation builds
   - Changelog updated

## Troubleshooting

### Common Issues

1. **Version Already Exists**

   ```bash
   # Delete local tag
   git tag -d v0.2.0
   
   # Delete remote tag (if needed)
   git push origin :refs/tags/v0.2.0
   ```

2. **Test Failures**

   ```bash
   # Run tests with detailed output
   cargo test --workspace -- --nocapture
   
   # Fix issues and retry
   ./scripts/test-release.sh
   ```

3. **Build Failures**

   ```bash
   # Check specific target build
   cargo build --target x86_64-unknown-linux-gnu --release
   
   # Update dependencies if needed
   cargo update
   ```

### Recovery Procedures

1. **Failed Release**

   ```bash
   # Delete failed release
   gh release delete v0.2.0
   git push origin :refs/tags/v0.2.0
   
   # Fix issues and retry
   git tag -d v0.2.0
   # ... fix issues ...
   git tag -a v0.2.0 -m "Release 0.2.0"
   git push origin v0.2.0
   ```

2. **Rollback Release**

   ```bash
   # Mark release as pre-release
   gh release edit v0.2.0 --prerelease
   
   # Or delete entirely
   gh release delete v0.2.0
   ```

## Security Considerations

- All release scripts validate inputs
- GitHub tokens have minimal required permissions
- Release assets are checksummed
- Binary signatures (planned for future releases)

## Monitoring

Monitor release health:

- GitHub Actions status
- Download metrics
- User feedback
- Issue reports

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for information about contributing to the release process.

---

For questions about the release process, open an issue or discussion on GitHub.
