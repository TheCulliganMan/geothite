# Meganium papercraft sculpture and animation

Meganium now has a connected mature quadruped body, a sloping two-stage neck,
a projecting muzzle and a fitted opening lower jaw. Six folded oval petals
form one flower around the neck. Paired curved golden antennae and a low,
tapered tail replace the earlier rods and disconnected spike collar.

The canonical asset is `models/battle_species/meganium.glb` under
`crates/crystal-voxel-view`: 33 anatomy parts, 4,520 triangles, 11,353 split-normal
vertices and 23 named joints. The previous same-species JSON is removed.
No duplicate STL, neutral mesh or generated Blender file is added. The older
multi-species Blender archive remains historical authoring material.

The standard glTF skin and idle/attack/hit rotations are authored in
`tools/meganium_sculpt.py` and `tools/meganium_glb.py`. Rebuild without Blender
or a content pack:

```sh
python3 tools/meganium_glb.py
python3 tools/test_meganium_glb.py
python3 tools/check-battle-species.py
python3 tools/check-johto-models.py
```

The pelvis/chest/neck/head chain, four support legs, two-joint tail, paired
two-joint antennae, six independently flexing petals and jaw give Meganium its
own mechanics. Soles and claws remain rooted. The jaw stops at neutral closure;
the lower petal opens away from the chest during an attack. Idle is periodic,
and action endpoints return to neutral for blending.

Neutral full height stays 1.48 authoring units and minimum Y stays zero.
The original pack height digits 511 convert to 5 feet 11 inches, or 1.8034 m,
through the existing runtime scale. Animated bounds only control visibility.
Body animation adds no projectiles or flash/sound events: original move
programs and the authoritative battle engine continue to own those events,
timing and damage. Camera and encounter positions are unchanged.

Stored-byte regressions cover deterministic export, topology and winding,
normalized skinning, fixed contacts, loop seams and blends, facial fit in
head-local space, deformation bounds and appendage clearance. The regression
for the jaw/lower petal rejects the earlier overclosed/intersecting key poses.
Runtime tests cover shared GPU resources, real skin bindings, physical scale
and dense animation visibility bounds.

The save-free native fixture accepts `--enemy-gust --starter meganium`.
Meganium uses its natural minimum evolution level 32 and retains its earlier
source-generated Razor Leaf learnset, representing declining Body Slam before
evolution. Its greater speed determines the first action through the ordinary
controller; the opponent uses its real trainer AI. Tests retain legal PP,
HP damage and survival for both actors. Body Slam still uses the honest classic
presentation fallback for its unsupported deformation; this fixture change does
not broaden move-renderer coverage. This is one species increment, not a
claim that all 251 species have completed the same art and animation review.
