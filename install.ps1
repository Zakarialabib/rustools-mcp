# Windows Installer for Rust Tools MCP
$ErrorActionPreference = "Stop"

$REPO_OWNER = "zakarialabib"
$REPO_NAME = "rustools-mcp"
$INSTALL_DIR = "$env:LOCALAPPDATA\rustools-mcp"
$BIN_NAME = "rustools-mcp.exe"

Write-Host "🚀 Installing rustools-mcp..." -ForegroundColor Cyan

# 1. Create Installation Directory
if (-not (Test-Path $INSTALL_DIR)) {
    New-Item -ItemType Directory -Force -Path $INSTALL_DIR | Out-Null
    Write-Host "Created installation directory: $INSTALL_DIR" -ForegroundColor Green
}

# 2. Determine Architecture
$arch = $env:PROCESSOR_ARCHITECTURE
if ($arch -eq "AMD64") {
    $asset_name = "rustools-mcp-windows-amd64.exe"
} else {
    Write-Error "Unsupported architecture: $arch"
}

# 3. Fetch Latest Release (Mocked logic for now, assumes direct download URL structure)
# In a real scenario, we'd query GitHub API to get the latest tag.
# For now, we'll assume the user builds it locally or we use a placeholder URL.
$download_url = "https://github.com/$REPO_OWNER/$REPO_NAME/releases/latest/download/$asset_name"

Write-Host "Downloading from: $download_url" -ForegroundColor Gray
# Invoke-WebRequest -Uri $download_url -OutFile "$INSTALL_DIR\$BIN_NAME" 
# NOTE: Commented out because the release doesn't exist yet. 
# Instructions for the user:
Write-Host "⚠️  NOTE: Since this is a dev environment, please build the binary manually:" -ForegroundColor Yellow
Write-Host "   cargo build --release" -ForegroundColor Yellow
Write-Host "   Copy-Item target\release\rustools-mcp.exe $INSTALL_DIR\$BIN_NAME" -ForegroundColor Yellow

# 4. Add to PATH
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$INSTALL_DIR*") {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$INSTALL_DIR", "User")
    Write-Host "Added $INSTALL_DIR to user PATH." -ForegroundColor Green
    Write-Host "Please restart your terminal for changes to take effect." -ForegroundColor Cyan
} else {
    Write-Host "$INSTALL_DIR is already in PATH." -ForegroundColor Gray
}

Write-Host "✅ Installation Setup Complete!" -ForegroundColor Green
