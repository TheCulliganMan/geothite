# Northwoods production 2D capture preparation

The fastest checked path uses the already built
`target/gym-controller/wasm-dev/examples/render_at_location`. It is a valid
macOS arm64 executable, SHA256
`9e6d7ca2b755d0a65910976ad05fc39fa7f0111ebd588e63c272b32f660ee6e7`.
Its fingerprint enables `bevy-shell`, `default`, `fullscreen-scaling`,
`location-tester` and `voxel-view`. The example requires `location-tester`;
`--view 2d` selects the actual production classic renderer. No new renderer or
pack exporter is needed for these ordinary map/art primitives.

The existing binary actually loaded and listed the completed 498-map city pack.
A headless capture of `MnR11C00R01` local `(39,5)` at noon completed successfully
in **2.37 seconds**. The original 640×576 image was personally viewed. Evidence
is retained under `renders/northwoods-native/capture-preflight-old/`; logs are
`.local/northwoods-capture-existing-binary-catalog.log` and
`.local/northwoods-capture-preflight-old.log`. This is old-pack binary/camera
preflight, not a capture of the proposed larger maps.

Two other arm64 binaries exist: `target/northstar-story/wasm-dev/examples/render_at_location`
(SHA256 `3f98cbcab2e7a0e8c8e90ad55ce1a2ffd6161b12adf3cff7b1d66dac6737cb0f`)
and `target/runtime-fps/wasm-dev/examples/render_at_location`
(SHA256 `4117fb61401d70756135a0848b84a339336e8c0092c2d2f55022021c164f8e61`).
Their fingerprints have the same features. The gym-controller executable is
the newest inspected build and is the one actually exercised here. Loading the
written expanded pack remains the compatibility check for its exact schema.

## Camera, time and player pose

Use an explicit single `--map`, `--x`, `--y`, `--view 2d`, `--hour 12` and
`--screenshot` for every decision frame. Batch `--maps` chooses nearest-walkable
map centers and therefore cannot reproduce these source decision positions.
The CLI has no facing option. Its new-location fixture uses native DOWN facing
and a standing Chris sprite; no walk sequence is requested. The example seeds
one level30 Totodile as a disposable render fixture. That is not earned regional
travel, an HM unlock or a save-state acceptance test.

With the existing `fullscreen-scaling` binary, a **640×576** capture uses three
physical pixels per source texel and reveals terrain outside the classic LCD.
The correct **160×144** classic comparison is the central physical crop
**`(80,72,560,504)`**, 480×432, reduced by exactly 3 with nearest-neighbor. Retain
the untouched whole image as the production capture. Do not resize its entire
640×576 output to 160×144; that changes camera composition. The preflight retains
that rejected conversion explicitly.

A second **320×288** capture had identical classic pixels after its exact 2×
reduction, but the executable exited1 because its screenshot validator requires
at least 640×576. That original and the failure log
`.local/northwoods-capture-preflight-lcd2x.log` are retained. This smaller capture
is rejected as the operational command; no validator was weakened.

Measured against the exact native standing CHRIS sprite, the successful classic
LCD places the **16×16 OAM frame top-left at `(80,72)`**, with zero mismatched
opaque sprite pixels. The historical planning helper places sprites at
`tile_y*16-4`; that would put a centered player at `(80,68)`, four pixels above
this executable's actual player. Current Rust's
`overworld_sprite_position_from_base` also addresses the tile top-left without
that subtraction. Preserve the actual capture and use its measured native
anchor when preparing a comparison; do not move the production player image to
make a planning overlay appear equal. This four-pixel discrepancy is explicit.

## Frozen locations

[Machine-readable anchors](northwoods-native-capture-anchors.json) records exact
layout hashes, global and native chunk-local coordinates, collision, noon and
DOWN facing. Every listed position was checked against the frozen source native
collision and rejected if occupied. They are Floor or TallGrass land cameras;
none requires seeding Surf or hidden event flags.

