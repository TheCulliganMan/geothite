#!/bin/sh
# Fetch pinned official MaleCNS inputs outside the repository. No builds.
set -eu
FLYGON_SOURCE_DIR="${1:?Usage: sh tools/flygon-data.sh SOURCE_DIR}"
mkdir -p "$FLYGON_SOURCE_DIR"
FLYGON_SOURCE_DIR=$(cd "$FLYGON_SOURCE_DIR" && pwd)
checksum() { shasum -a 256 "$1" | cut -d ' ' -f 1; }
fetch_source() {
  name=$1
  expected=$2
  url=$3
  if [ -f "$FLYGON_SOURCE_DIR/$name" ]; then
    [ "$(checksum "$FLYGON_SOURCE_DIR/$name")" = "$expected" ] || { echo "Existing $name has a different hash; refusing to overwrite" >&2; exit 1; }
    echo "Verified existing $name"
    return
  fi
  curl --fail --location --retry 3 --continue-at - --output "$FLYGON_SOURCE_DIR/$name.part" "$url"
  [ "$(checksum "$FLYGON_SOURCE_DIR/$name.part")" = "$expected" ] || { echo "Downloaded $name failed SHA-256 verification" >&2; exit 1; }
  mv "$FLYGON_SOURCE_DIR/$name.part" "$FLYGON_SOURCE_DIR/$name"
}
fetch_source annotations.feather 2177e246113e4cfbf1e7772ec37c6da1955ff22e8063d0b1f833101f99a9a3b2 https://storage.googleapis.com/flyem-male-cns/v1.0/connectome-data/flat-connectome/body-annotations-male-cns-v1.0-minconf-0.5.feather
fetch_source neurotransmitters.feather 95c9289220663abeb3409f3ad9e5a7f8a53f8093f5139d15502cd08da8879621 https://storage.googleapis.com/flyem-male-cns/v1.0/connectome-data/flat-connectome/body-neurotransmitters-male-cns-v1.0.feather
fetch_source edges.feather e35da783d1c686b2b58b3b87cd6a403ae43bfcfba8bff28e08ef752c1a56afc1 https://storage.googleapis.com/flyem-male-cns/v1.0/connectome-data/flat-connectome/connectome-weights-male-cns-v1.0-minconf-0.5.feather
printf 'Verified source data: %s\n' "$FLYGON_SOURCE_DIR"
