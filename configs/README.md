# μNet Configuration Management

This directory contains configuration templates and environment-specific configurations for μNet.

## Directory Structure

```
configs/
├── templates/           # Configuration templates
│   ├── production.toml    # Production-ready template with security hardening
│   ├── staging.toml       # Staging template mirroring production
│   └── development.toml   # Development template with debugging features
├── environments/        # Environment-specific configurations
│   ├── production/        # Production environment
│   │   ├── unet.toml       # Production configuration
│   │   └── .env.template   # Environment variables template
│   ├── staging/           # Staging environment
│   │   ├── unet.toml       # Staging configuration
│   │   └── .env.template   # Environment variables template
│   └── development/       # Development environment
│       ├── unet.toml       # Development configuration
│       └── .env.template   # Environment variables template
└── README.md            # This file
```

## Usage

### 1. Choose Your Environment

Select the appropriate environment configuration based on your deployment:

- **Development**: Local development with SQLite, relaxed security, verbose logging
- **Staging**: Pre-production environment mirroring production with reduced resources
- **Production**: Production-ready with PostgreSQL, security hardening, monitoring

### 2. Copy Configuration

Copy the environment-specific configuration to your deployment location:

```bash
# For production
cp configs/environments/production/unet.toml /etc/unet/config.toml

# For staging
cp configs/environments/staging/unet.toml /etc/unet/config.toml

# For development
cp configs/environments/development/unet.toml ./config.toml
```

### 3. Set Environment Variables

Copy and customize the environment variables template:

```bash
# For production
cp configs/environments/production/.env.template /etc/unet/.env
# Edit /etc/unet/.env with your actual values

# For staging
cp configs/environments/staging/.env.template /etc/unet/.env
# Edit /etc/unet/.env with staging values

# For development
cp configs/environments/development/.env.template ./.env
# Edit ./.env with local development overrides
```

### 4. Validate Configuration

Use the configuration validation tool to ensure your configuration is correct:

```bash
unet config validate --config /path/to/config.toml
```

## Configuration Templates

### Production Template (`templates/production.toml`)

Features:

- PostgreSQL database with connection pooling
- TLS/HTTPS enforcement
- JWT authentication required
- Comprehensive logging and monitoring
- Security hardening (network restrictions, geolocation)
- Resource management and throttling
- Cluster coordination and high availability
- Secrets management with HashiCorp Vault

### Staging Template (`templates/staging.toml`)

Features:

- PostgreSQL database (mirrors production)
- TLS enabled but more permissive
- Enhanced debugging and logging
- Reduced resource limits
- Shorter retention periods
- Auto-migration enabled for testing

### Development Template (`templates/development.toml`)

Features:

- SQLite database for simplicity
- Authentication disabled by default
- Verbose debug logging
- TLS disabled for ease of development
- Local Git repositories
- Minimal resource constraints
- All monitoring and clustering disabled

## Environment Variables

Configuration supports environment variable overrides using the `UNET_` prefix:

```bash
# Override database URL
export UNET_DATABASE_URL="postgresql://user:pass@host:5432/db"

# Override log level
export UNET_LOGGING_LEVEL="debug"

# Override server port
export UNET_SERVER_PORT=9080
```

Variable naming follows the configuration hierarchy with underscores:

- `database.url` → `UNET_DATABASE_URL`
- `server.port` → `UNET_SERVER_PORT`
- `logging.level` → `UNET_LOGGING_LEVEL`

## Security Considerations

### Production Security

1. **Secrets Management**: Use environment variables or external secret managers
2. **Database Credentials**: Never hardcode passwords in configuration files
3. **TLS Certificates**: Use proper certificates, not self-signed for production
4. **JWT Secrets**: Use cryptographically secure random strings (256-bit minimum)
5. **Network Security**: Configure appropriate IP allowlists and geolocation blocking

### Secret Storage

Production configurations use placeholder values that must be replaced:

```toml
# Use environment variables for secrets
jwt_secret = "${UNET_JWT_SECRET}"
url = "postgresql://user:${DB_PASSWORD}@host:5432/db"
```

### File Permissions

Ensure configuration files have appropriate permissions:

```bash
# Configuration files
chmod 640 /etc/unet/config.toml
chown unet:unet /etc/unet/config.toml

# Environment files (contain secrets)
chmod 600 /etc/unet/.env
chown unet:unet /etc/unet/.env
```

## Customization

### Creating Custom Templates

1. Copy an existing template as a starting point
2. Modify sections relevant to your environment
3. Update placeholder values with your specific requirements
4. Test with the validation tool

### Template Variables

Templates support placeholder syntax for common substitutions:

```toml
# Placeholder syntax examples
database_url = "{{ database_url | default('sqlite:./unet.db') }}"
cluster_name = "{{ cluster_name }}"
log_level = "{{ log_level | default('info') }}"
```

## Validation

The configuration validation tool checks:

- Required fields are present
- Database URL format is valid
- File paths exist for certificates and keys
- Network ranges are valid CIDR notation
- Resource limits are within reasonable bounds
- Cross-field dependencies are satisfied

Example validation:

```bash
# Validate configuration
unet config validate --config config.toml

# Validate with environment variables
unet config validate --config config.toml --env .env

# Validate and show resolved configuration
unet config validate --config config.toml --show-resolved
```

## Migration

### Upgrading Configurations

When upgrading μNet versions:

1. Check for new configuration options in release notes
2. Compare your configuration with the latest templates
3. Add new sections with appropriate defaults
4. Validate the updated configuration
5. Test in staging before production deployment

### Configuration Migration Tool

Use the configuration migration tool for version upgrades:

```bash
unet config migrate --from config-old.toml --to config-new.toml --version 1.1.0
```

## Troubleshooting

### Common Issues

1. **Database Connection Failures**
   - Check database URL format
   - Verify network connectivity
   - Confirm credentials are correct

2. **TLS Certificate Errors**
   - Verify certificate file paths
   - Check certificate validity and expiration
   - Ensure private key matches certificate

3. **Authentication Issues**
   - Verify JWT secret is set
   - Check token expiration settings
   - Confirm user roles and permissions

4. **Configuration Loading Errors**
   - Validate TOML syntax
   - Check file permissions
   - Verify environment variable names

### Debug Mode

Enable debug mode for detailed configuration loading information:

```bash
UNET_LOGGING_LEVEL=debug unet server --config config.toml
```

## Best Practices

1. **Version Control**: Store templates and environment configurations in version control
2. **Secret Management**: Use external secret managers for production
3. **Validation**: Always validate configurations before deployment
4. **Documentation**: Document any custom modifications
5. **Testing**: Test configuration changes in staging first
6. **Monitoring**: Monitor configuration reload events in production
7. **Backup**: Backup working configurations before making changes
