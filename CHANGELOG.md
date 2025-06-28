# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Release automation pipeline with GitHub Actions
- Changelog management and versioning support
- Release testing procedures
- Release announcement templates

## [0.1.0] - 2025-06-30

### Added
- Complete μNet network configuration management system
- Rust-based CLI tool (`unet-cli`) for network management
- REST API server (`unet-server`) with Axum framework
- Core library (`unet-core`) with shared functionality
- Configuration slicer (`config-slicer`) for hierarchical config diffing
- SQLite and PostgreSQL database support with SeaORM
- Policy engine with DSL for network configuration rules
- Template engine with MiniJinja for configuration templates
- SNMP polling and monitoring capabilities
- Git integration for configuration version control
- JWT authentication and RBAC authorization
- TLS/HTTPS support with certificate management
- Comprehensive logging, metrics, and alerting
- Horizontal scaling support with load balancing
- Resource management and performance optimization
- Container packaging (Docker, Kubernetes, Helm)
- Package distribution (Debian, RPM, Homebrew, Windows MSI)
- Deployment automation and configuration management
- Complete documentation with mdBook
- User guides, tutorials, and examples
- Security audit logging and vulnerability scanning
- Backup/recovery procedures and operational runbooks

### Security
- Input validation and sanitization
- Rate limiting and security audit logging
- Network access controls and secure credential storage
- Vulnerability scanning integration
- Security compliance documentation

[Unreleased]: https://github.com/example/unet/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/example/unet/releases/tag/v0.1.0