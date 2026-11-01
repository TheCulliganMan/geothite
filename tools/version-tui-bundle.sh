#!/bin/sh
# Version both standalone and in-place TUI surfaces with their Rust/glyph assets.
set -eu
cd "${1:?usage: version-tui-bundle.sh TUI_DIRECTORY}"
if test -s geothite.js && test -s geothite_bg.wasm; then
    test -s geothite.js
    test -s geothite_bg.wasm
    if command -v sha256sum >/dev/null 2>&1; then
        tui_hash=$(cat geothite.js geothite_bg.wasm browser.js bridge.js session.js browser.css ascii-frame.js ascii-mount.js | sha256sum | cut -d ' ' -f 1)
    else
        tui_hash=$(cat geothite.js geothite_bg.wasm browser.js bridge.js session.js browser.css ascii-frame.js ascii-mount.js | shasum -a 256 | cut -d ' ' -f 1)
    fi
    tui_name="geothite-$tui_hash"
    sed "s/geothite_bg\\.wasm/$tui_name.wasm/g" geothite.js > "$tui_name.js"
    mv geothite_bg.wasm "$tui_name.wasm"
    mv bridge.js "bridge-$tui_hash.js"
    mv session.js "session-$tui_hash.js"
    mv browser.css "browser-$tui_hash.css"
    mv ascii-mount.js "ascii-mount-$tui_hash.js"
    sed "s|./ascii-mount.js|./ascii-mount-$tui_hash.js|g" ascii-frame.js > "ascii-frame-$tui_hash.js"
    rm ascii-frame.js
    sed "s|./geothite.js|./$tui_name.js|g; s|./bridge.js|./bridge-$tui_hash.js|g; s|./session.js|./session-$tui_hash.js|g; s|./ascii-frame.js|./ascii-frame-$tui_hash.js|g" browser.js > "browser-$tui_hash.js"
    sed "s|/tui/browser.js|/tui/browser-$tui_hash.js|g; s|/tui/browser.css|/tui/browser-$tui_hash.css|g" index.html > index.html.next
    mv index.html.next index.html
    gzip -9 -c "$tui_name.wasm" > "$tui_name.wasm.gz"
    gzip -9 -c "$tui_name.js" > "$tui_name.js.gz"
    rm geothite.js browser.js
    shared_hash=$tui_hash
else
    shared_hash=$(cat ascii-frame.js ascii-mount.js | sha256sum | cut -d ' ' -f 1)
    mv ascii-mount.js "ascii-mount-$shared_hash.js"
    sed "s|./ascii-mount.js|./ascii-mount-$shared_hash.js|g" ascii-frame.js > "ascii-frame-$shared_hash.js"
    rm ascii-frame.js
fi
if test -f ../view-navigation.js; then
    sed "s|./tui/ascii-frame.js|./tui/ascii-frame-$shared_hash.js|g" ../view-navigation.js > ../view-navigation.next.js
    mv ../view-navigation.next.js ../view-navigation.js
    view_hash=$(cat ../view-navigation.js ../view-navigation.css | sha256sum | cut -d ' ' -f 1)
    mv ../view-navigation.js "../view-navigation-$view_hash.js"
    mv ../view-navigation.css "../view-navigation-$view_hash.css"
    sed "s|./view-navigation.js|./view-navigation-$view_hash.js|g; s|/view-navigation.css|/view-navigation-$view_hash.css|g" ../index.html > ../index.html.next
    mv ../index.html.next ../index.html
fi