| Image | Native map | Local core x,y | Global core x,y |
|---|---|---|---|
| Birch grass | MnR11C00R01 | 29,39 | 29,87 |
| Birch ledge | MnR11C00R01 | 33,41 | 33,89 |
| Birch stair | MnR11C00R01 | 37,43 | 37,91 |
| Palisade stair | MnR11C00R01 | 27,1 | 27,49 |
| Stoney tiers | MnR11C00R03 | 25,8 | 25,152 |
| Lester stair | MnR11C00R03 | 41,27 | 41,171 |
| R11 worst corridor | MnR11C00R03 | 48,8 | 48,152 |
| Safari Palisade cliff | MnBoundarySafariC00R01 | 29,5 | 29,53 |
| Safari Palisade grass | MnBoundarySafariC00R01 | 45,7 | 45,55 |
| Safari Northwind grass | MnBoundarySafariC01R00 | 25,33 | 89,33 |
| Safari Northwind stair | MnBoundarySafariC01R00 | 33,19 | 97,19 |
| Closed arrival | MnBoundarySafariC00R02 | 15,9 | 15,105 |
| Closed guards | MnBoundarySafariC00R02 | 12,6 | 12,102 |
| Safari widest clearing | MnBoundarySafariC00R02 | 29,6 | 29,102 |

## Ready commands after publication

The publisher must first set `CRYSTAL_NORTH_SHORE_EXPANSION_CONTROLLER_PACK` to
the exact written combined pack. It is the same actual pack used by the
controller cases, not a guessed destination. Confirm the catalog contains all
eight expanded R11 chunks and six Safari chunks, including the new southern
ones, with clipped dimensions. The capture process itself strictly verifies
and loads the pack before starting its headless Bevy app.

```sh
capture_pack="${CRYSTAL_NORTH_SHORE_EXPANSION_CONTROLLER_PACK:?publisher must select the written combined pack}"
capture_binary="$PWD/target/gym-controller/wasm-dev/examples/render_at_location"
capture_output="$(mktemp -d "$PWD/planning/minnesota/renders/northwoods-native/production-XXXXXX")"

CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_PREVIEW_MUTE=1 "$capture_binary" --pack "$capture_pack" --list-maps \
  > "$capture_output/native-map-catalog.txt"

capture_case() {
  CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
  CRYSTAL_PREVIEW_MUTE=1 CRYSTAL_RENDER_TEST_WINDOW_SIZE=640x576 \
    "$capture_binary" --pack "$capture_pack" --map "$2" --x "$3" --y "$4" \
    --view 2d --hour 12 --screenshot "$capture_output/$1.png" \
    > "$capture_output/$1.log" 2>&1
}
capture_case birch-grass MnR11C00R01 29 39
capture_case birch-short-ledge MnR11C00R01 33 41
capture_case birch-cave-stair MnR11C00R01 37 43
capture_case palisade-cave-stair MnR11C00R01 27 1
capture_case stoney-two-tiers MnR11C00R03 25 8
capture_case lester-cave-stair MnR11C00R03 41 27
capture_case worst-corridor MnR11C00R03 48 8
capture_case palisade-cliff MnBoundarySafariC00R01 29 5
capture_case palisade-grass-portage MnBoundarySafariC00R01 45 7
capture_case northwind-grass-portage MnBoundarySafariC01R00 25 33
capture_case northwind-cave-stair MnBoundarySafariC01R00 33 19
capture_case closed-entry MnBoundarySafariC00R02 15 9
capture_case closed-guards MnBoundarySafariC00R02 12 6
capture_case widest-clearing MnBoundarySafariC00R02 29 6
```

Inspect exit status, decoded image, player/terrain and logs for every capture;
a written file left by a failed run does not become passing evidence. Every
original and conversion must remain retained. The native capture archive
already stores encoded and GPU originals under each output's `.render-archive`.
Use existing `render_archive.save_render` / `render_stages.render_stage` for
subsequent crops and nearest-neighbor comparisons, with the crop above.

