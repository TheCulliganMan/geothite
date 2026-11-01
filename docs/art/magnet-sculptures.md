# Magnemite and Magneton sculptures

Current assets use the canonical skins under
[mechanical rigs](#canonical-mechanical-rigs). The original static review
below records the initial anatomy.

## Original static anatomy review

Original static models replace the thin rod magnets and oversized pupils with
smooth blue-silver bodies, broad white eyes, tiny pupils, cross-slotted face
bolts and threaded crown screws. Each magnet is a closed beveled horseshoe
with an outward opening and solid red and blue terminal blocks. Magneton uses
three physically intersecting bodies, with the lower units turned outward.

`tools/magnet_sculpt.py` authors both models without Blender or an external
asset. Magnemite has 18 parts and 19,316 triangles; Magneton has 54 parts and
57,948 triangles. Both retain their existing 0.96 height and ground at zero.
The main battle builder uses this recipe and bypasses its obsolete callback.

`tools/test_magnet_sculpt.py` independently checks closed, connected, outward
parts; finite unit normals and face winding; exact recipe parity, height and
triangle budgets; complete mechanical details; actual solid intersections
between screws, bodies, magnets and pole blocks; all three Magneton body
joins; surface-fitted eye and screw markings; and empty outward notches
between solid magnet arms. The first preview exposed incorrectly oriented
bevel walls; the topology test rejected them, and the wall winding was fixed.

All nine stored-mesh tests passed in 71.316 seconds. Final front, oblique and
rear previews were inspected. The installed canonical files match the tested
candidate bytes exactly. Layout validation passed, and storage validation
passed for 569 raw JSON model documents. Temporary candidates, reference
images, previews and logs were removed.

Fresh native presentation remains incomplete: the rebuilt reviewer returned
blank battle captures, as documented in [native review notes](native-pokemon-review.md).
The separate Rust all-251 loaded-mesh check passed after these replacements.
Software previews do not establish native integration or gameplay. The full visual
goal and usable rig search remain active. Temporary species candidates,
reference artwork, software renders and test logs were removed.

[EmblemSauce's rigged Magnemite](https://emblemsauce.itch.io/3d-rigged-pokemon-model)
advertises a freely downloadable rig but permits educational and portfolio
use and prohibits commercial use. Those terms do not establish permission to
redistribute its raw game asset in this repository. No source asset was
imported; the production sculptures remain original and static.


## Canonical mechanical rigs

`tools/magnet_glb.py` now writes both canonical skins. Runtime routing, strict
model readers and the Blender builder use the GLBs; old static JSON duplicates
are removed. Adaptive fitted eyes, simpler flat slot caps and smooth helical
thread normals remove redundant sampling without shrinking the models or
changing their open magnet silhouettes. Eye shading follows the actual parent
surfaces. Every original mechanical part remains, including the 3.5-turn crown
thread, broad eye with tiny pupil, three dark screw crosses per orb and all
red/blue pole blocks.

| Species | Logical/runtime parts | Vertices | Triangles | Joints | Accessors | Bytes | Authoring height | Verified pack height |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Magnemite | 18/9 | 5,966 | 9,140 | 7 | 64 | 286,008 | 0.96 | 0.3048 m |
| Magneton | 54/27 | 17,898 | 27,420 | 19 | 152 | 687,728 | 0.96 | 0.9906 m |

Material packing retains source order/ranges and computes influences from each
original part before grouping. The independent reader splits stored buffers
back into every logical part. Existing 20,000-vertex, 40,000-triangle,
64-primitive, 256-accessor and 1 MiB limits remain unchanged. Geometry pins:

- Magnemite logical: `616a56a4324733a3df92c7c5c58060ec77941d02839157b72045301df1fe43b3`
- Magnemite packed: `fcd9ef30d31caba9b51c6fc627b4bf0b68a20edf961777c9477acdb7b96904ce`
- Magneton logical: `ed29fd704f19641beba12fb100013b102bc69d55a3e8d95e45871ed52c8607e0`
- Magneton packed: `432c67fcf83876db5de5b5e9c0dc9ae27df2a6180760e7a0325e394dfccb7f8f`

Each orb has one body carrier with a coherent eye, two independent magnet
hinges, a crown carrier and two face-bolt carriers. Colored pole blocks share
their own horseshoe's carrier. Shafts, threads, caps and crown slots share the
crown carrier, which twists around the actual screw axis after each original
unit's Z turn. Both lower Magneton units retain their angled assembly. The
face bolts rotate their crosses about the front axis. Every part stays rigid;
there are no bending metal skins. Vertical magnet hinges retain the authoring
floor, and Magneton's three bodies stay physically joined through the clips.
Existing intentional battle/world hover is preserved.

Idle uses 81 keys over 3.2 seconds; attack/hit use 65 keys over one second.
Source cues own timing, with neutral endpoints and distinct unit phases.
All 16 combined Python checks pass in 47.361 seconds. They cover lossless
packing, exact neutral geometry, hierarchy/weights/binds, mechanical ownership,
closed solids and shading, occupied screw/magnet/pole/body joins through clips,
empty outward notches with solid arms, rigid lengths/winding, floor, independent
carriers, correct crown axes and strict reader mutation rejection. Four focused
Rust checks pass in 1.58 seconds for pack-size GPU activation, mechanical owners,
independent motion, floor/culling and 64 world floating phases with all weights
preserved and no extra limb motion.

Front, side and rear old/idle/attack/hit sheets were inspected. Both Blender
exports are byte-identical. Native evidence follows after paired review.
Shiny checks remain skipped, grass height is untouched and browser visual
acceptance remains open.


Both shared all-species geometry and exact source-cue timing checks passed
(2 tests, 90.99 seconds). The rebuilt native Magnemite/Pidgey Route 29 pair
was inspected at preserved state: frame 2450, checksum 720946100, LCD frame
3103 and virtual time 54.029684417. The coherent eye, face bolts, threaded
crown, open horseshoes and red/blue poles remain readable, with the correct
MAGNEMITE HUD and intentional floating clearance. Normal terrain admission
passed at the original 0.3048 m pack height and existing -110-degree review
yaw with normal support/omission/whole-body/frustum constraints active, no
fallback and no shader errors. Ignored evidence:
`target/pokemon-native-review/magnemite-normal`.


The native Magneton/Pidgey Route 29 pair was also inspected at preserved
state: frame 2803, checksum 4239097170, LCD frame 3441 and virtual time
61.380406917. All three joined bodies, eyes, face screws and colored open
magnets remain readable, with the correct MAGNETON HUD and intentional
floating clearance. Normal terrain admission passed at the original
0.9906 m pack height and existing -110-degree review yaw, with normal
support/omission/whole-body/frustum constraints active, no fallback and no
shader errors. Ignored evidence: `target/pokemon-native-review/magneton-normal`.
Shiny verification remains skipped; browser visual acceptance remains open.
