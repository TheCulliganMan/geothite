# Kangaskhan sculpture and parent/baby rig

`tools/kangaskhan_sculpt.py` retains the heavy armored mother, broad muzzle,
real open jaw with tongue and upper fangs, dark skull plate, raised clawed
fists, shoulder and thigh armor, grounded clawed feet, four dorsal plates and
heavy balancing tail. The cream pouch has a real open rim and hollow interior.
Its pale violet baby has a seated body, short neck, round head, cream ears,
fitted eyes and gleams, small mouth and two hands holding the rim. No external
geometry or rig is imported.

`tools/kangaskhan_glb.py` exports the canonical `kangaskhan.glb`: all 70 logical
solids packed into 46 runtime material groups, 18,743 vertices, 31,108 triangles,
20 joints, 252 accessors and 823,132 bytes. Redundant radial/interior sampling
is reduced within the existing limits; fitted paint refines against actual
stored parent surfaces. Source heights and proportions remain unchanged:
authoring height 1.40 and verified pack height 2.2098 m.

Logical neutral geometry digest:
`476e85961870526d4c0e2867fd693bd68a63cdf2f3862e43a00feca553b83f40`.
Packed runtime digest:
`d05bf3560699741a45d687e8c44745f0ba667d37fa551a0c1d7b16ef90cffc94`.

Mother body, belly, open pouch and dorsal armor share one carrier. Head,
jaw, ears, arms and wrists articulate independently, with coherent gaze,
fangs, fists and claws. Separate haunch carriers retain their fitted plates;
ankles blend into the fixed feet. The tail has two continuous carriers.
The baby has its own body and head, blended neck, separate ears and separate
arm/hand grip carriers. Its body turns about the seat while its hands retain
the rim. Grip carriers have explicit grip names so world locomotion does not
apply a walking arm swing to the baby. World walking uses two complete leg
chains and preserves all baby weights. Idle, attack and hit use source-cue
clocks and return to neutral endpoints.

The production registry, builder and asset readers use the GLB. Material
packing preserves each logical solid and all vertex/index/material/weight
ranges; the old static JSON is removed. All 19 combined Python checks pass,
covering exact geometry and packing, closed outward solids, shading, actual
roots, parent/baby fitted layers, pouch containment, empty interior beyond the
baby, both grips through motion, actual mouth aperture, fixed feet, winding,
strain, isolated joint motion and source-cue contracts. The baby's narrow seat
contact is checked with a shared occupied-volume witness in the cup floor;
the generic overlap grid skips that thin interval. Detached body and grip
negative controls remain active.

Three focused Rust checks pass for GPU activation at verified pack height,
runtime owners and independent mother/baby carriers, floor/culling through
all clips, plus two world support chains through 64 phases with baby weights
preserved and grip joints excluded from walking arm swing. Front, side and
rear poses of all clips were inspected. Blender exports are byte-identical.
Shiny checks remain skipped at the user's request. Browser visual acceptance
remains open.

Both shared all-species geometry and exact source-cue pose timing checks pass.
The rebuilt native Route 29 Kangaskhan/Rattata 3D/2D pair was inspected at
preserved state: frame 4570, checksum 339953756, LCD frame 5413 and virtual
time 99.975341419. The complete parent, seated baby and rim grips, open jaw,
raised fists and correct KANGASKHAN HUD remain readable; both feet meet the
terrain. Normal terrain admission passes at the original pack size and the
existing -110-degree review angle, with normal size/support/omission and
whole-body/frustum checks active. No shader errors were reported. Ignored
evidence is `target/pokemon-native-review/kangaskhan-normal`. Grass height is
untouched. Browser visual acceptance remains open.
