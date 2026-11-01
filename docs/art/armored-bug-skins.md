# Pineco and Forretress coherent armor skins

The existing detailed sculptures now use canonical seven-joint skins.
Neutral positions, corner normals, colors and triangle order remain identical
to the previous static models at f32 precision. Coherent packing substantially
reduces runtime parts while preserving every original source range.

| Species | Vertices / triangles | Source / runtime parts | Joints | Accessors | GLB bytes |
| --- | --- | --- | --- | --- | --- |
| Pineco | 5,418 / 3,900 | 54 / 13 | 7 | 75 | 226,572 |
| Forretress | 15,039 / 14,524 | 48 / 18 | 7 | 94 | 569,292 |

Pineco retains the dark tapered core, three six-plate armor rows, five tall
crown plates, stem and fitted ivory/red gaze. Each row has an independent
control; crown and stem have their own controls. Teal/navy interface vertices
share the same carriers and stay coincident through motion. The core's lowest
vertices remain on the identity floor root, with a blend to the body above.
The complete fitted gaze follows the fully weighted upper core.

Forretress retains the red core, two closed jagged hollow rocky hemispheres,
equatorial belt, annular eye rims, four genuinely hollow tapered barrels,
four recessed bores and supported front/back stone chips. The lower hemisphere
and all its chips stay on the identity root. A rear upper-shell hinge lifts
the upper hemisphere and its chips together. The red core carries the belt,
eyes and four independent barrel hinges. Each barrel and bore share a carrier.
There are no invented feet or new hover motion.

Idle lasts 3.4 seconds / 85 keys. Attack/hit last one second / 65 keys, have
neutral endpoints, rotation-only tracks and source-cue timing. Plate rotation,
stem sway, upper-shell opening and restrained port aiming retain occupied
attachments. Original 20k vertex, 40k triangle, 64 part/joint, 256 accessor and
1 MiB limits remain intact. Authoring height stays 0.93 for both models; local
pack sizes remain 0.6096 / 1.1938 m. Uniform scale and full animated bounds apply.

The local normal front/back sprites were inspected from the external pack.
The existing 3D teal/navy and gray/red palettes are preserved; the pack sprites
use blue-gray and purple/red instead, so exact palette parity is not claimed.
References and source/decoded pose sheets remain ignored under
`target/pokemon-native-review/armored-bug-*`.

The direct CLI, normal background builder and canonical GLBs produce identical
bytes. Both strict Python readers and Rust registry/GPU/world dispatch use the
same assets. Rebuild offscreen:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 tools/armored_bug_sculpt.py \
  --output target/armored-bug-rebuild
/Applications/Blender.app/Contents/MacOS/Blender --background --threads 2 \
  --python tools/build-battle-species.py -- \
  target/armored-bug-rebuild --only PINECO,FORRETRESS
