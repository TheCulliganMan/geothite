# Snubbull and Granbull sculptures and standard skins

`tools/bulldog_sculpt.py` authors the distinct bulldogs; `tools/bulldog_glb.py`
exports canonical standard skins. The species builder emits GLBs and retires
duplicate static JSON. Production neutral mesh lookup and GPU animation use
the same assets.

Snubbull retains all 39 parts, 16,555 vertices and 19,516 triangles. Its GLB is
635,380 bytes with 13 joints and 185 accessors. Digest:
`70065daa04c182094aef38bc6772adf416da67efaf666cc9bffe28b7b2854412`.
Granbull retains all 42 parts, 14,140 vertices and 19,048 triangles. Its GLB is
576,372 bytes with 13 joints and 193 accessors. Digest:
`386b9e7d3742f158136ec0acd0401377fefafc9dc1cf76afbdebbebb5467276e`.
Every original f32 position, normal and triangle order remains exact. Both fit
the unchanged 20,000-vertex, 256-accessor and one-MiB loader budgets.

Actual local normal front/back sprites and RGB555 palettes were inspected.
Snubbull uses coral (231,115,107), blue-gray (57,82,107), white and black.
Granbull uses purple (189,99,181), dark tongue purple (123,33,140), white and
black. Historical part labels retain pink/blue/ivory/rosy wording. Pack heights
200 and 407 give 0.6096 m and 1.397 m uniform production scales. Both retain
the normalized 1.06 height and floor origin. No extracted sprites ship with
the models.

An identity root fixes each pair of paws and their four fitted toe grooves.
Shins blend between those supports, leg-middle yaw and torso yaw. Body, neck
and collar surfaces share a head transition; Snubbull's four dress spots stay
rigid to the torso. Each folded ear and its fitted dark tip share an owner.
The fitted facial layers, jowls, upper fangs and muzzle share the head. Lower
jaws articulate independently; Granbull's enormous curved lower tusks stay
rigid to its jaw. Mouth cavities and the fitted tongue remain together on the
head. Fists own their fitted knuckle clefts and snug wrist cuffs, with blended
forearms joining them to the body. Ears, tail, arms, wrists and jaw use
rotation-only idle/attack/hit clips, neutral endpoints and source-cue timing.

Python 3.12 changed float summation and produced negative rather than positive
zero at a few mouth-patch centerline coordinates in Blender. The species-local
patch wrapper now passes an explicit original left-to-right centroid through
the fitting helper's optional `center` argument. Default behavior for other
callers is unchanged. Both focused Blender outputs are byte-identical to the
pinned GLBs; no geometry tolerance or loader validation is relaxed.

All seven stored-skin and seven migrated anatomy checks pass, covering exact
canonical reconstruction, bind identity, budgets, colors, fitted ownership,
planted paws/clefts, coherent jaw/tusks, winding and edge strain, physical joins,
folded ear markings and large canine support. Both focused production Rust
checks pass: GPU skinning at pack sizes and off-grid feature motion, ownership,
fixed support and culling bounds. Front and rear neutral software views were
inspected. Matched normal native 3D/2D reviews against Pidgey pass for both bulldogs.
Snubbull's dress spots, collar, folded ear tips and planted white paws read
clearly. Granbull's scale, curved lower tusks, jaw, cuffs and grounded paws
remain clear against retained grass. Rear details were also inspected in
neutral software views. Snubbull preserves core frame 2806, hash 2193979768,
LCD frame 3541 and time 65.505895167. Granbull preserves core frame 3311,
hash 2711752641, LCD frame 4126 and time 88.313459166. Both paired receipts
report `state_preserved=true`. Captures remain ignored local QA artifacts. Exact sprite-frame
motion, field gaits and browser review remain open. Shiny verification remains
skipped at the user's request.

The wider `every_exact_species_is_grounded_and_volumetric` and
`body_pose_uses_elapsed_idle_time_and_exact_current_attack_progress` regressions
also pass on these skins. They cover catalog geometry and elapsed-pose sampling,
without claiming visual acceptance for unrelated models. The native build
passes after an explicit f32 annotation fixes a shared dungeon UV closure.
