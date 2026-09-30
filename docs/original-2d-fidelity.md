# Original Crystal presentation is the reference

The 3D conversion must follow the original 2D designs, layouts, proportions,
palettes, poses, movement, effect sequences and timing. Added depth must not
introduce new choreography, decoration or effects. Existing modeled coverage
and performance measurements do not certify visual fidelity.

The current presentation is still a work in progress. The following matrix
separates source authority from appearance that still needs correction or review.

| Area | Current evidence | Status | Required next check or correction |
| --- | --- | --- | --- |
| Battle commands, PP, HP, damage and recharge | Production controller and battle engine; controller regressions and actual damage captures | Source authority retained | Keep all visual work outside these rules |
| Animation program clock, OAM, BGP/OBP and source displacement | Live source interpreter feeds `capture_immersive_source_frame`; recorded source frames/objects are retained | Source data retained | Compare original 2D and 3D views on matching authoritative frames |
| Battle camera | `battle_view::camera_pose` adds time-based sway | Known additional motion | Remove source-independent movement; retain only source-driven displacement |
| Battler pose and orientation | `battle_view::actor_pose` adds breathing, facing bias and generic transitions/recoil | Known additions | Match original facing and pose changes; audit every generic pose against its source sequence |
| Battle background and platforms | `battle_view::arena_mesh` adds biome scenery and staging | Not established by original battle canvas | Replace unsourced scenery with the original background treatment |
| Psychic and Hyper Beam styling | Source objects/palettes are retained, while `modeled_source_effect_pose` adds volumetric styling and aura geometry | Mixed source data and added styling | Verify shape, location, count and lifetime against original effects; remove unsupported additions |
| Battle HUD | `sync_immersive_battle_ui_layout` relocates panels and omits a prompt | Known layout deviation | Restore source layout/content while keeping it readable at the chosen display scale |
| Species models | All 251 normal species have first-pass geometry; four sculptures received further refinement | Individual fidelity not certified | Compare each silhouette, anatomy, proportion, palette and visible front/back pose with original sprites |
| People and overworld actors | Exact source identity and footing are preserved; authored articulated shapes/materials are used | Art fidelity not certified | Audit head/body proportions, clothing, palette, facing and walking poses against the relevant sprite frames |
| Buildings, props and surfaces | Source-aware complete-object bindings preserve map ownership and space | Placement coverage is not appearance fidelity | Compare original geometry cues, heights, colors and decoration; correct stretching and occlusion |
| Dynamic states and special appearances | Some states retain faithful source-art fallback | 3D coverage remains incomplete | Validate each state independently before replacing its source presentation |
| Sound | Existing source sound path remains authoritative; current demonstration clips are silent | Not verified by the silent clips | Compare source sound events and audio timing separately |
| Surf | Original program, OAM ribbon and palette sequence inspected; original LCD oracle selects vertical scroll (`rSCY`) | Proven current axis mismatch | Correct the existing horizontal mapping to vertical scroll, verify both views, then translate only the original sequence into 3D |

Each asset/effect needs a specific original reference, a documented 3D mapping,
matching-frame visual comparison and an explicit list of remaining differences.
An authored model, positive map count or successful mesh test alone does not
close that review. Source artwork and packs remain external; review captures
are verification artifacts, not bundled game content.

The unpublished special-room batch has additional confirmed identity errors: three
Goldenrod department-store roof binoculars were modeled as potted plants, and
Rocket Base B2F instrument equipment was modeled as a bookcase. Those bindings
must be corrected before that batch is published. The central roof inset also
needs source identification before retaining its current stair interpretation.
