#!/usr/bin/env bash
# Stage one flat archive for an rpi prebuilt extension release: dist/<name>-<version>-<target>.<ext>
set -euo pipefail

target="${1:?usage: pack.sh <target-triple> <version> [dist-dir]}"
version="${2:?usage: pack.sh <target-triple> <version> [dist-dir]}"
dist="${3:-dist}"
crate="rpi-adhd"

lib="$(ls "target/$target/release/"*.so "target/$target/release/"*.dylib "target/$target/release/"*.dll 2>/dev/null | head -1 || true)"
if [ -z "$lib" ]; then
  echo "no cdylib under target/$target/release" >&2
  exit 1
fi

mkdir -p "$dist"
base="$crate-$version-$target"
case "$lib" in
  *.dll) ext=zip ;;
  *)     ext=tar.gz ;;
esac

rm -f "$dist/$base.$ext"
# -C into the library's directory so the archive is flat, holding only the library.
if [ "$ext" = zip ]; then
  (cd "$(dirname "$lib")" && zip -q -j "$OLDPWD/$dist/$base.zip" "$(basename "$lib")")
else
  tar -czf "$dist/$base.tar.gz" -C "$(dirname "$lib")" "$(basename "$lib")"
fi

echo "$dist/$base.$ext"
