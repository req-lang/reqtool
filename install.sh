#!/usr/bin/env sh
set -e

REPO="req-lang/reqtool"
BINARY="reqtool"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Detect OS
case "$(uname -s)" in
  Linux)  OS="linux" ;;
  Darwin) OS="macos" ;;
  *)
    echo "Unsupported OS: $(uname -s)"
    exit 1
    ;;
esac

# Detect architecture
case "$(uname -m)" in
  x86_64 | amd64) ARCH="x86_64" ;;
  aarch64 | arm64) ARCH="aarch64" ;;
  *)
    echo "Unsupported architecture: $(uname -m)"
    exit 1
    ;;
esac

# Map to release artifact name
case "${OS}-${ARCH}" in
  linux-x86_64)   TARGET="x86_64-unknown-linux-musl" ;;
  linux-aarch64)  TARGET="aarch64-unknown-linux-musl" ;;
  macos-aarch64)  TARGET="aarch64-apple-darwin" ;;
  macos-x86_64)
    echo "Intel Macs are not supported. Please build from source."
    exit 1
    ;;
esac

# Resolve latest version if not specified
if [ -z "$VERSION" ]; then
  VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' \
    | sed 's/.*"tag_name": *"\(.*\)".*/\1/')
fi

if [ -z "$VERSION" ]; then
  echo "Failed to resolve latest version. Set VERSION manually: VERSION=v0.7.0 ./install.sh"
  exit 1
fi

URL="https://github.com/${REPO}/releases/download/${VERSION}/${BINARY}-${TARGET}"

echo "Installing ${BINARY} ${VERSION} (${TARGET}) to ${INSTALL_DIR}"

# Download
TMP=$(mktemp)
curl -fsSL "$URL" -o "$TMP"
chmod +x "$TMP"

# Install
if [ -w "$INSTALL_DIR" ]; then
  mv "$TMP" "${INSTALL_DIR}/${BINARY}"
else
  echo "Installing to ${INSTALL_DIR} requires elevated privileges."
  sudo mv "$TMP" "${INSTALL_DIR}/${BINARY}"
fi

echo "Done. Run 'reqtool --help' to get started."
