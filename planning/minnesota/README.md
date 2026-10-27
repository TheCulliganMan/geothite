# Minnesota mod pack implementation plan

Build an authored Minnesota region in Geothite using the original 2D tile language of the Johto and Kanto atlas rendered for this task. The region starts in Winona, passes through eight gym towns, follows the North Shore into canoe country, and ends at the Ely League. This folder is the implementation handoff. The [Grand Marais town design and review loop](playable/README.md) supersede its initial blockout; its gameplay is implemented and tested, with visual acceptance still open. The town, [North Shore Trail](routes/MN_R11.md) and [Gunflint Trail](routes/MN_R12.md) are preserved with native overlays and focused controller evidence. [League Approach](routes/MN_R13.md) has an individual granite route and shelter, and [Headwaters Trail](routes/MN_R14.md) now adds a quieter nine-chunk wetland/pine composition with a native wooden-rail crossing. Both have native overlays, production 2D review and explicit controller/MCP fixtures. The connected regional journey and hub gates remain pending. Generated packs remain ignored in `content-packs/`.

The user rejected both the small ImageGen route plates and the expanded illustrated replacement. **The actual Johto and Kanto tile render controls visual style.** The Minnesota overview controls regional geography and destination names only. Do not use the existing geographic map generator to compose this region.

**Current direction:** map sizes can vary. The problem is sparse, overdesigned space without worthwhile features. Choose each map's extent around landmarks, movement decisions, encounter placement and rewards; avoid padded walks and district/count quotas. [Central Lakes](routes/MN_R15.md) is one compact iteration, not a required size template. Review each complete map and its player-scale views for variety, feature density and pacing.

Read [STATUS.md](STATUS.md) for the precise delivered work, planning checks, and remaining implementation tasks.

The preserved prairie art pass gives Prairie Farm and Red River Prairie yellow grass
using native glyphs and palette7. [Prairie renders and replay](playable/prairie-grass.md)
use `content-packs/minnesota-prairie-review.crystalpack`, layered on the retained
Iron Range/North Shore geography revision. All generated packs remain ignored.

The current [garden/pine revision](playable/garden-and-pine-revisions.md) frames
R04's east garden, R06's northern junction and R07's north turn. It retains
the yellow prairie in `content-packs/minnesota-canopy-review.crystalpack`; eight
current controller cases, exact preservation and all377 audio checks pass.

[Gym designs](GYMS.md) specify all eight leaders' rooms, puzzles, trainer
parties, explicit move sets, rewards and field permissions. [City by city
progression](PROGRESSION.md) covers all nineteen hubs, the optional circuits,
Fairgrounds, rail outposts and the complete Winona-to-Ely journey. These expand
the existing story and route contracts; they do not claim a published gym
circuit or a completed earned regional playthrough.

