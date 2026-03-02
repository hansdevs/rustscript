#!/bin/sh
# ┌─────────────────────────────────────────┐
# │  RustScript Installer                   │
# │  https://github.com/hansdevs/rustscript │
# └─────────────────────────────────────────┘
#
# Just run:
#   curl -fsSL https://raw.githubusercontent.com/hansdevs/rustscript/main/install.sh | sh
#
# Environment overrides (optional):
#   RUSTSCRIPT_VERSION=v0.1.1  — pin a specific release
#   INSTALL_DIR=~/my/bin       — force a custom install path

set -e

REPO="hansdevs/rustscript"

# ── Friendly names ────────────────────────────────────────────────────────────

friendly_platform() {
    OS_RAW="$(uname -s)"
    ARCH_RAW="$(uname -m)"

    case "$OS_RAW" in
        Darwin)              OS_TAG="darwin";  OS_NICE="macOS" ;;
        Linux)               OS_TAG="linux";   OS_NICE="Linux" ;;
        MINGW*|MSYS*|CYGWIN*) OS_TAG="windows"; OS_NICE="Windows" ;;
        *) echo "  ✗ Sorry, $OS_RAW isn't supported yet."; exit 1 ;;
    esac

    case "$ARCH_RAW" in
        x86_64|amd64)  ARCH_TAG="x86_64";  ARCH_NICE="x86-64 (Intel/AMD)" ;;
        arm64|aarch64) ARCH_TAG="aarch64";  ARCH_NICE="ARM64 (Apple Silicon / Ampere)" ;;
        *) echo "  ✗ Sorry, $ARCH_RAW isn't supported yet."; exit 1 ;;
    esac

    if [ "$OS_TAG" = "windows" ]; then
        BINARY_NAME="rustscript-${OS_TAG}-${ARCH_TAG}.exe"
    else
        BINARY_NAME="rustscript-${OS_TAG}-${ARCH_TAG}"
    fi
}

# ── Pick the best install directory automatically ─────────────────────────────

pick_install_dir() {
    # Honour an explicit override first
    if [ -n "$INSTALL_DIR" ]; then return; fi

    # Walk the user's PATH and grab the first *user-writable* directory that
    # already exists — this is where the system already expects binaries to live.
    IFS=':'
    for dir in $PATH; do
        case "$dir" in
            "$HOME"/.local/bin|"$HOME"/.cargo/bin|"$HOME"/bin|"$HOME"/.bin)
                if [ -d "$dir" ] && [ -w "$dir" ]; then
                    INSTALL_DIR="$dir"
                    return
                fi
                ;;
        esac
    done
    unset IFS

    # Nothing matched — default to ~/.local/bin (the XDG standard user bin).
    INSTALL_DIR="$HOME/.local/bin"
}

# ── Check PATH and ask before downloading ─────────────────────────────────────

check_path() {
    # Already reachable? Nothing to do.
    case ":$PATH:" in
        *":$INSTALL_DIR:"*) NEEDS_PATH=0; return ;;
    esac

    NEEDS_PATH=1

    # Figure out which shell config to edit
    SHELL_NAME="$(basename "${SHELL:-/bin/sh}")"
    case "$SHELL_NAME" in
        zsh)  RC_FILE="$HOME/.zshrc" ;;
        bash)
            if [ -f "$HOME/.bash_profile" ]; then
                RC_FILE="$HOME/.bash_profile"
            else
                RC_FILE="$HOME/.bashrc"
            fi
            ;;
        fish) RC_FILE="$HOME/.config/fish/config.fish" ;;
        *)    RC_FILE="" ;;
    esac

    if [ -n "$RC_FILE" ] && [ -t 0 ]; then
        echo "  ⚠  $INSTALL_DIR isn't on your PATH yet."
        printf "     Add it to %s automatically? [Y/n] " "$RC_FILE"
        read -r yn
        case "$yn" in
            [Nn]*)
                ADD_TO_PATH=0
                ;;
            *)
                ADD_TO_PATH=1
                ;;
        esac
    else
        ADD_TO_PATH=0
    fi
}

