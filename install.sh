#!/bin/sh
# tezt installer for Linux and macOS. Downloads the prebuilt binary for your
# platform from the latest GitHub Release and installs it onto your PATH.
#
#   curl -fsSL https://raw.githubusercontent.com/BilagoNet/tezt/main/install.sh | sh
#
# Environment:
#   TEZT_INSTALL_DIR   where to install (default: ~/.local/bin)
#   TEZT_VERSION       a specific tag to install (default: latest)
set -eu

REPO="BilagoNet/tezt"
BIN_DIR="${TEZT_INSTALL_DIR:-$HOME/.local/bin}"

say() { printf 'tezt-install: %s\n' "$1"; }
die() {
    printf 'tezt-install: error: %s\n' "$1" >&2
    exit 1
}
need() { command -v "$1" >/dev/null 2>&1 || die "'$1' is required but was not found."; }

need curl
need tar

os="$(uname -s)"
arch="$(uname -m)"
case "$os" in
    Linux) os_triple="unknown-linux-gnu" ;;
    Darwin) os_triple="apple-darwin" ;;
    *) die "unsupported OS '$os'. On Windows use install.ps1, or 'pip install tezt'." ;;
esac
case "$arch" in
    x86_64 | amd64) cpu="x86_64" ;;
    arm64 | aarch64) cpu="aarch64" ;;
    *) die "unsupported architecture '$arch'." ;;
esac
target="${cpu}-${os_triple}"
archive="tezt-${target}.tar.gz"

tag="${TEZT_VERSION:-latest}"
if [ "$tag" = "latest" ]; then
    tag="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" |
        sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -n1)"
    [ -n "$tag" ] || die "no published release found yet — build from source or 'pip install tezt'."
fi

url="https://github.com/${REPO}/releases/download/${tag}/${archive}"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

say "downloading ${archive} (${tag})"
curl -fSL --proto '=https' --tlsv1.2 "$url" -o "$tmp/$archive" || die "download failed: $url"
tar -xzf "$tmp/$archive" -C "$tmp"

binary="$tmp/tezt-${target}/tezt"
[ -f "$binary" ] || binary="$tmp/tezt" # tolerate a flat archive layout
[ -f "$binary" ] || die "the archive did not contain the tezt binary."

mkdir -p "$BIN_DIR"
cp "$binary" "$BIN_DIR/tezt"
chmod 0755 "$BIN_DIR/tezt"
say "installed to $BIN_DIR/tezt"

case ":$PATH:" in
    *":$BIN_DIR:"*) ;;
    *) say "note: add $BIN_DIR to your PATH to run 'tezt' directly" ;;
esac
"$BIN_DIR/tezt" --version 2>/dev/null || true
