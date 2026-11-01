# Early starters and complete fossil access

This stage removes two unnecessary Champion gates. After delivering Tamarack's
first parcel to the Rochester clinic, return to the Winona field-station aide.
The two starters you did not choose are free level-5 research partners, one per
conversation. The original choice and previous Kindlit research claims are
respected. Declining or arriving with a full party preserves the offer. Leave
one party slot and return. The original caught-species rewards still run first.

Choose either free specimen from the Route 1 bluff researcher. Earn the Care
Badge and bring it to the signed Fossil Lab beside Rochester's clinic. After
the scientist restores the first level-10 partner, speak to the assistant in
the same room for the other free specimen. Restore that one too. The original
bluff researcher can also provide the alternate specimen after restoration;
both providers share one claim, preventing duplicate grants. Failed delivery
retains the specimen or offer. Trilume and Siphlet evolve at level 34.

The lab also directs players to Castorock's separate Route 3 cave discovery.
Use Strength to enter its incisor gallery and acknowledge the three jaw and
waterline observations. Refusal, RUN or defeat does not permanently remove an
uncaught Castorock; leave and reenter to retry. Castoroar evolves at level 32.

## Pack and verification

Pack: `content-packs/minnesota-early-research-v01.crystalpack`.
Parent: `content-packs/minnesota-leech-mainland-v01.crystalpack`.
Content identity:
`42f026e009fd591da612f6120b70c584fbe6a13bbd0dea85a6742fad579cfbc1`.
Pack SHA-256:
`1fd1bfe59ad965809741259f88a7c251e59e2add5317bfc5eacd17da7b9b7548`.
Resolve `world.json` for the current selection; later stages may inherit it.

Three production-controller cases pass both fossil choices, actual Route 1 to
lab travel, both restorations, all three starter choices, refusal, full party,
once-only claims and Continue. The earned case starts at New Game, completes
the actual first parcel delivery, and returns for all three starters before
any badge. Production stdio then continues that unchanged reached save through
the aide's completed dialogue, movement and Start menu: 36 calls, empty stderr.

The Rust preservation regression retains all 732 other complete maps, every
terrain/warp/actor, existing rewards and encounters, and exact native files,
audio bytes and audio manifest. Only the three research maps' relevant programs
and text change, together with two new starter-claim flags. The assistant's new
explicit facing command is included in the scoped metadata comparison.

```sh
CARGO_TARGET_DIR=target/verdigrim-scene CARGO_INCREMENTAL=0 \
  cargo build --locked -p crystal-assets --bin pack_minnesota_research_access
target/verdigrim-scene/debug/pack_minnesota_research_access \
  content-packs/minnesota-leech-mainland-v01.crystalpack \
  content-packs/minnesota-early-research-v01.crystalpack

CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-early-research-v01.crystalpack" \
  CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo test --locked -p crystal-bevy --lib early_research_ \
  -- --ignored --nocapture --test-threads=1
```

The publisher requires a fresh ignored output; choose another filename when
replaying the commands. The web location publisher recognizes this stage and
describes the actual early grants and same-room second fossil. Release
`20261031-early-research-29` preserves all 225 species pages and previous media;
local application, location and desktop/mobile browser checks pass.

The [full fauna successor](encounter-progression-v01/README.md) now inherits
these early services and closes all 225 acquisition paths. The smaller pack
recorded on this page retains its historical counts below.

This earlier stage still has its lineage's nine missing rare sources and
224-member compiled Dex. The separate comprehensive early-research candidate
includes all 225 species; its discovery and evolution checks do not turn this
smaller stage into a completed statewide collection or campaign. Biome
corrections and full-roster integration are documented in the successor.