```

Generated rebuild output and packs remain ignored and must never be staged.

Eighteen final stored Python checks pass in 11.019 seconds: ten source checks
and eight independently decoded skin checks. They cover exact source geometry,
closed connected outward solids, finite corner normals/winding, distinct
anatomy, actual occupied roots through poses, coincident plate interfaces,
parent-supported gaze/chips, visible gaze, open shell equator and unfilled
barrel/eye-rim apertures, hierarchy/inverse binds, every moving control,
coherent carriers, fixed ground support, positive posed winding/bounded strain,
neutral endpoints, source cues, names/limits and mutated byte/species/geometry
rejection. Source ray checks now use the existing indexed stored-triangle oracle
with identical barycentric tests/tolerances; no geometry threshold is relaxed.
The original eight baseline source checks also passed before conversion.

| Species | Source SHA256 | Packed SHA256 | GLB SHA256 |
| --- | --- | --- | --- |
| Pineco | `fb4c0dd8521ce539b3b024789b11b05641d5b9c88a12bff7fcc91a159b666d52` | `57da153652ef565881c64f8a0874e22e182a0c1ce7eee3985311a145835167f8` | `abe8c495990b38efe2191befb9fbe3f255d257b6b6d28b8ba145662fcd0b0c29` |
| Forretress | `a55e58ca3cb62964d17ca334965d012468af3c82b80da9b18f4fdb1054229427` | `5dbc71683b7f28a531a606b07e72f102a64dd20fa39fdd3938c2848429d19b5a` | `9024cb0808da635816b4fdf66d96e4207164a75ba9757034fcca60cf0b67c256` |

Four focused Rust `armored_bug_skin` checks pass in 0.98 seconds: GPU admission
at the original pack sizes for each species, registry/canonical identity with
fixed supports across all clips, and legless world locomotion preserving all
canonical bindings. Every world vertex stays above the floor and inside full
locomotion bounds; ground contact remains within 0.005 authoring units.
Both native review clients build in 49.79 seconds. An initial test compile hit
concurrent furniture-test API/import edits; the corrected shared tree passes
without this batch modifying furniture files.

Shared schema/physical-size and elapsed-idle/exact-current-attack-cue checks
pass in 88.00 and 113.59 seconds. Together with 18 stored Python and four
focused Rust checks, this is 24 scoped checks. It is not a full-suite or
gameplay-completion claim.


Six final same-state production 2D/3D pairs and two moving world frames were
inspected inline. Every capture used headless image targets and mute, with no
visible window, GUI review or focus change. Four front/world traces each have
32 synthetic frames across five Route36 positions. Both Route29 battles admit
the complete original 0.6096 / 1.1938 m models with uniform scale, support zero,
unchanged full-envelope/camera/terrain gates and no fallback. Both Pokémon
names are visible in the battle HUDs.

| Species / view | State frame / hash | LCD frame | Virtual seconds |
| --- | --- | --- | --- |
| pineco front | 1842 / 3542073100 | 1849 | 31.654771833 |
| pineco world | 1626 / 1357091974 | 1632 | 27.827735916 |
| pineco battle | 1975 / 3377594635 | 2666 | 45.65368125 |
| forretress front | 1881 / 2981210210 | 1887 | 32.282233625 |
| forretress world | 1676 / 1614682315 | 1684 | 28.8023175 |
| forretress battle | 1865 / 795807293 | 2458 | 42.119209292 |

Complete ignored receipts are at
`target/pokemon-native-review/armored-bug-capture-receipts.json`.
Both binaries contain the exact compressed current GLBs and headless/mute
controls; streamed SHA256, size and modification times remained identical
before and after all captures. Binary identities:

- `render_at_location`: `4ecc31dc36f35d900a851d820ff4a6d5bc73f737a4e43e78ea9ae5beaccb4f9b` (407,286,368 bytes).
- `immersive_battle_3d`: `96e88e82d217341eddbc4a5bd05893430b7858f7fddcbace3825e23a8cab70f9` (407,051,832 bytes).

Pineco's eyes remain visible beneath the crown from the front, while the
layered silhouette remains readable at battle distance. Small gaze details
fade at world scale and far-side plates are occluded. Its narrow core-tip
support can still look suspended under soft contact shadows even though the
root contacts and production placement are at zero. Forretress's eyes, rocky
shell and bores read clearly; front foliage overlaps one near port and side
views hide the far ports/gaze. Contact-shadow readability, especially for the
legless silhouettes, remains an open visual defect rather than a full visual
acceptance claim. Broader browser/camera/material/scenery coverage remains open.
No shiny verification, grass-height, AA/filtering, audio, pack publication or
gameplay changes belong to this batch.

Registration reaches 249 skin variants and 26 static species files.
Full-roster model and rig improvement remains active, including grounding
readability beyond the physical support checks.

The later [battle grounding follow-up](battle-grounding.md) adds faint shading
beneath low armor while preserving these exact model files and physical
contacts. New headless/muted battle pairs improve the narrow-tip floor cue;
world/browser/hardware-shadow coverage remains open.
