# Krabby and Kingler sculptures and rigs

The crab models retain their orange upper shells, ivory bellies, raised eyes,
cheek peaks and four walking legs. The pincers now have broad, flattened palms
and tapered movable jaws. Kingler has a dominant claw more than twice the
height of its smaller pincer, five crown points and two additional ivory jaw
teeth. Krabby retains equal claws and two crown points. Four pointed ivory
walking toes provide explicit ground contact. Visual references are the official
[Krabby](https://www.pokemon.com/us/pokedex/krabby) and
[Kingler](https://www.pokemon.com/us/pokedex/kingler) artwork. Reference images
remain outside the shipped assets.

`tools/crab_sculpt.py` authors the geometry and `tools/crab_glb.py` exports the
canonical skins. Legacy static JSON files are removed. The runtime, builder and
model checker all use the same canonical GLBs.

| Species | Parts | Vertices | Triangles | Joints | Accessors | Bytes | Authoring height | Pack height |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Krabby | 31 | 17,883 | 25,576 | 12 | 140 | 678,056 | 0.97 | 0.4064 m |
| Kingler | 36 | 18,026 | 25,908 | 12 | 156 | 674,292 | 0.97 | 1.2954 m |

The verified local pack supplies height digits 104 (RIVER CRAB) and 403
(PINCER), respectively. Neutral geometry digests:

- Krabby: `5f8f12d336b23cecec7ad64a1ef4ab89f387324289654f1fdbc722df900ff5ef`.
- Kingler: `77ef337fdce8469cd785876b536b3c92953ebd6a1b026f7b12001c2a592cfc36`.

The floor root remains fixed. A yaw-only body carries the complete shell and
face. Each arm blends from the shell into its own carrier and wrist. The
palm follows the wrist; a child jaw carrier closes independently. Four blended
leg carriers keep their knees attached while the walking toes stay planted
in battle. World locomotion groups each complete leg, knee and toe into one
walking chain. Idle, attack and hit use rotation-only keys, neutral endpoints
and the existing source-cue timing.

Eleven Python checks pass: canonical bounded containers, hierarchy and inverse
binds, anatomical owners, occupied roots through motion with a detached negative
control, floor contact, winding, strain, independent jaws, source-cue metadata,
closed outward solids, stored normals, family anatomy and fitted face clearance.
The existing 20,000-vertex, 40,000-triangle, 64-primitive, 256-accessor and 1 MiB
limits are retained. Blender generation is byte-identical. Front, side and rear
poses of all three clips were inspected for both species. Shiny checks are
skipped at the user's request.

Four focused Rust checks pass: both verified-size GPU paths, production
anatomical ownership and routing, planted battle toes, animated culling bounds,
independent jaws, and the 64-phase world walking regression. The existing
Paras/Parasect contact regression also passes in the same run.

Rebuild and check:

```sh
python3 tools/crab_sculpt.py --output crates/crystal-voxel-view/models/battle_species
python3 -m unittest discover -s tools -p 'test_crab*.py'
```

The rebuilt production client passed normal Route 29 terrain admission for both
species at original pack size. The paired 3D/2D Krabby encounter against Pidgey
preserves core frame 3412, checksum 2771847911, LCD animation frame 4151 and
virtual time 76.7544295. The Kingler encounter against Pidgey preserves core
frame 3538, checksum 1460042242, LCD animation frame 4257 and virtual time
79.629184374. Both inspected pairs show the full pincer silhouettes, clear
species labels and toes meeting the terrain. Kingler's dominant claw remains
much larger than its other pincer. Neither capture reports shader errors.
Evidence is ignored under `target/pokemon-native-review/krabby-normal` and
`target/pokemon-native-review/kingler-normal`.

The captures select the existing supported angle with
`CRYSTAL_BATTLE_REVIEW_YAW=-110`; normal scale, source support, scenery omission
and whole-body/frustum checks remain active. The two shared grounded/volumetric
and source-cue pose timing regressions pass. Grass height is untouched. Browser
visual acceptance remains open.
