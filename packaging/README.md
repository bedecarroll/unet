# μNet Package Distribution

This directory contains packaging configurations for distributing μNet across different platforms.

## Debian/Ubuntu Packages

### Prerequisites

```bash
# Install build dependencies
sudo apt-get update
sudo apt-get install build-essential debhelper devscripts dpkg-dev
sudo apt-get install cargo rustc pkg-config libssl-dev libsqlite3-dev libpq-dev

# Install Rust if not already available
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup toolchain install 1.85
rustup default 1.85
```

### Building Packages

```bash
# From the project root
cd packaging
./build-deb.sh
```

This creates two packages:

- `unet_0.1.0-1_amd64.deb` - CLI tool and core libraries
- `unet-server_0.1.0-1_amd64.deb` - HTTP server daemon

### Installation

```bash
# Install both packages
sudo dpkg -i unet_0.1.0-1_amd64.deb unet-server_0.1.0-1_amd64.deb

# Fix any dependency issues
sudo apt-get install -f

# Configure the service
sudo nano /etc/unet/config.toml

# Initialize the database
sudo -u unet unet migrate

# Start and enable the service
sudo systemctl start unet-server
sudo systemctl enable unet-server
```

### Package Contents

#### unet package

- `/usr/bin/unet` - CLI tool
- `/etc/unet/` - Configuration directory
- `/usr/share/unet/` - Example policies and templates
- `/usr/share/doc/unet/` - Documentation

#### unet-server package

- `/usr/bin/unet-server` - HTTP server daemon
- `/lib/systemd/system/unet-server.service` - Systemd service file
- `/var/lib/unet/` - Data directory (created by postinst)
- `/var/log/unet/` - Log directory (created by postinst)

### Configuration

Default configuration files are installed as examples in `/etc/unet/`:

- `config.toml.example` - Basic configuration
- `config-postgres.toml.example` - PostgreSQL configuration
- `config-load-balancer.toml.example` - Load balancer configuration

Copy and customize one of these as `/etc/unet/config.toml`.

### Service Management

```bash
# Check service status
sudo systemctl status unet-server

# View logs
sudo journalctl -u unet-server -f

# Restart service
sudo systemctl restart unet-server

# Stop service
sudo systemctl stop unet-server
```

### Uninstallation

```bash
# Stop and disable service
sudo systemctl stop unet-server
sudo systemctl disable unet-server

# Remove packages
sudo dpkg -r unet-server unet

# Purge packages (removes all data and configuration)
sudo dpkg -P unet-server unet
```

## Security Features

The packages include several security hardening features:

1. **Dedicated user account**: Service runs as `unet` user with minimal privileges
2. **Systemd security**: Service uses security-enhanced systemd configuration
3. **File permissions**: Proper ownership and permissions for configuration and data files
4. **Log rotation**: Automatic log rotation configured

## RPM Packages (RHEL/CentOS/Fedora)

### Prerequisites

```bash
# For RHEL/CentOS 8+
sudo dnf install rpm-build gcc rust cargo openssl-devel sqlite-devel postgresql-devel pkgconfig

# For RHEL/CentOS 7 (requires newer Rust from rustup)
sudo yum install rpm-build gcc openssl-devel sqlite-devel postgresql-devel pkgconfig
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup toolchain install 1.85
rustup default 1.85

# For Fedora
sudo dnf install rpm-build gcc rust cargo openssl-devel sqlite-devel libpq-devel pkgconfig
```

### Building Packages

```bash
# From the project root
cd packaging
./build-rpm.sh
```

This creates two packages:

- `unet-0.1.0-1.el8.x86_64.rpm` - CLI tool and core libraries
- `unet-server-0.1.0-1.el8.x86_64.rpm` - HTTP server daemon

### Installation

```bash
# Install both packages (RHEL/CentOS 8+)
sudo dnf install unet-*.rpm

# Install both packages (RHEL/CentOS 7)
sudo yum install unet-*.rpm

# Or use the generated install script
cd target/rpm-build/rpmbuild
./install-unet.sh

# Configure the service
sudo nano /etc/unet/config.toml

# Initialize the database
sudo -u unet unet migrate

# Start and enable the service
sudo systemctl start unet-server
sudo systemctl enable unet-server
```

### Package Contents

Same as Debian packages but using RPM paths:

- Configuration: `/etc/unet/`
- Binaries: `/usr/bin/`
- Data: `/var/lib/unet/`
- Logs: `/var/log/unet/`
- Service: `/usr/lib/systemd/system/unet-server.service`

## Homebrew Formula (macOS)

### Prerequisites

```bash
# Install Homebrew (if not already installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install Rust (required for building from source)
brew install rust
```

### Installation

#### Option 1: From Official Tap (when published)

```bash
# Add the μNet tap
brew tap example/unet

# Install μNet
brew install unet

# Install with PostgreSQL support
brew install unet --with-postgresql
```

#### Option 2: Local Formula Installation

```bash
# From the project root
brew install --formula packaging/homebrew/unet.rb
```

### Service Management

```bash
# Start the server service
brew services start unet-server

# Stop the server service  
brew services stop unet-server

# Restart the service
brew services restart unet-server

# Check service status
brew services list | grep unet
```

