# City cyclist overworlds

All 18 Minneapolis riders use dedicated NPC bicycle graphics, separate from
Chris/Kris and ordinary walking trainers. Nine use the blue-helmet commuter;
nine use the green-helmet rider with short hair. Both retain Crystal's compact
16×16 proportions, six directional/pedaling poses and original bicycle geometry.
The new heads remove Chris's backwards cap and Kris's large hair silhouette.

The selected sources are `01-commuter/native-pixels-v2.json` and
`02-city-rider/native-pixels-v1.json`. Each folder's `binding-v1.json` selects
nine existing actors by exact map, script, tile and previous sprite. Both are
`field_only` bindings with no trainer IDs: battle classes, portraits, names,
teams, rewards, dialogue, movement and progression remain unchanged. The native
encoder's complete-bank contract includes an unused portrait copied from the
accepted Cycle Tourist source; no new battle class or battle is registered.

## Build and run

The executed build used existing repository-built Rust tools:

```sh
CRYSTAL_SPRITE_BIN_DIR=target/verdigrim-scene/debug \
CRYSTAL_SPRITE_ENCODER=target/npc-sprites/npc_sprite_candidate \
  bash tools/build-city-cyclist-sprites.sh \
  content-packs/minnesota-minneapolis-mill-arts-v02.crystalpack \
  content-packs/minnesota-city-cyclists-v01.crystalpack
.local/route-grass-review/portable-tools-v01/geothite mcp \
  content-packs/minnesota-city-cyclists-v01.crystalpack
```

The final pack SHA256 is
`d2b51c4317a56d1aabafc7528342cfb120fa9b1600e60bfe9a79af87508e364d`;
its content identity is
`602da851faa3218a91335ca287bda6d584cfd616789c1f51ff6c4890d150a984`.
It extends the selected Mill/Arts streets revision, preserving its Duluth Mart
parent and all 732 maps. Use fresh ignored output/evidence names for reruns.
The tools' sources are `crates/crystal-assets/examples/npc_sprite_candidate.rs`
and `crates/crystal-assets/src/bin/pack_trainer_art.rs`; generated banks and packs
remain ignored.

Native comparison sheets, exact final game views and receipts are in
`target/city-cyclists-v01/review/index.html`. The comparison sheets include
original Crystal bicycle poses at a common palette, native 1× strips and equal
nearest-neighbor enlargement. The actual field views show both new NPC forms
beside the original red-capped player.

## Verified behavior

The four production-controller cycling regressions pass on this exact pack:
selected-move PP mutation and Continue for riders 01/13/18; a legal victory with
the authored return command and defeat flag; native patrol movement on its
existing loop; and declining the optional battle without starting it. These use
explicit adjacent-position and legal Minnesota party fixtures, including a
level-80 victory party. They do not establish ordinary-level balance or an earned
regional journey.

The production stdio client passed 22 JSON-RPC calls covering Start/Pack, actual
movement, downstairs dialogue ownership, saving and fresh-process reload, with
byte-empty stderr. The Rust verifier passed all 377 audio PCM hashes, frame
counts and loop ranges, plus desktop consumers. An independent raw-CBOR audit
checked every pack field: only the 18 intended actor sprites, matching event
mirrors, two palette registrations, 12 exact private resources and publication
identity/manifests changed. Player art, existing battle art/classes, every team,
AI, flags, scripts, geometry and audio are preserved.

## Carrying this into other work

Use `planning/minnesota/world.json` for the latest selected parent. Geometry
recomposition clones its existing actors, so retaining the selected parent
preserves these banks and sprites. For a fresh historical build, apply this
overlay after cyclist creation/dialogue refresh and map publication. The original
cyclist placement source intentionally remains the legacy foundation; the
field-only bindings are the final appearance override.

The paused 58-set regional trainer plan binds cyclist **battles** to Cycle Tourist
without replacing these fields. Its retained local assembly guard now accepts
the private NPC bike IDs as well as legacy bike IDs. This art revision does not
complete that separate paused publication or deploy a browser release.
