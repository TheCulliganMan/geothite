# Original Steelix sculpture and skin

The original angular sculpture retains ten irregular steel boulders, nine
buried volume-rooted joints, eight alternating spikes and a pointed tail.
Its broad toothed skull has fitted ivory eyes, red irises and dark pupils.
A separate low jaw connects through two metal hinges; four upper and three
lower squared teeth leave a real aperture backed by a recessed dark chamber.
The [original reference review](rock-serpents-source-review.md) records the
anatomy investigation. No external mesh or rig is imported.

The canonical asset is now `battle_species/steelix.glb`. Every original source
position, normal, color and triangle remains identical at f32 precision and
in logical source order. Coherent same-carrier/material packing reduces
48 logical parts to 32 runtime parts without changing geometry. It groups
paired spikes, skull details, hinges and each jaw's teeth, retaining every
logical range for strict readers.

| Vertices / triangles | Logical / runtime parts | Joints | Accessors | GLB bytes | Authoring height / pack length |
| --- | --- | --- | --- | --- | --- |
| 10,405 / 9,244 | 48 / 32 | 10 | 155 | 420,980 | 1.45 / 9.1948 m |

All existing 20k vertex, 40k triangle, 64 part/joint, 256 accessor and 1 MiB
limits remain intact. The normalized tail-to-head centerline remains
2.867623161993143, used in battle layout as `2.867_623_2 / 1.45`. The pack's
30 ft 2 in dimension remains a length, with uniform scale and full animation
bounds; no standing-height reinterpretation or admission-gate change is made.

Ten controls comprise floor root, seven successive rising-chain controls,
head and jaw. The lowest three boulders and tail remain fixed to the root.
Each upper boulder and its spikes stay rigid; buried joint vertices blend
along the actual adjacent-center axis. The head, fitted gaze, hinges, dark
chamber and upper teeth share one carrier. The jaw and complete lower teeth
rotate around the existing hinge axis. Stiff, restrained chain motion supports
the heavy head while the jaw opens for attack and responds separately to hit.
Idle lasts four seconds/85 keys; attack and hit last one second/65 keys.
Tracks rotate only, use source-relative cues and have neutral endpoints.

Twenty-two Python checks pass in 13.644 seconds: thirteen retained source
anatomy checks and nine independently decoded skin checks. They cover exact
neutral preservation, source ranges, hierarchy/inverse binds, all controls
moving, rigid armor/face/teeth owners, pinned supports, positive posed winding,
bounded joint strain, actual posed boulder/joint/spike/hinge/tooth roots,
fitted gaze, empty articulated aperture, original measured body datums,
source cues/endpoints and mutated source/bytes/species rejection. Source checks
now read the canonical GLB and compare f32 parity; their budgets are tightened
to the current runtime limits.

Three focused Rust checks pass in 0.78 seconds for original-size GPU admission,
canonical geometry/floor across clips and preserved world bindings/contact/full
locomotion bounds. Shared schema and exact elapsed-idle/current-attack-cue
checks pass in 99.98/99.76 seconds. All 24 fresh layout tests pass in 0.51
seconds, including the previously unresolved Steelix length fixture. This
supersedes the earlier stale-fixture failure. Together with the follower fixture regression below, there are 52 scoped checks;
this is not a full-suite or gameplay-completion claim. Both native clients
build in 58.40 seconds.

The direct sculpture CLI, background builder, candidate and canonical GLBs
match byte for byte. The common canonical reader restores all 48 logical
parts without retaining a parallel JSON model. Rebuild into ignored output:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 tools/steelix_sculpt.py \
  --output target/steelix-rebuild
/Applications/Blender.app/Contents/MacOS/Blender --background --threads 2 \
  --python tools/build-battle-species.py -- \
  target/steelix-rebuild --only STEELIX
```

Source geometry SHA256:
`87fe7010288e39368021ba4b84e9477f898cfc9e341d884f334cbaeeca40eceb`.
Packed geometry SHA256:
`1868650cc107f68412c8bd5614b9e2ae7db2b6f35b5db834e3213dc97ec556d0`.
GLB SHA256:
`7f7e568595edb71c6504127fd049b0b7af52d97e1af2bcf2583eba08c12073db`.
The decoded neutral/attack sheet was inspected inline, entirely offscreen.
The default Route36 review at (20,8) did not display the companion: its
original-size following gap is 562.20215 pixels, requiring nine clear source
steps, while that fixture walks four. A new production-publisher regression
reproduces the blocked short fixture and verifies a clear corridor at
EcruteakCity (20,21), retaining the full gap, original card size and unchanged
authoritative state. It passes in 22.57 seconds, making 52 scoped checks.
The default Route29 battle also rejects every checked original-size shot;
its final reason is `no readable higher shot on checked ground`. Those failed
captures are not visual approval. National Park admits the complete Steelix
and Nidoran-M models without fallback; its paired battle image is inspected.
The city front/world pairs completed in 63.784/46.405 seconds and were inspected
inline, alongside the moving world frame. Front clearly exposes the skull, gaze,
open jaw and rising chain; the wider world framing clips the giant companion
at the left edge and remains a visibility limitation. The National Park battle
pair completed in 144.329 seconds with both species names readable. All three
retain the same authoritative state between 2D and 3D. Both native binaries
remain identical across the completed city and battle groups (SHA256
`fa5ff5777692bc1ef4d7e862a687a9abc74f0a32cff53649640b99379ec7d10c`
for render_at_location,
`d4fa7a2d562cd71462473a41427ea48aa2e4c3b8f3d43c40866e5a8805498a25`
for immersive_battle_3d). No size, field-of-view,
collision, omission or admission rule is relaxed to obtain these reviews.
No shiny, grass-height, AA, audio,
pack or gameplay changes belong to this batch. Full-roster improvement remains
active.