If the existing binary rejects the actual new pack or a current renderer fix
is required, Root can perform one warm build with its chosen cache:

```sh
CARGO_TARGET_DIR=target/runtime-fps cargo build --locked --profile wasm-dev \
  -p crystal-bevy --features location-tester --example render_at_location
```

That command is prepared, not executed by this worker. Keep default features
when following the measured640×576 crop recipe. A `--no-default-features`
build omitting `fullscreen-scaling` changes the camera scale; it requires a new
preflight instead of silently reusing these crop coordinates. No Cargo, pack
publication, visible UI or shared renderer/source edits were performed here.

## Executed 505-map production review

The publisher's completed immutable pack was confirmed before it was opened:
`.local/minnesota-city-revisions/northwoods-3949ad8dff81537f9b664f914f5358825da40512fbfdae52c2fffa1a049441b7.crystalpack`,
file SHA256 `3949ad8dff81537f9b664f914f5358825da40512fbfdae52c2fffa1a049441b7`.
All 14 production 2D captures above then passed and were personally inspected.
The [decision sheet](../renders/northwoods-native/3949ad8dff81537f/production-1793064804698539000/production-decision-cameras.png)
shows every tested location, including the widest frames.
[Exact capture report](../renders/northwoods-native/3949ad8dff81537f/production-1793064804698539000/production-capture-report.json)
SHA256 is `8245ea3e6d53247e5d4cf69dfbabdd180ab56820ce9b95d744b9e8c935565c21`.

The shared gym-controller executable changed after the original preflight,
from 9e6d7ca2 to a newly built valid arm64 executable, SHA256
`1e32e21efe0d2b9db1067f9e760e278fd4e8bcb40067089bf6c5316c766279be`.
The launcher stopped on the hash mismatch before taking any production images.
A private ignored copy of the new executable was frozen and preflighted on the
actual 505 pack; all 14 final captures used that exact copy. No build was run by
this worker. The temporary compiled copy was removed after completion; the
original shared executable and all capture evidence remain.

The 14 successful processes totalled 205.138 seconds on the current system,
including repeated strict runtime startup. Each process used actual native
DOWN/noon, a 640×576 image target and the measured classic crop. On the new
executable, the Floor standing-player frame again matches `(80,72)` with zero
opaque-pixel mismatches; the historical four-pixel-up overlay mismatches 124
opaque pixels. No player shift was applied. Grass correctly masks the lower
body; stairs, ledge faces, Stoney's full offset tiers, Safari's cliff portages
and both closed gate officers/fence are visible.

All 14 raw GPU binary readbacks and JSON descriptors were independently checked
for 640×576 extent, byte length, sibling linkage and SHA256 identity. The final
production directory retains 380 PNG originals/archives/crops/conversions,
plus those raw binaries/descriptors, logs, exact command arguments and the
captured input manifest. Earlier rejected preflight images are also retained.
The R11 shore fork at global `(48,152)` remains the broadest ordinary road/lawn
composition; it is shown explicitly and left unchanged. These pictures prove
native presentation at disposable fixture positions. Safari admission remains
closed and production progression acceptance is separately owned by Root.

Before these captures, the exact native 505 pack also passed full frozen-source
comparison: R11 has 5,376 equal blocks and whole native/source pixels, all 128
original Johto blocks/collisions, three derived native recipes and the stable
eight-chunk IDs/20 connection records. Boundary has 3,584 equal blocks/pixels,
22 bodies, 18 signs,14 connection records/seven Water seams,476 closed perimeter
cells, 42 sealed-court cells and the relocated paired cabin, with all six
corruption fixtures rejected. See
[exact comparison summary](../renders/northwoods-native/3949ad8dff81537f/native-verify-1793064684803419000/verification-summary.json).