# ── Apply the PATH fix (called after install succeeds) ────────────────────────

apply_path_fix() {
    if [ "$NEEDS_PATH" = "0" ]; then return; fi

    if [ "$ADD_TO_PATH" = "1" ]; then
        if [ "$SHELL_NAME" = "fish" ]; then
            LINE="fish_add_path $INSTALL_DIR"
        else
            LINE="export PATH=\"$INSTALL_DIR:\$PATH\""
        fi
        echo "" >> "$RC_FILE"
        echo "# Added by RustScript installer" >> "$RC_FILE"
        echo "$LINE" >> "$RC_FILE"
        echo "  ✓ PATH updated in $RC_FILE — restart your terminal or run:"
        echo "    source $RC_FILE"
    else
        echo ""
        echo "  When you're ready, add this to your shell config:"
        echo "    export PATH=\"$INSTALL_DIR:\$PATH\""
    fi
}

# ── Resolve which version to install ──────────────────────────────────────────

resolve_version() {
    if [ -n "$RUSTSCRIPT_VERSION" ]; then
        VERSION="$RUSTSCRIPT_VERSION"
        return
    fi

    printf "  Checking latest release... "
    VERSION="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"//;s/".*//')"

    if [ -z "$VERSION" ]; then
        echo "failed."
        echo ""
        echo "  Couldn't reach GitHub. You can install a specific version:"
        echo "    RUSTSCRIPT_VERSION=v0.1.1 sh install.sh"
        exit 1
    fi
    echo "$VERSION"
}

# ── Download & install ────────────────────────────────────────────────────────

