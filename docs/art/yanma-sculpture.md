# Yanma dragonfly sculpture and standard skin

`tools/yanma_sculpt.py` authors the anatomy and `tools/yanma_glb.py` exports
canonical `models/battle_species/yanma.glb`. The species builder emits the GLB
and removes duplicate static JSON. There are 66 individually named source parts,
packed into 58 runtime primitives by joining each wing's three same-material
veins. Top-level source ranges recover every closed tube and its original order
for authoring checks. Primitive metadata retains the production loader's strict
`part` schema; no loader limit or validation rule is relaxed.

The original 20,268 vertices exceeded the skin loader's 20,000-vertex limit.
Reducing the small thorax's radial sampling and the tiny vein cross-sections
produces 19,976 vertices and 26,452 triangles. The 41 non-wing, non-thorax parts preserve exact
stored f32 positions, normals and triangle order. All twelve toes retain their
original geometry and ground contact. Shared identical normal, skin and
animation accessors keep the GLB at 243 accessors, below the existing 256 limit.
The final file is 755,228 bytes, with 16 joints and rest digest
`53cceeb9aa5759c6c8bace4f6e9dcb73b4f0d839dd6657e14b814d767c2c9711`.
The separately reviewed packed-runtime digest is
`85a1c4865714a1dd738973cce3244a9b4708e155890fe20061208182eed8114b`.
The loader hashes the 58 packed primitives; source reconstruction retains the
66-part source digest. Both forms remain independently pinned and checked.

Actual local Crystal front/back sprites and normal palettes were inspected.
RGB555 green (9,26,10) becomes sRGB8 (74,214,82), red (20,7,7) becomes
(165,57,57), with white and black. Head housings and limbs use green; the body,
wing bands and terminal paddles use red. Wings and glints are white, with black
lenses, veins, mouth and abdomen collars. Historical names retain gray/orange
labels for migration continuity. Geometry helpers convert sRGB to linear once.

The fitted head, lenses, glints, crown, mandibles and mouth share one owner.
Each of four complete wings, both fitted panels and its three veins share a
rigid wing owner, preserving their surface fit while flapping. Six leg middles
articulate between fixed forked toes and the yawing thorax. Three abdomen stages
carry the fitted collars and coherent terminal cap, fins and prongs. Idle,
attack and hit clips have an identity root, rotations only, neutral endpoints,
and delayed rear-wing/abdomen response. Authoritative battle cues own timing.

The local pack's size 311 means 3′11″ (1.1938 m). The renderer now uses Yanma's
nose-to-tail Z reference, following the long-body layout rule used by other
long-bodied species. This avoids applying that listed body dimension to the
raised-tail vertical span. The normalized 0.95 vertical span and floor origin
remain unchanged; scaling remains uniform and ratio-preserving.

Seven independent stored-skin and nine migrated anatomy checks pass, including
canonical reconstruction of all source parts, bind identity, source colors,
coherent wing coatings, fixed toes, clip endpoints, winding/edge strain, closed
geometry, actual surface fitting, six leg/twelve toe roots and collar contacts.
Neutral front and rear software views were inspected. Final focused Blender output is byte-identical and emits no duplicate JSON.
Both final production Rust checks pass: GPU skinning at the pack length and
off-grid sampling for coated wings, fixed toes, tail motion and culling bounds.
The native build passes. Matched normal 3D/2D review against Pidgey
shows the upward wing pairs and red panels clearly; the green head/legs, red
segmented tail and fixed toe contacts remain coherent. Near-side wings still
foreshorten at this angle. The paired receipt preserves core frame 2398, hash
469961496, LCD frame 3092 and virtual time 59.517074416
(`state_preserved=true`). Captures are ignored local QA artifacts. Exact
sprite-frame wingbeats, flight/field gaits and browser review remain open.
Shiny-specific checks and captures remain skipped at the user's request.

The two wider model regressions also pass on the corrected packed asset:
`every_exact_species_is_grounded_and_volumetric` and
`body_pose_uses_elapsed_idle_time_and_exact_current_attack_progress`. These
cover neutral geometry and elapsed pose sampling; they do not establish visual
acceptance for the whole catalog.

The front wings rest at 12 degrees and rear wings at 18 degrees of upward
dihedral. Every sail, both fitted panels and all three veins rotate together;
surface-fitting checks project along each actual wing normal. This exposes the
panels at low production camera elevations while retaining the floor origin.
