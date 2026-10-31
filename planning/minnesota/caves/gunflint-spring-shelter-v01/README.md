# Gunflint spring shelter

The complete 16×12 cave is rebuilt around a central rock fin with two dry paths,
a contained spring and separate climbable alcoves. Otta's camp now offers a
working free rest. The previous broad aisle, rectangular pool and cliff-adjacent
drop geometry are replaced by continuous native rock boundaries and ordinary
stair returns. This is an optional quiet shelter, with no new battle or HM gate.

`before.json` is the complete installed-room preimage. `after.json` is the
hand-authored terrain and rest dialogue. `contract.json` binds the original
native module, exterior warp and both actor anchors. `plan.json` records the
routes, service, cache and exact core/metatile coordinates. Native cave graphics,
night palette, music and audio programs are retained without copying assets.

- The mouth stays at `(17,23)`, with clear arrival `(17,22)`. Its original
  exterior pair is `MnR12C00R00 (40,15)`; native settling reaches safe floor
  `(40,16)`. The MCP move course also verifies the intervening raw arrival.
- The western and eastern dry passages join `(13,18)` to `(13,10)` around the
  fin. Both work without Surf. The spring cannot reach an unconnected map edge.
- The ordinary stair is native tile 30 at metatile `(3,4)`, walking column 6.
  The right-corner stair is native 36 at `(13,4)`, walking column 26. Both support
  climbing and descending; the corner's eastern wall blocks side entry.
- Otta remains at `(5,5)`, approached from `(6,5)` facing Left. His original
  conversation is retained, followed by an optional rest offer. No leaves HP,
  status and PP unchanged. Yes waits for acknowledgement before native
  `special HealParty` restores every partner, including fainted partners.
  The service remains free and repeatable without changing the blackout point.
- The original Max Ether remains at `(27,5)`, approached from `(26,5)`, with
  original once-only flag `EVENT_MN_R12_CAVE_ETHER`. Resting does not grant it.

The candidate is `content-packs/minnesota-gunflint-spring-shelter-v02.crystalpack`.
Selection is controlled by `planning/minnesota/world.json`; candidate publication
alone does not select the pack. The Rust publisher was built with
`CARGO_TARGET_DIR=target/verdigrim-scene CARGO_BUILD_JOBS=2 RUST_MIN_STACK=67108864
cargo build --locked -p crystal-assets --bin rebuild_gunflint_spring_shelter`
under the shared Cargo slot. The executed publication is recorded in
`renders/gunflint-spring-v631/publication-v637.json`, using the verified current
parent, full before/after sources, contract and a fresh ignored output. Pack
writing and strict readback use Rust. Do not overwrite historical outputs.

For fast source review:

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_cave_source.py \
  --pack content-packs/minnesota-iron-drill-core-v04.crystalpack \
  --source planning/minnesota/caves/gunflint-spring-shelter-v01/after.json \
  --output planning/minnesota/renders/gunflint-spring-next-source --watch
```

Use a fresh output directory. The measured warm source revision took 0.014 seconds.
The complete preview and all eight native camera views were visually inspected.
Every native frame matches the source at all 368,640 pixels. All earlier renders,
rejected passes, native originals and acted saves are retained.

`audit.py --pack PACK --output FRESH_DIRECTORY` checks the entire exact parent,
both dry branches, body-aware reachability, native pixels/audio preservation,
original cache/actors/warp/text, new rest programs and active Minnesota fishing
species, including rod time-group references. `check_mcp.py --help` gives the
independent production input course; its verifier reads native identity-bound
saves without changing them.

The rest controller case starts with explicit local setup: original Minnesota
Cindore/Fortigard at level 30, one poisoned at 1 HP, one fainted, all PP zero, no
Surf, badges or money. It proves refusal, pending acknowledgement, full native
party restoration and Continue. The separate course starts outside the actual
compact cave mouth and plays both dry paths, climbs, the cache and return.
Neither setup proves earned regional arrival or intended-level balance.

The independent MCP course passes 400 RPC calls from the injured local camp
save, using only production input after initial setup. It proves refusal/retry,
healing after acknowledgement, both climbs, the cache, actual exit and re-entry,
fresh-process Continue and the production Start menu. Native save reads prove
HP, PP, status, money, walking mode and cache quantity/flag; stderr stays empty.

[Whole before/after](../../renders/gunflint-spring-v631/audit-v648/whole-before-after.png),
[eight native player views](../../renders/gunflint-spring-v631/native-v638/MnR12Cave1F-production-cameras.png),
[actual spring-bank view](../../renders/gunflint-spring-v631/acted-v646/actual-spring-bank.png),
[production MCP record](../../renders/gunflint-spring-v631/mcp-v642/receipt.json).

Both controller cases pass in the default test profile; the final run is
`renders/gunflint-spring-v631/controller-v653.log`, with frozen binary and owned
source hashes in `controller-binary-v651.json`. Both affected crates pass
all-target Clippy with existing warnings, including a final check of the
corrected controller case. Changed Rust leaves pass formatting. Workspace
formatting still reports existing differences beginning at
`crates/offset-allocator/src/lib.rs:408`.

The accepted current source is `current.json`. The acceptance record binds the
completed checks and shared entrance reservation; the source index and selected
revision are in `planning/minnesota/world.json`. A later source-only parent
revision uses the exact same parent pack and is preserved during integration.
Broader regional map review remains active.