install() {
    URL="https://github.com/${REPO}/releases/download/${VERSION}/${BINARY_NAME}"

    echo ""
    echo "  ┌──────────────────────────────────────┐"
    echo "  │  Installing RustScript               │"
    echo "  └──────────────────────────────────────┘"
    echo ""
    echo "    Version      $VERSION"
    echo "    Platform     $OS_NICE · $ARCH_NICE"
    echo "    Destination  $INSTALL_DIR/rustscript"
    echo ""

    # ── Ask about PATH before touching anything ──
    check_path

    # ── Now download ──
    echo ""
    TMP="$(mktemp -d)"
    trap 'rm -rf "$TMP"' EXIT

    printf "  Downloading... "
    if ! curl -fSL -o "${TMP}/rustscript" "$URL" 2>/dev/null; then
        echo "failed."
        echo ""
        echo "  Could not download the binary."
        echo "  Make sure release $VERSION has a build for $OS_NICE $ARCH_NICE."
        echo "  Releases → https://github.com/${REPO}/releases"
        exit 1
    fi
    echo "done."

    chmod +x "${TMP}/rustscript"

    # ── Verify the download is a real binary, not an error page ──
    FILE_SIZE=$(wc -c < "${TMP}/rustscript" | tr -d ' ')
    if [ "$FILE_SIZE" -lt 50000 ]; then
        echo "  ✗ Download looks wrong (only ${FILE_SIZE} bytes)."
        echo "    Expected a binary, got something else."
        echo "    Releases → https://github.com/${REPO}/releases"
        rm -rf "$TMP"
        exit 1
    fi

    # Smoke-test: make sure it actually runs
    if ! "${TMP}/rustscript" --version >/dev/null 2>&1; then
        echo "  ✗ Binary downloaded but won't run on this system."
        echo "    This can happen if you're on an unsupported platform."
        echo "    Releases → https://github.com/${REPO}/releases"
        rm -rf "$TMP"
        exit 1
    fi

    INSTALLED_VERSION="$("${TMP}/rustscript" --version 2>&1)"
    printf "  ✓ Verified: %s\n" "$INSTALLED_VERSION"

    # Create the directory if needed, with sudo only when necessary
    mkdir -p "$INSTALL_DIR" 2>/dev/null || {
        echo "  Need sudo to create $INSTALL_DIR"
        sudo mkdir -p "$INSTALL_DIR"
    }

    if [ -w "$INSTALL_DIR" ]; then
        mv "${TMP}/rustscript" "${INSTALL_DIR}/rustscript"
    else
        echo "  Need sudo to write to $INSTALL_DIR"
        sudo mv "${TMP}/rustscript" "${INSTALL_DIR}/rustscript"
    fi

    echo "  ✓ Installed to $INSTALL_DIR/rustscript"

    # Apply the PATH fix we already got confirmation for
    apply_path_fix

    # Final check: can we actually find it on PATH now?
    echo ""

    # Create a starter project so "try it out" actually works
    STARTER_DIR="$HOME/rustscript-hello"
    STARTER_FILE="$STARTER_DIR/hello.rsx"
    if [ ! -f "$STARTER_FILE" ]; then
        mkdir -p "$STARTER_DIR"
        cat > "$STARTER_FILE" << 'RSX'
page {
    style {
        bg: "linear-gradient(135deg, #667eea 0%, #764ba2 100%)"
        fg: "#ffffff"
        font: "system-ui, sans-serif"
        pad: "0"
    }

    div {
        style {
            display: "flex"
            justify-content: "center"
            align-items: "center"
            min-height: "100vh"
        }

        div {
            style {
                bg: "rgba(255,255,255,0.15)"
                backdrop-filter: "blur(10px)"
                border-radius: "16px"
                pad: "3rem"
                text-align: "center"
                max-width: "500px"
            }

            h1 "Hello, RustScript!" {
                style { size: "2.5rem" margin-bottom: "0.5rem" }
            }
            p "You're up and running. Edit this file and try:" {
                style { size: "1.2rem" opacity: "0.9" }
            }
            p "rustscript serve hello.rsx" {
                style {
                    size: "1rem"
                    font: "'Menlo', monospace"
                    bg: "rgba(0,0,0,0.3)"
                    pad: "8px 16px"
                    border-radius: "8px"
                    margin-top: "1rem"
                    display: "inline-block"
                }
            }
        }
    }
}
RSX
        CREATED_STARTER=1
    else
        CREATED_STARTER=0
    fi

    if command -v rustscript >/dev/null 2>&1; then
        LIVE_VERSION="$(rustscript --version 2>&1)"
        echo "  ┌──────────────────────────────────────┐"
        echo "  │  You're all set!                     │"
        echo "  └──────────────────────────────────────┘"
        echo ""
        echo "  $LIVE_VERSION is ready to use."
        if [ "$CREATED_STARTER" = "1" ]; then
            echo ""
            echo "  A starter project was created at:"
            echo "    ~/rustscript-hello/hello.rsx"
            echo ""
            echo "  Try it right now:"
            echo "    cd ~/rustscript-hello"
            echo "    rustscript preview hello.rsx"
        fi
        echo ""
        echo "  Commands:"
        echo "    rustscript preview <file>     Open in browser"
        echo "    rustscript serve <file>       Dev server + live reload"
        echo "    rustscript build <file>       Compile to HTML"
        echo "    rustscript help               Full usage"
        echo ""
        echo "  Docs → https://github.com/${REPO}#readme"
    elif [ "$NEEDS_PATH" = "1" ] && [ "$ADD_TO_PATH" = "1" ]; then
        echo "  ┌──────────────────────────────────────┐"
        echo "  │  Almost there!                       │"
        echo "  └──────────────────────────────────────┘"
        echo ""
        echo "  Binary is installed and verified, but your terminal needs"
        echo "  to reload the PATH change. Run:"
        echo ""
        echo "    source $RC_FILE"
        if [ "$CREATED_STARTER" = "1" ]; then
            echo ""
            echo "  Then try:"
            echo "    cd ~/rustscript-hello"
            echo "    rustscript preview hello.rsx"
        else
            echo ""
            echo "  Then try:  rustscript help"
        fi
    else
        echo "  Done! Once your PATH includes $INSTALL_DIR, you're good to go."
        if [ "$CREATED_STARTER" = "1" ]; then
            echo "  A starter project is waiting at ~/rustscript-hello/hello.rsx"
        fi
    fi
    echo ""
}

# ── Go ────────────────────────────────────────────────────────────────────────

echo ""
echo "RustScript Installer :3"
echo ""

friendly_platform
pick_install_dir
resolve_version
install
