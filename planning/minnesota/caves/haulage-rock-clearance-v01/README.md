# Upper Haulage stair shoulder

The upper west boulder is set back from the front cliff face, leaving a clear
flat shoulder beside the original straight stair. One 32px metatile moves north
(two 16px walking tiles). The complete 18×14 room, native night palette, rock
inventory, stairs, actors, pressure network and five shaft pairs stay intact.

- [Whole revised room](../../renders/haulage-clearance-source-v1289/cf3efc669de1-1793496947957740000/whole-2x.png)
- [Native before/after at the same camera](../../renders/haulage-clearance-native-v1299/native-before-after.png)
- [Actually reached upper stair](../../renders/haulage-clearance-native-v1299/upper-stair.png)
- [Actually reached rock shoulder](../../renders/haulage-clearance-native-v1299/boulder-clearance.png)

## Source and physical contract

[`source-v01.json`](source-v01.json), SHA256
`1257acaf832e1986da0849e9cea2721e98bcf90eb0e2006de063557c5a46eaf2`,
binds the complete parent, map, private art and before/after rows. The two edits
swap metatile(4,3) `1d`→`0d` and (4,2) `0d`→`1d`.
[`terrain-v01.json`](terrain-v01.json) supports the existing fast whole-room loop.
Private alias `mn_iron_haulage_rock` assigns **floor** to `0d` for the original
pressure callback; stock cave `0d` is a cliff face. Preserve the private mapping.

Closed ordinary floor remains 170 west/136 east after all three actor bodies;
the opened room remains 310 connected cells. All active/passive shaft records
retain exact LADDER/FLOOR collision. The new rock blocks `(8,5)` and `(9,5)`
from floor y6. The front face permits standing at `(8,8)`/`(9,8)` but blocks
stepping south to y9. The real stair crosses x10,y10↔y7 in both directions.

## Candidate and fast rebuild

Parent: `content-packs/minnesota-reviewed-maps-v01.crystalpack`, SHA256
`83a63e4791f5de5bd72b8b874426b693bf03e3b2067665bfd1a4f7182aba58ba`.
Child: `content-packs/minnesota-haulage-rock-clearance-v01.crystalpack`, SHA256
`110e658e66449fd6ca3dc52fb7de5fa3f0f2bd827d8103aa838a32d63193229c`.
The [combined rebuild](../../revisions/reviewed-maps-v01/README.md) produces
those exact bytes as `minnesota-reviewed-maps-v02.crystalpack` and includes this
final source owner. Both remain unselected; the coastal selector is unchanged.

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_iron_haulage.py \
  --pack content-packs/minnesota-reviewed-maps-v01.crystalpack \
  --source planning/minnesota/caves/haulage-rock-clearance-v01/terrain-v01.json \
  --output planning/minnesota/renders/haulage-clearance-source-NEXT
CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-assets --bin revise_haulage_rock_clearance
target/verdigrim-scene/debug/revise_haulage_rock_clearance \
  content-packs/minnesota-reviewed-maps-v01.crystalpack \
  83a63e4791f5de5bd72b8b874426b693bf03e3b2067665bfd1a4f7182aba58ba \
  planning/minnesota/caves/haulage-rock-clearance-v01/source-v01.json \
  content-packs/minnesota-haulage-rock-clearance-NEXT.crystalpack
```

Use fresh ignored outputs and serialize Cargo. Source composition and physical
checks take 0.04 seconds after loading. Rust owns pack serialization.

## Verification

`renders/haulage-clearance-controller-v1297/` passes actual pressed-input rock/
cliff refusals, both stair directions, new shoulders, Continue; both guard
victories, independent pressure circuits, callback restoration and all five
shaft pairs; and the awarded rod course including zero-badge/no-HM early fishing.
The local native partners, placement, network-course badges and repel are setup
fixtures. No position, party or flags are rewritten after input. The clearance
course leaves both pressure flags unset and reloads a real reached rock save.

`renders/haulage-clearance-mcp-v1300/` independently repeats those physical
steps, Start and fresh Continue through two production processes: 120 calls,
empty stderr. All four played GPU originals in `haulage-clearance-native-v1299/`
were visually inspected. Actual save pose/facing and original night colors
match source: **1,474,560 pixels, zero differences** in
`haulage-clearance-native-comparison-v1301/`. Old/new matched-camera differences
are confined to the moved rock's rectangle `(224,0)–(320,192)`.

The installed audit in `haulage-clearance-installed-v1291/` proves all 742 other
complete maps and every other pack field exact; 258,048 terrain pixels match.
Its `receipt-v02.json` corrects the audio count from the empty embedded payload
field to the 602 unchanged native catalog entries. Independent Rust audio/
desktop verification passes every PCM hash, frame count and loop range
(`.local/haulage-clearance-audio-v1295.log`). Assets and Bevy all-target Clippy
pass (`haulage-clearance-publication-v1290`, `haulage-clearance-runtime-v1294`),
including `location-tester`. Wrong-parent/unlisted-source publication is refused.
Owned formatting/syntax pass; unrelated workspace formatting differences remain.

All originals and earlier passes are retained. This establishes the local
terrain correction, not earned mine access, intended-level balance or complete
regional acceptance. Saved terrain overrides are not migrated. Broader quality,
regional progression and verified final selection remain open.
