#!/usr/bin/env bash
set -euo pipefail

REPO="sigilco/intl-ai"
BINARY="intl-ai"

print_error() {
  printf '%s\n' "$*" >&2
}

case "$(uname -s)" in
  Linux)  OS="linux" ;;
  Darwin) OS="darwin" ;;
  *)
    print_error "Unsupported operating system: $(uname -s)"
    print_error "intl-ai binaries are currently available for Linux and macOS."
    print_error "Windows users can run 'npx @intl-ai/cli' or 'bunx @intl-ai/cli' instead."
    exit 1
    ;;
esac

case "$(uname -m)" in
  x86_64)        ARCH="x64" ;;
  arm64|aarch64) ARCH="arm64" ;;
  *)
    print_error "Unsupported architecture: $(uname -m)"
    exit 1
    ;;
esac

TARGET="bun-${OS}-${ARCH}"

VERSION="${INTL_AI_VERSION:-latest}"
INSTALL_DIR="${INTL_AI_INSTALL_DIR:-/usr/local/bin}"

if [ "$VERSION" = "latest" ]; then
  URL="https://github.com/${REPO}/releases/latest/download/${BINARY}-${TARGET}"
else
  URL="https://github.com/${REPO}/releases/download/${VERSION}/${BINARY}-${TARGET}"
fi

print_error "Installing ${BINARY} ${VERSION} for ${TARGET}..."

if [ ! -d "$INSTALL_DIR" ]; then
  mkdir -p "$INSTALL_DIR" || {
    print_error "Failed to create install directory: ${INSTALL_DIR}"
    exit 1
  }
fi

curl -fsSL "$URL" -o "${INSTALL_DIR}/${BINARY}"
chmod +x "${INSTALL_DIR}/${BINARY}"

printf '%s\n' "Installed ${BINARY} to ${INSTALL_DIR}"
