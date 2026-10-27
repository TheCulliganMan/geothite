#!/bin/sh
# Generate ignored, content-free distribution output. Pack is read only to hash.
set -eu
[ "$#" -ge 4 ] || { echo 'Usage: terminal-release.sh RELEASE OUTPUT PACK TARGET=BINARY…' >&2; exit 1; }
release=$1; output=$2; pack=$3; shift 3
case "$release" in ''|*[!a-zA-Z0-9_-]*) exit 1 ;; esac
script_dir=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
hash() {
  if command -v sha256sum >/dev/null; then sha256sum "$1" | cut -d ' ' -f 1
  else shasum -a 256 "$1" | cut -d ' ' -f 1; fi
}
mkdir -p "$output/downloads/terminal/$release"
cp "$script_dir/terminal-play.sh" "$output/downloads/terminal/$release/play.sh"
cp "$script_dir/../web-client/install.sh" "$output/install.sh"
{
  printf 'release %s\npack %s\nlauncher %s\n' "$release" "$(hash "$pack")" "$(hash "$script_dir/terminal-play.sh")"
  for target_binary in "$@"; do
    target=${target_binary%%=*}; binary=${target_binary#*=}
    case "$target" in ''|*[!a-zA-Z0-9_-]*) exit 1 ;; esac
    sh "$script_dir/terminal-package.sh" "$binary" "$target" "$output/downloads/terminal/$release"
  done
} > "$output/downloads/terminal/current.txt"