The [Northstar Survey story](STORY.md) follows the player's first parcel into a
Team Lode cover-up: stolen instruments, falsified copper-nickel test readings,
and a plan to force Kettlurk to power the mine's pumps. The villains have
credible motives and knowingly harmful methods. Main-chapter reveals and sample
dialogue are attached to the existing milestones in `story.json`. The native
Q06 foreman, separate meter recovery and Grand Rapids handover now join the
native Duluth reveal and Q07 chief/receiver ending in the 502-map
`content-packs/minnesota-northstar-harbor.crystalpack`. It retains the
repaired three-city source, prairie art and earlier terrain. Actual handover
opens the guarded R10 walking link; victory or pickup alone cannot do so.
See [Q06's native contract](routes/MN_R09.md#actors-rewards-and-q06) and the
[current evidence](STATUS.md). The harbor reading identifies Kettlurk, confirms
that Lode continued after its own warnings and sends the true readings only
after all nine pages are acknowledged. Upstream earned chapters, Duluth's
regional travel links, League completion and the Kettlurk encounter remain open.

[One-way path contracts and renders](LEDGES.md) document the compact routes' ledges, safe landings and uphill returns. Their fast review loops include a movement overlay and reject decorative drops with trivial bypasses.

## Read in this order

1. [ART.md](ART.md): source pixels, scale, palette, and composition requirements.
2. [WORLD.md](WORLD.md) and [world.json](world.json): region graph, map names, chunk boundaries, and connections.
3. [STORY.md](STORY.md) and [story.json](story.json): story, badges, persistent flags, and progression gates.
4. [routes](routes): one detailed route dossier and one explicit terrain layout per route.
5. [towns](towns): town dimensions, ports, required destinations, and interior inventory.
6. [IMPLEMENTATION.md](IMPLEMENTATION.md): repository integration and work sequence.
7. [ACCEPTANCE.md](ACCEPTANCE.md): evidence required before a map, story chapter, or pack is accepted.

[actors.json](actors.json) holds 12 named story interaction anchors, and [interiors.json](interiors.json) holds home, shop, clinic, gym, cave and League room navigation reservations. [ROUTES.md](ROUTES.md) is the compact route-by-route index.

## Current design inventory

| Record | Count | Meaning |
| --- | ---: | --- |
| Logical routes | 20 | Geography/story links; size is determined by worthwhile play |
| Towns and major outdoor landmarks | 19 | Includes lake and wilderness hubs |
| Route map chunks | Derived by `tools/check_plan.py` | Up to32×24 metatiles; clipped extents vary as individual revisions freeze |
| Hub interiors | 64 | Clinics, shops, gyms, home, field station, story rooms, League |
| Route interior reservations | Derived from route dossiers | Caves, farmhouses, tool shed, keeper/ranger rooms and lake cabin |
| Gym battles | 8 | Explicit parties, levels, badge slots and repeat behavior |
| Main story quests | 9 | Q00 through Q08 |
| Side quests | 4 | SQ01 through SQ04 |

Route and hub counts describe planning, not connected gameplay. See
[STATUS.md](STATUS.md) and each route runbook for current native evidence.
All twenty route rasters have native publication evidence in the471-map route
composition. Recent garden/pine, prairie and geography revisions have focused
production-controller and preservation evidence; consult STATUS.md for their
actual scope. Hub integration, connected earned story travel and final visual
acceptance remain open. The atlas verifies
actual source/native equality before using teal and marks superseded packs amber.
Map size follows useful features, not a region-wide compactness rule.

## Source priority

The graph and story conditions are explicit design decisions. Preserve them while authoring individual maps. The route JSON files are planning schemas; the implementing agent must lower them to the typed Rust pack structures. Grand Marais has a separate implemented Rust builder that reads its town authoring JSON.

The `.layout.json` files contain authored brush coordinates and original composition strokes. They contain no copied base-map block arrays. `tools/render_reference.py` reads the user-supplied pack in memory and draws those strokes with its existing metatiles. It does not infer geography, synthesize source assets, or export a pack.

If an illustration, dossier, collision result, and gameplay result disagree, resolve the conflict in this order: gameplay safety, authored graph/story contract, actual source art, then illustrative geography. Record any change to an endpoint or quest gate in the JSON and its dependent tests.

## Inspect the work

Open [the single connected progress atlas](renders/atlas/minnesota-current-connected.png), [the local render gallery](renders/index.html), [the Route 1 map and directional contract](routes/MN_R01.md), and [the native-scale Route 29 reference](renders/source/Route29.png). The atlas includes all current terrain, distinguishes earlier blockouts and marks unfinished hub links; see [its regeneration command](ATLAS.md). Renders and the copied visual references are ignored because they depend on local content.

**Keep every render for reuse, including rejected passes.** Save full-size originals
before conversion, native crops, converted images, overlays, rooms and previews.
Map review tools use `tools/render_archive.py`: each output and its previous alias
are retained byte-for-byte in an adjacent `history/` directory, named with the
full SHA-256 of the encoded file. Convenient current filenames are additional
copies. Conversion or encoding failure must not remove the original. Render
history is a local reusable deliverable; do not delete it during build cleanup.
Production screenshots also retain original images and exact native readback
bytes with format/dimension descriptors under a sibling
`.render-archive/<output-filename>/` directory before decoding or conversion.
This includes original creature art, sprite sheets and visual references outside
the map render folder. Retain all current planning images without converting them:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/render_archive.py \
  planning/minnesota
```

Historical creature renders and exports also have content-addressed copies in
`renders/creature-archive/`, including the older archive from the build cache.
Its immutable inventories preserve original paths and hashes for reuse. See
[the creature archive command](pokemon/README.md).

Regenerate source references with:

```sh
python3 -m venv .local/minnesota-planning-env
.local/minnesota-planning-env/bin/pip install Pillow cbor2
.local/minnesota-planning-env/bin/python planning/minnesota/tools/render_reference.py \
  content-packs/core-modular.browser.crystalpack planning/minnesota/renders/source
.local/minnesota-planning-env/bin/python planning/minnesota/tools/check_plan.py \
  --pack content-packs/core-modular.browser.crystalpack --render
```

Run `python3 planning/minnesota/tools/check_plan.py` for a plan-only check without a pack or image dependencies. This checks authored records and dependencies; it is not a gameplay test.

## Start the implementation

The **Minnesota State Fairgrounds** are explicitly in scope as a dedicated
playable destination connected to St. Paul. The [fairgrounds contract](zones/state-fairgrounds.json)
includes the midway, livestock/4-H barns, dairy/butter exhibit, food row, show
ring and repeat-safe activities. Its actual city entrance is being reserved;
the grounds and their native gameplay are not published yet.

The [Glacier Line source contract](transit/README.md) adds the requested
Detroit Lakes–Staples–St. Cloud–St. Paul–Red Wing–Winona rail spine. The complete
St. Paul depot public room now has native city doors and all four return carpets,
verified in [the local city candidate](playable/st-paul.md). St. Cloud's public
station door, both carpets and Continue also pass in [its native successor](playable/st-cloud.md).
Both train platforms stay sealed. Courier/rail progression, the other station bindings and native
animated travel remain implementation work. The progress atlas must distinguish
that local public-room connection from the unimplemented rail network.

Keep Grand Marais and North Shore stable. Keep the individually authored Gunflint, League Approach and Headwaters overlays and their checked production renders/controller behavior available. Its hub ports and the connected journey remain integration work. Continue the same individual process across the remaining routes; the full twenty-route objective stays intact. The planned initial regional journey remains Winona, eastern Route 1 and Rochester's first gym. Prove connected production-controller progression before describing the region as playable.

Boundary Waters is now designed as a **Safari-style wildlife preserve**, with a free ranger basecamp and separate optional capture expeditions. The [gameplay contract](zones/boundary-waters-safari.json) fixes candidate habitats, session budgets, capture commands, progression safeguards and required Rust work; the [implementation design](playable/boundary-waters-safari.md) records the runtime handoff. Its four candidate preserve chunks are not yet registered or included in the map inventory.
