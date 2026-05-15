#!/usr/bin/env sh
set -eu

repo="${AGENTHUB_REPO:-AubakirovArman/agenthub}"
version="${AGENTHUB_VERSION:-latest}"
install_dir="${AGENTHUB_INSTALL_DIR:-$HOME/.agenthub/bin}"
artifact="${AGENTHUB_ARTIFACT:-}"

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "agenthub installer: missing required command: $1" >&2
    exit 1
  fi
}

detect_asset() {
  os="$(uname -s)"
  arch="$(uname -m)"
  case "$os" in
    Linux) os_part="unknown-linux-gnu" ;;
    Darwin) os_part="apple-darwin" ;;
    *) echo "agenthub installer: unsupported OS: $os" >&2; exit 1 ;;
  esac
  case "$arch" in
    x86_64|amd64) arch_part="x86_64" ;;
    arm64|aarch64) arch_part="aarch64" ;;
    *) echo "agenthub installer: unsupported architecture: $arch" >&2; exit 1 ;;
  esac
  echo "agenthub-$arch_part-$os_part.tar.gz"
}

download() {
  url="$1"
  output="$2"
  if command -v curl >/dev/null 2>&1; then
    curl -fL "$url" -o "$output"
  elif command -v wget >/dev/null 2>&1; then
    wget -O "$output" "$url"
  else
    echo "agenthub installer: install curl or wget" >&2
    exit 1
  fi
}

need_cmd tar
need_cmd mktemp

asset="$(detect_asset)"
tmp="$(mktemp -d "${TMPDIR:-/tmp}/agenthub-install.XXXXXX")"
trap 'rm -rf "$tmp"' EXIT INT TERM

if [ -n "$artifact" ]; then
  archive="$artifact"
else
  archive="$tmp/$asset"
  if [ "$version" = "latest" ]; then
    url="https://github.com/$repo/releases/latest/download/$asset"
  else
    url="https://github.com/$repo/releases/download/$version/$asset"
  fi
  download "$url" "$archive"
fi

tar -xzf "$archive" -C "$tmp"
binary="$(find "$tmp" -type f -name agenthub | head -n 1)"
if [ -z "$binary" ]; then
  echo "agenthub installer: archive does not contain agenthub binary" >&2
  exit 1
fi

mkdir -p "$install_dir"
cp "$binary" "$install_dir/agenthub"
chmod +x "$install_dir/agenthub"

echo "agenthub installed to $install_dir/agenthub"
echo "Add this directory to PATH if needed:"
echo "  export PATH=\"$install_dir:\$PATH\""
