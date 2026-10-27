#!/bin/sh
# sixloom installer: downloads the prebuilt binary for this platform from the
# latest GitHub release, or builds from source with cargo if none matches.
#
#   curl -fsSL https://raw.githubusercontent.com/datamonsterr/sixloom/main/install.sh | sh
#   ... | sh -s -- --version v0.1.0 --dir ~/.local/bin
set -eu

REPO="datamonsterr/sixloom"
VERSION="latest"
DIR="${SIXLOOM_INSTALL_DIR:-$HOME/.local/bin}"

while [ $# -gt 0 ]; do
  case "$1" in
    --version) VERSION="$2"; shift 2 ;;
    --dir) DIR="$2"; shift 2 ;;
    -h|--help) sed -n '2,6p' "$0" 2>/dev/null || true; exit 0 ;;
    *) echo "unknown option: $1" >&2; exit 2 ;;
  esac
done

os=$(uname -s); arch=$(uname -m)
case "$os-$arch" in
  Linux-x86_64) target="x86_64-unknown-linux-musl" ;;
  Linux-aarch64|Linux-arm64) target="aarch64-unknown-linux-musl" ;;
  Darwin-x86_64) target="x86_64-apple-darwin" ;;
  Darwin-arm64) target="aarch64-apple-darwin" ;;
  *) target="" ;;
esac

mkdir -p "$DIR"
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT

if [ -n "$target" ]; then
  if [ "$VERSION" = "latest" ]; then
    url="https://github.com/$REPO/releases/latest/download/sixloom-$target.tar.gz"
  else
    url="https://github.com/$REPO/releases/download/$VERSION/sixloom-$target.tar.gz"
  fi
  if curl -fsSL "$url" -o "$tmp/sixloom.tar.gz" 2>/dev/null; then
    tar xzf "$tmp/sixloom.tar.gz" -C "$tmp"
    install -m 0755 "$tmp/sixloom" "$DIR/sixloom"
    echo "sixloom installed to $DIR/sixloom"
    case ":$PATH:" in *":$DIR:"*) ;; *) echo "add $DIR to your PATH" ;; esac
    exit 0
  fi
  echo "no prebuilt binary for $target ($VERSION); building from source" >&2
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found. Install Rust (https://rustup.rs) and rerun, or download a release manually." >&2
  exit 1
fi
ref=""
[ "$VERSION" != "latest" ] && ref="--tag $VERSION"
# shellcheck disable=SC2086
cargo install --locked --git "https://github.com/$REPO" $ref --root "$tmp/root" sixloom
install -m 0755 "$tmp/root/bin/sixloom" "$DIR/sixloom"
echo "sixloom installed to $DIR/sixloom"
