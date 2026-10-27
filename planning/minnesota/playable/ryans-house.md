# Ryan's House on the Baptism River

The original cabin publication is `content-packs/minnesota-northstar-ryans-house.crystalpack`:
510 maps,251 species,251 moves,377 audio programs. SHA-256:
`c0c15e1193c816bcf6e5272af5c3dc716deed588d5b0d2070c01a59e723e2dc5`.
Its exact input is the509-map drainage/ICE candidate documented in
[duluth-links.md](duluth-links.md). The story, both Duluth links and expanded
Northwoods are retained. Generated packs and all render outputs remain ignored.

The integrated 515-map story successor retains this entire house/river module.
Its native cabin case was replayed successfully on that exact pack; see
[the current combined build and matching waterfall save](minneapolis.md).

## Place and physical contract

The small native wooden cabin is on the east bank beside the existing three-block
Baptism cascade in `MnR11C00R00`. Its full facade uses original Johto blocks
2c/2d above2e/2f, at metatile origin `(11,9)`. A short lawn porch joins the
existing waterfall viewing shelf and winding walking branch. It is an optional
home with no badge, field-move, Safari fee or story requirement.

The actual DOOR is `(23,21)`; its safe FLOOR return is `(23,22)`. Exterior warp1
enters `MnRyansHouse` at passive FLOOR record3 `(5,6)`. Exterior record2 is the
nontriggering return destination. The5×4 traditional-house interior has a wood
floor, complete table/seating, cupboards, shelves and a wall picture. Ryan stands
at `(6,3)`, talks through native pages and welcomes the player to the river.
Both actual carpet exits `(5,7)` and `(6,7)` return to the same safe porch.
The readable sign is at `(26,22)`, approached from `(26,23)`.

Only seven original metatiles change: four facade pieces, two porch lawn blocks
and the sign. Every river/cascade tile, water edge, existing actor, regional
connection, story command and original room stays exact. No item, event,
badge, money or healing reward is attached to the house. Native map identity is
North Shore group41/map10; lighthouse4 and all eight exterior IDs remain stable.
HOME, Fly and blackout destinations are preserved; no house recovery is added.

The canonical dossier/layout, `playable/ryans-house.json`, `interiors.json` and
world route reference agree. The previous native terrain/dossier are the frozen
`revisions/northwoods-expansion/after/MN_R11.*.json` files. New publication must
compose the cabin after the expansion and Duluth connections, rather than
claiming the older northern expansion alone includes it.

## Evidence

- The full-route source check passes walking endpoints, door/porch reachability,
  water boundaries and ledges. The plan now contains158 planned maps.
- The external-pack preservation case passes: exactly the seven declared bank
  blocks and additional sign/warp records, plus the one new room and native
  registration, change. Every other original data field, all art and all audio
  remain exact. Split map payloads reconstruct both actual modules. Duplicate
  publication and an active-carpet arrival substitution are rejected.
- The production-controller case passes: real door entry, Ryan's dialogue with movement ownership, sign reading, both carpet exits, indoor/outdoor Continue and unchanged semantic event/engine flags, badges, party, bag and money. Map transitions may materialize false temporary flag entries; logical flag values are checked across both key sets.
- Actual stdio MCP passes the door, Ryan dialogue, both real carpet exits and START on the written510-map pack; stderr remains byte-empty.
- Whole current North Shore, cabin detail, complete room and160×144 approach
  views were rendered and inspected. Actual headless noon/down-facing2D images
  were inspected too. Source/native porch comparison is0/1,024,000 pixels;
  the complete interior is0/327,680. The latter uses the renderer's indoor
  palette and full-room4× framing; initial day-palette/3× assumptions were
  corrected in the reference tool, with every earlier image retained.
- All377 PCM hashes, frame counts and loop ranges pass on the written pack.
  Scoped Rust formatting passes. Workspace formatting and Clippy retain the
  unrelated blockers documented in `duluth-links.md`; no broad green claim.

All original images, conversions, crops and rejected comparisons remain under
`renders/ryans-house/`. These are local visual checks; user visual approval is
still open. Earlier story milestones in the review save are explicit setup,
not an earned new-game journey.

## Executed build and publication

```sh
CARGO_TARGET_DIR=target/northstar-story cargo build --profile wasm-dev --locked \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  -p crystal-assets --bin pack_ryans_house
target/northstar-story/wasm-dev/pack_ryans_house \
  content-packs/minnesota-northstar-drainage-ice.crystalpack \
  planning/minnesota/revisions/northwoods-expansion/after/MN_R11.layout.json \
  planning/minnesota/routes/MN_R11.layout.json \
  planning/minnesota/revisions/northwoods-expansion/after/MN_R11.json \
  planning/minnesota/routes/MN_R11.json \
  planning/minnesota/playable/ryans-house.json \
  content-packs/minnesota-northstar-ryans-house.crystalpack
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_ryans_house.py \
  --pack content-packs/minnesota-northstar-ryans-house.crystalpack --verify-published
target/northstar-story/wasm-dev/verify_crystalpack \
  content-packs/minnesota-northstar-ryans-house.crystalpack --audio --desktop
```

The asset case is `cargo test --locked -p crystal-assets --test ryans_house -- --ignored`.
Controller build/run uses the same pinned toolchain/profile and two package
optimization settings as `duluth-links.md`, filter `ryans_house_controller_`,
`RUST_MIN_STACK=33554432`, `CRYSTAL_DULUTH_STORY_TEST_PACK` set to the absolute
510-map pack, and `CRYSTAL_DULUTH_STORY_SAVE_DIR` set to
`$PWD/.local/ryans-house-review-saves`. The shared fixture helper supplies an
initial legal partner and position, with all eight badges false. Every action
thereafter uses production input and Continue.

A retained production save starts on the waterfall approach:

```sh
target/northstar-story/wasm-dev/geothite play \
  content-packs/minnesota-northstar-ryans-house.crystalpack \
  --load .local/ryans-house-review-saves/ms10-ryans-house-porch-15703-1793067017364132000.crystalsave
```

Walk up to enter. `mcp` in place of `play` starts the checked production stdio
client. The same directory retains the actually visited interior save. Every
render remains available, including the first mismatched reference generations.
