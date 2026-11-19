# The fen, river and prairie investigations

## Layered Fen

From Northwoods, the player can examine an exposed root in the saplings, a reflective cut in the north bank of a pool and moss on the eastern rock face. The original room made the first three observations on those features, then required three different, unmarked squares to act on them. Its observer also described the pool as above the root. The revised features work for both steps. The last observation explains the order. An out-of-order touch or a conversation with the observer identifies the next landmark and preserves progress. The old search positions and story flags still work. Opening the hollow gives a dry 31-step route from the crossing to a speaking position beside Mitwist. The battle remains optional.

Crystal's Ilex Forest remembers the player's progress when placing Farfetch'd and changes the apprentice's reply. Here the existing trail and battle are preserved, and the observer remembers which feature the player needs. Polished Crystal's Moss Rock in Ilex Forest provided a reference for a single recognizable feature in a wooded area. The Minnesota root, bank and moss drawings are original. [Crystal Ilex Forest](https://github.com/pret/pokecrystal/blob/master/maps/IlexForest.asm) · [Polished Crystal Ilex Forest](https://github.com/Rangi42/polishedcrystal/blob/master/maps/IlexForest.asm)

## Lake Pepin

The sketcher stands on a grassy bank. A small animated curl of foam now sits on the adjacent water where the player examines the eddy. The sketcher gives a bearing and tells the player to take a canoe or use Surf. The skipper directs the player to the low pine branches visible at the second observation. The route around the bend between the two people is 166 steps. From the pines, the sheltered inlet is 71 water steps north and west. The people remember which observations are still missing and can repeat the directions to the inlet. Reaching Barbelisk, declining its battle, withdrawing and returning retain their original conditions.

## Field Margin

Entering from Route 16, the grooves are in the southern patch of bare dirt. The caretaker explains that the player needs to step back out of the field and return to see the tracks change. From the northern patch the two stones are to the west. The northern stone has the buried edge and the southern stone is the old plow marker. Their entire solid faces accept examination. After the player reports the marker, the caretaker identifies the clearing where he saw the turf lift. Approaching Kensigard from any of its four sides starts the original reveal. Twelve native search squares and all four reveal approaches connect to the entrance across the same 399 walking squares.

## Build and verify

The [Field receipt](field-build.json) and [Intro receipt](intro-build.json) identify both external Town Returns parents and the expected children. Run from the repository root with the matching parent and a fresh output path. For the commands below, set `parent` and `child` to the paths from one receipt. Keep both editions' archives separate. Existing saves belong to their original archive identity.

```sh
CARGO_INCREMENTAL=0 cargo build --locked -j1 -p crystal-assets --features pack-authoring --example pack_fen_trail
target/debug/examples/pack_fen_trail "$parent" "$child"
checks=planning/minnesota/revisions/fen-trail-v01
uv run --offline --python 3.14t --with cbor2 --with pillow python "$checks/check-preservation.py" "$parent" "$child"
uv run --offline --python 3.14t --with cbor2 --with pillow python "$checks/check-landmarks.py" "$parent" "$child"
uv run --offline --python 3.14t --with cbor2 --with pillow python "$checks/check-fen.py" "$child" --parent "$parent"
uv run --offline --python 3.14t --with cbor2 --with pillow python "$checks/check-pepin.py" "$child"
uv run --offline --python 3.14t --with cbor2 --with pillow python "$checks/check-field.py" "$child"
MN_FEN_TRAIL_PACK="$PWD/$child" RUST_MIN_STACK=67108864 CARGO_INCREMENTAL=0 cargo test --locked -j1 -p crystal-assets --lib installed_field_resolves_the_visible_faces_and_all_four_approaches -- --ignored --test-threads=1
MN_FEN_TRAIL_PACK="$PWD/$child" RUST_MIN_STACK=67108864 CARGO_INCREMENTAL=0 cargo test --locked -j1 -p crystal-assets --lib owned_canoe_reaches_pepin_pine_observation_without_a_surf_partner -- --ignored --test-threads=1
```

Both editions passed the static comparisons and the engine interaction test. Six maps differ from each parent. All 138 original Johto terrain blocks retain their morning, daytime and night pixels. The root retains Headbutt, the water retains Surf and its original moving tiles, and the moss retains its solid bank. The four-frame Pepin foam uses two additional native tiles. The map comparisons include all other rooms, encounters, flag addresses, trainers, movement permissions and audio. Native camera renders verified the visible landmarks and revealed Kensigard. The route and quest results are compiled checks without a playthrough.
