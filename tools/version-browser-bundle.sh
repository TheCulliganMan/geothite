#!/bin/sh
# Publish generated wasm-bindgen glue and WASM as one content-addressed pair.
set -eu
cd "${1:?usage: version-browser-bundle.sh WEB_DIRECTORY}"
test -s crystal-bevy.js
test -s crystal-bevy_bg.wasm
grep -Fq "'./crystal-bevy.js'" index.html
grep -Fq "'crystal-bevy_bg.wasm'" crystal-bevy.js
bundle_hash=$(cat crystal-bevy.js crystal-bevy_bg.wasm | sha256sum | cut -d ' ' -f 1)
bundle_name="crystal-bevy-$bundle_hash"
sed "s/crystal-bevy_bg\.wasm/$bundle_name.wasm/g" crystal-bevy.js > "$bundle_name.js"
mv crystal-bevy_bg.wasm "$bundle_name.wasm"
gzip -9 -c "$bundle_name.wasm" > "$bundle_name.wasm.gz"
gzip -9 -c "$bundle_name.js" > "$bundle_name.js.gz"
sed "s|'./crystal-bevy.js'|'./$bundle_name.js'|g" index.html > index.html.versioned
mv index.html.versioned index.html
rm -f crystal-bevy.js crystal-bevy.js.gz crystal-bevy_bg.wasm.gz
