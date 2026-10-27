#!/bin/sh
set -eu
script=$0
while [ -L "$script" ]; do
  link=$(readlink "$script")
  case "$link" in /*) script=$link ;; *) script=$(dirname "$script")/$link ;; esac
done
release_dir=$(CDPATH= cd -- "$(dirname "$script")" && pwd)
home_dir=$(CDPATH= cd -- "$release_dir/../.." && pwd)
pack_hash=$(cat "$release_dir/pack-id")
pack="$home_dir/packs/$pack_hash/game.crystalpack"
save="$home_dir/saves/$pack_hash.crystalsave"
command=play
case ${1:-} in play|mcp|dump) command=$1; shift ;; esac
if [ "${1:-}" = --session ]; then
  session=${2:?--session requires a name}
  case "$session" in *[!A-Za-z0-9_-]*|'') echo 'Session names use ASCII letters, digits, hyphens or underscores' >&2; exit 1 ;; esac
  [ "${#session}" -le 64 ] || { echo 'Session names must be at most 64 characters' >&2; exit 1; }
  save="$home_dir/saves/$pack_hash-session-$session.crystalsave"
  shift 2
fi
if [ -f "$save" ] || [ -f "$save.bak" ]; then
  exec "$release_dir/geothite-native" "$command" "$pack" --save "$save" --load "$save" "$@"
fi
exec "$release_dir/geothite-native" "$command" "$pack" --save "$save" "$@"
