# Croconaw and Feraligatr model and rig update

The [October 20 reference and posture revision](crocodile-geodude-fidelity.md)
supersedes the neutral geometry, pose, counts and pins in the historical
records below.

Current camera/follower follow-up: The shared trainer-eye and physical following-distance change now exposes
Feraligatr’s eyes above its jaw in the reviewed ordinary first-person view
at the original 2.3114 m size. Broader world/scenery acceptance remains open.
See [physical companion spacing](pokemon-following-distance.md) for the
current checks, captures and remaining limitations. Earlier review records
below retain their original evidence and conclusions.


## Canonical authoring and runtime registration

`tools/crocodile_sculpt.py` preserves the distinct continuous juvenile body and
broad-shouldered adult, broad crocodile snouts, fitted eyes/nostrils, all teeth,
all armor plates, red cranial/dorsal/tail crests and the original 1.30 authoring
height. `tools/crocodile_glb.py` exports 14-joint canonical models with strict
readers and lossless source-part ranges. Both canonical GLBs replace their old JSON in the 198-entry runtime registry.
Native visual acceptance remains pending; the software sheets are authoring
evidence only.

The actual jaw cages now use two bounded Catmull-Clark steps with smooth corner
normals. The authored body profiles use four longitudinal samples and 48 radial
segments. Arms, legs and tails follow curved profiles; palms are rounded and
juvenile fingertips/toes taper into rounded ends. Adult mouth corners curve
between their real upper and lower jaw roots. Intentional red crest folds and
armor bevels remain crisp. No global mesh subdivision or palette replacement
was applied.

A physical contact check rejected teeth that stopped short of the jaw. Their
hidden roots now enter an occupied interval of the actual triangulated jaw,
while the authored visible tips stay fixed. Croconaw's tail diamond was beside
its laterally curved tail; both tail crests now follow the actual sampled
centerline rather than x=0. Original Y/Z outlines and all named solids remain.

Feraligatr's old planar armor caps could bridge unsupported shoulder space or
sit behind the hock. Plates now use their actual upper arm, forearm or hock
surface, validate their entire footprints, retain real bevels and adapt the
cap triangulation to surface curvature. `fitted_adaptive_contour` has an optional
maximum surface error for these caps; its default is unchanged. Re-exporting
Cubone and Marowak with the new helper preserved their canonical bytes.

The hierarchy is root, body, head, hinged lower jaw, left/right upper arms,
elbows and wrists, supported thighs and two tail carriers. Body/belly/neck
weights remain continuous. Skull paint and cranial crests share the head;
upper teeth follow the upper jaw, lower teeth follow the lower jaw, and mouth
interiors/corners blend between them. Fingers and adult claws share their
wrists. Fitted armor uses its true limb's field, and tail crests use the same
field as the tail. Battle feet and matching toe claws remain fixed.

Idle uses 81 keys over 3.2 seconds; attack/hit each use 65 keys over one second.
Endpoints return to neutral and production source cues own action timing.
Material grouping retains every original solid, index and original-part weight;
readers restore all 49/80 logical parts. Runtime groups keep complete feet and
claws separate by side for world walking. Existing 20,000 vertices, 40,000
triangles, 64 parts, 256 accessors and 1 MiB limits remain unchanged.

| Model | Vertices | Triangles | Logical / runtime parts | Joints | Accessors | Bytes |
|---|---:|---:|---:|---:|---:|---:|
| Croconaw | 13,558 | 21,444 | 49 / 28 | 14 | 172 | 611,216 |
| Feraligatr | 16,776 | 24,448 | 80 / 41 | 14 | 226 | 750,960 |

Croconaw: 
Logical geometry pin: `21a8baf8555e83fe873f7694e88b68010e84ba28f8f855895176f0a6b2a9f592`. Packed pin: `5e931c599dd42d7e2ebbf57fa8174ae66023d02dea8875b09730c1efc1a1697f`.

Feraligatr: 
Logical geometry pin: `b5b3a8c4469fbc80406847fddb9e3f70914abf8afe8c56e73470ac98768a076d`. Packed pin: `d80bb181fae2556e57b3ef65e60fb42f4e816f7008c5e72ca1db8d1979719cad`.

All 13 Python checks pass: seven stored skin checks in 94.600 seconds and six
anatomy checks in 32.691 seconds. They cover exact neutral/packed geometry and
source ranges, hierarchy/binds/weights, independent carriers, fixed supports,
actual occupied limb/tooth/crest joins through poses, armor clearance, positive
winding, bounded edge strain, coherent owners, fitted surfaces and mutation
rejection. Front/side/rear old, idle, attack and hit sheets were inspected after
the final geometry changes. Native close and elevated evidence is recorded below.

