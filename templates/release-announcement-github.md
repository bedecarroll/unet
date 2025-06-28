# μNet {{version}} Released 🚀

We're excited to announce the release of μNet {{version}}! This release brings significant improvements to network configuration management and automation.

## 🎯 What's New

{{#if major_features}}
### Major Features
{{#each major_features}}
- {{this}}
{{/each}}
{{/if}}

{{#if improvements}}
### Improvements
{{#each improvements}}
- {{this}}
{{/each}}
{{/if}}

{{#if bug_fixes}}
### Bug Fixes
{{#each bug_fixes}}
- {{this}}
{{/each}}
{{/if}}

{{#if breaking_changes}}
## ⚠️ Breaking Changes

{{#each breaking_changes}}
- {{this}}
{{/each}}

Please review the [migration guide]({{migration_guide_url}}) for upgrade instructions.
{{/if}}

## 📥 Installation

### Quick Install

```bash
# Download and install latest release
curl -sSL https://github.com/example/unet/releases/download/v{{version}}/install.sh | bash
```

### Manual Download

Download the appropriate binary for your platform from the [releases page](https://github.com/example/unet/releases/tag/v{{version}}):

- **Linux (x86_64)**: [unet-linux-x86_64.tar.gz](https://github.com/example/unet/releases/download/v{{version}}/unet-linux-x86_64.tar.gz)
- **Linux (musl)**: [unet-linux-x86_64-musl.tar.gz](https://github.com/example/unet/releases/download/v{{version}}/unet-linux-x86_64-musl.tar.gz)
- **macOS (Intel)**: [unet-macos-x86_64.tar.gz](https://github.com/example/unet/releases/download/v{{version}}/unet-macos-x86_64.tar.gz)
- **macOS (Apple Silicon)**: [unet-macos-aarch64.tar.gz](https://github.com/example/unet/releases/download/v{{version}}/unet-macos-aarch64.tar.gz)
- **Windows**: [unet-windows-x86_64.zip](https://github.com/example/unet/releases/download/v{{version}}/unet-windows-x86_64.zip)

### Package Managers

```bash
# Homebrew (macOS/Linux)
brew install unet

# Debian/Ubuntu
wget https://github.com/example/unet/releases/download/v{{version}}/unet_{{version}}_amd64.deb
sudo dpkg -i unet_{{version}}_amd64.deb

# RHEL/CentOS/Fedora
wget https://github.com/example/unet/releases/download/v{{version}}/unet-{{version}}-1.x86_64.rpm
sudo rpm -i unet-{{version}}-1.x86_64.rpm

# Docker
docker pull ghcr.io/example/unet:{{version}}
```

## 📖 Documentation

- **Getting Started**: [User Guide](https://example.github.io/unet/user_getting_started.html)
- **Configuration**: [Configuration Tutorial](https://example.github.io/unet/user_config_tutorial.html)
- **API Reference**: [API Documentation](https://example.github.io/unet/api_reference.html)
- **Full Documentation**: [μNet Documentation](https://example.github.io/unet/)

## 🔧 Upgrade Instructions

{{#if upgrade_notes}}
### From Previous Versions

{{#each upgrade_notes}}
- {{this}}
{{/each}}
{{/if}}

For detailed upgrade instructions, see the [upgrade guide]({{upgrade_guide_url}}).

## 🐛 Known Issues

{{#if known_issues}}
{{#each known_issues}}
- {{this}}
{{/each}}
{{else}}
No known issues at this time.
{{/if}}

## 🤝 Contributing

We welcome contributions! Please see our [contributing guide](https://github.com/example/unet/blob/main/CONTRIBUTING.md) for details.

## 📝 Full Changelog

For a complete list of changes, see the [CHANGELOG.md](https://github.com/example/unet/blob/v{{version}}/CHANGELOG.md).

## 🙏 Acknowledgments

{{#if contributors}}
Special thanks to all contributors who made this release possible:

{{#each contributors}}
- @{{this}}
{{/each}}
{{/if}}

## 🔗 Links

- [Release Downloads](https://github.com/example/unet/releases/tag/v{{version}})
- [Documentation](https://example.github.io/unet/)
- [Issue Tracker](https://github.com/example/unet/issues)
- [Discussions](https://github.com/example/unet/discussions)

---

**Release Date**: {{release_date}}  
**Release Manager**: {{release_manager}}

For questions or support, please visit our [discussions page](https://github.com/example/unet/discussions) or [open an issue](https://github.com/example/unet/issues/new).