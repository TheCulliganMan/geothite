# East bench return drift

This terrain-only candidate improves `MnR09Cave1F`, the mine entrance near Nell.
The eastern working previously required retracing the same stair and central
bench. Nine metatiles now form a lower drift under its southern wall. A native
south-facing drop joins that drift to the upper bench; the old stairs remain
the ordinary uphill return. Continuous bedrock separates the levels. There are
no decorative loose rocks, new field-move gates, rewards or actors.

## Comparison and intent

The actual original Union Cave 1F in the selected pack was rendered alongside
Crystal Legacy's local `maps/UnionCave1F.blk`, revision
`77e6b3859349698ad0a7a8a93cd27f78860d3e4a`. Their terrain layouts are identical:
Legacy retains the mainline long through-route, rock-enclosed bends, side
stairs and water-separated areas. That is a useful restraint, rather than
evidence that every mod enlarges its dungeons. The primary
[mainline event source](https://raw.githubusercontent.com/pret/pokecrystal/master/maps/UnionCave1F.asm)
places the Route 32 and Route 33 entrances at opposite ends, with two optional
basement stairs. Minnesota's mine is smaller and has a different story role;
its missing piece is a distinct return trip from the eastern working, not more
rooms or copied mainline encounters.

Both reference layouts were rendered with the supplied pack's native cave
art. This comparison establishes their terrain, not Legacy's exact in-game
palette or sprite presentation. Reference images and all candidate iterations
are retained under `target/cave-pass-20261118/`.

## Geometry contract

- Map size remains 12×11 metatiles, with forced native night palette.
- South mouth `(13,19)`, clear arrival `(13,18)`, and its exterior pairing stay
  exact. Both existing drill-approach warp records also remain exact, including
  the intentionally blocked `(19,11)` cell. The lower drift never opens it.
- Existing Nell `(19,7)`, foreman `(9,5)`, cache `(7,5)` and hidden pickup
  `(11,16)` retain their locations, scripts, rewards and flags.
- The new lip is `(16,12)` / `(17,12)`, crossing the wall course at y13 to
  `(16,14)` / `(17,14)`. The wall course closes both flanks at x15 and x18.
- The lower drift bends west across y16 to meet the original entrance passage.
  All three existing stairs remain; uphill access uses the original route.
- No Surf, Strength, Rock Smash, Cut or Flash dependency is introduced. Existing
  story permission checks and the active Minnesota roster are untouched.

## Build and review

The integration owner registers `northstar_mine_return` in `pack_authoring.rs`.
For a combined pack call `northstar_mine_return::install` before its final
identity/save-migration step. It changes only the cached map blocks and their
encoded payload. The preimage guard includes the complete existing terrain.

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build --locked -p crystal-assets \
  --bin pack_mine_return --bin mine_return_geometry
target/debug/pack_mine_return content-packs/minnesota-crossings-v01.crystalpack \
  content-packs/minnesota-mine-return-v01.crystalpack
target/debug/mine_return_geometry content-packs/minnesota-crossings-v01.crystalpack \
  content-packs/minnesota-mine-return-v01.crystalpack
.local/minnesota-review-env/bin/python planning/minnesota/caves/r09-return-drift-v01/review.py \
  content-packs/minnesota-crossings-v01.crystalpack \
  content-packs/minnesota-mine-return-v01.crystalpack target/cave-pass-20261118/verified-v02
```

The candidate builder and strict readback passed. Candidate SHA256:
`6fd91357294064cc3b25be5ec43dc653acab692779ff6e83384b3938e65f4e8a`.
The compiled movement graph measures 24→15 actions from Nell’s interaction
approach `(19,8)` to the clear mouth arrival `(13,18)`, with 136 reachable
positions with a return to the entrance, both successful downward drops and
blocked reverse movement. No field move is enabled. All three existing warp
positions, collisions and reciprocal destinations remain exact; the perimeter,
upper rim and both drop flanks are closed.

The complete pack comparison allows only nine changed terrain blocks, their
encoded map payload and derived identity. All 742 other maps and all events,
programs, actors, parties, encounters, species, art, audio and runtime files are
exact. The whole-room comparison and three native-size camera pairs were
visually inspected in `target/cave-pass-20261118/verified-v02/`. The mainline and
Legacy reference renders have identical SHA256 hashes. Workspace formatting
passes. The shared release owner owns all-target Clippy; unrelated existing
library dead-code warnings were present during the targeted build.

The graph checker uses compiled native movement primitives with stationary actors and
warp apertures as terminal destinations. It does not create a game session or
exercise story scripts. Native-size renders are read-only compositions of the
compiled terrain and original sprite art, not gameplay captures. The combined
release owner must include the changed room in saved-terrain migration. This
candidate is not selected or deployed.
