# Minnesota exterior stair landings

Current integration: `outdoor_terrain_integrated_revision` is selected in
`world.json`. See `../outdoor-terrain-integration-v01/README.md` for the ordered
authoring sources, exact composition and final568-call/15-view native proof.
The older unselected/pending statements below document historical stage packs.

Current parent-cohort verification is complete:68 requested native GPU views
pass with zero pixel mismatches and were reviewed in retained contact sheets;
all scheduled native controller/roster/animation regressions and both required
asset/Bevy all-targets Clippy commands pass. `regional-image-review-v129.json`
records the image review, and the complete logs remain under this revision's
render directory. Existing workspace warnings are retained; the full-workspace
format check has unrelated pre-existing failures. Scoped changed Rust passes.
The subsequent R10 cache/west-terrace pack has separate exact native input,
audio and image receipts. Its next carrier-ridge revision remains separate work.
No active world selection has changed. Older pending/rejected-stage notes below
describe their historical preimages, rather than the final verification state.

User direction: remove the miscolored cave-floor tile above **all outside
stairs**, preserving the correct stair shape, native gray steps and surrounding
outdoor terrain. Right-corner stairs use36; a local30 reference is not a global
replacement rule.

The selected-parent inventory found23 imported outdoor stair placements:
Duluth11, Boundary Waters5, Iron Range3, North Shore3 and western R10 one.
Duluth's authored terrace revision fixes its shapes and geometry separately.
This final regional art pass retains all other terrain IDs, collision, events,
trainers, rooms, connections, programs and audio.

`recipes.json` defines the eight native-art row joins in four private outdoor
aliases. Front12/30 stairs retain outdoor72's two upper glyph rows. Right-corner36
retains outdoor69's upper rows, including its vertical side wall. Keep the
original native lower stair rows and palettes. The selected North Shore has131 blocks and has no36 recipe yet. The Rust
builder first appends native36 at83, then applies the outdoor69 landing. This
new right-corner art supports the three corrected North Shore descents. No source glyph pixels, palette entries or graphical bank addresses
are changed; seven existing metatile definitions select outdoor rows and one native36
metatile is appended with unused glyph slots.

The Rust `revise_outdoor_stair_landings PACK SHA256 OUTPUT` builder requires
Duluth's corrected terrace family first, rejects private art shared outside
Minnesota exteriors, and certifies every native lower stair glyph and collision.
The CLI checks the exact parent hash before and after publication, requires a
fresh output path and verifies the written pack. Run after the latest selected
parent has received the reviewed Duluth terrace revision; do not roll back
unrelated concurrent map or story fixes.

Readonly preview/audit:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_outdoor_stair_landings.py \
  --parent content-packs/minnesota-current-duluth-cliff-stairs-v03.crystalpack \
  --output planning/minnesota/renders/outdoor-stair-landings-v01/source-next
