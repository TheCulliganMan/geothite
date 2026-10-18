# Ho-Oh rainbow phoenix sculpture

## Canonical phoenix skin (2026-10-15)

The canonical source is now `models/battle_species/ho_oh.glb`, authored by
`tools/ho_oh_sculpt.py` and `tools/ho_oh_glb.py`. The obsolete static JSON was
removed. All 207 source parts remain recoverable in order from 38 runtime
primitives. Same-carrier red flight vanes, ivory bands, green tips and warm
coverts pack into separate coherent banks. Gold tail vanes and their pale
terminals, crest petals, fixed digits and talons also pack by carrier/color.
No third-party geometry is used.

The model has 19,747 vertices and 25,200 triangles, down from 81,715 and
118,516. Outlines retain their authored curve sampling while interior feather
subdivision is reduced. Head, crest, mouth, haunch, shin, digit and talon
sampling is reduced selectively. The ivory belly uses conforming local cap
refinement at its original thickness: a coarse candidate clipped the breast
and was rejected. The shared helper was extracted from Lugia without changing
Lugia's generated GLB bytes. Body, neck, both beaks and both feet preserve all
six original source parts exactly after float32 conversion: positions,
normals, triangles and colors. Their retained geometry SHA-256 is
`ff91b55266642639ceff0b02e907dd38191e49dcc3724b9060c18a9e0dc4d529`.

The ten-joint skin fixes feet, eight gripping digits and all ivory talons on
the identity floor root. Supported shins and haunches bridge to the yawing
breast. Neck and gaze articulate independently. Each complete wing bank moves
with its own shoulder, preserving both painted faces. The lower beak has an
opening joint, and the entire golden fan has a separate yawing tail carrier.
Idle, attack and recoil contain rotation keys only and return literally to
neutral. Authoring height remains 1.45. The verified local pack supplies
height digits 1206 (12 feet 6 inches), or 3.81 metres, classification RAINBOW.

The source geometry pin is
`67e4806121613127491e740385e7974e9afdb7b32f52dbe8722674ffa277214b`;
the packed runtime pin is
`4e246f222c484ed35240a4bd22ce63b024b482e85b707d9968dbea9da3fac218`.
The skin uses 179 accessors within unchanged renderer limits. The common
Blender builder exports the same canonical GLB. Neutral, idle, attack and
recoil front/rear software views were inspected against the original static
model. No detached paint or visible loss of the tricolor silhouette was seen.

Thirteen stored anatomy checks, seven decoded-skin checks and three focused
Rust routing/rig/GPU checks passed. They cover closed solids, normals,
self-intersections, actual fitted-layer clearance, occupied roots, a real
open jaw with a filler negative control, coherent carriers, fixed floor
contacts, tail height, posed winding/strain, neutral endpoints and culling
bounds. The jaw-volume oracle merges duplicate shared-edge crossings before
pairing occupied intervals. The wider grounding/volume and source-cue timing
regressions passed. Shiny verification remains skipped as requested.

The final canonical GLB is 800,780 bytes. Blender regeneration is byte-identical
and the production native executable built successfully. Current full-scale
walking-encounter staging remains unresolved: Route 29 found 78 supported
ground candidates but rejected every camera composition; Route 38 found no
supported complete-body candidates. A wider 1400 by 700 Route 29 viewport
still rejected the camera. These are actual native rejections, not accepted
visual captures. No full-size native 3D/2D acceptance is claimed for this skin.
Source dimensions, animated footprint, terrain admission and grass height were
kept intact. The previously recorded static-era arena capture is historical evidence
for the earlier model only. Large-bird native staging and browser acceptance
remain open.


A subsequent camera follow-up corrected radial-versus-view-depth far-plane
units and added a second 60/68/75-degree search only after all ordinary stages
fail. All 48 terrain and three battle-view integration checks pass; Articuno's
native 3D/2D regression also passes. Ho-Oh still fails the current Route 29
retained-neighborhood/scenery proof in both tiers. Full-size native acceptance
remains open; no geometry or scale change was used to force it through.


## Initial static sculpture history

