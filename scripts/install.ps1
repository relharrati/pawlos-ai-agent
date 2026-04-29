# pawlos Windows Installer
# Run: iwr -useb https://raw.githubusercontent.com/relharrati/pawlos-ai-agent/master/scripts/install.ps1 | iex

[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$ErrorActionPreference = "Stop"

$REPO = "relharrati/pawlos-ai-agent"
$BRANCH = "master"
$BIN_NAME = "pawlos"
$INSTALL_DIR = "$env:LOCALAPPDATA\Programs\pawlos"
$CONFIG_DIR = "$env:USERPROFILE\.pawlos"

# Colors (cyan/purple theme)
$ACCENT = "Cyan"
$INFO = "DarkGray"
$SUCCESS = "Green"
$WARN = "Yellow"
$ERROR = "Red"
$BOLD = "Bold"

# Random taglines
$TAGLINES = @(
    "Ready to assist",
    "It compiles! Ship it!",
    "Powered by coffee and ambition",
    "AI at your service",
    "Now with 20% more intelligence",
    "Click clack goes the code",
    "Don't look at the logs",
    "sudo make me a sandwich",
    "404: Boredom not found",
    "Your AI bestie"
)
$TAGLINE = $TAGLINES[(Get-Random -Maximum $TAGLINES.Count)]

function Write-Info($msg) { Write-Host "[·] $msg" -ForegroundColor $INFO }
function Write-Warn($msg) { Write-Host "[!] $msg" -ForegroundColor $WARN }
function Write-Success($msg) { Write-Host "[✓] $msg" -ForegroundColor $SUCCESS }
function Write-Err($msg) { Write-Host "[✗] $msg" -ForegroundColor $ERROR; exit 1 }

# Banner
Write-Host ""
Write-Host "  _____   __          ___           ____   _____ " -ForegroundColor $ACCENT
Write-Host " |  __ \ /\ \        / / |         / __ \ / ____|" -ForegroundColor $ACCENT
Write-Host " | |__) /  \ \  /\  / /| |  ______| |  | | (___  " -ForegroundColor $ACCENT
Write-Host " |  ___/ /\ \ \/  \/ / | | |______| |  | |\___ \ " -ForegroundColor $ACCENT
Write-Host " | |  / ____ \  /\  /  | |____    | |__| |____) |" -ForegroundColor $ACCENT
Write-Host " |_| /_/    \_\/  \/   |______|    \____/|_____/ " -ForegroundColor $ACCENT
Write-Host ""
Write-Host "  $TAGLINE" -ForegroundColor $INFO
Write-Host ""

$arch = $env:PROCESSOR_ARCHITECTURE
if ($arch -eq "AMD64") { $target = "x86_64-pc-windows-msvc" }
elseif ($arch -eq "ARM64") { $target = "aarch64-pc-windows-msvc" }
else { Write-Err "Unsupported: $arch" }

Write-Info "Architecture: $target"

# Create directories
if (!(Test-Path $INSTALL_DIR)) { 
    New-Item -ItemType Directory -Path $INSTALL_DIR -Force | Out-Null 
}
if (!(Test-Path $CONFIG_DIR)) { 
    New-Item -ItemType Directory -Path $CONFIG_DIR -Force | Out-Null 
}

$exePath = Join-Path $INSTALL_DIR "$BIN_NAME.exe"

# Try download latest release
Write-Info "Checking for pre-built binary..."
$apiUrl = "https://api.github.com/repos/$REPO/releases/latest"

try {
    $response = Invoke-RestMethod -Uri $apiUrl -UseBasicParsing -TimeoutSec 10
    $asset = $response.assets | Where-Object { $_.name -match "pawlos.*windows.*\.exe" } | Select-Object -First 1
    
    if ($asset) {
        Write-Info "Downloading $($asset.name)..."
        Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $exePath -UseBasicParsing
        Write-Success "Downloaded binary"
    } else {
        throw "No pre-built binary"
    }
} catch {
    Write-Warn "No pre-built binary found"
    Write-Info "Building from source..."
    
    if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Err "Rust not installed. Install from: https://rustup.rs"
    }
    
    $tmp = Join-Path $env:TEMP "pawlos-build"
    git clone --depth 1 "https://github.com/$REPO.git" $tmp 2>$null
    Push-Location $tmp
    cargo build --release -p cli 2>&1 | Select-Object -Last 5
    Copy-Item "target\release\pawlos.exe" $exePath -Force
    Pop-Location
    Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue
    Write-Success "Built from source"
}

# Add to PATH (current + persistent)
$pathEntry = $INSTALL_DIR
$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($currentPath -notlike "*$pathEntry*") {
    [Environment]::SetEnvironmentVariable("Path", "$currentPath;$pathEntry", "User")
    $env:Path = "$env:Path;$pathEntry"
    Write-Success "Added to PATH"
} else {
    Write-Info "Already in PATH"
}

# Create config dirs
New-Item -ItemType Directory -Path "$CONFIG_DIR\memories" -Force | Out-Null
New-Item -ItemType Directory -Path "$CONFIG_DIR\agents" -Force | Out-Null
New-Item -ItemType Directory -Path "$CONFIG_DIR\skills" -Force | Out-Null
New-Item -ItemType Directory -Path "$CONFIG_DIR\logs" -Force | Out-Null
New-Item -ItemType Directory -Path "$CONFIG_DIR\vector_db" -Force | Out-Null
New-Item -ItemType Directory -Path "$CONFIG_DIR\mcp_servers" -Force | Out-Null

# Done
Write-Host ""
Write-Host "     P A W L - O S   u r   a g e n t   |   b u d d y" -ForegroundColor Cyan
Write-Host ""
Write-Success "pawlos installed!"
Write-Info "Installed to: $exePath"
Write-Host ""
Write-Info "Run: pawlos"
Write-Info "Or:  pawlos onboard"
Write-Host ""