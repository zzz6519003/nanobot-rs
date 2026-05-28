#!/usr/bin/env bash
# Install system-level build dependencies for nanobot-rs.
# Safe to run on Linux, macOS, and Windows (Git Bash / MSYS2).
# Usage: ./scripts/install-deps.sh
set -euo pipefail

install_protoc_linux() {
  echo "[install-deps] Installing protoc via apt..."
  sudo apt-get update -y
  sudo apt-get install -y protobuf-compiler
}

install_protoc_macos() {
  if command -v protoc &>/dev/null; then
    echo "[install-deps] protoc already installed: $(protoc --version)"
    return
  fi
  echo "[install-deps] Installing protoc via brew..."
  brew install protobuf
}

install_protoc_windows() {
  if command -v protoc &>/dev/null; then
    echo "[install-deps] protoc already installed: $(protoc --version)"
    return
  fi
  echo "[install-deps] Installing protoc via choco..."
  choco install protoc --yes
}

case "$(uname -s)" in
  Linux*)   install_protoc_linux ;;
  Darwin*)  install_protoc_macos ;;
  MINGW*|MSYS*|CYGWIN*) install_protoc_windows ;;
  *)
    echo "[install-deps] Unknown OS: $(uname -s). Please install protoc manually." >&2
    exit 1
    ;;
esac

echo "[install-deps] protoc: $(protoc --version)"
echo "[install-deps] Done."
