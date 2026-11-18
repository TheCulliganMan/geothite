# Private Minnesota field review

The verified v2 candidate includes the rewritten first-visit and return
conversations about Pendulop, Birchit, Lakernaut and Loobit. The first candidate
is retained as superseded review history after its town prose was rejected.

This composes five separately owned revisions on the verified raw Lakeside pack,
`content-packs/minnesota-lakeside-v01.crystalpack`, SHA256
`e56ded3d55b8364e8756a4a02eb530fb74f52c03951c1c01d962441f7e2193d9`.
The result is a review product, not a selected pack or a public deployment.
The current Mothlight local selection and the Lakeside rollout remain separate.

The leaves are `northstar_river_terrace`, `northstar_mine_return`,
`minnesota_hidden_route_clues`, `northstar_route_challenges`, and
`northstar_town_responses`. Their focused source directories retain the design
comparisons, native renders and individual checks. The combined builder calls
their final installers without duplicating their authoring data.

## Private save identity

The candidate appends `northstar-field-review-v2` (the superseded first
candidate used `northstar-field-review-v1`). The full runtime modpack
ID therefore differs from Lakeside and Mothlight. The candidate removes
`data/northstar-save-compatibility.json` instead of claiming that existing
public saves are safe to migrate. No save files are opened or rewritten.
Fresh saves belong to this private review identity. This change is not a new
public compatibility schema.

## Future public migration review

Two maps need explicit saved-terrain and actor reconciliation before any future
release accepts old saves:

- `MnR03C00R00`: eight metatile changes add a supply shelf and a downward return.
  The existing item ball moves while keeping its original collection flag.
  Reconcile old geometry and its cached object position, retain collected-item
  state, and check safe player positions around both ledge lanes and landings.
- `MnR09Cave1F`: nine metatile changes open a lower drift beneath the east bench.
  Reconcile cached geometry and ensure in-room player positions remain safe.
  Keep the existing drill-access warp collisions, quest state, actors and pickups.

The five clue stamps retain their original collision quadrants; their rewards,
positions and receipt flags do not move. They still need their updated visual
blocks applied when a future release refreshes cached map geometry. Trainer and
conversation changes must retain existing battle and story flags. This review
does not authorize or implement public migration.

## Reproduction

Use the shared existing Cargo cache at one job after coordination with other
owners. Do not run a game session or deploy anything.

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build --locked -p crystal-assets \
  --bin pack_northstar_field_review --bin mine_return_geometry
target/debug/pack_northstar_field_review \
  content-packs/minnesota-lakeside-v01.crystalpack \
  content-packs/minnesota-field-review-v02.crystalpack \
  target/field-review-20261118/manifest-v02.json
target/debug/mine_return_geometry content-packs/minnesota-lakeside-v01.crystalpack \
  content-packs/minnesota-field-review-v02.crystalpack
```

The CLI requires fresh ignored outputs, verifies the exact parent hash, applies
all five preimage-guarded installers, checks composition ownership, derives the
private identity, verifies runtime structure and strictly reads back the pack.
Its generated manifest enumerates actual changed maps, fields, metatile counts,
runtime assets and save identity. It remains ignored with the generated pack.
All source work stays in the repository; no disassembly or extracted art catalog
is added. The final v2 candidate passed strict readback and ownership checks. Its SHA256 is
`c602c1d7be1594ffd7112a1752f265a80c3631b7bd4e8f1c74ffdd1e6c5b58ae`, with content
identity `f2de249613f5987b7722a959391a0d7f8a03953ca3ecaffd1eff80bce2b42250`.
All four focused suites passed on both Crossings and Lakeside: terrace (1),
clues (1), trainer checks (2), and town replies (1). The exact tests were listed
before execution; earlier zero-test results were rejected.

The combined native mine graph retains 136 reachable positions with safe return,
24→15 actions from Nell to the mouth, both downward drops and blocked reverses.
Its desktop/audio verifier passed all 602 PCM hashes, frame counts and loop
ranges. The combined town renderer checked all 32 installed pages. No gameplay
session, public deployment or player-save mutation was performed.

The generated manifest, logs and native storyboards remain under
`target/field-review-20261118/`. The selected local pack remains Mothlight v10.
The two saved-terrain reviews above remain prerequisites for future public
migration, not completed migration work.

## Actual combined map manifest

All 743 maps remain; 11 change and the other 732 are byte-for-byte equal as
compiled map records. Exactly 22 metatiles change across seven maps.

| Map | Changed metatiles | Purpose |
| --- | ---: | --- |
| `MnDuluth` | 0 | First-visit and return conversations |
| `MnR03C00R00` | 8 | Supply shelf, moved original Ether, return drop and sign |
| `MnR07C00R01` | 1 | One native visual clue for an unchanged hidden reward |
| `MnR08C00R01` | 1 | One native visual clue for an unchanged hidden reward |
| `MnR09C01R00` | 1 | One native visual clue for an unchanged hidden reward |
| `MnR09Cave1F` | 9 | Lower mine drift and return drop |
| `MnR10C01R00` | 1 | One native visual clue for an unchanged hidden reward |
| `MnR11C00R02` | 1 | One native visual clue for an unchanged hidden reward |
| `MnR16C00R00` | 0 | Existing trainer dialogue and revised parties |
| `MnR16C01R00` | 0 | Existing trainer dialogue and revised parties |
| `MnStCloud` | 0 | First-visit and return conversations |

Only map blocks, scripts, their compiled mirrors, the three owned trainer records,
and clue tile banks change in game data. Map dimensions, connections, warp and
background event records, encounters, species, moves, audio, reward definitions
and story flags remain unchanged. Original clue collision quadrants are retained.
Runtime-file changes are restricted to the clue art and removal of the public
save-policy file. The report gains only the private review marker.

## Final source checks

`cargo fmt --all -- --check` and scoped diff checks passed. Current-source
`cargo clippy --locked -j 1 -p crystal-assets --all-targets` passed after the final town rewrite. The trainer test-only helper was moved
above its test module before the final check.
There are no diagnostics in the five new leaf files or the combined builder.
Existing unrelated warnings remain, as documented in
`target/route-challenges-v01/lint-receipt.md`; the final combined check is
`target/field-review-20261118/v02-clippy.log`.

All 37 combined town storyboard PNGs are pixel-identical to the individually
accepted installed renders (PNG file encoding differs). Their receipt binds the
combined pack hash. No scene, timing or earned-progression claim is made.
