# Lynx Fire starter

Kindlit → Rimepelt → Borealynx is the Fire → Fire → Fire/Ice starter,
with stable source IDs MN_274–276 and family MN_F125. The withdrawn water
proposal retains MN_F124 / MN_271–273; its IDs were not reused.

All three remain quadrupedal, with short bobtails, broad paws and tufted ears.
The progression is a small banked ember, a lean coal-bodied prowler with a
shoulder flame mantle, and a broad winter lynx with a blazing chest, low solid
frost mantle and frozen snowshoe paws. The final design removes tall armor
spikes and layered fur. Starter totals are 318/405/530 with level16/36 evolutions.

The user-requested v8 redraw gives the family compact, closed-mouth faces,
clearer eye clusters, and a broader orange adult chest beneath one low frost
mantle. Enemy fronts face left toward the player. Player rear portraits show
the backs of the ears, nape and shoulders aimed toward the upper-right opponent;
their eyes are hidden.

Cindore → Pelletor → Mesablast remains active as the ordinary Iron Range
family MN_F002. Its identities, profiles, evolution chain and palettes were
preserved; only its Fire starter role was removed. The master order places the
lynx between the Grass and Water starters and the furnace family in Iron Range.

## Selected art

`front-back.imagegen-v8.png` is the selected built-in ImageGen source;
`prompt-v8.txt` records its exact prompt. All eight returned source sheets and
prompts remain alongside it. `reference-mapping.json` records their roles.
Native redesign12 in `manifest.json` is the selected pixel version.

Native fronts are56×56 and rear portraits48×48. Every normal view contains
exactly four RGB555 inks, including white and black. The adult uses white,
orange, pale blue and black. Native PNGs, indices, 2bpp tiles and immutable
conversion history remain ignored under `target/minnesota-pokemon/native/MN_F125/`.

The broad earlier native8 face stencils were rejected as boxy and remain in
`native-refinement-v8.rejected.json`. Native9/10/11 and the full pre-redesign
manifest in `redesign-20261027-v1/manifest-before.json` remain retained.
Redesign12 uses source-center sampling and small guarded front refinements:
5 Kindlit eye pixels, 8 Rimepelt eye pixels, and 13 Borealynx eye/paw pixels.
Rear portraits require no native edits. Source-center, area-vote and refined
candidates and their settings are retained for comparison.

The Rust converter applies each edit only when its declared previous ink
matches. It synchronizes rows, PNG pixels and tile indices. The Rust pack loader
independently repeats source conversion and edits, rejecting undeclared painting,
stale coordinates, invalid inks, changed provenance or mismatched PNG/2bpp data.
No unrecorded changes were made to generated PNGs.

## Redesign12 verification

Inspected the source sheet and all six rebuilt native views at native size and
8x. The active manifest preserves each original four-ink palette, all types,
profiles and evolution data. Rebuilt with the existing Rust converter:

```sh
target/agent-minnesota-species/debug/examples/minnesota_sprite_sheet --family MN_F125
```

A temporary three-species fixture was loaded through the real Rust catalog and
pack compiler using `northwoods-moves-v2.crystalpack` as the immutable base.
This repeats source-center conversion and guarded edits and checks palette,
PNG, rows, 2bpp and the exact review-sheet hash. The focused pack verified 471
species, 264 moves and 377 audio programs with matching PCM hashes, frame counts
and loop ranges using `verify_crystalpack --audio --desktop`. The temporary
fixture and review pack were removed after verification. The latest native
outputs remain in `target/minnesota-pokemon/native/MN_F125/` for the next export.

The Rust website exporter also consumed the new six frames and generated their
normal and shiny variants. This is source/native/pack verification; no live game
session or website publication was performed.

## Earlier source-v7 verification

The changed Rust leaves pass direct rustfmt checks. Fifteen focused Rust tests
pass, including guarded/atomic edits, PNG/row/tile synchronization, rejected
undeclared painting, and unchanged legacy conversion. The authored-catalog
regression also passes for the final lynx, retained furnace line and existing
Musketunge control family: actual party creation, level evolutions, PP spending,
damage, dex/save references and exact preservation of base audio/runtime bytes.

The final focused local pack is
`content-packs/minnesota-lynx-review.crystalpack`: the verified 251-species base
plus those nine approved review species, with the existing13-move extension.
The temporary nine-species test fixture was used to isolate approved artwork
and is removed after verification. This is an artwork/content review pack,
not a full regional-catalog publication or a new starter-offer script.

Executed native build and final pack verification:

```sh
CARGO_TARGET_DIR=target/agent-minnesota-species cargo build --locked \
  -p crystal-assets --example minnesota_sprite_sheet --bin pack_minnesota_pokemon \
  --bin pack_northwoods_moves --bin export_minnesota_pokedex_web \
  --config 'profile.dev.package.crystal-assets.opt-level=0' \
  --config 'profile.dev.package.crystal-core.opt-level=0'
target/agent-minnesota-species/debug/examples/minnesota_sprite_sheet --family MN_F125
target/agent-minnesota-species/debug/pack_northwoods_moves \
  content-packs/minnesota-lynx-review-species.crystalpack \
  content-packs/minnesota-lynx-review.crystalpack
target/agent-minnesota-species/debug/verify_crystalpack \
  content-packs/minnesota-lynx-review.crystalpack --audio --desktop
```

The earlier full candidate `northwoods-lynx-moves-v1.crystalpack` verified470
species,264 moves and377 audio programs, with matching PCM hashes, frame counts
and loop ranges; it predates the last facial-outline edits. A later full rebuild
was correctly blocked by the separate MN_F076 manifest marked `user-rejected`.
That rejection was not overridden. The earlier full website snapshot's complete
integration test also encounters that family's ongoing Swaylet/Mitwist rename;
it is not reported as a passing final publication.

Workspace `cargo fmt --all -- --check` still reports existing differences
beginning in `crates/offset-allocator`. Required all-target Clippy remains blocked
by existing `clippy::bad_bit_mask` at `crates/crystal-core/src/multiplayer/mod.rs:1332`.
Those unrelated files were not repaired by this work. No website deployment,
fullscreen launch or interactive-playability claim is made.
