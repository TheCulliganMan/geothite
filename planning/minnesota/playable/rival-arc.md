# Rival arc implementation and acceptance

The [current 704-map story composition](../revisions/current-story-v01/README.md)
replays these authored chapters on compact geography and preserves accepted
R05 grass, R01 limestone cave and the written League finale. Use that current
publication instead of the historical arc-v8 below.

> **Historical story candidate; withdrawn from current geography publication.**
> Arc-v8 and its frozen parents contain the retired oversized North Shore /
> Gunflint layout. Keep their saves and render originals for historical evidence.
> Do not branch or publish a new pack from them. The 11 `pack_northstar_*`
> publication commands now require the compact canonical footprints and ports.
> Rebuild the story overlays on the canonical compact geography first; never
> transplant the old outdoor modules or migrate these saves to a new identity.

This revision extends the existing Minnesota campaign and both authored lairs.
The authoritative scene dialogue and narrative contracts are in
`planning/minnesota/story-arc-v3.json`; the physical final chamber is authored in
`planning/minnesota/lairs/rival-arc/intake.json`. Faction teams and legal moves
remain in the single regional policy, `planning/minnesota/story-pokemon.json`.

Reed has six battles: Winona, Rochester, ICE relay, St Paul, the mine and the
Boundary intake. The Minneapolis recruitment, St Cloud intervention and optional
St Cloud station return make his choices visible between battles. He pursues
paid status, knowingly betrays Ellis and guards a crew he knows is endangered.
Losing authority does not redeem him.

Rochester introduces Ivy's missing crew and a displaced Gillpup. St Paul brings
Mina and Finn into Reed's transfer work. The St Cloud scene releases Sera through
an acted intervention; its refusal can be retried. The mine retains its separate
haulage, ventilation, feed and drive operations. The harbor contains a physical
locking cradle and a live call from Ivy at the intake.

The Boundary chief battle, radio link, final rival battle, local shutdown,
creature escape and crew clearance are separate steps. The shelter hatch stays
physically closed until the live link is acknowledged. Kettlurk moves through
the opened gate into actual water before disappearing; Ivy and Finn walk clear.
Restoration waits until those actions and dialogue acknowledgments finish.
Continue restores the opened gate, hidden actors and final crew positions.

Tamarack, Ellis, Jo, Mina and Nell have changed return dialogue. Unrelated route
trainers and League member Halden no longer share Reed's name. The original
Ryan cabin, gatehouses, route geometry, lair networks, species records and audio
are preserved. New machinery uses private native-art tileset extensions.

## Validation status

The initial structural build and later native results below belong to the
historical arc-v8 lineage. Its current compact-geography replacement has a
separate [publication and acceptance record](../revisions/current-story-v01/README.md).
Final dialogue labels are now revised by `story.json`'s `dialogue_overhaul` after
the arc overlay; old page counts cannot certify those newer pages. Chapter
fixtures do not establish an earned full campaign.

The builder requires the unbound `northstar-iron-workings-v1` parent with native
Kettlurk and Gillpup overworld banks, and rejects an already region-bound pack.
Apply the regional species binder last. Keep all outputs ignored and use a new
filename for each reviewed publication; never overwrite a frozen input.

## Historical executed publication

The historical candidate is `content-packs/minnesota-northstar-story-arc-v8.crystalpack`:
567 maps, 474 installed species records, SHA-256
`9a9269140978b9186aa5263f9524dae3034f3b66b49ec47089fb893894a23779`.
The installed count includes preserved source regions; Minnesota appearances
are bound and audited against the active Minnesota roster.

```sh
CARGO_TARGET_DIR=target/northstar-story CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0 \
RUST_MIN_STACK=67108864 cargo build --locked --profile wasm-dev \
  -p crystal-assets --features test-fixtures \
  --bin pack_northstar_rival_arc --bin pack_northstar_regional_cast
RUST_MIN_STACK=67108864 target/northstar-story/wasm-dev/pack_northstar_rival_arc \
  content-packs/minnesota-northstar-arc-art-v1.crystalpack \
  planning/minnesota content-packs/minnesota-northstar-rival-arc-v10.crystalpack
RUST_MIN_STACK=67108864 target/northstar-story/wasm-dev/pack_northstar_regional_cast \
  content-packs/minnesota-northstar-rival-arc-v10.crystalpack \
  planning/minnesota/story-pokemon.json planning/minnesota/pokemon/roster.json \
  planning/minnesota/pokemon/roster-curation.json \
  planning/minnesota/pokemon/species-profiles.json \
  content-packs/minnesota-northstar-story-arc-v8.crystalpack
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_rival_arc.py \
  content-packs/minnesota-northstar-rival-arc-v10.crystalpack
```