The verified local pack reports CROCONAW = 307 / BIG JAW = 1.0922 m and
FERALIGATR = 707 / BIG JAW = 2.3114 m. Their runtime scale must retain these
heights. Shiny checks remain skipped by request; grass height is untouched.


Both models are registered in the 198-entry authored runtime catalog, and the
legacy JSONs are removed. The main Blender builder and pure source CLI export
the canonical GLBs. The published strict reader/hierarchy/packing check passes
(0.818 seconds); four focused Rust checks pass in 2.02 seconds for both actual
pack-size GPU activations, fixed supports/jaw/wrist/tail ownership and bounds,
and 64 phases of complete two-foot walking with non-leg owners preserved.

Blender initially rejected a bit mismatch. Independent comparison found only
20 signed-zero jaw position components across both models; all numeric values
were identical. Crocodile position zeros are now canonical positive zero.
Independent stored-buffer comparison proves numeric geometry, normals, indices,
weights, hierarchy, binds and clips are unchanged. Strict hashes remain in
force, and both Blender exports now match the canonical bytes exactly.
Native battle acceptance is recorded below; world and browser review remain open.


The shared all-species grounded/volumetric geometry and exact production
source-cue timing checks both pass against the current 198-entry registry
(2 tests, 124.73 seconds). Fresh battle and world review executables build
successfully; all four current canonical compressed payloads are present in
the battle executable. Multi-view native review continues below.


Native Croconaw/Sentret Route 29 pair inspected at preserved state: frame
5213, checksum 1248705665, LCD frame 6187, virtual time 137.530594338. The
original 1.0922 m body, rounded broad snout, coherent eyes/paint, rooted teeth,
rounded fingers/toes and attached tail diamond remain readable against the
retained terrain. Correct CROCONAW HUD; no fallback or shader error. Normal
terrain admission passed at the existing -110-degree review yaw with support,
omission, whole-body and frustum constraints intact. Ignored evidence:
`target/pokemon-native-review/croconaw-normal`.

Native Feraligatr/Sentret Route 29 pair also inspected at preserved state:
frame 4638, checksum 2873737362, LCD frame 5613, virtual time 128.680985458.
The original 2.3114 m size is retained. The whole red-crested silhouette, broad
rounded jaws, seated teeth, clean V belly, curved limbs/armor, counterbalancing
tail and grounded clawed feet fit in the actual scene. Diffuse fog light leaves
the blue/cream/red anatomy readable. Correct FERALIGATR HUD; no fallback or
shader error. Normal terrain admission passed at the existing -110-degree
review yaw and normal support/omission/whole-body/frustum constraints. Ignored
evidence: `target/pokemon-native-review/feraligatr-normal`.

World first-person review is not accepted yet. `render_at_location` with
CUBONE in NewBarkTown (5,8), image-target paired capture and first-person
camera aborts with a Compute Task Pool stack overflow. It reproduced with
both RALA and LR ordinary inputs. The debugger diagnosis is recorded below;
no model resizing or admission relaxation was used. Browser review remains open.


World QA stack diagnosis: LLDB stopped in
`GameDataSet::apply_runtime_mutation_command_with_checksum` at its stack probe.
The unoptimized native QA override reserves 0x1b1000 + 0xf00 = 1,777,408 bytes
(about 1.70 MiB) in that function, before its caller frames. This is the
Compute Task Pool overflow, not an observed mesh/bind failure. Native world
review was rebuilt with the repository's normal optimized `crystal-assets`
profile (opt-level 2), retaining the same geometry, sizes, controller inputs and
cameras. No runtime source or thread-stack limit was changed. World review
remains unaccepted until the actual paired captures pass. Diagnostic evidence:
`target/pokemon-native-review/cubone-world-debug.log` and
`target/pokemon-native-review/world-stack-disassembly.log`.


The normal optimized-assets build resolves the world QA stack overflow. New
Bark's exit-scene image was rejected as model evidence. Clear Route36 (20,8),
noon stationary source/native pairs now show both crocodiles at their actual
pack sizes. Croconaw first-person frame 2229/checksum 2960780844 and elevated
frame 1721/checksum 1822599684 preserve state. The broad jaw, round belly,
curved fingers and attached tail diamond read in front and side views. Close
inspection still exposes abrupt thigh/hock transitions that need further
organic contour work; world polish is not fully accepted.

