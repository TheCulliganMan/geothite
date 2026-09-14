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
sed "s|'./crystal-bevy.js'|'./$bundle_name.js'|g; s|./crystal-bevy_bg.wasm|./$bundle_name.wasm|g" index.html > index.html.versioned
mv index.html.versioned index.html
rm -f crystal-bevy.js crystal-bevy.js.gz crystal-bevy_bg.wasm.gz

# The audio worker and its Rust module must also update as one versioned pair.
audio_hash=$(cat crystal-audio.js crystal-audio_bg.wasm audio-worker.js | sha256sum | cut -d ' ' -f 1)
audio_name="crystal-audio-$audio_hash"
sed "s/crystal-audio_bg\.wasm/$audio_name.wasm/g" crystal-audio.js > "$audio_name.js"
mv crystal-audio_bg.wasm "$audio_name.wasm"
gzip -9 -c "$audio_name.wasm" > "$audio_name.wasm.gz"
gzip -9 -c "$audio_name.js" > "$audio_name.js.gz"
sed "s|'./crystal-audio.js'|'./$audio_name.js'|g; s|./crystal-audio_bg.wasm|./$audio_name.wasm|g" audio-worker.js > audio-worker.versioned.js
mv audio-worker.versioned.js "audio-worker-$audio_hash.js"
sed "s|'./audio-worker.js'|'./audio-worker-$audio_hash.js'|g" index.html > index.html.versioned
mv index.html.versioned index.html
rm -f crystal-audio.js crystal-audio_bg.wasm.gz audio-worker.js

# Optional Flygon: version both worker imports against one neural module pair.
if test -s flygon/crystal_flygon_bg.wasm; then
    flygon_hash=$(cat flygon/crystal_flygon.js flygon/crystal_flygon_bg.wasm | sha256sum | cut -d ' ' -f 1)
    flygon_name="crystal_flygon-$flygon_hash"
    sed "s/crystal_flygon_bg\\.wasm/$flygon_name.wasm/g" flygon/crystal_flygon.js > "flygon/$flygon_name.js"
    mv flygon/crystal_flygon_bg.wasm "flygon/$flygon_name.wasm"
    gzip -9 -c "flygon/$flygon_name.wasm" > "flygon/$flygon_name.wasm.gz"
    for worker in flygon-worker flygon-view-worker; do
        sed "s|flygon/crystal_flygon.js|flygon/$flygon_name.js|g; s|flygon/crystal_flygon_bg.wasm|flygon/$flygon_name.wasm|g" "$worker.js" > "$worker.next"
        mv "$worker.next" "$worker.js"
        worker_hash=$(sha256sum "$worker.js" | cut -d ' ' -f 1)
        mv "$worker.js" "$worker-$worker_hash.js"
        for ui in flygon.js flygon-view.js; do
            sed "s|./$worker.js|./$worker-$worker_hash.js|g" "$ui" > "$ui.next"
            mv "$ui.next" "$ui"
        done
    done
    rm flygon/crystal_flygon.js
fi
