# Route 1 current grass, stairs and local directions

Selected revision: `route_grass_r01_stair_guidance_revision`.
Pack: `content-packs/minnesota-current-r01-stair-guidance-v01.crystalpack`.
SHA256: `e6af050b697b6a9d52735713d5eabe0c069110f69ecdc5b81707e181b7ee30b2`.

The current exterior has 136 grass walking quadrants, an unavoidable four-step
Winona-side grass crossing in either direction, and ordinary cave12 bluff stairs.
The former tall-cliff hop is closed. The old sign, walker, farmer and trainer
messages still promised that removed drop; five existing text programs now
explain the east stairs. Page boundaries, the farmer's Berry/delivery guidance,
battle roots, flags, rewards and input ownership remain intact.

Only global/local text programs and their parsed text caches change in this
child. All 732 maps retain identical geometry, collision, actors, events, teams
and encounters. All runtime art, audio and other content remain exact. Brainerd's
roof repair, Bemidji's stairs, accepted State Fair buildings, warm North Shore and
Minnesota's native-2D policy are preserved.

## Verification

- The whole current source route and three native views were actually inspected:
  western grass, grass/stair approach and the eastern through-path. All three
  source/native comparisons have zero pixel mismatches. The later text-only
  child preserves their complete terrain/art inputs exactly.
- A real starter/town checkpoint was followed with 1,651 production stdio calls:
  both route directions, ten native wild encounters and RUN, ordinary stair
  ascent/descent, old high-cliff denial, actual reached-save Continue, Items and
  Balls pockets. Source and input saves stayed unchanged; stderr was empty.
- The child passed a fresh native New Game starter decline/retry, gift/repeat and
  Continue case using the retained primary-checkout controller v27. The child
  checkpoint was played forward normally, without retagging or editing saves.
- The child guidance review passed 251 stdio calls and four acknowledged reads:
  farmer before/after Continue, walker and sign. Movement is blocked while text
  owns input. Source/input saves stay unchanged and stderr is empty. Trainer
  replacements are verified in the global/local native text mirrors; no trainer
  defeat was fabricated for this text repair.
- Guarded publisher build, affected-crate all-target Clippy and owned Rust
  formatting pass. Existing unrelated compiler warnings and workspace-wide
  formatting differences remain recorded in the logs.
- `post-selection-proof-v01.json` independently verifies the selected pack hash,
  source index, all 640 layout metatiles, five global/local text mirrors and
  preservation of every parent source binding.

Retained failed review attempts are diagnostic evidence, not passing cases.
The older controller binary's route planner indexed the original Johto bank and
failed on the appended stair block after playing the opening. The first stdio
harness expected `battle` instead of the actual `wildbattle` status. The first
child-text harness assumed farmhouse arrival (4,7); actual arrival is the carpet
at (4,9). Its reached save was continued through native inputs for the corrected
review. No game state was patched to satisfy these harness assumptions.

## Current source

`current-native-v02.layout.json` materializes the installed exterior.
`current-exterior.contract-v01.json` is explicitly an installed exterior review
contract: required grass, actual stairs, closed cliff and retired historical
jump contracts. The full base-phase route builder inputs remain in their
original source paths. Their old one-way geometry is historical for this stage;
the selected current index points to the scoped current contract and layout.
Native unchanged actors, rooms and encounter tables remain authoritative in the
bound pack. `guidance-before-v01.json` and `guidance-after-v01.json` are the five
exact program preimages/replacements consumed by the Rust repair.

Reproduce to a fresh output with the retained Rust publisher:

```sh
.local/route-grass-review/portable-r01-guidance-tools-v01/revise_r01_stair_guidance \
  content-packs/minnesota-current-brainerd-mart-roof-v01.crystalpack \
  e343cd9c65f7c876946be4b59878eabc564c9c7f42a7f29f54cb1a2fac89396d \
  planning/minnesota/revisions/route-grass-v01/current-r01-successor-review-v01/guidance-before-v01.json \
  planning/minnesota/revisions/route-grass-v01/current-r01-successor-review-v01/guidance-after-v01.json \
  content-packs/minnesota-current-r01-stair-guidance-reproduction.crystalpack
```

The broader route/region grass objective remains active. The new used-surface
inventory in `../current-route-ledge-review-v01/` binds 28 of 53 numbered-route
chunks to unchanged used terrain/actors/events/connections; 25 chunks have later
features needing successor evidence. This Route 1 review resolves one of those
changed chunks. That inventory is not whole-goal or earned-campaign completion.
No commit, push or external publication was requested or made. Packs, captures,
saves, build output and generated verification material remain ignored.