`tools/ho_oh_sculpt.py` authors an original individual battle sculpture. Its reviewed
candidate is installed as a dedicated battle model and dispatched by the pure
battle-species builder. Ho-Oh enters the exact registry and leaves the shared
species list. Its Rust routing regression passed;
its fresh muted native GPU capture has passed and been inspected. The original
`actor_props/icon_ho_oh.mesh.json` remains unchanged. Its current SHA-256 is
`0970611b867db722dea1d55706d4db9e0c643ec2aaf30238e68514038d22012e`.

The [official Ho-Oh artwork](https://sg.portal-pokemon.com/pokedex/0250/)
was visually inspected in an image-only background browser tab, then closed.
The [Space Bird source review](legendary-bird-rig-research.md) yielded no
accessible model file or validated skeleton. The initial sculpture was original and
static; no third-party geometry or reference pixels were imported.

A tapered red body supports a fitted broad ivory belly, long curved red
throat, fitted green collar and thin ivory collar edge. The compact red head
has small fitted white eyes, red irises and dark pupils. Five rounded gold
crest petals root in the skull. A curved gold upper beak and open lower jaw
frame a recessed dark oral lining. Two rising red wings carry 24 separate
broad flight feathers and twelve smaller warm shoulder coverts. Every flight
feather has an ivory terminal band and green tip on both sides, fitted to its
actual parent skin. Nine broad gold tail vanes fan backward, each with pale
gold tips on both sides. Ivory feathered thighs support strong dark shins,
feet, three forward digits and a rear digit per foot, with ivory talon hooks.

The first software views exposed pointed tips and thin legs. Flight vanes
now use rounded blunt ends and a broader fan. The feet have thicker grounded
knuckles and digits. The authoring check caught a green tip beyond its white
parent; green outlines now clip from the actual inset white outline. The
revised candidate passed skin, root, normals and closed-solid checks but
exceeded the 120,000-triangle budget. Reducing the rounded curve sampling
preserved its silhouette while bringing it below that limit. A test code
error was also corrected before the final full run; it supplies no geometry
passing evidence.

The initial static source had 207 parts and 118,516 triangles, height 1.45 and
floor zero. Its SHA-256 is
`01a2f59baf066890372fa1d76bffb0b94776841ed0881da045611f290cceec2b`.
Final front, oblique and rear software renders exited zero and were individually
inspected. Thirteen final candidate tests passed in 29.229 seconds; the installed
canonical run passed all thirteen in 28.496 seconds. These independently
check topology, face normals, height, grounding, budgets, real anatomical roots,
nonadjacent crossings and fitted color vertices, triangle centers and edges.
Both actual empty mouth space and filled-mouth rejection are required, along
with a visible recessed oral lining and a detached-flight-feather negative
control. A Rust regression requires the exact battle path to resolve the new
geometry, direct icon routing to remain unchanged, and Ho-Oh to occur exactly
once in the dedicated registry. Canonical layout validation passed, and storage
validation passed for the current shared tree of 575 raw JSON model documents.

The current all-251 Rust regression passed with one matching test, zero
failures, 1.16 seconds and 1,266 filtered tests. The compiled test binary
contains the full current hashes for Moltres, Lugia and Ho-Oh. The fresh
production build exited zero after 21 minutes 40 seconds including cache waits.
Ho-Oh's separate dedicated-model/icon-routing regression passed with one
matching test, zero failures, 0.05 seconds and 1,271 filtered tests. Other
package targets contained no matching tests.

The fresh production executable contains the full canonical Ho-Oh hash and
compiled mute switch. Its muted owned-image GPU capture exited zero in 2.67
seconds; renderer write time was 0.88 seconds. The 2,000 by 1,400 image has
34,314 colors and was inspected in full. Both red flight fans show their white
bands and green tips, and the crest, ivory belly, gold tail and strong dark
feet are visible with the arena, opponent and battle HUD. The far wing is
foreshortened; software front and rear views cover its outline and color layers.
Canonical model hash and executable inode, size and modification time were
unchanged before and after capture and across all three replacement trials.

All trials set `CRYSTAL_PREVIEW_MUTE=1` before process creation. Both music and
effect sinks start at zero before source append, and music refresh remains
zero. No system volume or other application was changed. The full 251-species
appearance objective and separate live-window capture issue remain open.
Generated captures and logs are ignored verification scratch, not deliverables.
