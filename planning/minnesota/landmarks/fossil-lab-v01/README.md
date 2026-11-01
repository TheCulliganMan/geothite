# Rochester Fossil Lab

The signed lab stands between Rochester's clinic and mart. The Driftless bluff
researcher now directs the player there. Its scientist restores LENS FOSSIL into
TRILUME or SPIRE FOSSIL into SIPHLET at level 10 after the Care Badge, with a free
party slot. The specimen is consumed only after the native gift succeeds.
The saved restoration flags and post-Champion second-choice exchange remain
unchanged. Neither fossil becomes an ordinary wild encounter.

The complete native facade, brick apron and sign use eight existing modern-town
metatiles. The 5×5 lab uses original ElmsLab equipment, bookcases and exit carpet.
ElmsLab, OaksLab and RuinsOfAlphResearchCenter were inspected as native reference
interiors. The supplied pack provides all art and collision; no new graphic or
3D presentation was introduced. The room has a restoration machine, specimen
bench and an assistant explaining where to obtain the fossils.

`lab.json` binds the exact parent identity and exterior preimages, full indoor
rows, actor/bench positions, clear arrival and both exits. The Rust builder
checks the native map slot, collision and conservative dry access. It retains
shared outdoor helper programs verbatim and namespaces the moved restoration
programs inside the lab, preserving every native item, gift and event operand.

| Native contract | Core tile / warp |
| --- | --- |
| Exterior public approach | Rochester (24,12) |
| Sign / reader approach | (26,10) / (26,11) |
| Door / passive return | (23,9), warp11 / (23,10), warp12 |
| Indoor passive arrival | (5,8), warp3 |
| Scientist / caller | (3,4) / (3,5) |
| Bench / reader | (6,3) / (6,4) |
| Both carpet exits | (4,9), warp1 and (5,9), warp2; outward Down returns to city12 |

Executed build/publication from the repository root:

```sh
CARGO_TARGET_DIR=target/verdigrim-scene CARGO_INCREMENTAL=0 \
RUST_MIN_STACK=67108864 cargo build --locked -p crystal-assets \
  --bin pack_minnesota_fossil_lab --example publish_pokedex_locations
RUST_MIN_STACK=67108864 target/verdigrim-scene/debug/pack_minnesota_fossil_lab \
  content-packs/minnesota-landmark-habitats-v02.crystalpack \
  planning/minnesota/landmarks/fossil-lab-v01/lab.json \
  content-packs/minnesota-fossil-lab-v01.crystalpack
```

The immutable child has 735 maps and content identity
`eee053cc55a1f3ddcc98c9cab722cf9b2a2630794993bb0d6b1cdfbe249bcbe0`.
Only Rochester and the bluff directions change among the 734 original maps;
732 are exact. The new room is the only addition. All native art, 377 audio
programs, encounters, fishing tables, trainer parties, items, event catalog,
HOME/recovery and train service remain unchanged. Audio PCM hashes, frame
counts and loop ranges pass.

Four production-controller cases verify the street/sign, bench, restoration of
both specimens, both real returns and reentry, Save/Continue, the existing
choice/refusal/exchange, badge/full-party retry and level-34 evolutions. The
local cases explicitly declare their initial badge/item/party or evolution
fixture; they do not establish earned statewide collection. An external asset
regression also checks strict preservation and rejects stale parents, occupied
native IDs, blocked actor sites, bad exits and duplicate installation.

Native 2D source and actual reached-session captures were inspected under the
ignored `planning/minnesota/renders/fossil-lab-v01/`. The static union of warp,
connection and conditional map-command edges connects this room to Winona; it
is a structural check, not proof of an earned campaign.

The matching web release updates both fossil families' natural-habitat notes,
restoration location and requirements. Its existing profiles, learnsets, art and
audio are preserved. Read `world.json` for current selection and lineage.

The production stdio client passes74 calls across both restoration/return cases,
including the native nickname choice and Start menu, with byte-empty stderr.
The live web release passes desktop/mobile checks; all2203 deployed files match
the local release. Scoped formatting and both affected crates all-target Clippy
pass with existing warnings and no diagnostics in the new code. The full
workspace format check still reports unrelated offset-allocator, pack-core and
web-server differences. No new Cargo target directory was created; the shared
cache remains in use by active reviews. Free disk after checks is169GiB.