```

Without `--pack`, this composes native rows in memory for source review only;
it does not create or modify a game pack. With `--pack CHILD`, it checks exact
whole-pack preservation against the immediate post-Duluth parent, lower stair
pixels, outdoor landing pixels, unchanged gameplay data and every unrelated
runtime file. It saves whole source maps, per-stair views and a native camera
list. Retain all renders, including rejected versions.

Status: the native outdoor-stair-repair v02 candidate exists and remains
unselected. Exact preservation and all377 audio checks pass. Duluth's continuous
production MCP journey passes; the remaining controller and refreshed renderer
checks are in progress. Historical fixtures are local traversal setup, not earned
campaign progress. Do not report source images as native screenshots.


## Statewide cliff-edge correction

The user expanded the repair to all cliff-adjacent ledges and stairs. All42
native hop runs across103 Minnesota outdoor maps have been inspected. The
explicit `cliff-joins.json` candidate contains45 direct changes across20 maps:
33 hop cells close existing tall cliff faces; Brainerd's false upper ledge
becomes the continuous vertical side wall; three cells repair two low banks; and six cells move
three North Shore descents to native36 right corners. Two additional cells join
Boundary Waters' detached lower retaining wall. Four boulder relocations add
eight more changed cells, for53 cells total. Duluth's high-cliff hop closures are
a separate earlier stage with14 closed gameplay lanes.

R19's small bank remains a useful native one-way ledge. Its left cliff piece
becomes a low bank, its right cliff piece joins the existing pine boundary,
and the original central hop remains. R03's isolated grass bank keeps its hops
and replaces its oversized rock end with a matching grassy ledge. Neither bank
should be turned into a tall wall. No new hop may land in the bordering pines.

Every changed module has a complete native preimage guard; no placeholder
preimages remain. The plan preserves all actors, events, gates and connections.
Production publication, exact preservation, native visual review and controller
verification remain required before selection.

The source audit retained ordinary walking returns for the other accessible
cliff faces. R06's optional lookout is deliberately Cut-gated: its correct
return uses the real Cut replacement below the eastern ramp. Do not remove the
Cut tree or open an extra entrance to force a pre-Cut path check to pass. R05's
southeastern flagged hop had a rock landing already; closing it removes an
unusable jump rather than making a new crossing.

Fifteen original native74 crossings were inspected by direction. Four are
proper north-facing ramps adjoining native70 and remain native ramps. Eleven
south-facing crossings use native12 lower steps under clean
outdoor72 landings. The earlier upper74-cap preview was rejected: a rock lip
must not be pasted above a south-facing staircase. These native repairs remain
unselected pending production input and fresh renderer acceptance. Each needs correct side
walls, real lower footing, no flat bypass around its steps, and preservation of
its gate and anchor contract. Shared Johto/Kanto art cannot be changed globally;
use Minnesota exterior-only aliases for any new stair art and collision.


## Pictured Boundary Waters boulders

The user's latest reference is the Northwind upper pair in
`MnBoundarySafariC01R00`, at metatiles(13,9) and(15,9). The explicit
`boulder_moves` entries move each north by one32px map metatile to(13,8)
and(15,8), restoring native72 at the old positions. Their original native0a
art and collision stay exact. The native right-hand staircase at(16,9), nearby
grass, pines, actors and events retain their contracts. The source before/after
is retained under `renders/outdoor-stair-landings-v01/cliff-boulders-v08/`.
This preview is source review; native publication/capture and production walking
checks remain required before selecting the candidate.

The statewide boulder inventory retains31 candidate views. Those other boulders
are not claimed fixed by moving this pair. In particular, boulders replacing
side-wall/corner tiles need their actual retaining geometry restored; shifting
them north into another wall would repeat the same mistake.


Latest lower-cliff correction: `MnBoundarySafariC01R00`(15,10) uses native6f
and(16,10) uses native72. These connect the previously detached side wall at
(15,11) to the existing right corner at(17,10). The upper stair foot remains
walkable, with a westward bend around the inner corner to the lower terrace.
Do not round off the detached wall without restoring its missing connection.
The explicit terrain plan has45 changes plus four boulder movements: the upper
pair above and the lower pair at(8,13)/(10,13), moved to(8,12)/(10,12).

## Native evidence and remaining work

`renders/outdoor-stair-landings-v01/publication-v42.json` records the exact four
native stages. The final unselected stair-repair v02 SHA256 is
`cc639313098f1ea797cfdfb52f4d0ae950b5f47fa134ea30a85a333934ba375e`.
The landing, cliff and ramp preservation receipts are respectively
`landing-preservation-v42/receipt.json`, `cliff-preservation-v43/receipt.json` and
`stair-preservation-v43/receipt.json` under the same render folder. All377 PCM
hashes, frame counts and loop ranges pass in `build-audio-v42.log`.

Duluth's production MCP proof is `duluth-stdio-v49/receipt.json`:917 actual calls,
11 flights in both directions,21 step-side denials,14 high-cliff lanes denied in
both directions, real autosaves/Continue and zero stderr bytes. Its initial
placement and legal partners are explicit local setup, not earned access.

`native-v43/` retains preliminary native geometry captures from the earlier
renderer. The four new private stair aliases require the refreshed renderer's
animation registration and icy rustle checks; do not use those preliminary
images to claim animation acceptance. Existing native Duluth plateau views
remain pixel-exact in `native-plateaus-v40/capture-progress.json`.

The next additive terrain plans are `../outdoor-boulder-setbacks-v01/plan.json`
(ten straight-front setbacks) and `../outdoor-cliff-corners-v01/source-plan.json`
(seven actual corners). Neither has been selected. Keep their complete original
maps, before/after crops, failed publication logs and static topology receipts.

The regional production MCP replay now passes all26 non-Duluth/non-Cut courses:
22 repaired stairs and four retained north-facing ramps, with304 actual RPC
calls, both directions, appropriate side denials, unchanged initial fixtures
and zero stderr. Evidence: `outdoor-stdio-v55/receipt.json` and
`outdoor-mcp-proof-v55.json` under the render folder. Combined with the earlier
Duluth journey this verifies37 courses. R06's native Cut-gated stair remains
a separate required case; no gate was removed to make these courses pass.
The failed incomplete Repel setup is retained in `outdoor-fixtures-v54.log`;
v55 pairs the initial counter with MAX_REPEL before the first input.

The separate R06 Cut-gated stair now passes all three production MCP permission
cases on the ten-boulder v02 child: absent move and absent Hive Badge both deny;
with both, native Cut opens the tree, the player walks through and returns up
the stair. Native save validation confirms unchanged complete party/HP/PP, bag,
money and badges; the actual opened block is saved as3c. See
`cut-stdio-v61/receipt.json`, `cut-native-save-check-v64.log` and
`cut-mcp-proof-v64.json`. This adds the38th distinct verified outdoor course
to the earlier37; it does not claim earned Cut or permanent tree persistence.

All49 preliminary v43 native geometry cameras completed with zero differing
pixels. Keep their renderer provenance: fresh alias animation/icy-rustle and
updated GPU proof remain required. The ten-boulder native child passes full
preservation/audio, but its v60 input run has nine passing cases and one real
Iron Range trainer-sight failure. The separate
`../outdoor-trainer-sightlines-v01/plan.json` corrects that authored facing;
do not call the uncorrected boulder candidate accepted.
