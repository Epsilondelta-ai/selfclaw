#!/usr/bin/env bash
#
# SelfClaw Installer
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/Epsilondelta-ai/selfclaw/main/scripts/install.sh | bash
#   curl -fsSL https://raw.githubusercontent.com/Epsilondelta-ai/selfclaw/main/scripts/install.sh | bash -s -- --no-onboard
#
# Options:
#   --no-onboard    Skip the onboarding wizard after installation
#   --version VER   Install a specific version (default: latest)
#
set -euo pipefail

REPO="Epsilondelta-ai/selfclaw"
INSTALL_DIR="${SELFCLAW_INSTALL_DIR:-/usr/local/bin}"
VERSION=""
NO_ONBOARD=false

# ── Parse arguments ──────────────────────────────────────────────────

while [[ $# -gt 0 ]]; do
    case "$1" in
        --no-onboard)
            NO_ONBOARD=true
            shift
            ;;
        --version)
            VERSION="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# ── Helpers ──────────────────────────────────────────────────────────

info()    { echo "  [info]  $*"; }
success() { echo "  [ ok ]  $*"; }
error()   { echo "  [err]  $*" >&2; }
fatal()   { error "$*"; exit 1; }

detect_platform() {
    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux)  os="linux" ;;
        Darwin) os="macos" ;;
        *)      fatal "Unsupported OS: $os. Use WSL2 on Windows." ;;
    esac

    case "$arch" in
        x86_64|amd64)   arch="x86_64" ;;
        arm64|aarch64)   arch="aarch64" ;;
        *)               fatal "Unsupported architecture: $arch" ;;
    esac

    echo "${os}-${arch}"
}

check_command() {
    command -v "$1" &>/dev/null
}

# ── Main ─────────────────────────────────────────────────────────────

main() {
    echo ""
    echo "  ┌──────────────────────────────────────────┐"
    echo "  │         SelfClaw Installer                │"
    echo "  │                                           │"
    echo "  │  A fully autonomous AI agent that         │"
    echo "  │  discovers its own reason for existence.  │"
    echo "  └──────────────────────────────────────────┘"
    echo ""

    # Detect platform.
    local platform
    platform="$(detect_platform)"
    info "Platform: $platform"

    # Check for required tools.
    for cmd in curl tar; do
        if ! check_command "$cmd"; then
            fatal "Required command not found: $cmd"
        fi
    done

    # Determine version.
    if [[ -z "$VERSION" ]]; then
        info "Fetching latest release..."
        VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
            | grep '"tag_name"' \
            | head -1 \
            | sed -E 's/.*"tag_name":\s*"([^"]+)".*/\1/')

        if [[ -z "$VERSION" ]]; then
            # No release yet — fall back to building from source.
            info "No pre-built release found. Building from source..."
            install_from_source
            return
        fi
    fi
    info "Version: $VERSION"

    # Download binary.
    local archive_name="selfclaw-${VERSION}-${platform}.tar.gz"
    local download_url="https://github.com/${REPO}/releases/download/${VERSION}/${archive_name}"

    info "Downloading ${archive_name}..."
    local tmp_dir
    tmp_dir="$(mktemp -d)"
    trap 'rm -rf "$tmp_dir"' EXIT

    if ! curl -fsSL "$download_url" -o "${tmp_dir}/${archive_name}" 2>/dev/null; then
        info "Pre-built binary not available for $platform."
        info "Building from source instead..."
        install_from_source
        return
    fi

    # Extract and install.
    info "Extracting..."
    tar -xzf "${tmp_dir}/${archive_name}" -C "$tmp_dir"

    info "Installing to ${INSTALL_DIR}..."
    if [[ -w "$INSTALL_DIR" ]]; then
        cp "${tmp_dir}/selfclaw" "${INSTALL_DIR}/selfclaw"
    else
        info "Requires sudo for ${INSTALL_DIR}"
        sudo cp "${tmp_dir}/selfclaw" "${INSTALL_DIR}/selfclaw"
    fi
    chmod +x "${INSTALL_DIR}/selfclaw"

    success "Installed selfclaw to ${INSTALL_DIR}/selfclaw"

    post_install
}

install_from_source() {
    # Check for Rust toolchain.
    if ! check_command cargo; then
        info "Rust not found. Installing via rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        export PATH="$HOME/.cargo/bin:$PATH"
    fi

    info "Cloning repository..."
    local tmp_dir
    tmp_dir="$(mktemp -d)"
    trap 'rm -rf "$tmp_dir"' EXIT

    git clone "https://github.com/${REPO}.git" "${tmp_dir}/selfclaw"
    cd "${tmp_dir}/selfclaw"

    info "Building (release mode)... This may take a few minutes."
    cargo build --release

    info "Installing to ${INSTALL_DIR}..."
    if [[ -w "$INSTALL_DIR" ]]; then
        cp "target/release/selfclaw" "${INSTALL_DIR}/selfclaw"
    else
        info "Requires sudo for ${INSTALL_DIR}"
        sudo cp "target/release/selfclaw" "${INSTALL_DIR}/selfclaw"
    fi
    chmod +x "${INSTALL_DIR}/selfclaw"

    success "Built and installed selfclaw to ${INSTALL_DIR}/selfclaw"

    post_install
}

post_install() {
    # Verify installation.
    echo ""
    if check_command selfclaw; then
        success "selfclaw is in PATH"
        selfclaw --version 2>/dev/null || true
    else
        echo ""
        echo "  selfclaw was installed to ${INSTALL_DIR}/selfclaw"
        echo "  but it's not in your PATH."
        echo ""
        echo "  Add this to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
        echo "    export PATH=\"${INSTALL_DIR}:\$PATH\""
        echo ""
    fi

    # Initialize.
    echo ""
    info "Initializing SelfClaw..."
    "${INSTALL_DIR}/selfclaw" init 2>/dev/null || selfclaw init

    # Onboarding.
    if [[ "$NO_ONBOARD" == "false" ]]; then
        echo ""
        info "Starting onboarding wizard..."
        "${INSTALL_DIR}/selfclaw" onboard 2>/dev/null || selfclaw onboard
    else
        echo ""
        success "Installation complete!"
        echo ""
        echo "  Next steps:"
        echo "    selfclaw onboard    # Setup wizard"
        echo "    selfclaw run        # Start the agent"
        echo "    selfclaw doctor     # Check health"
        echo ""
    fi
}

main "$@"
