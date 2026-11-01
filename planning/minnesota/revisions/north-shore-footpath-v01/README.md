# North Shore footpath to Grand Marais

[Fresh selected-pack verification](current-selected-review.md) confirms the
complete mainland round trip still works without Surf in the selected local
International Falls successor, with new retained renders and production input records.

The main coastal path now reads as a continuous land route: seven river
crossings use native gravel and gray side rails, replacing eight pink brick
blocks that visually resembled building walls. The two-tile-wide crossings,
shore landings and all existing gameplay collision remain intact. This is a
bridge readability repair, not a claim that previously disconnected collision
was discovered. The prior and repaired current terrain both have a dry course.

The repaired pack was played from Duluth through the actual shore gate, along
North Shore to Grand Marais's west entrance, and back to Duluth. The party had
zero Surf moves at every saved checkpoint and stayed in Normal walking mode.
The 1,953-call production-controller replay uses explicit pre-input setup:
Duluth `(64,37)`, original Minnesota Cindore/Fortigard at level 80, supplies,
eight badges and the existing lighthouse signal. Those story prerequisites
remain enforced. This does not claim earned campaign access.

`plan.json` lists the eight exact cells, native glyph columns, endpoint
connections and gameplay contract. `before.json`/`after.json` are complete
native chunk preimages/candidates. `current-native-chunks-v01.json` and
`current-exterior.contract-v01.json` bind the installed route. The active
selection and composed source index are in `planning/minnesota/world.json`.

Executed Rust publication, using the selected parent at publication:

```sh
CARGO_TARGET_DIR=target/verdigrim-scene CARGO_BUILD_JOBS=2 RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-assets --bin rebuild_north_shore_footbridges
target/verdigrim-scene/debug/rebuild_north_shore_footbridges \
  content-packs/minnesota-current-r13-boundary-apron-v03.crystalpack \
  7d3987bfbf0378aed960495cc20981c0d258eb391f33630b8e3de6c2686e2e7d \
  planning/minnesota/revisions/north-shore-footpath-v01/before.json \
  planning/minnesota/revisions/north-shore-footpath-v01/after.json \
  content-packs/minnesota-north-shore-footbridges-v01.crystalpack
```

The publisher requires a fresh ignored output. It rejects stale full-route
preimages and unrelated edits. It copies existing native metatile glyphs; no
original PNG, 2bpp, palette, building block, script, actor, encounter, team,
connection, ledge, stair, waterfall, water boundary or audio program changes.
The exact-parent audit covers all 732 maps; 728 are entirely unchanged.

For a fresh whole-route review, use the read-only renderer with a new output
directory. Add `--terrain planning/minnesota/revisions/north-shore-footpath-v01/after.json`
when previewing against the parent before publication. The measured warm source
render and static dry graph took 0.991 seconds after loading the external pack;
the installed version took 0.856 seconds. Every output keeps native originals,
all seven crossing crops and the static dry-course overlay. The overlay is
planning evidence; the recorded controller replay establishes actual movement.

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_north_shore_footbridges.py \
  --pack content-packs/minnesota-north-shore-footbridges-v01.crystalpack \
  --output planning/minnesota/renders/north-shore-footpath-next-review
```

[Whole native route](../../renders/north-shore-footpath-v563/installed-v569/whole-original.png),
[dry course](../../renders/north-shore-footpath-v563/installed-v569/dry-course-overview.png),
[all nine production views](../../renders/north-shore-footpath-v563/native-v569/production-decisions.png),
[acceptance](../../renders/north-shore-footpath-v563/acceptance-v572.json).
Every native view matches the source at all 368,640 pixels; all 377 audio programs
pass PCM hash, frame-count and loop-range verification. Scoped all-target Clippy
passes with existing warnings; new Rust leaves pass formatting. Workspace-wide
formatting still reports existing differences beginning at
`crates/offset-allocator/src/lib.rs:408`.

This addresses the North Shore mainland trip. Deeper cave quality, wider regional
map review and earned campaign verification remain ongoing work.
