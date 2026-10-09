#!/bin/sh
# Prebuilt, user-local install. No sudo, Rust, npm, or source checkout required.
set -eu
umask 077
base=${GEOTHITE_DOWNLOAD_URL:-https://geothite.ryanculligan.com}
home_dir=${GEOTHITE_HOME:-${XDG_DATA_HOME:-$HOME/.local/share}/geothite}
case "$home_dir" in /*) ;; *) echo 'GEOTHITE_HOME must be an absolute path' >&2; exit 1 ;; esac
play=yes
case ${1:-} in '' ) ;; --no-play) play=no ;; *) echo 'Usage: sh install.sh [--no-play]' >&2; exit 1 ;; esac
case "$(uname -s):$(uname -m)" in
  Darwin:arm64) platform=aarch64-apple-darwin ;;
  Darwin:x86_64) platform=x86_64-apple-darwin ;;
  Linux:x86_64) platform=x86_64-unknown-linux-gnu ;;
  Linux:aarch64|Linux:arm64) platform=aarch64-unknown-linux-gnu ;;
  *) echo 'Unsupported platform. Use https://geothite.ryanculligan.com/tui/ or Linux via WSL.' >&2; exit 1 ;;
esac
for tool in curl gzip mktemp awk cut readlink tty; do command -v "$tool" >/dev/null || { echo "Required: $tool" >&2; exit 1; }; done
command -v sha256sum >/dev/null || command -v shasum >/dev/null || { echo 'Required: sha256sum or shasum' >&2; exit 1; }
hash() {
  if command -v sha256sum >/dev/null; then sha256sum "$1" | cut -d ' ' -f 1
  else shasum -a 256 "$1" | cut -d ' ' -f 1; fi
}
fetch() { curl --fail --location --silent --show-error --retry 2 --connect-timeout 15 "$1" -o "$2"; }
scratch=$(mktemp -d "${TMPDIR:-/tmp}/geothite-install.XXXXXXXX")
cleanup() {
  status=$?
  trap - EXIT
  rm -f "$scratch/manifest" "$scratch/binary.gz" "$scratch/native" "$scratch/pack"
  rmdir "$scratch"
  exit "$status"
}
trap cleanup EXIT
trap 'exit 130' HUP INT TERM
fetch "$base/downloads/terminal/current.txt" "$scratch/manifest"
# Manifest contains only a release slug and SHA256 checksums, never shell code.
release=$(awk '$1 == "release" {print $2}' "$scratch/manifest")
binary_hash=$(awk -v p="$platform" '$1 == p {print $2}' "$scratch/manifest")
pack_hash=$(awk '$1 == "pack" {print $2}' "$scratch/manifest")
case "$release" in ''|*[!a-zA-Z0-9_-]*) echo 'Invalid release manifest' >&2; exit 1 ;; esac
if [ -z "$binary_hash" ]; then echo "No prebuilt release for $platform yet. Use https://geothite.ryanculligan.com/tui/" >&2; exit 1; fi
for checksum in "$binary_hash" "$pack_hash"; do
  case "$checksum" in *[!0-9a-f]*|'') echo 'Invalid checksum' >&2; exit 1 ;; esac
  [ ${#checksum} -eq 64 ] || { echo 'Invalid checksum length' >&2; exit 1; }
done
fetch "$base/downloads/terminal/$release/$platform.gz" "$scratch/binary.gz"
[ "$(hash "$scratch/binary.gz")" = "$binary_hash" ] || { echo 'Executable checksum mismatch' >&2; exit 1; }
gzip -dc "$scratch/binary.gz" > "$scratch/native"
pack_dir="$home_dir/packs/$pack_hash"
if [ ! -f "$pack_dir/game.crystalpack" ] || [ "$(hash "$pack_dir/game.crystalpack")" != "$pack_hash" ]; then
  fetch "$base/realtime-clock.browser.crystalpack" "$scratch/pack"
  [ "$(hash "$scratch/pack")" = "$pack_hash" ] || { echo 'Content checksum mismatch; rerun after publication finishes.' >&2; exit 1; }
  mkdir -p "$pack_dir"
  mv "$scratch/pack" "$pack_dir/game.crystalpack"
fi
release_dir="$home_dir/releases/$release-$platform"
mkdir -p "$release_dir" "$home_dir/bin" "$home_dir/saves"
chmod 755 "$scratch/native"
mv "$scratch/native" "$release_dir/geothite-native"
fetch "$base/downloads/terminal/$release/play.sh" "$release_dir/play.sh.new"
launcher_hash=$(awk '$1 == "launcher" {print $2}' "$scratch/manifest")
[ "$(hash "$release_dir/play.sh.new")" = "$launcher_hash" ] || { rm -f "$release_dir/play.sh.new"; echo 'Launcher checksum mismatch' >&2; exit 1; }
chmod 755 "$release_dir/play.sh.new"
mv "$release_dir/play.sh.new" "$release_dir/play.sh"
printf '%s\n' "$pack_hash" > "$release_dir/pack-id"
ln -s "$release_dir/play.sh" "$home_dir/bin/geothite.new"
mv -f "$home_dir/bin/geothite.new" "$home_dir/bin/geothite"
if [ "$play" = yes ]; then
  # Restore actual terminal input rather than curl's pipe. The Rust client uses
  # the portable tty poller so reopened terminals also work on macOS.
  if terminal=$(tty 2>/dev/null </dev/tty); then exec "$home_dir/bin/geothite" 0<>"$terminal" >"$terminal"
  else echo "No interactive terminal; run \"$home_dir/bin/geothite\" to play." >&2; exit 1; fi
fi
