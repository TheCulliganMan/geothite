#!/bin/sh
# Package a native executable, not game content. Output must stay ignored.
set -eu
[ "$#" = 3 ] || { echo 'Usage: terminal-package.sh BINARY TARGET OUTPUT_DIRECTORY' >&2; exit 1; }
mkdir -p "$3"
gzip -9 -c "$1" > "$3/$2.gz"
if command -v sha256sum >/dev/null; then checksum=$(sha256sum "$3/$2.gz" | cut -d ' ' -f 1)
else checksum=$(shasum -a 256 "$3/$2.gz" | cut -d ' ' -f 1); fi
printf '%s %s\n' "$2" "$checksum"
