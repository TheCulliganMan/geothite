# Four bounded route terrain revisions

The reviewed local successor is
`content-packs/minnesota-route-quality-next-v01.crystalpack`, SHA256
`1186a24371251f741db88cf35a7863145ed5423da4ad3d7c6485c8f242753ecb`.
It contains660 maps and preserves the installed three cities and all their
rooms. Exactly211 metatiles change across seven existing route chunks. These
are hand-authored terrain improvements, not final whole-route or regional
Johto parity, user visual approval or earned campaign progression.

[Four complete routes at one scale](../renders/route-quality-next/whole-review-v02/four-routes-review.png)
and [Minneapolis–river park–St Paul at their actual connected positions](../renders/route-quality-next/whole-review-v02/twin-cities-connected-review.png)
show the current native-source geometry. Full-resolution originals, rejected
passes, crops, conversions and untouched GPU captures remain retained. Dark
space above and below the river park is outside the known maps, not invented
connecting terrain. The older region-wide atlas remains historical.

| Route | Bounded change | Native cells |
| --- | --- | --- |
| R04 Twin Cities river park | An attached tree tongue makes the east-bank road turn; lower groves close loose lawn shoulders. All bridges, readings, trainers and grass choices remain. |6 |
| R08 Cedar/Long Lake | Wooded headland, reachable pier drop with grassy ordinary return, and a bent south inlet join the actual lake silhouette. |92 |
| R18 Mille Lacs | Walking follows the lake bank, dune bends and optional fenced shoreline while preserving the mandatory six-grass neck, piers, Surf islet and ranger reports. |73 |
| R19 Prairie–Oak | Pine headland, two-lane native oak descent and a tighter watch pocket preserve the direct trainer versus eight-grass alternative, original ramps and Headbutt tree. |40 |

Each new drop has blocked reverse and side routes and a real ordinary return.
R08's pier return is locally required; the longer183/185-step regional lake
alternative remains honestly available. Its shortest ordinary return lanes
are13/11 steps with at least four grass entries. R19's new oak return lanes
are9/11 steps with at least three grass entries. Existing ledges and their
previous contracts are retained. A permanently blocking native Headbutt tree
can serve as a side guard; an openable Cut tree cannot substitute for one.

## Authoritative source and publication

The eight canonical files are `routes/MN_R{04,08,18,19}{,.layout}.json`.
Their guarded promotion preserves exact previous bytes under
[`revisions/route-quality-next-v01/`](../revisions/route-quality-next-v01/).
Only the reviewed layout strokes, declared dossier fields and bounded
`terrain_revision` evidence metadata were promoted. No town, room, party,
quest or world reservation was rewritten during this promotion.

The strict Rust terrain publisher revises existing native modules without
re-lowering editorial actors or programs. The executed order is R04→R18→R19→R08
from `minnesota-three-cities-tight-final-v02.crystalpack`. The frozen
[publication receipt](../renders/route-quality-next/publication-receipt-v02-e23e84d678fb2433e5f76bb3fcde79a3aff49a7f2a712993fd882bf6e5e7caba.json)
records all parent/stage hashes, binary hash, original failed attempts and
byte-exact R04 reproduction. The original R04 execution binary hash was not
captured before its rebuild; that reproduction closes the specific gap.

The executable accepts `PACK BEFORE_LAYOUT AFTER_LAYOUT DOSSIER OUTPUT`:

```sh
CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=1 \
  cargo build --profile wasm-dev --locked -p crystal-assets --bin revise_authored_route
target/runtime-fps/wasm-dev/revise_authored_route \
  content-packs/minnesota-three-cities-tight-final-v02.crystalpack \
  planning/minnesota/renders/route-quality-next/r04/v04/before.layout.json \
  planning/minnesota/renders/route-quality-next/r04/v04/after.layout.json \
  planning/minnesota/renders/route-quality-next/r04/v04/dossier.json \
  content-packs/minnesota-route-quality-r04-v01.crystalpack
```

Use the receipt's next stages and frozen inputs for the remaining three steps.
R08 requires its `v05/MN_R08.publication.{previous,candidate}.layout.json`
pair, which includes the six already-installed story caps. The canonical
route layout deliberately remains separate from that story overlay. Publication
uses `integration-inputs/R08.native-anchored.dossier.json`: three old planned
trainer names differ from the installed SOREN/ADA/MARA identities, while their
parties match. Only that frozen publication dossier reconciles the names;
neither native identities nor another author's planned roster are overwritten.