### Configuration and Data Locations

Configuration files:

- Apple Silicon: `/opt/homebrew/etc/unet/`
- Intel Macs: `/usr/local/etc/unet/`

Data and logs:

- Apple Silicon: `/opt/homebrew/var/lib/unet/`, `/opt/homebrew/var/log/unet/`
- Intel Macs: `/usr/local/var/lib/unet/`, `/usr/local/var/log/unet/`

### Creating a Homebrew Tap

```bash
# Initialize a new tap repository
cd packaging
./create-homebrew-tap.sh init

# Test the formula
./create-homebrew-tap.sh test

# Update formula (after making changes)
./create-homebrew-tap.sh update
```

### Uninstallation

```bash
# Stop the service
brew services stop unet-server

# Uninstall the package
brew uninstall unet

# Remove the tap (if using custom tap)
brew untap example/unet
```

## Windows Installer (MSI)

### Prerequisites

```powershell
# Install WiX Toolset
choco install wixtoolset

# Install Rust with MSVC target
rustup target add x86_64-pc-windows-msvc

# Install Visual Studio Build Tools (for Rust MSVC compilation)
```

### Building the Installer

```powershell
# From the project root
cd packaging\windows
.\build-msi.ps1
```

This creates:

- `unet-0.1.0-x64.msi` - Windows MSI installer package

### Installation

```powershell
# Interactive installation
.\unet-0.1.0-x64.msi

# Silent installation
msiexec /i "unet-0.1.0-x64.msi" /quiet

# Using helper script
.\install-unet.ps1

# Silent install with helper
.\install-unet.ps1 -Quiet
```

### Installation Features

The installer supports selective installation:

- **Core Components** - CLI tool, configuration, documentation (required)
- **μNet Server** - HTTP server daemon and Windows service (optional)  
- **Examples and Templates** - Sample policies and templates (optional)

### Installation Layout

- **Program Files**: `C:\Program Files\uNet\`
- **Application Data**: `C:\ProgramData\uNet\`
- **Service**: Windows service "UnetService"
- **Environment**: Binaries added to system PATH

### Service Management

```powershell
# Start the service
Start-Service UnetService

# Stop the service
Stop-Service UnetService

# Check status
Get-Service UnetService

# Configure startup
Set-Service UnetService -StartupType Automatic
```

### Uninstallation

```powershell
# Interactive uninstall
msiexec /x "unet-0.1.0-x64.msi"

# Silent uninstall
msiexec /x "unet-0.1.0-x64.msi" /quiet

# Using helper script
.\install-unet.ps1 -Uninstall
```

## Directory Structure

```
packaging/
├── README.md                    # This file
├── build-deb.sh               # Debian package build script
├── build-rpm.sh               # RPM package build script
├── create-homebrew-tap.sh     # Homebrew tap management script
├── debian/                    # Debian packaging files
│   ├── changelog              # Package changelog
│   ├── compat                 # Debian compatibility level
│   ├── control                # Package metadata and dependencies
│   ├── copyright              # License information
│   ├── rules                  # Build rules
│   ├── unet.install          # File installation rules for CLI package
│   ├── unet.postinst         # Post-installation script for CLI
│   ├── unet.postrm           # Post-removal script for CLI
│   ├── unet-server.install   # File installation rules for server package
│   ├── unet-server.postinst  # Post-installation script for server
│   ├── unet-server.postrm    # Post-removal script for server
│   ├── unet-server.prerm     # Pre-removal script for server
│   └── unet-server.service   # Systemd service file
├── homebrew/                 # Homebrew packaging files
│   └── unet.rb              # Homebrew formula
├── rpm/                      # RPM packaging files
│   ├── SPECS/
│   │   └── unet.spec         # RPM spec file
│   ├── SOURCES/              # Source files for RPM build
│   └── unet-server.service   # Systemd service file for RPM
└── windows/                  # Windows MSI packaging files
    ├── README.md             # Windows-specific documentation
    ├── unet.wxs              # WiX installer source
    ├── build-msi.ps1         # PowerShell build script
    ├── License.rtf           # License file (generated)
    ├── unet-service.exe      # Service wrapper (generated)
    └── configure.exe         # Post-install config (generated)
```

## Troubleshooting

### Build Issues

1. **Missing Rust toolchain**: Install Rust 1.85+ using rustup
2. **Missing system dependencies**: Install build-essential, debhelper, and dev libraries
3. **OpenSSL linking issues**: Ensure libssl-dev is installed

### Runtime Issues

1. **Service won't start**: Check configuration syntax and permissions
2. **Database connection issues**: Verify database credentials and connectivity
3. **Permission denied**: Ensure unet user has access to necessary directories

### Common Commands

```bash
# Check package information
dpkg -l | grep unet
dpkg -s unet
dpkg -L unet

# Check service logs
sudo journalctl -u unet-server --no-pager
sudo tail -f /var/log/unet/unet-server.log

# Test CLI tool
unet --version
unet nodes list
unet health
```

## Support

For issues related to packaging:

1. Check the troubleshooting section above
2. Review system logs: `sudo journalctl -u unet-server`
3. Check application logs: `/var/log/unet/`
4. Report issues at: <https://github.com/example/unet/issues>
