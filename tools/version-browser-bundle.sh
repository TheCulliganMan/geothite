#!/bin/sh
# Publish generated wasm-bindgen glue and WASM as one content-addressed pair.
set -eu
cd "${1:?usage: version-browser-bundle.sh WEB_DIRECTORY}"
if [ "${2:-}" != "--flygon-only" ]; then
test -s crystal-bevy.js
test -s crystal-bevy_bg.wasm
if test -f renderer-bundle.js; then
    grep -Fq "'./crystal-bevy.js'" renderer-bundle.js
else
    grep -Fq "'./crystal-bevy.js'" index.html
fi
grep -Fq "'crystal-bevy_bg.wasm'" crystal-bevy.js
bundle_hash=$(cat crystal-bevy.js crystal-bevy_bg.wasm | sha256sum | cut -d ' ' -f 1)
bundle_name="crystal-bevy-$bundle_hash"
sed "s/crystal-bevy_bg\.wasm/$bundle_name.wasm/g" crystal-bevy.js > "$bundle_name.js"
mv crystal-bevy_bg.wasm "$bundle_name.wasm"
gzip -9 -c "$bundle_name.wasm" > "$bundle_name.wasm.gz"
gzip -9 -c "$bundle_name.js" > "$bundle_name.js.gz"
sed "s|'./crystal-bevy.js'|'./$bundle_name.js'|g; s|./crystal-bevy_bg.wasm|./$bundle_name.wasm|g" index.html > index.html.versioned
mv index.html.versioned index.html
if test -f renderer-bundle.js; then
    # The selector probes this manifest on every startup. Ship its WebGL pair
    # even when this build has no optional WebGPU artifact.
    printf '{"webgl":{"module":"./%s.js","binary":"./%s.wasm"}' "$bundle_name" "$bundle_name" > renderer-bundles.json
    if test -s crystal-bevy-webgpu_bg.wasm; then
        gpu_hash=$(cat crystal-bevy-webgpu.js crystal-bevy-webgpu_bg.wasm | sha256sum | cut -d ' ' -f 1)
        gpu_name="crystal-bevy-webgpu-$gpu_hash"
        sed "s/crystal-bevy-webgpu_bg\\.wasm/$gpu_name.wasm/g" crystal-bevy-webgpu.js > "$gpu_name.js"
        mv crystal-bevy-webgpu_bg.wasm "$gpu_name.wasm"
        gzip -9 -c "$gpu_name.wasm" > "$gpu_name.wasm.gz"
        gzip -9 -c "$gpu_name.js" > "$gpu_name.js.gz"
        printf ',"webgpu":{"module":"./%s.js","binary":"./%s.wasm"}' "$gpu_name" "$gpu_name" >> renderer-bundles.json
        rm -f crystal-bevy-webgpu.js crystal-bevy-webgpu_bg.wasm.gz
    fi
    printf '}\n' >> renderer-bundles.json
    sed "s|./crystal-bevy.js|./$bundle_name.js|g; s|./crystal-bevy_bg.wasm|./$bundle_name.wasm|g" renderer-bundle.js > renderer-bundle.next.js
    selector_hash=$(sha256sum renderer-bundle.next.js | cut -d ' ' -f 1)
    mv renderer-bundle.next.js "renderer-bundle-$selector_hash.js"
    sed "s|./renderer-bundle.js|./renderer-bundle-$selector_hash.js|g" index.html > index.html.versioned
    mv index.html.versioned index.html
    rm renderer-bundle.js
fi
rm -f crystal-bevy.js crystal-bevy.js.gz crystal-bevy_bg.wasm.gz

# Worker protocols and their Rust modules must update together as well.
cpu_workers=0
for role in terrain actor; do
    module="crystal-$role-worker"
    if ! test -s "${module}_bg.wasm"; then continue; fi
    cpu_workers=1
    worker_hash=$(cat "$module.js" "${module}_bg.wasm" "$role-worker.js" | sha256sum | cut -d ' ' -f 1)
    worker_name="$module-$worker_hash"
    sed "s/${module}_bg\\.wasm/$worker_name.wasm/g" "$module.js" > "$worker_name.js"
    mv "${module}_bg.wasm" "$worker_name.wasm"
    gzip -9 -c "$worker_name.wasm" > "$worker_name.wasm.gz"
    sed "s|./$module.js|./$worker_name.js|g; s|./${module}_bg.wasm|./$worker_name.wasm|g" "$role-worker.js" > "$role-worker-$worker_hash.js"
    for client in terrain-worker-client.js actor-worker-bridge.js; do
        sed "s|./$role-worker.js|./$role-worker-$worker_hash.js|g" "$client" > "$client.next"
        mv "$client.next" "$client"
    done
    rm "$module.js" "$role-worker.js"
done
if test "$cpu_workers" = 1; then
    # Dependency order: shared leaves first, then the importing bridges/page.
    for client in cpu-worker-lifecycle cpu-worker-pool terrain-worker-client terrain-worker-bridge actor-worker-bridge; do
        client_hash=$(sha256sum "$client.js" | cut -d ' ' -f 1)
        mv "$client.js" "$client-$client_hash.js"
        for source in index.html *.js; do
            sed "s|./$client.js|./$client-$client_hash.js|g" "$source" > "$source.next"
            mv "$source.next" "$source"
        done
    done
fi

# The audio worker and its Rust module must also update as one versioned pair.
audio_hash=$(cat crystal-audio.js crystal-audio_bg.wasm audio-worker.js | sha256sum | cut -d ' ' -f 1)
audio_name="crystal-audio-$audio_hash"
sed "s/crystal-audio_bg\.wasm/$audio_name.wasm/g" crystal-audio.js > "$audio_name.js"
mv crystal-audio_bg.wasm "$audio_name.wasm"
gzip -9 -c "$audio_name.wasm" > "$audio_name.wasm.gz"
gzip -9 -c "$audio_name.js" > "$audio_name.js.gz"
sed "s|'./crystal-audio.js'|'./$audio_name.js'|g; s|./crystal-audio_bg.wasm|./$audio_name.wasm|g" audio-worker.js > audio-worker.versioned.js
mv audio-worker.versioned.js "audio-worker-$audio_hash.js"
sed "s|'./audio-worker.js'|'./audio-worker-$audio_hash.js'|g; s|./crystal-audio_bg.wasm|./$audio_name.wasm|g" index.html > index.html.versioned
mv index.html.versioned index.html
rm -f crystal-audio.js crystal-audio_bg.wasm.gz audio-worker.js
fi

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
    view_hash=$(sha256sum flygon-view.js | cut -d ' ' -f 1)
    mv flygon-view.js "flygon-view-$view_hash.js"
    sed "s|./flygon-view.js|./flygon-view-$view_hash.js|g" flygon.js > flygon.next.js
    mv flygon.next.js flygon.js
    ui_hash=$(sha256sum flygon.js | cut -d ' ' -f 1)
    mv flygon.js "flygon-$ui_hash.js"
    sed "s|./flygon.js|./flygon-$ui_hash.js|g" flygon.html > flygon.next.html
    mv flygon.next.html flygon.html
    rm flygon/crystal_flygon.js
fi
