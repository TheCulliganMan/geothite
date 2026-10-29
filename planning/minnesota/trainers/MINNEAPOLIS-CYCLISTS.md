# Minneapolis cycling trainers

Eighteen native bicycle riders follow clockwise street circuits of 32–48
walking tiles in the civic, eastern neighborhood and market/lake streets.
The player talks to a rider and answers Yes/No: Yes starts a native trainer
battle, No closes the conversation and riding resumes. These are script actors
with zero sight radius, so passing a cyclist never starts a sight battle.
Each rider has a distinct name, two active Minnesota Pokemon, and a persistent
native defeat flag. Parties start at levels 18–23 and retain the parent's
scaling setting (disabled in this parent). Native CHRIS_BIKE/KRIS_BIKE art
supplies the bicycles.

The source is `minneapolis-cyclists.json`. No terrain, doors, connections,
existing actors, story or artwork is replaced. Publication checks every loop
tile against native floor collision and existing actors/events. Loop extents
fit the native four-bit movement fields. Crossings use native actor collision;
cyclists wait for occupied tiles. Only actors near the player advance, matching
the production map-object simulation. Initial battle approaches and ordinary
city access are checked during publication.

Build from the existing original trainer cast descendant (preserving all 33
previously added trainers):

```sh
CARGO_TARGET_DIR=target/trainer-cast cargo run --locked --profile wasm-dev \
  -p crystal-assets --bin pack_minneapolis_cyclists -- \
  content-packs/minnesota-trainer-cast-33-v01.crystalpack \
  planning/minnesota/trainers/minneapolis-cyclists.json \
  content-packs/minnesota-minneapolis-cyclists-v01.crystalpack
```

The output must be fresh. Generated packs and review images remain ignored.
Verification results are recorded after execution below.

## Publication and preservation

Published `content-packs/minnesota-minneapolis-cyclists-v01.crystalpack` from the
33-trainer-cast parent. The written pack passes the Rust runtime verifier.
The read-only audit confirms 18 additions, 32–48 tile loops, script actors with
zero sight radius, and unchanged other maps, existing actors, species, native
art and audio manifest. Whole-city and district reviews are retained under
`.local/minneapolis-cyclists-v01/published-review/`; earlier previews and rejected
placement attempts remain retained alongside them.

```sh
.local/minnesota-review-env/bin/python \
  planning/minnesota/tools/review_minneapolis_cyclists.py \
  content-packs/minnesota-trainer-cast-33-v01.crystalpack \
  content-packs/minnesota-minneapolis-cyclists-v01.crystalpack NEW_REVIEW_DIRECTORY
```

Focused Rust formatting passes. Whole-workspace formatting reports unrelated
existing differences. Required asset Clippy stops in `crystal-core` at the
existing denied `clippy::bad_bit_mask` in `multiplayer/mod.rs:1332`, before
checking the changed asset code.


## Runtime checks

Two production controller regressions pass against the published pack: riders
actually move while staying on their authored perimeter without a sight battle,
and declining the Yes/No prompt avoids a battle. The headless native map capture
is retained at `.local/minneapolis-cyclists-v01/native-review/cyclists-native.png`.

The three battle fixtures (riders 01, 13 and 18) also pass through the public
`VisibleShellController`: talk, accept Yes, enter the native battle, save/load
that battle, and execute a selected attack with actual PP consumption. A small
standalone Rust proof links the existing production libraries to avoid rebuilding
the entire test harness again. Its source, binary and logs remain under the
ignored review directory. Initial party, position and facing are explicit setup;
all conversation and battle input uses the production controller.

The first combined test run had two passing tests and one test-assumption
failure before battle input: it incorrectly required parent level scaling to be
enabled. The regression now derives the expected level from the parent's setting;
the three public-controller battle fixtures verified the actual unscaled pack.
Neither global level scaling nor unrelated trainer balance was changed.

Both requested all-target Clippy commands (assets and controller) stop at the
pre-existing core bad-bit-mask error. No full lint pass is claimed.

The final standalone No check also confirms that the pending prompt resolves,
text closes, no battle opens, and the player's position stays unchanged.
`yes-no-final-proof.log` records all four passing public-controller fixtures.
