# Rocky Duluth shoreline and open route fences

This revision adds38 original native boulders along Duluth's grassy shore,
removes the requested route fences, and removes19 buoy metatiles already backed
by Route16's solid tree boundary. Exposed outer water buoys remain intact.
The two floating fisherman decks, their boarding faces, and the open inner harbor
are preserved. Rocks do not disconnect any existing reachable walking terrain.

The exact parent is `r10_fitted_fork_revision`, SHA256
`1f16d2b3ca473cc5e937140b7ba6f3e190bcaf3718dd6854b40acd8e98a6ddff`. The newer fitted R10 cave and both North Shore
cafes stay exact. `sources.json` inherits that parent's authoring index and
replaces only the six current exterior layout references and their route dossiers.
The frozen dossiers use installed Minnesota trainer teams and encounters;
parent review prose is historical context, not a new gameplay claim.

Route10 loses17 rail cells. Route12 already contains no fences and remains
byte-for-byte unchanged. Route15 loses82 post/rail cells, using plain signboards
at its three existing sign anchors and a tree at one structural woodland edge.
Route16 revises94 fence cells: structural bridge/field rails become native earth
banks while other aprons open. Its native hayfield hop, western return and
trainer/grass choice remain valid. Route17 loses10 wooden rail cells while its
Cut tree, side boulders, ice and lower lake remain exact.

Routes15 and16 explicitly freeze their installed shore fitting before the edits;
automatic fitting otherwise puts fence posts back into the water. The route
parser now accepts explicit native source edges for these two routes. No source
pixels, collision definitions or tileset files change. All260 changed metatiles
are enumerated by the layouts; all725 other complete maps and every actor,
script, room, encounter, roster entry, trainer and audio program remain exact.

## Executed Rust publication

```sh
CARGO_TARGET_DIR=.local/route-grass-native-target CARGO_INCREMENTAL=0 \
CARGO_BUILD_JOBS=1 RUST_MIN_STACK=67108864 \
  cargo build --offline --profile wasm-dev --locked \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0 \
  -p crystal-assets --bin revise_shoreline_and_fences
.local/route-grass-native-target/wasm-dev/revise_shoreline_and_fences \
  planning/minnesota/revisions/shore-rocks-open-routes-v01/plan.json \
  content-packs/minnesota-current-shore-rocks-open-routes-v02.crystalpack
```

The publisher requires a fresh ignored output path, checks the exact parent SHA,
checks complete source terrain and current native anchors, validates each route's
geometry and compact geography, and strictly reads back the written pack.
Use another fresh ignored output name for a replay. The verified output is SHA256
`7af6ebc795b0280a7d74626315da4331e42b50dcb62090e48a29d5d211ee48ff`, content identity
`1bcddce2f787d6af538f6e28426f48659ed7a3f4286e308ce3104fda44ed5847`. A second execution using the final sources
reproduced it byte for byte.

## Verification

The external Rust shore regression passes native art/event/water preservation,
source-art drift rejection and repeat rejection. The source is
`crates/crystal-assets/tests/duluth_harbor.rs`; the exact completed-library Rust
test link and execution prerequisites are retained in the ignored verification
folder. All five route revisions pass full source/native geometry checks.

Production MCP inputs cross the opened fences on Routes10,15,16 and17, verify
shore-rock collision and Surf access, and cross the removed buoy row before the
existing trees stop further northward Surf. Both floating decks still board,
talk, launch Surf, deny the exposed southern buoy edge, return and Continue.
All154 calls have empty stderr. These start from explicit
local QA fixtures; they do not claim earned regional progression.

Twelve headless native before/after images were reviewed. Six matched-position
comparisons have zero changed pixels outside the contracted metatiles. Image
alignment is checked against unchanged native terrain; the buoy view's settled
camera is24px higher. Full map previews and every saved input/capture receipt
remain under `planning/minnesota/renders/shore-rocks-open-routes-v01`.

```sh
target/northstar-story/debug/verify_crystalpack \
  content-packs/minnesota-current-shore-rocks-open-routes-v02.crystalpack \
  --audio --desktop
```

All377 PCM hashes, frame counts and loop ranges pass. The affected crate's
all-targets Clippy passes with unrelated warnings, and changed Rust formatting
passes. Whole-workspace formatting still reports unrelated differences. Packs,
images, saves, logs and generated receipts remain ignored; no web deployment
is included. `world.json` owns the current local selection.