The read-only review passes: closed/open chamber access, every scripted creature,
crew and player path, actor anchors and exact original block pixels across all
four private tileset imports at morning/day/night/indoor palettes. The initial
paired native capture exposed offscreen action; the current revision stages the
player at the outflow and then the crew instead.

### Grunt coverage revision

The rival overlay now adds 24 ordinary sight trainers: eight ICE headquarters
guards, twelve Lode mine guards, and four allied intake guards. Headquarters
coverage totals 14 ordinary guards across six floors, excluding Reed and the
enforcer. The mine has twelve additional ordinary guards across its six
underground/approach maps; the foreman and Reed remain separate encounters.
All added grunts have two or three Minnesota partners, native legal moves,
individual sightlines, dialogue, and persistent defeat flags. Existing recovery
services and puzzle controls are preserved. The new candidate is
`minnesota-northstar-story-arc-v8.crystalpack`, built from unbound arc-v10.
SHA-256: `9a9269140978b9186aa5263f9524dae3034f3b66b49ec47089fb893894a23779`.
The native `rival_arc_all_twenty_four_grunts_battle_and_persist_with_chapter_parties`
case passed for all 24 guards. Each fight uses production input and ordinary
chapter parties, checks the actual individual defeat event, then saves and
continues in the same pack. No encounter grants restoration. The mine and
intake cases include lead knockouts with the remaining party finishing the fight.
This is individual-encounter evidence; the continuous headquarters course and
New Game through Hall of Fame review remain in progress. The strict Rust
publication-preservation test also passed (27.51 seconds), including original
floor geometry, puzzle events and warps on the newly guarded floors.

### Executed candidate-v8 acceptance

- Seven native rival/grunt/worker/climax/aftermath cases passed in 797.50 seconds,
  including all 24 individual added grunts and all three starter branches.
- Regional preservation/party-contract test passed in 25.16 seconds.
- Production stdio MCP Rochester, intake and aftermath replays passed with
  byte-empty stderr, actual visible choices, dialogue and the production Start menu.
- Paired 2D/2.5D captures of Kettlurk entering the water and Ivy/Finn clear of
  the machinery were inspected under `renders/story-arc-v3/native-v8/`.
- The continuous level-23 headquarters trial exhausted its four carried potions
  and lost to the enforcer after using the nurse. The review route now restocks
  at the real Minneapolis Mart using earned money before that fight; the first starter branch has now cleared the enforcer through that real
  restock route, while the remaining branches are still running. Boss parties and player levels were not changed.
- The fresh earned campaign is still running. Its chapter checkpoints must stay
  with candidate-v8. It is not yet Hall-of-Fame acceptance.

`cargo fmt --all -- --check` reports existing formatting differences outside the
new rival modules. The full assets all-target Clippy attempt encountered a
missing unrelated `tests/split_rock_clinic.rs` source; the assets library and
combined assets/Bevy library-test Clippy checks completed with existing warnings.
The final Bevy all-target Clippy check passed after the native shop-route fix
with existing warnings. Scoped formatting checks pass for the rival modules
and revised headquarters review.

## Compact geography audit — 2026-10-28

The arc-art-v1 parent and story-arc-v8 have R11 terminal24×24 and lower
Gunflint32×24, retain the retired R12 eastern/bottom chunks, and lack Cedar.
The approved `minnesota-superior-compact-v03.crystalpack` has terminal4×24,
Gunflint32×32 aggregate (lower chunk32×8), Cedar20×12 and Marais20×19.
It has no installed Northstar lair overlay, so it cannot simply replace the
old arc parent in the final rival command. Story overlays must be reapplied
in order while preserving the canonical outdoors. The acted-opening source
also pins an old parent hash and must receive a reviewed rebase contract.

The gate in `northstar_publication_geometry.rs` rejects the obsolete dimensions,
retired map identities and old compact-port offsets before every CLI story
overlay or regional cast binding. Historical runtime tests remain useful for
local story logic; they do not establish acceptance of rebased canonical packs.
The three-starter continuous HQ course passed after real Mart restocking
(825.66seconds). The earned New Game run failed at the enforcer after2399.76
seconds using the older test executable; no Hall-of-Fame completion is claimed.
