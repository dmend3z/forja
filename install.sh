#!/bin/sh
# forja installer — downloads the latest release binary from GitHub
# Usage: curl -fsSL https://raw.githubusercontent.com/dmend3z/forja/main/install.sh | sh
set -e

REPO="dmend3z/forja"
INSTALL_DIR="${FORJA_INSTALL_DIR:-$HOME/.local/bin}"
BINARY_NAME="forja"

main() {
    os=$(uname -s | tr '[:upper:]' '[:lower:]')
    arch=$(uname -m)

    case "$os" in
        darwin) os="apple-darwin" ;;
        linux)  os="unknown-linux-gnu" ;;
        *)
            echo "Error: unsupported OS: $os"
            exit 1
            ;;
    esac

    case "$arch" in
        x86_64|amd64)  arch="x86_64" ;;
        arm64|aarch64) arch="aarch64" ;;
        *)
            echo "Error: unsupported architecture: $arch"
            exit 1
            ;;
    esac

    target="${arch}-${os}"
    tarball="${BINARY_NAME}-${target}.tar.gz"

    echo "  Detecting platform: ${target}"

    # Get latest release tag
    tag=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"//;s/".*//')

    if [ -z "$tag" ]; then
        echo "Error: could not determine latest release"
        exit 1
    fi

    url="https://github.com/${REPO}/releases/download/${tag}/${tarball}"

    echo "  Downloading ${BINARY_NAME} ${tag}..."

    # Create install directory
    mkdir -p "$INSTALL_DIR"

    # Download and extract
    tmpdir=$(mktemp -d)
    trap 'rm -rf "$tmpdir"' EXIT

    if ! curl -fsSL "$url" -o "${tmpdir}/${tarball}"; then
        echo "Error: failed to download ${url}"
        echo "Check https://github.com/${REPO}/releases for available binaries"
        exit 1
    fi

    tar -xzf "${tmpdir}/${tarball}" -C "$tmpdir"
    mv "${tmpdir}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

    echo "  Installed to ${INSTALL_DIR}/${BINARY_NAME}"

    # Check if install dir is in PATH
    case ":$PATH:" in
        *":${INSTALL_DIR}:"*) ;;
        *)
            echo ""
            echo "  Warning: ${INSTALL_DIR} is not in your PATH"
            echo "  Add it with: export PATH=\"${INSTALL_DIR}:\$PATH\""
            echo ""
            ;;
    esac

    # Prompt to run forja init
    printf "  Run forja init now? [Y/n] "
    read -r answer
    case "$answer" in
        [nN]*)
            echo ""
            echo "  Run 'forja init' when you're ready to get started"
            ;;
        *)
            echo ""
            "${INSTALL_DIR}/${BINARY_NAME}" init
            ;;
    esac
}

main
