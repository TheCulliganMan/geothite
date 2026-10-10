# PR #5 visual polish

The goal is to polish the whole PR. Model availability, topology checks and
successful compilation do not establish finished art. The catalog currently
contains 571 static JSON models, eight individual animated GLBs and 75 human
scenes in the shared catalog: 654 neutral review entries. This includes
scenery variants and non-Pokémon props, not 654 distinct Pokémon.

## Review method

`tools/render-model-review.py` renders actual canonical geometry, node
transforms, normals and linear materials into ignored review sheets. It needs
NumPy and Pillow for inspection only; neither becomes a game dependency.

```sh
python3 tools/render-model-review.py crates/crystal-voxel-view/models \
  --size 240 --out target/model-polish/all
```

These are orthographic neutral asset inspections. They do not replace animated
pose review, the production camera, actual map composition or GPU gameplay
verification. In-game readability at normal physical size is a final gate.

Each asset needs a recognizable silhouette, correct proportions, coherent
anatomy or construction, fitted surface details, restrained source-appropriate
color and deliberate normals. Shared primitives are useful construction tools;
the final forms must distinguish species and character identities. Original
map footprints, source collision, actor placement, physical species dimensions,
authored scripts and attack effects remain authoritative.

## Current assessment

The baseline complementary-species sheets have been visually inspected. Many families
still need reconstruction, rather than a material adjustment: detached ball
heads and bellies, cylindrical limbs, protruding eyeballs and repeated family
silhouettes dominate the existing first pass. Clear baseline examples include the
Machop family (now reconstructed below), Abra/Alakazam, Drowzee/Hypno, evolution pairs such as
Croconaw/Feraligatr, and the generic quadrupeds. The more recently sculpted
starters, Gengar and birds also require final camera and motion review.

Representative human, furniture and exterior sheets have been inspected. Human
stance, face fitting, garment silhouettes and distinguishing body proportions
remain open. Furniture must be assessed in its room as well as in isolation;
flat repeated façades, interior trim and architecture need map-specific review.
The full 28-sheet neutral catalog is generated, but uninspected sheets are not
counted as reviewed or polished.

| Workstream | Required finish | Status |
| --- | --- | --- |
| All 251 Pokémon | Individual silhouettes/anatomy/materials; appropriate animation and special states; normal-size battle review | Open |
| All 75 humans | Distinct proportions/outfits/faces; planted locomotion and interactions; map-context review | Open |
| Furniture and machinery | Coherent constructions, clear openings, fitted controls, actor clearance and interactions | Open |
| Buildings and exterior props | Distinct source layouts, intentional trim/materials, camera/reveal behavior and approaches | Open |
| Interiors, floors and boundaries | Complete joined forms, source-faithful surfaces, puzzle states, collision and room composition | Open |
| Battle presentation | Art readability, cue-aligned motion, effects placement, switch/faint/capture/return and physical scale | Open |
| Native/browser integration | Actual gameplay and GPU review, portable source regeneration, content-exclusion and full regressions | Open |

## Ampharos reconstruction

Ampharos now has one continuous pear torso, slender neck and rounded muzzle,
instead of stacked body/head spheres and a neck rod. Broad tapered flippers
replace stick arms with ball hands. Its eyes, forehead beacon, subtle smile and
ivory chest fit the surface. Rounded striped horns, two neck bands and a curved
striped tail carry the species silhouette; broad paired feet stay on Y=0.

`tools/ampharos_sculpt.py` is its standard-library authoring recipe. The existing
`ampharos.mesh.json` remains its sole canonical geometry file; the legacy
Ampharos branch is removed from the Blender batch builder. The builder delegates
to the recipe. Explicit seven-decimal source precision removes near-zero libm
noise from normals. The unchanged 1.35-unit height retains the existing runtime
physical scaling contract. There are 26 closed anatomy parts, 4,700 triangles
and 2,402 vertices.

```sh
python3 tools/ampharos_sculpt.py
python3 tools/test_ampharos_sculpt.py
```

Four stored-model checks pass: exact recipe reproduction, one connected oriented
closed solid per part, finite unit normals agreeing with real triangle winding,
outward chest faces, paired anatomy and grounded height. Model inspection found
and corrected an inverted belly-panel edge closure before publication. Neutral
before/after views are reviewed. Final in-game battle review and articulation
remain open; this increment does not finish Ampharos or the broader catalog.

## Machop-family reconstruction

The three previous models shared a sphere head, cylindrical limbs and nearly
the same torso. Their replacements have distinct anatomical profiles:

- Machop has a larger head relative to its narrower young body, a curved tail,
  lower arms, broad palms and planted three-toed feet.
- Machoke has a long-legged, angular wrestler silhouette, shaped deltoids,
  narrower elbows, fitted red arm markings and a waist-following belt/trunks.
- Machamp has a broader chest and shoulders, separate upper and lower arm
  attachments, four independent squared hands, thick thighs and broad lips.

Each torso joins pelvis, waist, chest, neck, jaw and cranium into one closed
surface. Pectoral relief is part of that surface; the initial detached chest
volumes were rejected during inspection. All faces have fitted almond eyes
and shallow pupils, with three swept cranial plates rather than pointed
ornaments. The static source heights stay 0.85/1.15/1.45 units, retaining the
existing pack-derived physical scale. No animation or attack event is added.

`tools/machop_sculpt.py` authors all three canonical JSON models. The batch
builder delegates to it and no longer contains the old Machop-family geometry.
`tools/sculpt_geometry.py` shares closed surface construction with Ampharos;
the refactor reproduces Ampharos's canonical JSON exactly. Species anatomy
and palettes remain in their own recipes.

| Species | Closed parts | Triangles |
| --- | ---: | ---: |
| Machop | 32 | 3,628 |
| Machoke | 41 | 4,144 |
| Machamp | 51 | 5,216 |

The four new family checks verify actual stored topology, oriented closure,
finite unit normals agreeing with triangles, source reproduction, scale,
planted toes, fitted eyes, two/four arm anatomy and distinct proportional
widths. They caught inward-facing smooth normals at two early elbow shapes;
the elbow geometry was revised before acceptance. Front, three-quarter and
rear model renders have been reviewed. The existing species validator passes
on the four remodeled species as an explicitly scoped subset. All 957 voxel
tests pass with three existing ignored checks. The four Ampharos checks also
continue to pass.

```sh
python3 tools/machop_sculpt.py
python3 tools/test_machop_sculpt.py
```

This is a static anatomy increment. In-game camera/readability, animated
clearance and species articulation remain open for all three models, alongside
the full catalog's remaining workstreams.
