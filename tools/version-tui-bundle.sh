#!/bin/sh
# Version the painter adapter together with its pinned glyph painter.
set -eu
cd "${1:?usage: version-tui-bundle.sh TUI_DIRECTORY}"
hash=$(cat ascii-frame.js ascii-mount.js | sha256sum | cut -d ' ' -f 1)
mv ascii-mount.js "ascii-mount-$hash.js"
sed "s|./ascii-mount.js|./ascii-mount-$hash.js|g" ascii-frame.js > "ascii-frame-$hash.js"
rm ascii-frame.js
sed "s|./tui/ascii-frame.js|./tui/ascii-frame-$hash.js|g" ../view-navigation.js > ../view-navigation.next.js
mv ../view-navigation.next.js ../view-navigation.js
view_hash=$(cat ../view-navigation.js ../view-navigation.css | sha256sum | cut -d ' ' -f 1)
mv ../view-navigation.js "../view-navigation-$view_hash.js"
mv ../view-navigation.css "../view-navigation-$view_hash.css"
sed "s|./view-navigation.js|./view-navigation-$view_hash.js|g; s|/view-navigation.css|/view-navigation-$view_hash.css|g" ../index.html > ../index.html.next
mv ../index.html.next ../index.html
