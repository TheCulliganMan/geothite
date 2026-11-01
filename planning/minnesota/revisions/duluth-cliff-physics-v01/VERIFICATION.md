# Duluth cliff repair verification

The selected local pack is `minnesota-current-duluth-cliff-stairs-v03`, composed
onto State Fair buildings v01. Its SHA256 is
`6878059cebf9826b319b840bbcb2a8f243a73556c68d00cc0d135ad4a597265f`;
content identity is
`0ee91fd21eaff19a2094f0214a6d0245751836e08c7c33a06a09a543eb687b56`.

All evidence is retained under `renders/duluth-cliff-physics-v01/`. Local party
and position fixtures finish before input. Gameplay uses the production
`VisibleShellController`, including the text client's ordinary press path.

| Check | Result |
| --- | --- |
| Current-parent eleven stair flights, both directions, bag/party preservation, Save/Continue | Pass, 61.53 seconds |
| Fresh ten-lane ledge unit replay, downward hop, blocked reverse, walked stair return | Pass, 50.27 seconds on reviewed v02 |
| Every real door and both room carpets, passive arrivals and retained southern ledge loop | 38 doors, 76 room returns pass, 143.03 seconds on reviewed v02 |
| Production text client: supplies, garden stairs, sign-adjacent ledge and walked return | 215 calls pass on reviewed v02, empty stderr |
| Fresh current-parent text client: continued stair state, ascent/descent, production Start menu | 32 calls pass, empty stderr, input save unchanged |
| Exact preservation against selected Fair parent | 29 changed Duluth metatiles; all 729 other maps and entire normalized residual pack exact |
| Current versus reviewed composition | Complete Duluth module, tileset, art and all non-Fair maps/art exact |
| Native art | Existing 140 metatiles pixel exact; appended `8c` exactly reproduces cave36 pixels and collision |
| Native camera comparison | Five inspected 640×576 images, zero differing pixels against native source compositor |
| New current-parent native image and whole town | Pixel identical to reviewed composition |
| Audio/Desktop pack verification | All 377 PCM hashes, frame counts and loop ranges pass |
| Active Minnesota roster | All three exterior trainer teams, local fishing/gift/wild species and their evolution targets pass current roster/retirement closure; visible actors are human/item sprites |
| `cargo clippy --locked -p crystal-assets --all-targets` | Pass, existing warnings; no new Duluth-module diagnostics |
| `cargo clippy --locked -p crystal-bevy --all-targets` | Pass, existing warnings; no new Duluth-module diagnostics |
| Scoped Rust formatting | Pass |
| `cargo fmt --all -- --check` | Existing workspace differences, starting in `offset-allocator/src/lib.rs:408`; unrelated files were not reformatted |

The v02 gameplay evidence carries onto v03 through an independent exact
module/art equivalence check, rather than relabeling old saves or old run logs.
The latest-parent stair replay, continued text-client journey and native image
were also executed directly on v03.

The eleven flights and ten clear ledge lanes have ordinary approaches/landings
with actor bodies included. Ordinary uphill ledge returns are 11–27 walking
steps. All 38 passive exterior returns are reachable without hopping. Both
regional connection offsets and outdoor spawn513 at (41,32) remain exact.

Important evidence files:

- `current-fair-parent-preservation-v04.json`: full residual preservation,
  negative mutations and exact equivalence to reviewed v02.
- `current-source-and-executables-v03.json`: source and binary bindings taken
  before the current-parent replay.
- `current-stdio-supply-v03.json` and `stdio-*-v01.json`: actual RPC streams,
  errors, stderr, save hashes and destinations.
- `pixel-comparison-v01/receipt.json`: all five complete source comparisons.
- `current-pack-native-equivalence-v03.json`: current pack identity and new
  native capture comparison.
- `fast-preview-loop-benchmark-v01.json`: retained before/after source previews,
  warm timings, complete native terrain equality and production actor origins.
- `final-receipt-v03.json`: final artifact inventory and selected revision.

The older v01 controller receipt's whole-test-file hash was collected after the
ten-lane source extension. Its named nine-lane log remains earlier coverage;
the fresh Cargo ten-lane replay supplies the complete current-source result.
The original logs and receipt remain retained rather than silently rewritten.

The first source pass closed the garden's rear gap and disconnected northern
streets; the accepted source uses a real stair there. The first publisher run
correctly refused a missing outdoor-spawn inventory. The first published pack
had a seven-step eastern ledge return; the accepted source moves that drop one
metatile farther from the stair. The next audit initially undercounted the sign
block's clear lower quadrant; all ten lanes are now verified with real input.

This verifies the requested cliff repair and its integration. Broader city
composition, story and earned statewide progression remain separate reviews.
