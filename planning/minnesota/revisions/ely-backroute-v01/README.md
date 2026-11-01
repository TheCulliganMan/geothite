# Ely snow and the Split Rock back route

Requested: falling snow in Ely, a playable Pokémon Center south of Split Rock,
and a new Ely–North Shore link that avoids the long southern approach. Work is
solo. The compact Superior/Grand Marais geography is the authoritative parent;
the later shore-gate branch still has the superseded broad coastal layout and
is not silently substituted for it.

## Geography and services

The actual parent is `content-packs/minnesota-superior-compact-v03.crystalpack`,
689 maps, SHA256 `032304d0fd1d51863a2c3bf2562b20728e22c7535d1c194555621bc8e85b4899`.
Ely is at world metatile `(163,-342)`. Its east boundary touches the new
7×12 wooded pass at `(183,-342)`, which touches R11's west boundary at
`(190,-352)` (the R11 chunk origin). The pass makes a short northward dogleg through the real gap.
It has snowy western pines, a warmer grassy coastal bend and a trail guide.
The northern and southern edges remain solid pines; there are no fake water
seams or unpaired entrances.

| Seam | Native core lanes | Reciprocal offset |
| --- | --- | --- |
| Ely east ↔ pass west | Ely(39,14/15) ↔ pass(0,14/15) | 0 / 0 |
| Pass east ↔ R11 C00R02 west | pass(13,10/11) ↔ shore(0,30/31) | −10 / +10 |
| Existing Ely north ↔ R13 | unchanged | unchanged |

The complete original Johto Pokémon Center facade occupies R11 world metatiles
`(16..17,63..64)`, below the Split Rock lighthouse at `(25,49)`. Its real door
is core `(33,33)` in `MnR11C00R02`, with passive exterior return `(33,34)`.
`MnSplitRockPokecenter1F` uses the native clinic floorplan, a nurse at `(3,1)`,
PC at `(8,0)` and safe interior arrival `(3,5)`. Healing is the production
PokecenterNurseScript with its own dynamically allocated recovery point. Existing
League/badge/survey checks, towns, trainers and Cut gates are retained.

Ely's private `mn_ely_snow` alias preserves all original Kanto glyphs, recipes
and building silhouettes. Six exact native winter-pine glyphs occupy previously
unused slots96–101 and one new fully blocking pine block83 surrounds the town.
The two decorative partial Cut canopies become ordinary pines, closing two
unneeded lawn corner cells; no reward or shortcut was gated by those trees.
The green palette receives snow colors. Falling snow uses a bounded presentation pool in 2D and the existing
3D particle system. Indoors and the warm North Shore are excluded. Snow does
not alter movement, encounter or battle-weather rules.

## Source loop and verification status

`prepare.py --epoch NEW_NAME` authors explicit cells and rows, checks legal
actor-aware ordinary walking and renders whole maps using unchanged source
pixels. It never writes a content pack. Every epoch, original and conversion
is retained. `source-v08` incorporates the requested pines and was visually reviewed: the clinic facade is complete,
its door faces its forecourt, and the pass occupies the actual gap at one scale.
No user visual acceptance is implied.

Native publication, controller round trip, healing/PC/Continue, snowfall unit
verification and headless 2D captures passed. The replay publication is byte
identical. See [VERIFICATION.md](VERIFICATION.md) for executed commands, evidence
and scope. No full campaign playthrough or final 3D visual acceptance is claimed.

The pass guide stands at core `(8,14)` in its optional grassy spur. The native
18-column dialogue points west to Ely and east to Split Rock's Center, explains
the shorter return, and names the inland snowline. There is no reward/story
flag or admission charge. Both grass bands lie across the actual back road.
The Center's exterior active/passive warp indices are3/4; its two indoor exit
carpets are1/2 and passive arrival is3. The old lighthouse's indices1/2 remain
unchanged. Original R11 trainers and pickups are not relocated.

Pine source review history is immutable: v03 revealed that Kanto block37 was
part of the lodge facade, so v04 excludes it from tree replacement. All lodge
rows and both doors are preserved. Only actual canopy blocks0f/32/33 become
the new pine block83. The extra colors and six imported glyphs remain confined
to Ely's private alias; the original Kanto catalog is unchanged.


The accepted v08 clinic stands on the south bank. Every original WATER/cliff
quadrant and the existing brick stream crossing remain exact. Native publication
also rejects any R11 patch that replaces water, cliffs, ice or field gates.
The source compiler flattens overlapping editor strokes into one original-parent
preimage per cell before publication. The CLI takes an explicit reviewed contract
SHA256, so future source edits do not require recompiling a hardcoded hash.
