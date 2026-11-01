# Arbok hood, fanged jaw and skin

Arbok replaces a generic spherical-headed serpent and flat polygon hood with
an original rounded cobra mantle, fitted dark/ivory/red false eyes and threat
chevron, angular upper muzzle, real open mouth, rooted curved ivory fangs,
dark throat, pink tongue and separate lower jaw. A continuous purple coil and
muted ventral strip retain low ground contact and a raised pointed tail. The
user-supplied pack's normal front/back sprites guide the anatomy and mask;
exact palette parity is not claimed.

| Vertices / triangles | Parts | Joints | Accessors | GLB bytes | Authoring height |
| --- | --- | --- | --- | --- | --- |
| 11,554 / 18,900 | 25 | 8 | 147 | 538,504 | 1.18 |

The previous static assembly had 1,256 vertices, 2,376 triangles and 15 parts.
Existing 20k vertex, 40k triangle, 64 part/joint, 256 accessor and 1 MiB limits
remain unchanged. Original authoring height, pack dimension (11 ft 6 in,
3.5052 m), longitudinal calibration `2.3298596 / 1.18`, uniform scale, full
animated bounds and model-admission gates are preserved.

Eight controls articulate the floor root, raised neck/body, head, tail base,
tail tip, two outer hood folds and jaw. The shared immutable snake centerline
and radius recipe retains the already checked low coil. The Ekans GLB stays
byte-identical (`7d73c37140684a2973ef7c3c9632b0a54cd6433a7b0784a14e55a6c3cb6a1a02`).
Longitudinal surface queries blend the raised neck and tail while retaining
the low coil on the floor root. Complete face/fangs share the head carrier;
jaw and tongue share the mandible. Idle lasts 3.4 seconds/85 keys; attack/hit
last one second/65 keys, rotate only, follow source cues and return to neutral.

The initial mantle sat behind the cylindrical neck, hiding its central threat
mark. Its actual profiles now cross the front of the neck; triangle checks
require the marking to remain visible ahead of that neck. An early broad
hood blend stretched tiny fitted-mask edges. The complete mask now shares
the central chest carrier, while unmarked outer folds have local pivots and
restrained independent motion. The upper bridge follows the head and the
lower bridge blends into the neck. No geometric limit or test tolerance was
relaxed. Posed fitted-feature checks compare related parts in their actual
carrier frame, retaining the deformed hood as the tested parent surface.

Twenty-one Python checks pass in 15.145 seconds: twelve source anatomy checks
and nine independently decoded skin checks. They cover closed connected
outward solids, finite normals/winding, dimensions/budgets, uncrossed coil
and hood, occupied roots/detached negative controls, fitted feature vertices,
edges and centers, unobstructed mask, genuinely empty mouth space and exposed
fangs, ventral support, exact canonical geometry/hierarchy/inverse binds,
every control moving, coherent carriers, fixed floor, positive posed winding,
bounded strain, posed roots and hood-mask support, source cues/endpoints and
mutated source/bytes/species rejection.

Direct sculpture CLI and background Blender builder reproduce the canonical
asset byte for byte:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 tools/arbok_sculpt.py \
  --output target/arbok-rebuild
/Applications/Blender.app/Contents/MacOS/Blender --background --threads 2 \
  --python tools/build-battle-species.py -- \
  target/arbok-rebuild --only ARBOK
```

Source and packed SHA256:
`2db1012c5b11414aee4a5b75c1517805dcf4be91989a2f854a74c6df71830790`.
GLB SHA256:
`3c78d4db0047da6a95b17fdd7689a49175731abac32565c322914b60c6c141d3`.
The decoded neutral/attack source sheet was inspected inline, offscreen.
The production first pass exposed overly round false eyes, a broad red crescent
and real eyes set too high on the brow. The final mask has angular pale eyes,
peaked/notched dark brows, compact red centers and a dark-bordered angular red
chevron; the real eyes sit lower on the front of the muzzle. Independent rays
require occupied brow peaks and empty notches, and every complete mask part
must share the chest carrier. All joint nodes, inverse binds, clip metadata,
times and quaternion keys remain byte-identical across this refinement.
Three final focused Rust checks pass in 0.65 seconds after refinement. The final
shared anatomy/order/physical-size schema check passes in 132.62 seconds,
making 25 scoped checks overall. Both final native clients rebuild in 1 minute
23 seconds. Three refined same-state production 2D/3D pairs and two moving
front/world frames were inspected inline, entirely headless and muted.
Front/world/battle capture clocks are 101.468/68.875/110.982 seconds.
The complete Arbok and Pidgey models admit without fallback; both HUD labels
are readable and every receipt retains the authoritative state. The existing
length calibration, uniform scale and physical support zero pass the capture
postchecks at the original 3.5052 m pack dimension.

The final front view exposes the real pupils below the brow, peaked hood
markings, outlined red chevron, open mouth and grounded low coil. The battle
view keeps the complete hood, fanged head and raised tail visible. Stationary
world camera step 6 remains substantially obstructed by foreground branches;
the moving frame exposes the model through existing foliage fading. Broader
world visibility and palette parity remain open. The first-pass smiling-mask
renders are superseded by this refined group.

Both final clients retain exact SHA256 identities throughout all captures:
`render_at_location` is
`920ed8334698b973733d5776975a6922ca3355580b57dd2d817ceeffc2ccca7e`;
`immersive_battle_3d` is
`38801ec31d0c3f06aad256cd46589dcbd0e42c2ab010ad4785fe86fdfd16553b`.
Final receipts and review images remain ignored under
`target/pokemon-native-review/arbok-refined-*`. Registry count is 257 skin
variants and 18 static species files.
These are scoped checks, not a full-suite or gameplay-completion claim.
No shiny, grass-height, AA, audio, pack or gameplay changes belong to this batch.
Full-roster model and rig improvement remains active.
