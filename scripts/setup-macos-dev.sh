#!/usr/bin/env bash
set -euo pipefail

project_root="$(cd "$(dirname "$0")/.." && pwd)"
typst_root="$project_root/tools/typst"
version="0.15.1"
architecture="$(uname -m)"

case "$architecture" in
  arm64|aarch64)
    target="aarch64-apple-darwin"
    checksum="48f62ed034aa3a7978309579ac6ca00045e2ef0da73114e8af27cfd8e74dc05a"
    ;;
  x86_64)
    target="x86_64-apple-darwin"
    checksum="7f9fdd9584866245de9a79e0add8f9236fae6f40a8a45e2c4771ccc14db4e0fa"
    ;;
  *)
    echo "Unsupported macOS architecture: $architecture" >&2
    exit 1
    ;;
esac

if [[ -x "$typst_root/typst-$target/typst" ]]; then
  "$typst_root/typst-$target/typst" --version
  exit 0
fi

archive="$(mktemp -t tradedesk-typst).tar.xz"
trap 'rm -f "$archive"' EXIT
curl -fL "https://github.com/typst/typst/releases/download/v$version/typst-$target.tar.xz" -o "$archive"
echo "$checksum  $archive" | shasum -a 256 -c -
mkdir -p "$typst_root"
tar -xJf "$archive" -C "$typst_root"
"$typst_root/typst-$target/typst" --version
