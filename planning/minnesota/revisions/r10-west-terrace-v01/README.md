# R10 cache west terrace

This is a separate, unselected correction on cache-cliff v04. It closes the
cache's west face and front with native cliff corners and a real southern
stair, removing two boulders that previously substituted for its wall. The
older carrier terrace remains separate; keep its east retaining face and its
western stair return. The remaining western spur/carrier-stair boulders are
unresolved and are not certified by this correction.

## Exact contour

Coordinates are32px metatiles in the48×20 route. Only ten cells change in
`MnR10C00R00`; the second native chunk, reciprocal seam, actors, events, teams,
pickups, cave warps, story conditions and lake boundary stay exact.

| Segment | Coordinates | Native recipe |
| --- | --- | --- |
| West face | (11,4..5) |83: left glyph columns of68, right columns of03 |
| Southwest turn | (11,6) |6c |
| Grass-lined front | (12..15,6) |84: upper glyph rows of03, lower rows of72 |
| Southern stair | (16,6) | Existing82: outdoor brown landing with native12 treads |
| Recessed front turn | (17,6) |6e |
| Mine forecourt west face | (17,7) |68 |

The stair's inside position is core(33,12), tread(33,13), and outside landing
(33,14). Both tread sides must reject repeated movement. The southwest turn
and mine recess remain solid retaining corners. Existing carrier stair82 at
meta(5,10) remains required to return from the lower road to the carrier rim;
the new cache flight does not replace that climb.

Private83 and84 allocate no glyphs, images, palettes or foreground masks.
The Rust art stage preserves all131 existing metatiles and uses the original
per-glyph palettes. Native collision is WALL/GRASS/WALL/GRASS for83 and
GRASS/GRASS/WALL/WALL for84. Do not flatten their rock faces or import a pale
cave-floor cap.

## Habitat and rejected passes

The MAX_REPEL stays at core(27,9). The southern stair is deliberately at x16;
placing it at x15 produced a three-step bypass via the walkable north rim.
A brown front strip produced a two-step bypass. The final grass-lined front
and rightward stair retain at least four encounter steps from every ordinary
approach, even after other actor bodies are cleared. The pine neck still
requires six grass steps. Native UP_WALL lateral movement and directional
entry/departure count in this check.

The source collision check is
`renders/r10-western-cliff-v01/west-grass-v116.json`; it is not production
gameplay proof. `rejected-three-step.layout.json` and its dossier preserve the
failed stair position. `rejected-carrier-bypass.layout.json` preserves the
failed attempt to remove the lower carrier face. Earlier full-resolution
renders remain under `west-terrace-v115/` and `west-terrace-v118/`.
The wind-grass review point moves from core(27,13), now the front wall, to
(27,11), inside the actual grass. This is an authoring camera contract, not a
runtime actor relocation.

## Publication and review

The Rust CLI is `revise_r10_west_terrace`. Its arguments are parent pack,
exact parent SHA256, before layout, after layout, dossier, fresh art output,
and fresh terrain output. It verifies the parent, preserves storage, reads
both written stages back, and validates the complete revised route through
the existing native terrain publisher. Neither stage selects an active pack.

`r10_closed_west_terrace_rejects_dry_front_lip` is an opt-in regression with
`CRYSTAL_R10_WEST_TERRACE_PACK` set to cache-cliff v04 and
`CRYSTAL_R10_CACHE_SOURCE_ROOT` set to the revisions directory. It reads all
external inputs at runtime, validates the new complete terrace, and rejects
the dry front's actual native two-step bypass. The native build/publication
cohort is `renders/r10-western-cliff-v01/west-*-v119.*`.

`audit_r10_cache_art.py --west-terrace` checks exact raw joins, all four palette
banks, every original metatile and whole-pack preservation, with unrelated
actor/foreground mutations rejected. The generic terrain audit checks only
the declared ten-cell change and rejects unrelated map, actor, art and audio
drift. Audio hashes/frame counts/loop ranges, production stair and wall input,
cache access, Iva, Continue, native GPU review and required scoped Clippy must
pass before selecting or claiming this candidate playable.


## Written native candidate

The normal Rust CLI published
`content-packs/minnesota-current-r10-west-terrace-v01.crystalpack`, SHA256
`e6c2126e57e1349ae73ba42c6139d8e1cc7f3b61298e2df892b0d122866da536`.
`west-publication-v119.json` binds the exact source hashes and both native stages.
The art audit passes both joins, all131 old metatiles in all four palette banks,
whole-pack preservation and two unrelated mutations. All377 audio PCM hashes,
frame counts and loop ranges pass in `west-audio-v119.log`. The terrain audit passes
752 checks for exactly ten cells in one map and rejects all four unrelated
mutations. The actual native grass regression passes as well.

The whole written route and reciprocal seam were inspected in
`renders/r10-western-cliff-v01/west-native-whole-v125/`. These retained originals
use native pixels and initial actor poses; they are a source atlas, not a GPU
or walking receipt. The current source preview rendered in0.978s after pack
loading. Parent-cohort asset and Bevy all-targets Clippy now pass in the v66
logs. Broader integration remains pending; the new carrier-corner API needs
its own asset check. The active world selection is unchanged.

The fresh full-workspace formatting check still fails on unrelated existing
files, starting in `offset-allocator`; the complete log is `west-fmt-v122.log`.
The three changed Rust leaf files pass scoped `rustfmt --check`.

## Production input and visual review

`west-local-review-proof-v127.json` binds the exact e6c2126e native pack to208
successful production RPCs:34 for both stair flights and the southwest turn,
99 for the cache/west wall/front/actual reached-save Continue and Start,57 for
the east face, and18 for Iva's native battle menu. Initial native identity,
position, legal Minnesota partners and Repel are explicit fixtures. No state
is repaired after input. This does not claim collection, a completed battle
turn, earned regional access or statewide completion.

The failed12-call cache replay is retained in `west-native-stdio-v121/`:
the test path tried to cross the reward actor at(27,9). The corrected path goes
around it through y8, using the same pack and unchanged initial fixture.
`west-cache-stdio-v126/` passes; the already successful east-face and Iva cases
were not rerun. Every repeated wall probe attempts three directional presses.

All six unmodified native GPU captures in `west-native-gpu-v126/` pass with
zero pixel mismatches and were inspected individually: both sides of the new
stair, the grass-lined west face, southwest turn, mine recess and reward.
The source atlas and all earlier rejected renders remain preserved. The
western spur/carrier boulder stack still needs its own contour correction;
this pass closes the cache terrace only. Do not select this pack until the
remaining integration/source checks have been reviewed.
