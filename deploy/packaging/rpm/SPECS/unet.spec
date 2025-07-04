Name:           unet
Version:        0.1.0
Release:        1%{?dist}
Summary:        Network configuration management and automation tool

License:        MIT
URL:            https://github.com/example/unet
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust >= 1.85
BuildRequires:  cargo >= 1.85
BuildRequires:  gcc
BuildRequires:  openssl-devel
BuildRequires:  sqlite-devel
BuildRequires:  postgresql-devel
BuildRequires:  pkgconfig

# Runtime dependencies
Requires:       openssl
Requires:       sqlite
Requires(pre):  shadow-utils
Requires(post): systemd
Requires(preun): systemd
Requires(postun): systemd

%description
μNet is a comprehensive network configuration management and automation
tool designed for modern network infrastructure. It provides:

* Declarative configuration management using templates and policies
* Real-time network state monitoring via SNMP
* Git-based version control for network configurations
* REST API for programmatic access
* Command-line interface for operators
* Policy-driven configuration validation
* Template-based configuration generation

%package server
Summary:        μNet HTTP server for network configuration management
Requires:       %{name} = %{version}-%{release}

%description server
HTTP server component of μNet that provides REST API endpoints for
network configuration management. Features include:

* RESTful API for configuration management
* JWT-based authentication and RBAC authorization
* TLS/HTTPS support with certificate management
* Prometheus metrics and monitoring
* Background tasks for SNMP polling
* Database support (SQLite and PostgreSQL)

This is the server daemon that should be deployed on infrastructure
management servers.

%prep
%autosetup

%build
# Set environment variables for build
export OPENSSL_NO_VENDOR=1
export CARGO_TARGET_DIR=%{_builddir}/target

# Build the workspace
cargo build --release --workspace --bins

%install
# Create directories
install -d %{buildroot}%{_bindir}
install -d %{buildroot}%{_sysconfdir}/unet
install -d %{buildroot}%{_unitdir}
install -d %{buildroot}%{_datadir}/unet/policies
install -d %{buildroot}%{_datadir}/unet/templates
install -d %{buildroot}%{_docdir}/%{name}
install -d %{buildroot}%{_localstatedir}/lib/unet
install -d %{buildroot}%{_localstatedir}/log/unet

# Install binaries
install -m 755 target/release/unet %{buildroot}%{_bindir}/unet
install -m 755 target/release/unet-server %{buildroot}%{_bindir}/unet-server

# Install configuration files
install -m 644 config.toml %{buildroot}%{_sysconfdir}/unet/config.toml.example
install -m 644 config-postgres.toml %{buildroot}%{_sysconfdir}/unet/config-postgres.toml.example
install -m 644 config-load-balancer.toml %{buildroot}%{_sysconfdir}/unet/config-load-balancer.toml.example

# Install systemd service file
install -m 644 packaging/rpm/unet-server.service %{buildroot}%{_unitdir}/unet-server.service

# Install example policies and templates
if [ -d policies ]; then
    cp -r policies/* %{buildroot}%{_datadir}/unet/policies/
fi
if [ -d templates ]; then
    cp -r templates/* %{buildroot}%{_datadir}/unet/templates/
fi

# Install documentation
install -m 644 README.md %{buildroot}%{_docdir}/%{name}/
if [ -d docs/book ]; then
    cp -r docs/book %{buildroot}%{_docdir}/%{name}/html
fi

%pre
# Create unet user and group
getent group unet >/dev/null || groupadd -r unet
getent passwd unet >/dev/null || \
    useradd -r -g unet -d %{_localstatedir}/lib/unet -s /sbin/nologin \
    -c "μNet service user" unet

%post
# Set up default configuration if none exists
if [ ! -f %{_sysconfdir}/unet/config.toml ]; then
    if [ -f %{_sysconfdir}/unet/config.toml.example ]; then
        cp %{_sysconfdir}/unet/config.toml.example %{_sysconfdir}/unet/config.toml
        chown root:unet %{_sysconfdir}/unet/config.toml
        chmod 640 %{_sysconfdir}/unet/config.toml
        echo "Default configuration created at %{_sysconfdir}/unet/config.toml"
        echo "Please review and customize the configuration before starting the service."
    fi
fi

# Set up log rotation
if [ ! -f %{_sysconfdir}/logrotate.d/unet ]; then
    cat > %{_sysconfdir}/logrotate.d/unet << 'EOF'
%{_localstatedir}/log/unet/*.log {
    daily
    missingok
    rotate 14
    compress
    delaycompress
    notifempty
    copytruncate
    su unet unet
}
EOF
fi

%post server
%systemd_post unet-server.service

# Don't automatically start the service - let the admin configure it first
echo "μNet server service installed but not started."
echo "Please:"
echo "1. Review and customize %{_sysconfdir}/unet/config.toml"
echo "2. Initialize the database: unet migrate"
echo "3. Start the service: systemctl start unet-server"
echo "4. Enable on boot: systemctl enable unet-server"

%preun server
%systemd_preun unet-server.service

%postun
# Remove user and group on uninstall
if [ $1 -eq 0 ]; then
    userdel unet >/dev/null 2>&1 || :
    groupdel unet >/dev/null 2>&1 || :
    rm -f %{_sysconfdir}/logrotate.d/unet
fi

%postun server
%systemd_postun_with_restart unet-server.service

%files
%license LICENSE-MIT
%doc %{_docdir}/%{name}/README.md
%{_bindir}/unet
%dir %attr(750,root,unet) %{_sysconfdir}/unet
%config(noreplace) %{_sysconfdir}/unet/config.toml.example
%config(noreplace) %{_sysconfdir}/unet/config-postgres.toml.example
%config(noreplace) %{_sysconfdir}/unet/config-load-balancer.toml.example
%{_datadir}/unet/
%dir %attr(755,unet,unet) %{_localstatedir}/lib/unet
%dir %attr(755,unet,unet) %{_localstatedir}/log/unet
%if 0%{?rhel} >= 8 || 0%{?fedora} >= 28
%doc %{_docdir}/%{name}/html/
%endif

%files server
%{_bindir}/unet-server
%{_unitdir}/unet-server.service

%changelog
* Mon Jun 30 2025 μNet Development Team <dev@unet.example.com> - 0.1.0-1
- Initial release of μNet network configuration management tool
- Features:
  * Declarative configuration management with templates and policies
  * Real-time SNMP monitoring and state tracking
  * Git-based version control integration
  * REST API with JWT authentication and RBAC
  * Command-line interface for network operators
  * Policy engine with custom DSL for validation rules
  * Template engine with MiniJinja for configuration generation
  * Database support for SQLite and PostgreSQL
  * TLS/HTTPS support with certificate management
  * Prometheus metrics and comprehensive monitoring
  * Horizontal scaling with load balancer compatibility
  * Container support with Docker and Kubernetes manifests
- Security features:
  * Input validation and sanitization
  * Rate limiting and DDoS protection
  * Security audit logging
  * Vulnerability scanning integration
  * Secure credential storage with external secret managers
- Observability:
  * Structured logging with multiple output formats
  * Prometheus metrics with custom business metrics
  * Multi-channel alerting and escalation procedures
  * Grafana dashboards and operational runbooks
- Performance optimizations:
  * Multi-layer caching and connection pooling
  * Async processing and resource management
  * Memory optimization and graceful degradation
  * Benchmarking framework and performance monitoring