# Driftless fossil gallery below the starter hollow

The original lit Route1 hollow now leads to a complete optional12×10 lower
limestone gallery. A west fossil ledge, low cleft and separate east sample
alcove give the cave a clear exploration sequence. Both climbs use fitted
native30/36 stairs. The room is dry, quiet and enclosed; there is no new random
walking battle, exterior connection or admission gate.

The existing guide still gives HM FLASH after the Care Badge. The original
Antidote and lit upper loop remain intact. The guide gains directions to the
lower gallery. Real native Flash illuminates the lower room; without it, the
fossil wall cannot be read. Acknowledging all three fossil pages reveals the
sample cache on the eastern shelf. The final page tells the player where to
look. The native Rare Candy reward is once-only; a full item pocket preserves
it for retry after native Toss. Leaving for the outdoors resets Flash normally.

The dialogue takes its local setting from the Minnesota Geological Survey's
[Paleozoic geology](https://cse.umn.edu/mgs/paleozoic-geology): southeastern
Minnesota's limestone records an ancient shallow sea and its fossil life.
The cave's architecture and reward are fictional game design.

The verified local review revision is
`content-packs/minnesota-driftless-fossil-gallery-v03.crystalpack`. Active selection is
recorded in `planning/minnesota/world.json` after complete acceptance.
The parent is the selected Rock Cut shelf-drop pack bound by `contract.json`.
The older combined `field-use-v01` gallery/current proposal is unselected and
has a different gallery floorplan; it is not a build parent. This revision
installs the gallery only. It does not add that proposal's Leech Lake current.

- [Complete connected levels](../../renders/driftless-fossil-gallery-v774/connected-v790/connected-levels-2x.png)
- [Whole gallery, lit source](../../renders/driftless-fossil-gallery-v774/whole-lit-source-v01.png)
- [Actual dark/lit native cameras](../../renders/driftless-fossil-gallery-v774/native-v788/actual-dark-and-lit-cameras.png)
- [Native gameplay acceptance](../../renders/driftless-fossil-gallery-v774/controller-v787/receipt.json)
- [Production stdio replay](../../renders/driftless-fossil-gallery-v774/mcp-v788/receipt.json)
- [Complete local acceptance](../../renders/driftless-fossil-gallery-v774/acceptance-v792.json)
- [Exact full-parent audit](../../renders/driftless-fossil-gallery-v774/audit-v786.json)

## Connections and state

Coordinates are16px native walking tiles. Metatiles are32px.

| Contract | Native coordinates and behavior |
| --- | --- |
| Original world entrance | `MnR01C00R00` CAVE `(18,17)` enters upper hollow warp1 `(9,11)`; original exit still returns to exterior passive Floor `(18,18)` |
| Down stair in upper hollow | Native block02→0b at metatile `(5,4)`; active warp2 `(11,9)` targets lower gallery passive warp2 `(18,17)` |
| Return from lower gallery | Active warp1 `(19,17)` targets upper hollow passive warp3 `(10,9)`; both passive tiles remain stable Floor |
| Fossil ledge | Native30 at metatile `(3,4)`; ordinary lane x6,y7–10; side wall `(7,9)` |
| Fossil wall | Read at `(6,4)`, approach `(5,4)` facing right; requires real `STATUSFLAGS_FLASH`, then all printed pages and acknowledgment |
| Sample alcove | Native36 at metatile `(8,6)`; lane x16,y11–14; side wall `(17,13)` |
| Sample cache | `(19,9)`, approach `(18,9)` facing right; native RARE_CANDY1 after the fossil clue |
| Darkness | Native DARK palette/phone selector4 in both map attributes and runtime metadata; native Flash resolves to NITE |

`EVENT_MN_LAMP_GALLERY_READ` records acknowledged fossil knowledge.
`EVENT_MN_LAMP_GALLERY_CACHE` records successful delivery.
`EVENT_MN_LAMP_GALLERY_HIDDEN` controls temporary object visibility separately.
Capacity refusal never sets the delivery claim. Object callbacks restore the
correct visibility after reentry and Continue. The original HM gift dispatch,
Antidote program, original actors and world warp are preserved exactly.
Only the original guide's text receives an appended hint.

The full native Flash source already exists in the supplied pack. This publisher
adds its executable root and three retained native definitions to the active
closure. It imports no external source dump, art, catalog or audio program.
All other original global programs remain exact. The new room inherits the
Driftless landmark; it adds no Fly or blackout recovery destination.

## Verification and limits

Three fresh default-profile controller cases cover actual original HM gift and
PACK teaching, badge refusal, both stairs in both directions, darkness refusal,
all three fossil pages and input lock, a held-page save/Continue, visible cache
reveal, successful native delivery, once-only persistence, full-pocket refusal,
native Toss/retry, paired passive returns and the original outdoor exit/reset.
They use declared initial local positions and a legal Minnesota Beetynamo;
Care Badge, known-move and full-pocket setups are explicit fixture variants.
They do not establish an earned regional campaign.

The native review loads actual reached saves without scene overrides. Two dark
views are inspected visually; seven illuminated frames are compared at all
368,640 pixels against native NITE source. The whole two-level source render is
also inspected. Every original, rejected, converted, cropped and GPU render is
retained. The first candidate's inherited NITE menu selector was rejected and
corrected to DARK in both native metadata paths before acceptance.

The full-parent audit removes only the new gallery, its three flags, paired
stair, appended hint and native Flash closure, and recovers the complete parent.
All732 other existing maps, original art/runtime files, audio, catalog data and
recovery destinations remain exact. All377 audio programs pass hashes, frame
counts and loop ranges. Scoped all-target Clippy and changed Rust formatting
pass; pre-existing workspace formatting differences remain.

## Executed Rust publication and fast review

```sh
CARGO_TARGET_DIR=target/verdigrim-scene CARGO_BUILD_JOBS=2 RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-assets --bin build_driftless_fossil_gallery
target/verdigrim-scene/debug/build_driftless_fossil_gallery \
  content-packs/minnesota-r05-shelf-drop-v01.crystalpack \
  71a40f87927ed85404c631869295b091225ac0e26c76fef0d084adf32e5f8d67 \
  planning/minnesota/caves/driftless-fossil-gallery-v01/gallery.json \
  planning/minnesota/caves/driftless-fossil-gallery-v01/contract.json \
  content-packs/minnesota-driftless-fossil-gallery-v03.crystalpack
```

Outputs must be fresh and ignored. The publisher verifies the complete current
parent module, preserves the native gift/item command tables, validates access
and paired stairs, and performs strict readback. Deliberately disabling the
Flash gate, blocking the eastern climb or presenting a stale parent contract
is rejected without writing an output.

The complete illuminated source rerender measured0.056 seconds after loading
art. `--watch` creates a separate immutable version on each source edit:

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_cave_source.py \
  --pack content-packs/minnesota-driftless-fossil-gallery-v03.crystalpack \
  --source planning/minnesota/caves/driftless-fossil-gallery-v01/gallery.json \
  --palette nite --watch \
  --output planning/minnesota/renders/driftless-next-source
```

The source preview represents illumination and revealed cache placement;
it does not bypass the actual runtime Flash or fossil-reading gates.

The fresh test binary is produced with the default-profile
`cargo test --locked -p crystal-bevy --lib --no-run --message-format=json` build.
The executed three-case filter is
`minnesota_field_use::fossil_gallery:: --ignored --nocapture --test-threads=1`.
Set `CRYSTAL_WINONA_TEST_PACK` to the exact pack,
`CRYSTAL_DRIFTLESS_GALLERY_SOURCE` to the frozen source, and fresh
`CRYSTAL_DRIFTLESS_GALLERY_REVIEW_DIR` / `CRYSTAL_MAP_RETAIN_SAVES_DIR` paths.
The read-only `audit.py`, `check_mcp.py` and `review_native.py` beside this
runbook expose their required inputs with `--help`.

Broader map/cave quality and earned campaign review remain active. This is a
local review revision and does not publish a browser release.
