# Choosing a research partner in Winona

Once Ivy receives Tamarack’s reading in Rochester, the aide at the Winona Field Station offers the two partners who stayed behind. A player who started with Kindlit can ask about Birchit first, hear the introduction and decide whether to take it. Pouchid is still there for a later visit. The menu shows the actual two species for each starting partner. B and Cancel both end the conversation. Declining a particular Pokémon or arriving with six party members leaves that partner available.

Each adoption uses that species’ original introduction, level-5 partner, nickname prompt and collection record. Once the player has claimed one, the aide goes straight to the one who remains. After both, the aide returns to the original farewell. The record follows the adoption even if the partner has moved to the PC. Existing saves use their original pack.

## Driftless Lamp Gallery

After using Flash and reading the fossil shelf, the player can find a Rare Candy. If the Items pocket is full, the game now says “No room for the RARE CANDY.” The cache stays in place. Making room allows collection on the same visit or after saving and returning. The original discovery, sound and receipt still happen on a successful pickup.

## Reproduce

The [archive check](check-pack.py) takes a [Field](field-build.json) or [Intro](intro-build.json) Marais Supplies parent and its corresponding child. It compares every map, the typed gift and menu records, and the embedded art and audio. The only changed rooms are the Winona Field Station and the Driftless Lamp Gallery.

Run from the repository root with absolute paths to the matching parent, a fresh child output and a fresh directory for local captures.

```sh
export CARGO_INCREMENTAL=0 RUST_MIN_STACK=67108864
cargo run --locked -j1 -p crystal-assets --example pack_ely_challenge -- --research-choices "$parent" "$child"
uv run --offline --no-project --with cbor2 --with pillow python planning/minnesota/revisions/research-choices-v01/check-pack.py "$parent" "$child"
export CRYSTAL_WINONA_TEST_PACK="$child" CRYSTAL_DRIFTLESS_GALLERY_REVIEW_DIR="$frames/gallery-saves"
export MN_RESEARCH_CHOICE_FRAMES="$frames"
cargo test --locked -j1 -p crystal-bevy --lib -- --ignored --test-threads=1 early_research_starters_all_choices early_research_starters_existing_claims winona_aide_uses_the_players_actual_stage fossil_gallery_controller_full_pocket_claim
```

The native Winona checks cover all three starting partners, either way to cancel, a full party, a deposit at the clinic PC and collection after Save/Continue. Both editions also passed the journey from a new Kindlit game through Ivy’s clinic and back to the aide before the first badge. The player chose Birchit first, returned for Pouchid and kept all three through Save/Continue. The Gallery check uses Flash, examines the fossil wall, reads the refusal, saves, makes room in the Pack and returns for the candy. For the full first-delivery journey, run `early_research_starters_earned_first_delivery_returns_before_any_badge`. Set `CRYSTAL_NORTHSTAR_CANONICAL_STORY_SOURCE` to the matching edition’s authored story when both editions share a working tree. Each has its own version of Mom’s opening.

The [South Lake movement checks](../fen-waterline-v01/README.md) cover walking the pebble bank, stepping into the canoe, paddling and landing. Both editions keep the same Fen geography and movement art as Marais Supplies.
