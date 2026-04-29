#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────
#  pawlos — one-line installer
#  Usage: curl -sSL https://raw.githubusercontent.com/relharrati/pawlos-ai-agent/master/scripts/install.sh | bash
# ─────────────────────────────────────────────────────────────
set -euo pipefail

REPO="https://github.com/relharrati/pawlos-ai-agent"
BRANCH="master"
BIN_NAME="pawlos"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
CONFIG_DIR="$HOME/.pawlos"

# Colors (cyan/purple theme)
BOLD='\033[1m'
ACCENT='\033[38;2;0;229;204m'     # cyan
ACCENT_BRIGHT='\033[38;2;0;255;255m'  # bright cyan
INFO='\033[38;2;136;146;176m'     # muted blue
SUCCESS='\033[38;2;0;229;204m'    # cyan
WARN='\033[38;2;255;176;32m'      # amber
ERROR='\033[38;2;230;57;70m'       # red
MUTED='\033[38;2;90;100;128m'    # gray
NC='\033[0m' # No Color

# Random taglines
TAGLINES=(
    "Ready to assist"
    "It compiles! Ship it!"
    "Powered by coffee and ambition"
    "AI at your service"
    "Now with 20% more intelligence"
    "Click clack goes the code"
    "Don't look at the logs"
    "sudo make me a sandwich"
    "rm -rf /problems"
    "404: Boredom not found"
    "I for one welcome our new robot overlords"
    "It's not a bug, it's a feature"
    "Works on my machine"
    "git push --force"
    "sudo rm -rf /tmp"
    "Bracket balanced, ship it!"
    "Your AI bestie"
    "ur buddy"
)

TAGLINE="${TAGLINES[$((RANDOM % ${#TAGLINES[@]}))]}"

print_banner() {
    echo -e "${ACCENT}${BOLD}"
    cat <<'EOF'
  _____   __          ___           ____   _____ 
 |  __ \ /\ \        / / |         / __ \ / ____|
 | |__) /  \ \  /\  / /| |  ______| |  | | (___  
 |  ___/ /\ \ \/  \/ / | | |______| |  | |\___ \ 
 | |  / ____ \  /\  /  | |____    | |__| |____) |
 |_| /_/    \_\/  \/   |______|    \____/|_____/ 
EOF
    echo -e "${NC}${INFO}  ${TAGLINE}${NC}"
    echo ""
}

# UI helpers
ui_info()  { echo -e "${MUTED}·${NC} $*"; }
ui_warn()  { echo -e "${WARN}!${NC} $*"; }
ui_success() { echo -e "${SUCCESS}✓${NC} $*"; }
ui_error() { echo -e "${ERROR}✗${NC} $*" >&2; exit 1; }

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux) echo "linux" ;;
        Darwin) echo "macos" ;;
        *) echo "unknown" ;;
    esac
}

# Ensure curl/wget
ensure_downloader() {
    if command -v curl &>/dev/null; then return 0; fi
    if command -v wget &>/dev/null; then return 0; fi
    
    ui_info "curl/wget not found, attempting to install..."
    
    if command -v apt-get &>/dev/null; then
        apt-get update -qq && apt-get install -y -qq curl wget 2>/dev/null && return 0
    elif command -v yum &>/dev/null; then
        yum install -y curl wget 2>/dev/null && return 0
    elif command -v apk &>/dev/null; then
        apk add curl wget 2>/dev/null && return 0
    fi
    
    ui_error "curl or wget required. Install manually: sudo apt install curl wget"
}

# Download binary or build from source
download_or_build() {
    local target="$1"
    local ext=""
    [[ "$(detect_os)" == "mingw"* ]] && ext=".exe" || [[ "$(uname -s)" == "Windows" ]] && ext=".exe"
    
    local url="${REPO}/releases/download/latest/${BIN_NAME}-${target}${ext}"
    local dest="${INSTALL_DIR}/${BIN_NAME}${ext}"
    
    mkdir -p "$INSTALL_DIR"
    ensure_downloader
    
    ui_info "Trying to download pre-built binary..."
    if curl -sfL "$url" -o "$dest" 2>/dev/null; then
        chmod +x "$dest"
        ui_success "Downloaded binary"
        return 0
    fi
    
    # No release found - build from source
    ui_warn "No pre-built binary found"
    ui_info "Building from source (requires Rust)..."
    
    if ! command -v cargo &>/dev/null; then
        ui_error "Cargo not found. Install Rust: https://rustup.rs"
    fi
    
    local tmp=$(mktemp -d)
    cd "$tmp"
    git clone --depth 1 -b "${BRANCH}" "${REPO}.git" pawlos_src 2>/dev/null || \
    git clone --depth 1 "${REPO}.git" pawlos_src
    
    cd pawlos_src
    cargo build --release -p cli 2>&1 | tail -5
    
    cp "target/release/${BIN_NAME}${ext}" "$dest"
    chmod +x "$dest"
    cd / && rm -rf "$tmp"
    
    ui_success "Built from source"
}

# Ensure PATH
ensure_path() {
    if [[ ":$PATH:" == *":$INSTALL_DIR:"* ]]; then
        return 0
    fi
    
    local shell_rc=""
    [[ -n "${BASH_VERSION:-}" ]] && shell_rc="$HOME/.bashrc"
    [[ -n "${ZSH_VERSION:-}" ]] && shell_rc="$HOME/.zshrc"
    
    if [[ -n "$shell_rc" ]]; then
        if ! grep -q "$INSTALL_DIR" "$shell_rc" 2>/dev/null; then
            echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$shell_rc"
            ui_info "Added to PATH in $shell_rc"
            ui_info "Run: source $shell_rc"
        fi
    else
        ui_info "Add to PATH: export PATH=\"$INSTALL_DIR:\$PATH\""
    fi
}

# Bootstrap config dirs
bootstrap_dirs() {
    mkdir -p "$CONFIG_DIR/memories" "$CONFIG_DIR/agents" "$CONFIG_DIR/skills" \
           "$CONFIG_DIR/logs" "$CONFIG_DIR/vector_db" "$CONFIG_DIR/plugins" "$CONFIG_DIR/mcp_servers"
}

# Install MCP deps
install_mcp_deps() {
    ui_info "Installing MCP dependencies..."
    
    if command -v npm &>/dev/null; then
        npm install -g @modelcontextprotocol/server-filesystem \
            @modelcontextprotocol/server-fetch \
            @modelcontextprotocol/server-github \
            @modelcontextprotocol/server-brave-search 2>/dev/null || true
    fi
    
    # npx versions for MCP
    command -v npx &>/dev/null && {
        npx -y @modelcontextprotocol/server-filesystem --help >/dev/null 2>&1 || true
    } || true
    
    ui_success "MCP dependencies ready"
}

# Main
main() {
    print_banner
    
    ui_info "Detected: $(detect_os)"
    
    # Install
    download_or_build "$(uname -m)"
    
    # PATH
    ensure_path
    
    # Config
    bootstrap_dirs
    
    # MCP
    install_mcp_deps
    
    # Done
    echo ""
    echo -e "${ACCENT}     P A W L - O S   u r   a g e n t   |   b u d d y${NC}"
    echo ""
    ui_success "pawlos installed!"
    echo ""
    ui_info "Run: pawlos"
    ui_info "Or:  pawlos onboard (first-time setup)"
    echo ""
}

main "$@"