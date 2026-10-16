# Bellossom dancer sculpture

`tools/bellossom_sculpt.py` authors the original canonical Bellossom asset.
The battle species builder dispatches it before Blender batching and rejects
the old shared flower path. Sunkern and Sunflora retain their dedicated recipes.
Bellossom keeps its 0.95 presentation height and floor origin.

The dancer has a continuous small green torso and broad green head. Short
curved leaf-like arms form a raised arm and an arm crossing the chest. Glossy
dark eyes carry two white catchlights each; fitted green cheek patches frame a white
open smile, and the facial patches follow the actual stored head surface.

The skirt has fifteen closed curved leaves: five broad red outer leaves,
five alternating white inner leaves, and five darker rear leaves. Each leaf
has a rounded transverse section, a narrow root inside the torso, a broad
middle and a tapered pointed end. These replace the old sparse flat triangles.

Two crown flowers have six broad rounded red petals each. Their faces
are subdivided and gently curled. Thick green cups have raised annular lips,
descending inner walls and recessed dark floors. The petal roots remain below
the cavity floors, avoiding the initial visible petal intersections. The final
asset contains 45 closed parts and 26,980 triangles.

Silhouette and palette were checked against the
[official Bellossom artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/182.png).
External research found CC Attribution listings for
[Bellossom](https://sketchfab.com/3d-models/bellossom-8b77abfbcc9c472ba68d72609b682bc5)
and [a fan-authored Bellossom](https://sketchfab.com/3d-models/bellossom-9b2fb72a8a3c464aa6427bdad690d90b).
The metadata did not establish an original usable rig. No external mesh or
textures were imported. The initial sculpture used the static native base-color path. The standard
skin below supersedes it.

`python3 -m unittest discover -s tools -p test_bellossom_sculpt.py` checks
closed connected topology, winding, finite unit normals, deterministic recipe
parity, height, budget, part counts, fitted facial geometry and front triangle
clearance, actual torso/arm/skirt roots, all twelve petal roots, both crown
attachments and the actual recessed floors with clearance across each well.
Front/rear software reviews and native battle captures assess model
presentation, not gameplay behavior.

Initial static verification passed all eight Bellossom regressions. The final front and
rear software views and native battle capture were visually inspected. The Rust
catalog regression passed for the 220-species battle catalog; it verifies grounding and volume
rather than visual quality. Layout and storage validation passed for all 571
JSON assets. Temporary review images, reference artwork and logs were removed
after inspection.


## Standard skin and pack palette (2026-10-14)

The sole canonical runtime model is now `battle_species/bellossom.glb`, exported
by `tools/bellossom_glb.py`; duplicate static JSON is retired. The original stored
JSON first matched its original recipe exactly. All 30 non-skirt parts retain
indexed f32 triangle positions; petal normals now share connected smooth patches.
The fifteen skirt leaves keep their analytic controls but use 16 transverse
samples and 21 rows instead of 20 and 25. This reduces the original 29,611
vertices/31,900 triangles to 18,524 vertices/26,980 triangles. No existing loader
limit is raised. The new asset is 739,892 bytes with 25 joints and 227 accessors.

The supplied pack's front/back/normal palettes agree: RGB555 red (27,8,6) and
green (6,17,8) expand to RGB8 (222,66,49) and (49,140,66), alongside white and
black. Its inspected front sprite establishes the red/white skirt, green body
and crown centers. These replace the prior green/gold skirt and pale-green body.
The dark rear skirt remains an authored shade of the source red. Height digits
104 establish 0.4064 m (1′4″), preserving the 0.95-unit authored span.

The standard rig articulates torso, neck, head, both two-stage arms, fifteen
individual skirt leaves and two complete flower crowns. Each crown's cup,
petals and recessed inset share one rigid owner. Face parts share the head.
The lowest skirt tips stay root-owned; rotation-only idle/attack/hit clips have
neutral endpoints and a stationary root. Crown and arm-tip responses lag the
body brace. These are authored anatomical responses, not verified source-frame
motion or gaits. Shinies reuse this model and the existing pack palette path.

Neutral digest:
`21bc079e44b6abd8d1012172d7fb1c909e679d18d143ceafc613241861785f57`.
Six independent stored-skin checks, all eight anatomy checks and byte-identical
Blender regeneration pass. Both fresh Rust registry/GPU checks pass, including all-vertex off-grid
animation bounds, floor clearance, planted tips and coherent crown ownership.
All 81 standard variant schemas pass, and full art validation passes for
61 editable sources, 609 runtime models in 609 files and 758 embedded references.
The native build and paired normal/shiny visual review are pending. The standard registry
contains 81 variants spanning 56 species, plus the separate legacy Pidgeotto path.


A neutral front/rear software inspection caught black flower-well insets reading
as extra eyes. The inspected source sprite instead establishes green centers.
Both insets now use the verified green. An executed before/after audit against
the earlier asset in this task's build cache proves the complete binary buffer,
node/skin/animation definitions and all mesh schema identical; only two material
assignments and the neutral digest changed. The corrected front software view
was inspected, six stored tests pass again, and Blender reproduces the corrected
file byte-for-byte. The eight anatomy checks, both Rust integration checks, all
81 schemas and broad art validation above preceded this material-only revision.
Both final-revision Rust registry/GPU checks now pass against the corrected
asset. The final native rebuild passes. The paired production review is recorded below. Temporary software previews and focused builder output
were removed after inspection.


## Matched native field review (2026-10-14)

Both normal/shiny Route 29 3D/2D pairs were personally inspected and preserve
state. Normal versus Pidgey: core frame 3073, checksum 430773371, LCD frame 3872,
time 76.65057125. Shiny versus Sentret: core frame 3428, checksum 471454070,
LCD frame 4273, time 86.457739664. The normal green body/centers and red/white
skirt are readable; the shiny blue body/centers and orange-red flowers follow
the source palette change on the same mesh, joints and clips. Face patches,
arms, skirt layers and both crowns stay attached. Physical height is unchanged.
The two encounters use different camera compositions; each individual 3D/2D
pair shares its recorded state.

This accepts native rest-pose/rig/palette integration. The contact shadow remains
weak, so complete grounding polish is not accepted. Detailed source-frame motion,
gaits, browser review and the all-species objective remain open. QA ran under
concurrent load without a shared graphics lease; capture durations are not
performance benchmarks. Captures, receipts and logs remain ignored.

The executed field-review command uses `CRYSTAL_RENDER_TEST_PARTY_SPECIES=BELLOSSOM`,
`CRYSTAL_RENDER_TEST_PARTY_SHINY=0` or `1`, `CRYSTAL_RENDER_TEST_WALK_INPUT=synthetic`,
`CRYSTAL_RENDER_TEST_HOLD_BATTLE=1`, `CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`,
`CRYSTAL_RENDER_TEST_CAPTURE_TIMEOUT_SECS=300` and `CRYSTAL_PREVIEW_MUTE=1` with
`target/pokemon-models/debug/examples/immersive_battle_3d`, the local
`content-packs/core-modular.browser.crystalpack`, `--walking-encounter`,
`--walk` set to `RALA` repeated twenty times, `--size 1000x700`, and the paired
`--screenshot`/`--second` paths. This selects only a disposable party lead;
controller movement, encounter rolls, scripts and retained-map admission remain
authoritative.
