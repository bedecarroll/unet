# PowerShell script to build MSI installer for μNet
# Usage: .\build-msi.ps1 [-Clean] [-Debug]

param(
    [switch]$Clean,
    [switch]$Debug
)

$ErrorActionPreference = "Stop"

# Configuration
$ProjectRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$PackagingDir = "$ProjectRoot\packaging\windows"
$BuildDir = "$ProjectRoot\target\windows-installer"
$WixSourceFile = "$PackagingDir\unet.wxs"
$WixObjFile = "$BuildDir\unet.wixobj"
$MsiFile = "$BuildDir\unet-0.1.0-x64.msi"

Write-Host "Building Windows MSI installer for μNet..." -ForegroundColor Green
Write-Host "Project root: $ProjectRoot"
Write-Host "Build directory: $BuildDir"

# Function to check for required tools
function Test-Prerequisites {
    Write-Host "Checking prerequisites..." -ForegroundColor Yellow
    
    # Check for WiX Toolset
    $wixPath = Get-Command "candle.exe" -ErrorAction SilentlyContinue
    if (-not $wixPath) {
        Write-Error "WiX Toolset not found. Please install WiX Toolset 3.11+ from https://wixtoolset.org/"
        exit 1
    }
    
    $lightPath = Get-Command "light.exe" -ErrorAction SilentlyContinue
    if (-not $lightPath) {
        Write-Error "WiX Light tool not found. Please ensure WiX Toolset is properly installed."
        exit 1
    }
    
    # Check for Rust/Cargo
    $cargoPath = Get-Command "cargo.exe" -ErrorAction SilentlyContinue
    if (-not $cargoPath) {
        Write-Error "Cargo not found. Please install Rust from https://rustup.rs/"
        exit 1
    }
    
    Write-Host "All prerequisites found." -ForegroundColor Green
}

# Function to clean previous builds
function Remove-BuildArtifacts {
    Write-Host "Cleaning previous build artifacts..." -ForegroundColor Yellow
    
    if (Test-Path $BuildDir) {
        Remove-Item -Recurse -Force $BuildDir
    }
    
    # Clean Rust build
    Push-Location $ProjectRoot
    try {
        & cargo clean
    }
    finally {
        Pop-Location
    }
    
    Write-Host "Clean completed." -ForegroundColor Green
}

# Function to build Rust binaries
function Build-RustBinaries {
    Write-Host "Building Rust binaries..." -ForegroundColor Yellow
    
    Push-Location $ProjectRoot
    try {
        # Build for Windows x64
        $env:CARGO_TARGET_DIR = "$ProjectRoot\target"
        
        if ($Debug) {
            & cargo build --workspace --bins --target x86_64-pc-windows-msvc
            $script:BinaryPath = "$ProjectRoot\target\x86_64-pc-windows-msvc\debug"
        } else {
            & cargo build --release --workspace --bins --target x86_64-pc-windows-msvc
            $script:BinaryPath = "$ProjectRoot\target\x86_64-pc-windows-msvc\release"
        }
        
        if ($LASTEXITCODE -ne 0) {
            throw "Rust build failed with exit code $LASTEXITCODE"
        }
        
        # Verify binaries exist
        $requiredBinaries = @("unet.exe", "unet-server.exe")
        foreach ($binary in $requiredBinaries) {
            $binaryPath = Join-Path $BinaryPath $binary
            if (-not (Test-Path $binaryPath)) {
                throw "Required binary not found: $binaryPath"
            }
        }
        
        Write-Host "Rust binaries built successfully." -ForegroundColor Green
    }
    finally {
        Pop-Location
    }
}

# Function to prepare build environment
function Initialize-BuildEnvironment {
    Write-Host "Preparing build environment..." -ForegroundColor Yellow
    
    # Create build directory
    New-Item -ItemType Directory -Force -Path $BuildDir | Out-Null
    
    # Copy binaries to expected locations for WiX
    $targetRelease = "$ProjectRoot\target\release"
    New-Item -ItemType Directory -Force -Path $targetRelease | Out-Null
    
    Copy-Item "$BinaryPath\unet.exe" "$targetRelease\unet.exe" -Force
    Copy-Item "$BinaryPath\unet-server.exe" "$targetRelease\unet-server.exe" -Force
    
    # Create placeholder service wrapper (would need actual implementation)
    $serviceExe = "$PackagingDir\unet-service.exe"
    if (-not (Test-Path $serviceExe)) {
        Write-Host "Creating placeholder service executable..." -ForegroundColor Yellow
        # This would normally be a proper Windows service wrapper
        Copy-Item "$BinaryPath\unet-server.exe" $serviceExe -Force
    }
    
    # Create configuration helper (would need actual implementation)
    $configExe = "$PackagingDir\configure.exe"
    if (-not (Test-Path $configExe)) {
        Write-Host "Creating placeholder configuration executable..." -ForegroundColor Yellow
        # This would normally be a configuration utility
        Copy-Item "$BinaryPath\unet.exe" $configExe -Force
    }
    
    # Create license file if it doesn't exist
    $licenseRtf = "$PackagingDir\License.rtf"
    if (-not (Test-Path $licenseRtf)) {
        Write-Host "Creating license file..." -ForegroundColor Yellow
        @"
{\rtf1\ansi\deff0 {\fonttbl {\f0 Times New Roman;}}
\f0\fs24
μNet Network Configuration Management Tool\par
\par
Licensed under the MIT License\par
\par
Copyright 2025 μNet Development Team\par
\par
Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:\par
\par
The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.\par
\par
THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.\par
}
"@ | Out-File -FilePath $licenseRtf -Encoding ASCII
    }
    
    Write-Host "Build environment prepared." -ForegroundColor Green
}

