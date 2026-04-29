#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────
#  pawlos — one-line installer
#  Usage: curl -sSL https://raw.githubusercontent.com/relharrati/pawlos-ai-agent/main/scripts/install.sh | bash
#  Or:    npx pawlos-ai
# ─────────────────────────────────────────────────────────────
set -euo pipefail

REPO="https://github.com/relharrati/pawlos-ai-agent"
BRANCH="master"
BIN_NAME="pawlos"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
CONFIG_DIR="$HOME/.pawlos"

# ── helpers ────────────────────────────────────────────────────
info()  { printf '\033[1;36m[pawlos]\033[0m %s\n' "$*"; }
ok()    { printf '\033[1;32m[  ok  ]\033[0m %s\n' "$*"; }
err()   { printf '\033[1;31m[ err  ]\033[0m %s\n' "$*" >&2; exit 1; }

# ── detect OS and arch ────────────────────────────────────────
detect_target() {
    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux)   os="linux" ;;
        Darwin)  os="darwin" ;;
        CYGWIN*|MINGW*|MSYS*) os="windows" ;;
        *) err "Unsupported OS: $os" ;;
    esac

    case "$arch" in
        x86_64|amd64) arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *) err "Unsupported architecture: $arch" ;;
    esac

    echo "${os}-${arch}"
}

# ── download binary or build from source ───────────────────────────────────────────
download_binary() {
    local target="$1"
    local version="${2:-latest}"
    local ext=""
    [[ "$target" == windows* ]] && ext=".exe"

    local url="${REPO}/releases/download/${version}/${BIN_NAME}-${target}${ext}"
    local dest="${INSTALL_DIR}/${BIN_NAME}${ext}"

    mkdir -p "$INSTALL_DIR"

    # Try to download pre-built binary first
    info "Trying to download pawlos for ${target}..."
    if command -v curl &>/dev/null; then
        if curl -sfL "$url" -o "$dest" 2>/dev/null; then
            ok "Downloaded pre-built binary"
            return 0
        fi
    fi

    # If download fails, build from source
    info "No pre-built binary found. Building from source..."
    if ! command -v cargo &>/dev/null; then
        err "cargo not found. Install Rust: https://rustup.rs"
    fi

    local temp_dir=$(mktemp -d)
    cd "$temp_dir"
    git clone --depth 1 --branch "${BRANCH:-master}" "${REPO}.git" pawlos_src 2>/dev/null || \
    git clone --depth 1 "${REPO}.git" pawlos_src
    cd pawlos_src
    cargo build --release -p cli --quiet
    cp "target/release/${BIN_NAME}${ext}" "$dest"
    cd /
    rm -rf "$temp_dir"
    ok "Built from source"
}
        err "Neither curl nor wget found. Please install one."
    fi

    chmod +x "$dest"
    ok "Binary installed to $dest"
}

# ── ensure install dir is on PATH ─────────────────────────────
ensure_path() {
    case ":$PATH:" in
        *":$INSTALL_DIR:"*) ;; # already on PATH
        *)
            local shell_rc=""
            if [[ -n "${BASH_VERSION:-}" ]]; then
                shell_rc="$HOME/.bashrc"
            elif [[ -n "${ZSH_VERSION:-}" ]]; then
                shell_rc="$HOME/.zshrc"
            fi

            if [[ -n "$shell_rc" ]]; then
                echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$shell_rc"
                info "Added $INSTALL_DIR to PATH in $shell_rc"
                info "Run: source $shell_rc"
            else
                info "Add this to your shell profile: export PATH=\"$INSTALL_DIR:\$PATH\""
            fi
            ;;
    esac
}

# ── bootstrap config directory ────────────────────────────────
bootstrap_dirs() {
    mkdir -p \
        "$CONFIG_DIR/memories" \
        "$CONFIG_DIR/agents" \
        "$CONFIG_DIR/skills" \
        "$CONFIG_DIR/logs" \
        "$CONFIG_DIR/vector_db" \
        "$CONFIG_DIR/plugins" \
        "$CONFIG_DIR/mcp_servers"
    ok "Config directory: $CONFIG_DIR"
}

# ── install Python dependencies for MCP servers ────────────────
install_mcp_deps() {
    info "Installing MCP server dependencies..."
    
    # Core MCP packages
    pip install --quiet \
        mcp \
        httpx \
        uvicorn \
        2>/dev/null || true
    
    # Common MCP servers
    pip install --quiet \
        @modelcontextprotocol/server-filesystem \
        @modelcontextprotocol/server-fetch \
        @modelcontextprotocol/server-brave-search \
        @modelcontextprotocol/server-memory \
        @modelcontextprotocol/server-github \
        2>/dev/null || true
    
    # Node packages for MCP (via npx)
    command -v npx &>/dev/null && {
        npx -y @modelcontextprotocol/server-filesystem --help >/dev/null 2>&1 || true
        npx -y @modelcontextprotocol/server-github --help >/dev/null 2>&1 || true
    } || true
    
    ok "MCP dependencies installed"
}

# ── main ──────────────────────────────────────────────────────
main() {
    local target version
    target="$(detect_target)"
    version="${PAWLOS_VERSION:-latest}"

    info "Installing pawlos (target: $target, version: $version)"

    # If building from source (dev mode)
    if [[ "${BUILD_FROM_SOURCE:-0}" == "1" ]]; then
        info "Building from source..."
        if ! command -v cargo &>/dev/null; then
            err "cargo not found. Install Rust: https://rustup.rs"
        fi
        cargo install --path crates/pawlos-cli --locked
        ok "Built and installed via cargo"
    else
        download_binary "$target" "$version"
    fi

    bootstrap_dirs
    ensure_path
    install_mcp_deps

    echo ""
    ok "pawlos installed! Run: pawlos"
    echo ""
}

main "$@"
