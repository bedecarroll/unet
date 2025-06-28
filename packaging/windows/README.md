# Windows MSI Installer for μNet

This directory contains the Windows Installer package configuration using WiX Toolset.

## Prerequisites

### Required Tools

1. **WiX Toolset 3.11+**

   ```powershell
   # Download and install from https://wixtoolset.org/
   # Or install via Chocolatey
   choco install wixtoolset
   ```

2. **Rust Toolchain**

   ```powershell
   # Install Rust
   Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "rustup-init.exe"
   .\rustup-init.exe
   
   # Add Windows MSVC target
   rustup target add x86_64-pc-windows-msvc
   ```

3. **Visual Studio Build Tools** (for Rust MSVC target)

   ```powershell
   # Install Visual Studio Build Tools or Visual Studio Community
   # Ensure C++ build tools and Windows SDK are installed
   ```

### Optional Tools

- **MSI Validation Tool** (msival2.exe) - for installer validation
- **Orca** - for MSI inspection and debugging

## Building the Installer

### Quick Start

```powershell
# From the packaging/windows directory
.\build-msi.ps1
```

### Build Options

```powershell
# Clean previous builds
.\build-msi.ps1 -Clean

# Build debug version
.\build-msi.ps1 -Debug

# Clean and rebuild
.\build-msi.ps1 -Clean
.\build-msi.ps1
```

### Build Output

The build process creates:

- `target/windows-installer/unet-0.1.0-x64.msi` - Main installer
- `target/windows-installer/install-unet.ps1` - Installation helper script

## Installation

### Interactive Installation

```powershell
# Run the MSI installer
.\unet-0.1.0-x64.msi

# Or use the helper script
.\install-unet.ps1
```

### Silent Installation

```powershell
# Silent install
msiexec /i "unet-0.1.0-x64.msi" /quiet

# Silent install with logging
msiexec /i "unet-0.1.0-x64.msi" /quiet /l*v install.log

# Using helper script
.\install-unet.ps1 -Quiet
```

### Installation Features

The installer supports selective feature installation:

- **Core Components** - CLI tool, configuration, documentation (required)
- **μNet Server** - HTTP server daemon and Windows service (optional)
- **Examples and Templates** - Sample policies and templates (optional)

## Installation Layout

After installation, μNet is organized as follows:

### Program Files

- **Installation Directory**: `C:\Program Files\uNet\`
  - `bin\` - Executables (unet.exe, unet-server.exe)
  - `config\` - Configuration templates
  - `docs\` - Documentation
  - `examples\` - Example policies and templates

### Application Data

- **Data Directory**: `C:\ProgramData\uNet\data\`
- **Logs Directory**: `C:\ProgramData\uNet\logs\`

### Registry

- **Installation Info**: `HKLM\Software\uNet\`
- **User Settings**: `HKCU\Software\uNet\Installed\`

### Environment

- μNet binaries are added to the system PATH
- Environment variables are set for service configuration

## Service Management

The installer creates a Windows service for the μNet server:

```powershell
# Start the service
Start-Service UnetService

# Stop the service
Stop-Service UnetService

# Check service status
Get-Service UnetService

# Configure service startup
Set-Service UnetService -StartupType Automatic
```

### Service Configuration

The service is configured with:

- **Name**: UnetService
- **Display Name**: μNet Network Configuration Management Service
- **Startup**: Manual (configurable)
- **Account**: Local System
- **Recovery**: Automatic restart on failure

## Configuration

### Default Configuration

The installer creates example configuration files:

- `config.toml.example` - Basic configuration
- `config-postgres.toml.example` - PostgreSQL configuration
- `config-load-balancer.toml.example` - Load balancer configuration

### First-Time Setup

1. **Copy and customize configuration**:

   ```powershell
   cd "C:\Program Files\uNet\config"
   copy config.toml.example config.toml
   notepad config.toml
   ```

2. **Initialize the database**:

   ```powershell
   unet migrate
   ```

3. **Start the service**:

   ```powershell
   Start-Service UnetService
   ```

## Uninstallation

### Interactive Uninstallation

```powershell
# From Programs and Features (Add/Remove Programs)
# Or run the MSI again and choose Remove

# Using MSI directly
msiexec /x "unet-0.1.0-x64.msi"

# Using helper script
.\install-unet.ps1 -Uninstall
```

### Silent Uninstallation

```powershell
# Silent uninstall
msiexec /x "unet-0.1.0-x64.msi" /quiet

# Silent uninstall with logging
msiexec /x "unet-0.1.0-x64.msi" /quiet /l*v uninstall.log

# Using helper script
.\install-unet.ps1 -Uninstall -Quiet
```

## Troubleshooting

### Build Issues

1. **WiX Toolset not found**:
   - Ensure WiX Toolset is installed and in PATH
   - Restart PowerShell after installation

2. **Rust compilation fails**:
   - Verify Rust toolchain: `rustc --version`
   - Install MSVC target: `rustup target add x86_64-pc-windows-msvc`
   - Install Visual Studio Build Tools

3. **Missing dependencies**:
   - Check project builds: `cargo check --workspace`
   - Ensure all required system libraries are available

### Installation Issues

1. **Installer won't run**:
   - Run as Administrator
   - Check Windows Installer service is running
   - Verify MSI file integrity

2. **Service won't start**:
   - Check configuration file syntax
   - Verify database connectivity
   - Review Windows Event Log

3. **Permission errors**:
   - Ensure proper file permissions
   - Check User Account Control (UAC) settings
   - Run as Administrator when necessary

### Common Commands

```powershell
# Check installation
unet --version
unet health

# View service logs
Get-WinEvent -LogName Application | Where-Object {$_.ProviderName -eq "UnetService"}

# Check installation registry
Get-ItemProperty "HKLM:\Software\uNet"

# Verify files
Get-ChildItem "C:\Program Files\uNet" -Recurse
```

## Development

### Modifying the Installer

1. **Edit WiX source**: `unet.wxs`
2. **Update build script**: `build-msi.ps1`
3. **Test changes**: `.\build-msi.ps1 -Debug`
4. **Validate**: Use Orca or msival2.exe

### Adding Features

1. **Define new components** in `unet.wxs`
2. **Add to appropriate features**
3. **Update file references**
4. **Test installation scenarios**

### Debugging

```powershell
# Enable MSI logging
msiexec /i "unet-0.1.0-x64.msi" /l*v debug.log

# View detailed log
notepad debug.log

# Use Orca for MSI inspection
orca unet-0.1.0-x64.msi
```

## File Structure

```
packaging/windows/
├── README.md              # This file
├── unet.wxs              # WiX source file
├── build-msi.ps1         # Build script
├── License.rtf           # License for installer (generated)
├── banner.bmp            # Installer banner (optional)
├── dialog.bmp            # Installer dialog image (optional)
├── unet-service.exe      # Service wrapper (generated)
└── configure.exe         # Post-install configuration (generated)
```

## Support

For Windows-specific issues:

1. Check Windows Event Log
2. Review MSI installation logs
3. Verify Windows Installer service
4. Report issues at: <https://github.com/example/unet/issues>