## Actual native evidence

The [independent preservation audit](../renders/route-quality-next/native-preservation-v01/proof-8dce57b002aca42e345e0a227af0c5eac2d4621cb8041e78a0b5636b4a68ca3b.json)
passes719 checks and rejects four meaningful corruptions. Restoring only the
declared block arrays and their uniquely owned encoded payloads makes the
entire dataset equal to the parent. All unrelated maps, cities, rooms, scripts,
events, encounters, trainer parties/names, source art, runtime files, reports,
HOME and recovery records remain exact. Raw pack identity is independently
recomputed. All377 audio PCM hashes, frame counts and loop ranges pass on the
exact final pack.

Thirty-eight actual640×576 headless production frames match14,008,320 source
pixels exactly: [R04](../renders/route-quality-next/native-r04-v02/production-decisions.png),
[R08](../renders/route-quality-next/native-r08-v01/production-decisions.png),
[R18](../renders/route-quality-next/native-r18-v01/production-decisions.png) and
[R19](../renders/route-quality-next/native-r19-v01/production-decisions.png).
The connected-route reviewer uses actual immediate neighbors/offsets, native
border repetition, foreground priority and the production camera clamp. It
does not fit an image translation to hide an error. Water animation has one
consistent native phase throughout each complete frame. These frames use
explicit local empty-party placements at noon, south-facing; they do not
prove earned access. Root inspected the whole maps, paired source views,
all four final native decision sheets and the weakest additional views.

[Twenty production controller cases pass in one final run](../renders/route-quality-next/controller-v05/controller-proof.json).
They cover both new drop lanes, reverse/side denial and ordinary returns,
old drops/ramps, whole shore trips, native Cut denial/removal/reload, rod-report
subsets/retries, Surf/islet reward/return, Headbutt battle with actual PP use,
the populated Twin Cities journey, two trainer victories against four opponents,
three wild encounters, actual Ether restoration/consumption and Continue.
Inputs use `VisibleShellController::press`; initial party/badge/position/RNG or
quest-subset fixtures are explicit. R19's separate zero-grass watch case starts
with its trainer defeated and does not claim an earned victory.

Earlier failed runs remain retained. One assumed raw flag-map representation
stayed identical across a native seam; the corrected check compares all logical
event/engine values. Pickup assertions now read the actual item name. The
populated R04 course walks the new elbow rather than repeatedly pushing into
its tree. Native battle narration owns input before any underlying replacement
cursor; the final course no longer swaps RNG after setup. No position, flags,
inventory or RNG are rewritten during gameplay to rescue a test.

Five focused asset traversal tests pass, including local versus regional
ordinary-return contracts, invalid/bypass negatives and permanent-tree versus
Cut side guards. Changed Rust leaves pass formatting. The current global
`cargo fmt --all -- --check` retains unrelated offset-allocator differences.
Assets and Bevy all-target Clippy completes with `location-tester`, `--locked`
and the existing `never_loop`, `bad_bit_mask`, `approx_constant` and
`useless_conversion` allowances; warnings occur outside the changed leaves.
The canonical shared route checks pass after promotion, including R08's
permanent-tree side guard and three rejected walkable/openable replacements,
and the existing R18/R19 adversarial cases. The global plan check passes with
20 routes,19 hubs,57 chunks,88 contracts and288 planned maps. Its symbolic
badge/champion reachability is not executed campaign evidence.

## Continue the quality work

These four bounded passes leave other weak portions visible. R01, R02, R03
and R05 have isolated source candidates awaiting their own native/input
acceptance; they are not included in the211 changed cells above. Caves and
the rest of the twenty-route region still require continued sight review.

The64×48 city baseline still has excessive empty lawn. The latest city work
is a smaller hand recomposition with the same useful shops, rooms and
activities, compared against actual Goldenrod/Saffron player cameras. See
the [installed-city recomposition contract](../cities/RECOMPOSITION.md);48×36
is an exploration ceiling, not a minimum size. No smaller city has yet been
published or accepted. The separate native State Fair664-map candidate,
other gym/story successors and Northwoods creature work are distinct lineages
until explicitly composed and reverified.
