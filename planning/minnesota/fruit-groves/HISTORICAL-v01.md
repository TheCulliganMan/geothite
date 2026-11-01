Historical superseded lineage. Do not build or play these packs.

# Minnesota berry and apricorn stops

The combined delivery is `content-packs/minnesota-fruit-and-ice-v03.crystalpack`.
The [frozen-water runbook](../regions/ice/FROZEN-WATER.md) records its final overlay.

`groves.json` is the explicit placement contract: 228 native fruit-tree actors
on 93 Minnesota outdoor map chunks. There are 148 berry trees and 80 apricorn
trees. Each Mississippi wooded reach has ten berry stops, one at each named
bank. The accepted twenty-trainer routes, grass, supplies and field care are
carried forward onto the newer 713-map shore-gate parent.

All nine native berry items and seven apricorn colors are represented. Trees
use the original `SPRITE_FRUIT_TREE` art and `fruittree` script, including the
native obtained-item, empty-tree and full-pack messages. Each tree has its own
saved harvest flag. The production daily reset clears fruit harvest flags;
permanent item pickups and story flags remain saved.

Trees occupy existing Floor beside woodland and grass. Their eight neighboring
walking tiles remain clear; entrances, coordinate events, signs and seams keep
passing room. Review checks approaches from existing paths, warps, gates or
Surf shores, and rechecks all river activities with the tree bodies present.
Narrow or enclosed chunks without a safe site are listed in `skipped`; nearby
chunks of those routes still have harvests. No terrain was expanded to meet a
tree quota.

The authoring helper selects recorded sites deterministically from real native
collision. It writes source JSON only. The Rust publisher owns pack mutations,
preimage and collision validation, native script lowering, metadata and strict
written-pack verification. Carried-forward river teams and encounters use the
active original Minnesota roster. Unrelated encounters and teams are preserved.

Executed publication command (the output must be fresh):

```sh
target/river-fun/wasm-dev/pack_minnesota_fruit_groves \
  content-packs/minnesota-shore-gate-v03.crystalpack \
  planning/minnesota/fruit-groves/groves.json \
  content-packs/minnesota-fruit-groves-v02.crystalpack
```

Build the publisher with the pinned toolchain:

```sh
CARGO_TARGET_DIR=target/river-fun cargo build --locked --profile wasm-dev \
  -p crystal-assets --bin pack_minnesota_fruit_groves \
  --config profile.wasm-dev.package.crystal-core.opt-level=0 \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0
```

Read-only written-pack review and whole-map source-art renders:

```sh
.local/minnesota-review-env/bin/python \
  planning/minnesota/tools/review_fruit_groves.py \
  --output planning/minnesota/renders/fruit-groves-v01/source05
```

Render folders are immutable epochs under the ignored
`planning/minnesota/renders/fruit-groves-v01/`. Earlier 231-tree layouts remain
there; the final contract removes enclosed-pocket placements. Packs and native
fixture saves remain ignored. Native fixtures explicitly provide initial
locations and a Minnesota partner; they do not establish earned regional
progression.

Written-pack identity: `600b948606613c7189f121dc993087b430876938a00e4c99bc3badb496503ca3`.
SHA-256: `b872cb36064000e58421008f52490ebf4ef7b98f22f32bc9c96f319b718b2a75`.
The `source05/proof.json` receipt confirms all 228 clear walking rings, route
access, both twenty-trainer reaches, original actors/events, other map payloads,
art and audio. The written river teams and wild encounters pass the active
Minnesota roster check. Both whole-route renders and twelve regional examples
were visually inspected.

The focused daily-harvest regression and all 23 native field-item unit tests
pass. A production stdio-MCP session on the written pack passes movement, Start
menu, the Winona home warp/Mom dialogue and byte-empty stderr. Workspace-wide
formatting already fails in unrelated source; the three new Rust leaf files
pass their scoped formatting check. The broad core test invocation is blocked
by existing missing radio fixtures and `save_sharing` lacking its test-fixture
feature; the focused core unit binary passes.
