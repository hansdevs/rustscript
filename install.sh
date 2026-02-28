#!/bin/sh
# RustScript installer — no Rust toolchain required.
# Usage:  curl -fsSL https://raw.githubusercontent.com/user/rustscript/main/install.sh | sh
#
# Override variables:
#   RUSTSCRIPT_VERSION  — tag to install (default: latest)
#   INSTALL_DIR         — where to put the binary (default: /usr/local/bin)

set -e

REPO="user/rustscript"   # ← Update this to the real GitHub org/repo
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# ── Detect OS & Architecture ──────────────────────────────────────────────────

detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Darwin)  OS_TAG="darwin" ;;
        Linux)   OS_TAG="linux" ;;
        MINGW*|MSYS*|CYGWIN*) OS_TAG="windows" ;;
        *)
            echo "Error: Unsupported OS: $OS"
            exit 1
            ;;
    esac

    case "$ARCH" in
        x86_64|amd64)   ARCH_TAG="x86_64" ;;
        arm64|aarch64)   ARCH_TAG="aarch64" ;;
        *)
            echo "Error: Unsupported architecture: $ARCH"
            exit 1
            ;;
    esac

    if [ "$OS_TAG" = "windows" ]; then
        BINARY_NAME="rustscript-${OS_TAG}-${ARCH_TAG}.exe"
    else
        BINARY_NAME="rustscript-${OS_TAG}-${ARCH_TAG}"
    fi
}

# ── Resolve version ──────────────────────────────────────────────────────────

resolve_version() {
    if [ -n "$RUSTSCRIPT_VERSION" ]; then
        VERSION="$RUSTSCRIPT_VERSION"
    else
        echo "Fetching latest release..."
        VERSION="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
            | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"//;s/".*//')"
        if [ -z "$VERSION" ]; then
            echo "Error: Could not determine latest version."
            echo "Set RUSTSCRIPT_VERSION manually, e.g.:"
            echo "  RUSTSCRIPT_VERSION=v0.1.0 sh install.sh"
            exit 1
        fi
    fi
}

# ── Download & Install ───────────────────────────────────────────────────────

install() {
    URL="https://github.com/${REPO}/releases/download/${VERSION}/${BINARY_NAME}"

    echo ""
    echo "  RustScript Installer"
    echo "  ────────────────────"
    echo "  Version:  $VERSION"
    echo "  Platform: ${OS_TAG}-${ARCH_TAG}"
    echo "  Binary:   $BINARY_NAME"
    echo "  Install:  $INSTALL_DIR/rustscript"
    echo ""

    TMP="$(mktemp -d)"
    trap 'rm -rf "$TMP"' EXIT

    echo "Downloading $URL ..."
    if ! curl -fSL -o "${TMP}/rustscript" "$URL"; then
        echo ""
        echo "Error: Download failed."
        echo "Check that release $VERSION exists and has a binary for ${OS_TAG}-${ARCH_TAG}."
        echo "Releases: https://github.com/${REPO}/releases"
        exit 1
    fi

    chmod +x "${TMP}/rustscript"

    # Try installing to INSTALL_DIR, use sudo if needed
    if [ -w "$INSTALL_DIR" ]; then
        mv "${TMP}/rustscript" "${INSTALL_DIR}/rustscript"
    else
        echo "Need sudo to install to $INSTALL_DIR"
        sudo mv "${TMP}/rustscript" "${INSTALL_DIR}/rustscript"
    fi

    echo ""
    echo "✓ rustscript installed to ${INSTALL_DIR}/rustscript"
    echo ""
    echo "  Try it:  rustscript preview app.rsx"
    echo ""

    # Verify it's on PATH
    if ! command -v rustscript >/dev/null 2>&1; then
        echo "⚠  ${INSTALL_DIR} is not in your PATH."
        echo "   Add it:  export PATH=\"${INSTALL_DIR}:\$PATH\""
        echo ""
    fi
}

# ── Main ─────────────────────────────────────────────────────────────────────

detect_platform
resolve_version
install