# Function to compile WiX source
function Build-WixInstaller {
    Write-Host "Compiling WiX installer..." -ForegroundColor Yellow
    
    Push-Location $ProjectRoot
    try {
        # Compile WiX source to object file
        Write-Host "Running candle.exe..." -ForegroundColor Cyan
        & candle.exe -ext WixUtilExtension -out $WixObjFile $WixSourceFile
        
        if ($LASTEXITCODE -ne 0) {
            throw "WiX compilation failed with exit code $LASTEXITCODE"
        }
        
        # Link object file to create MSI
        Write-Host "Running light.exe..." -ForegroundColor Cyan
        & light.exe -ext WixUIExtension -ext WixUtilExtension -out $MsiFile $WixObjFile
        
        if ($LASTEXITCODE -ne 0) {
            throw "WiX linking failed with exit code $LASTEXITCODE"
        }
        
        Write-Host "MSI installer created successfully." -ForegroundColor Green
        Write-Host "Output: $MsiFile" -ForegroundColor Cyan
    }
    finally {
        Pop-Location
    }
}

# Function to test the installer
function Test-Installer {
    Write-Host "Testing installer..." -ForegroundColor Yellow
    
    if (-not (Test-Path $MsiFile)) {
        throw "MSI file not found: $MsiFile"
    }
    
    # Get MSI properties
    Write-Host "MSI Properties:" -ForegroundColor Cyan
    $msiInfo = Get-ItemProperty $MsiFile
    Write-Host "  File: $($msiInfo.Name)"
    Write-Host "  Size: $([math]::Round($msiInfo.Length / 1MB, 2)) MB"
    Write-Host "  Created: $($msiInfo.CreationTime)"
    
    # Test MSI validation (if available)
    $msiValidate = Get-Command "msival2.exe" -ErrorAction SilentlyContinue
    if ($msiValidate) {
        Write-Host "Running MSI validation..." -ForegroundColor Cyan
        & msival2.exe $MsiFile /v
    }
    
    Write-Host "Installer testing completed." -ForegroundColor Green
}

# Function to create installation script
function New-InstallationScript {
    $installScript = "$BuildDir\install-unet.ps1"
    
    @"
# Installation script for μNet MSI installer
# Run as Administrator

param(
    [switch]`$Uninstall,
    [switch]`$Quiet
)

`$MsiFile = "`$PSScriptRoot\unet-0.1.0-x64.msi"

if (-not (Test-Path `$MsiFile)) {
    Write-Error "MSI file not found: `$MsiFile"
    exit 1
}

if (`$Uninstall) {
    Write-Host "Uninstalling μNet..." -ForegroundColor Yellow
    if (`$Quiet) {
        Start-Process "msiexec.exe" -ArgumentList "/x `"`$MsiFile`" /quiet" -Wait
    } else {
        Start-Process "msiexec.exe" -ArgumentList "/x `"`$MsiFile`"" -Wait
    }
} else {
    Write-Host "Installing μNet..." -ForegroundColor Green
    if (`$Quiet) {
        Start-Process "msiexec.exe" -ArgumentList "/i `"`$MsiFile`" /quiet" -Wait
    } else {
        Start-Process "msiexec.exe" -ArgumentList "/i `"`$MsiFile`"" -Wait
    }
}

Write-Host "Installation completed." -ForegroundColor Green
"@ | Out-File -FilePath $installScript -Encoding UTF8
    
    Write-Host "Installation script created: $installScript" -ForegroundColor Green
}

# Main execution
try {
    # Handle clean parameter
    if ($Clean) {
        Remove-BuildArtifacts
        Write-Host "Clean completed." -ForegroundColor Green
        exit 0
    }
    
    # Check prerequisites
    Test-Prerequisites
    
    # Clean previous builds
    Remove-BuildArtifacts
    
    # Build Rust binaries
    Build-RustBinaries
    
    # Prepare build environment
    Initialize-BuildEnvironment
    
    # Build WiX installer
    Build-WixInstaller
    
    # Test installer
    Test-Installer
    
    # Create installation script
    New-InstallationScript
    
    Write-Host ""
    Write-Host "Windows MSI installer built successfully!" -ForegroundColor Green
    Write-Host "Installer: $MsiFile" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "To install:" -ForegroundColor Yellow
    Write-Host "  msiexec /i `"$MsiFile`""
    Write-Host "  or run: $BuildDir\install-unet.ps1"
    Write-Host ""
    Write-Host "To uninstall:" -ForegroundColor Yellow
    Write-Host "  msiexec /x `"$MsiFile`""
    Write-Host "  or run: $BuildDir\install-unet.ps1 -Uninstall"
}
catch {
    Write-Error "Build failed: $_"
    exit 1
}