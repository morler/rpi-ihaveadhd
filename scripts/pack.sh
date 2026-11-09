#!/usr/bin/env bash
# Stage one flat archive: dist/<crate>-<version>-<target>.tar.gz holding only the cdylib.
set -euo pipefail

target="${1:?usage: pack.sh <target-triple> <version>}"
version="${2:?usage: pack.sh <target-triple> <version>}"
dist="dist"
crate="rpi-adhd"

lib="$(ls "target/$target/release/"*.so "target/$target/release/"*.dylib "target/$target/release/"*.dll 2>/dev/null | head -1 || true)"
if [ -z "$lib" ]; then
  echo "no cdylib under target/$target/release" >&2
  exit 1
fi

mkdir -p "$dist"
base="$crate-$version-$target"
rm -f "$dist/$base.tar.gz"
# tar.gz on every platform: Windows runners ship bsdtar but not zip.
tar -czf "$dist/$base.tar.gz" -C "$(dirname "$lib")" "$(basename "$lib")"
echo "$dist/$base.tar.gz"
