# Minnesota mod pack implementation plan

Minnesota uses native 2D only for the entire session. 3D and 2.5D are unavailable
through desktop menus, F3, browser controls, saved preferences and previews,
including on original maps retained in the regional pack. Text view remains available.

Minnesota is an authored Geothite region using native Crystal art. The journey
starts in Winona, follows eight gyms to Duluth, resolves the Northstar Survey
in Boundary Waters and ends at the Ely League. This folder contains the source
plans, profiles and versioned implementation records. Generated packs stay
ignored in `content-packs/`.

Read [the story profile](STORY.md), [progression](PROGRESSION.md) and
[gym profiles](GYMS.md) for the current authored direction. The
[current story composition record](revisions/current-story-v01/README.md)
and [world.json](world.json) identify their particular publications and later
regional revisions. Chapter records retain the evidence and limits of their
named packs. A newer source file does not retroactively update an old pack.
An uninterrupted ordinary-party campaign remains unclaimed.

The user rejected both the small ImageGen route plates and the expanded illustrated replacement. **The actual Johto and Kanto tile render controls visual style.** The Minnesota overview controls regional geography and destination names only. Do not use the existing geographic map generator to compose this region.

**Current direction:** map sizes can vary. The problem is sparse, overdesigned space without worthwhile features. Choose each map's extent around landmarks, movement decisions, encounter placement and rewards; avoid padded walks and district/count quotas. [Central Lakes](routes/MN_R15.md) is one compact iteration, not a required size template. Review each complete map and its player-scale views for variety, feature density and pacing.

Read [STATUS.md](STATUS.md) for the precise delivered work, planning checks, and remaining implementation tasks.

Map publications have their own versioned records, including the
[prairie art](playable/prairie-grass.md),
[garden and pine edges](playable/garden-and-pine-revisions.md),
[North Shore](routes/MN_R11.md) and [Gunflint](routes/MN_R12.md).
Use the regional revision pointers for the intended parent rather than selecting
an older pack from one of these records by name alone.

The Northstar story follows Tamarack's delivery to Ivy into Ellis's rescue,
Mina's shipment request, the endangered mine crew and Lode's attempt to use
Kettlurk to drain the lakes. Reed knowingly joins ICE and repeatedly guards the
operation. The ending requires the local drive shutdown and witnessed release;
the chief battle and radio link alone cannot restore the survey.

[One-way path contracts and renders](LEDGES.md) document the compact routes' ledges, safe landings and uphill returns. Their fast review loops include a movement overlay and reject decorative drops with trivial bypasses.

## Read in this order

1. [ART.md](ART.md): source pixels, scale, palette, and composition requirements.
2. [WORLD.md](WORLD.md) and [world.json](world.json): region graph, map names, chunk boundaries, and connections.
3. [STORY.md](STORY.md) and [story.json](story.json): story, badges, persistent flags, and progression gates.
4. [routes](routes): one detailed route dossier and one explicit terrain layout per route.
5. [towns](towns): town dimensions, ports, required destinations, and interior inventory.
6. [IMPLEMENTATION.md](IMPLEMENTATION.md): repository integration and work sequence.
7. [ACCEPTANCE.md](ACCEPTANCE.md): evidence required before a map, story chapter, or pack is accepted.

[actors.json](actors.json) contains base story anchors;
[interiors.json](interiors.json), the chapter sources and their Rust builders
supply additional rooms and actors. [ROUTES.md](ROUTES.md) indexes the twenty
logical routes. Counts in early blockouts are not a current pack inventory.

## Story conventions

Follow [STORY.md's source and writing conventions](STORY.md#sources-and-writing-conventions).
Use supplied names, plain speech and concrete requests. Do not invent a
biography, regional custom or completed test result to fill out a profile.
Minnesota appearances use only the active roster after curation; base import
species and historical test parties are not current regional teams.

`story.json` provides the base contracts, while its `dialogue_overhaul` owns
revised text labels after chapter overlays. `story-events-v2.json` and
`story-arc-v3.json` supply acted scenes; `story-pokemon.json` supplies regional
binding. Preserve that composition order. Historical chapter records are
supporting evidence for their own versions, not alternative current profiles.

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

## Implementation records

Use [STATUS.md](STATUS.md) and the revision fields in [world.json](world.json)
to locate the relevant publication. The
[current story record](revisions/current-story-v01/README.md) documents compact
geography, the recurring rival, both lairs and the acted intake release. The
[interior/story child](interior-life/current-story-v01/README.md) has a separate
identity and verification scope. Neither record certifies later source edits.

The [State Fair](playable/state-fairgrounds.md),
[Glacier Line](transit/README.md),
[Boundary preserve](playable/boundary-waters-safari.md) and
[free basecamp](playable/boundary-basecamp.md) have independent implementation
records. Keep optional activities separate from mandatory story progress.
Safari payment, food purchases, catches and local repairs cannot gate the
rescue or League release. Do not infer admission or passenger service from the
presence of a design document.

Keep accepted terrain and reusable render originals. Follow the repository's
Rust pack-build path and exact-parent guards, verify the resulting pack and
use production-controller evidence before calling a new composition playable.
