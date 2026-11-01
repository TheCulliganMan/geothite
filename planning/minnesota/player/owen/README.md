# Owen sprite assets

Assets only. Engine/player-selection/save integration was removed from the shared checkout at the user's request. No build or deployment is required to inspect these files.

- Front: 56x56; back: 48x48; trainer card: 40x56.
- Walking, bike and surf: six 16x16 frames, stacked down/up/left idle then down/up/left step. Mirror left for right.
- Fishing body: three 16x8 lower-body patches, down/up/left.
- Shared four-color RGB555 palette. PNG, native 2bpp and gbcpal supplied for every asset.
- JSON contains editable, hand-authored palette-index grids.

PNG palette index 0 is white, as in the native source pipeline. Preserve enclosed white shirt/eye highlights when deriving transparency.

The source photo guided short curls, beard, navy zip hoodie and adult build. The photo is not included. Chris and Kris reference pixels are not included or copied.

The earlier ignored Owen crystalpacks and engine test receipts are experimental, not supported playable releases. Do not use them with the restored engine. Sprite generation is complete; player integration is deferred.

Authoritative source: `native-pixels-v2.json`. Version 1 is retained as the rejected palette/bike draft. Delivery: `target/owen/delivery/owen-sprites.zip`.