Feraligatr's elevated side view preserves the complete large silhouette,
attached tail, crest and jaws beside the trainer. The first-person view
(frame 3126/checksum 3758876406) is a useful close body/hand inspection but
crops the head at this ordinary follower distance. It is not full-body or
face acceptance. Arm/leg transitions also remain angular at close range.
No model size, follower gap, support or camera bounds were changed to fit
the model. Native evidence: `croconaw-world-first`, `croconaw-world-elevated`,
`feraligatr-world-first`, `feraligatr-world-elevated` in the ignored review
directory. Browser, moving-world and remaining close contour work stay open.
Shiny verification remains intentionally skipped.


### Native close-view correction: elbows, knees and feet

The first close native views exposed flat limb closing faces at elbows and
knees. Both upper limbs now continue around the bend and taper inside their
adjoining segments; lower roots also start inside the upper volumes. Curved
limb rings use 24 sides with three longitudinal samples per control interval.
Instep geometry increases locally from 18×10 to 28×16. No global subdivision,
body resizing, rig hierarchy, cue timing, fixed-foot ownership or world gap
changes were used. The 49/80 logical parts and 28/41 runtime parts remain.
Knee armor follows the joined thigh/hock front envelope with the same leg
weight field. Geometry pins and the table above describe this revision.

A new independent stored-triangle test finds actual planar closing rings and
checks their interiors remain occupied by adjoining limbs through idle, attack
and hit samples. It fails on the previous meshes at an exposed elbow and
passes on both revisions. All 14 Python anatomy/skin checks pass in 75.859 s;
four focused Rust activation/support/walking checks pass in 1.41 s. Both
Blender exports match canonical bytes exactly. The local old/new front/side/
rear neutral/attack/hit sheets show rounded knees and elbows with coherent
feet. Existing 20k vertices, 40k triangles, 64 parts, 256 accessors and 1 MiB
runtime limits remain unchanged. Native acceptance for these revised meshes
is recorded below; earlier receipts describe the preceding revision.


Final contour checks: the added independent self-intersection test found a
fold in the first Croconaw hock revision. Its buried root now continues along
the shin axis inside the thigh, eliminating the fold while retaining the
rounded knee. All 15 final Python checks pass (182.047 s under simultaneous
native review/build load), including planar-cap coverage through clips and
uncrossed limb surfaces. Four focused Rust checks pass again (2.07 s). Both
final Blender exports remain byte-identical. No production limits changed.

Final Feraligatr native Route29/Pidgey battle pair preserves frame 6109,
checksum 1660363885, LCD 7075 and virtual time 143.102177209. The full original
2.3114 m silhouette fits normal terrain admission, with rounded limb joins,
planted clawed feet and readable jaws/crest/armor. Correct FERALIGATR HUD,
modeled species correct, no fallback. Elevated Route36 frame 1703/checksum
4127077509 preserves the complete side silhouette at gameplay scale. The
first-person upward view (frame 3152/checksum 731979012) inspects the underside
of the jaws and shoulder surfaces; the jaw still blocks the eyes at ordinary
follower distance. Full first-person face readability remains unaccepted.
Evidence: ignored `feraligatr-contour-first`, `feraligatr-contour-elevated`,
`feraligatr-contour-battle`. Browser and moving-world acceptance remain open.


The full exact-species schema test now passes (98.13 s), including both
updated crocodile geometry counts and retained hierarchy/physical spans. Its
first run exposed only a stale Heracross vertex fixture: independent GLB
accessors report 19,588, and the current source exporter reproduces the
unchanged canonical 814,644 bytes. The fixture and Heracross art note now
reflect that count. Final native binaries contain both exact canonical GLBs,
verified against decompressed Cargo-generated gzip payloads and those actual
payload bytes in the executable. All non-limb/non-limb-armor geometry and
weights, the complete hierarchy, inverse binds and clips match the previous
models exactly.


Final Croconaw Route29/Sentret pair preserves frame 3134/checksum
4096534027, LCD 3799, virtual time 67.246804418. The original 1.0922 m body
passes normal terrain admission with no fallback or source-card substitution.
The rounded knees, elbows and feet read clearly with the unchanged juvenile
body, broad jaws, eye/belly markings and tail diamond. Correct CROCONAW HUD.
The final Route36 first-person and elevated pairs also preserve state and
show supported feet and coherent limb contours at close and gameplay scales.
No camera/admission/physical-scale workaround was used. Final evidence:
`croconaw-contour-first`, `croconaw-contour-elevated`,
`croconaw-contour-battle` under `target/pokemon-native-review/`. Browser and
moving-world visual acceptance remain open. Shiny checks are skipped.
