# pawlos Windows Installer
# Run: iwr -useb https://raw.githubusercontent.com/relharrati/pawlos-ai-agent/main/scripts/install.ps1 | iex

[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$ErrorActionPreference = "Stop"

# Config
$REPO = "relharrati/pawlos-ai-agent"
$BRANCH = "master"
$BIN_NAME = "pawlos"
$INSTALL_DIR = "$env:LOCALAPPDATA\Programs\pawlos"
$CONFIG_DIR = "$env:USERPROFILE\.pawlos"

function Write-Info($msg) { Write-Host "[pawlos] $msg" -ForegroundColor Cyan }
function Write-Ok($msg)   { Write-Host "[  ok  ] $msg" -ForegroundColor Green }
function Write-Err($msg)  { Write-Host "[ err  ] $msg" -ForegroundColor Red; exit 1 }

Write-Host ""
Write-Host "  ____    _             _           _           " -ForegroundColor Magenta
Write-Host " |  _ \  | |           | |         | |         " -ForegroundColor Magenta
Write-Host " | | | | | |_  _   _  | |_  _   _ | |_        " -ForegroundColor Magenta
Write-Host " | | | | | __|| | | | | __|| | | || __|       " -ForegroundColor Magenta
Write-Host " | |_| | | |_ | |_| | | |_ | |_| || |_        " -ForegroundColor Magenta
Write-Host "  \___/   \__| \__,_|  \__| \__,_| \__|       " -ForegroundColor Magenta
Write-Host ""
Write-Info "Installing pawlos..."
Write-Host ""

# Detect architecture
$arch = $env:PROCESSOR_ARCHITECTURE
if ($arch -eq "AMD64") { $target = "x86_64-pc-windows-msvc" }
elseif ($arch -eq "ARM64") { $target = "aarch64-pc-windows-msvc" }
else { Write-Err "Unsupported architecture: $arch" }

# Create directories
if (!(Test-Path $INSTALL_DIR)) { 
    New-Item -ItemType Directory -Path $INSTALL_DIR -Force | Out-Null 
}
if (!(Test-Path $CONFIG_DIR)) { 
    New-Item -ItemType Directory -Path $CONFIG_DIR -Force | Out-Null 
}

# Download latest release
$apiUrl = "https://api.github.com/repos/$REPO/releases/latest"
Write-Info "Fetching latest release..."

try {
    $response = Invoke-RestMethod -Uri $apiUrl -UseBasicParsing
    $downloadUrl = $response.assets | Where-Object { $_.name -match "pawlos.*windows.*\.exe" } | Select-Object -First 1
    
    if (!$downloadUrl) {
        # Try to find any exe
        $downloadUrl = $response.assets | Where-Object { $_.name -match "\.exe$" } | Select-Object -First 1
    }
    
    if ($downloadUrl) {
        Write-Info "Downloading $($downloadUrl.name)..."
        $exePath = Join-Path $INSTALL_DIR "$BIN_NAME.exe"
        Invoke-WebRequest -Uri $downloadUrl.browser_download_url -OutFile $exePath -UseBasicParsing
        Write-Ok "Downloaded to $exePath"
    } else {
        Write-Info "No pre-built binary found. Building from source..."
        # Note: Requires Rust/Cargo
        if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
            Write-Err "Rust not installed. Install from https://rustup.rs"
        }
        $tempDir = Join-Path $env:TEMP "pawlos-build"
        git clone --depth 1 "https://github.com/$REPO.git" $tempDir 2>$null
        Push-Location $tempDir
        cargo build --release -p cli --quiet
        Copy-Item "$tempDir/target/release/pawlos.exe" $exePath
        Pop-Location
        Remove-Item $tempDir -Recurse -Force
        Write-Ok "Built from source"
    }
} catch {
    Write-Err "Failed to download: $_"
}

# Add to PATH
$pathEntry = $INSTALL_DIR
$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($currentPath -notlike "*$pathEntry*") {
    [Environment]::SetEnvironmentVariable("Path", "$currentPath;$pathEntry", "User")
    Write-Ok "Added to PATH"
} else {
    Write-Info "Already in PATH"
}

# Create config directory structure
New-Item -ItemType Directory -Path "$CONFIG_DIR\memories" -Force | Out-Null
New-Item -ItemType Directory -Path "$CONFIG_DIR\agents" -Force | Out-Null
New-Item -ItemType Directory -Path "$CONFIG_DIR\skills" -Force | Out-Null
New-Item -ItemType Directory -Path "$CONFIG_DIR\logs" -Force | Out-Null
New-Item -ItemType Directory -Path "$CONFIG_DIR\vector_db" -Force | Out-Null
New-Item -ItemType Directory -Path "$CONFIG_DIR\mcp_servers" -Force | Out-Null

Write-Host ""
Write-Host "  P A W L - O S   u r   a g e n t   |   b u d d y" -ForegroundColor Cyan
Write-Host ""
Write-Ok "pawlos installed!"
Write-Info "Run: pawlos"
Write-Info "Or:  pawlos onboard (first time setup)"
Write-Host ""