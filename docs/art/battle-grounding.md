# Grounding beneath low Pokémon silhouettes

Battle presentation now keeps two separate shading registrations. The existing
tight shadow remains attached to the actual floor-connected support region.
A faint additional layer registers nearby low surfaces, including Pineco's
armor overhanging its narrow tip. This improves the floor cue without moving,
rescaling or reshaping the Pokémon.

The additional footprint clips the neutral mesh at 30% of its height and
includes exact triangle/ceiling intersections. Thus tessellation does not
resize the registration, and high plumes/crowns do not determine its extent.
It is cached once per actor alongside the existing physical contact bounds.
The soft ellipse fits within this clipped horizontal footprint: scale 0.40
with the existing mesh's outer radius 1.25. Its material opacity is 0.45,
multiplying the existing vertex falloff into an 18% center and 9.9% middle.
This is a portable approximation of nearby ground occlusion, not ray-traced
ambient occlusion or a replacement physical support region.

Floating species and source-card fallbacks retain the existing shadow only.
Both layers follow source displacement, battler visibility, capture reveal
and classic-view hiding. The physical support registration continues to own
grass contact information. Admission, original pack size, complete animated
bounds, source-ground witnesses, camera fitting and terrain/omission gates
are unchanged. The overworld shadow renderer is outside this change.

The implementation is in `crates/crystal-voxel-view/src/battle_view.rs` and uses
the existing unlit, alpha-blended vertex-color shadow mesh. It adds one small
cached bounds record and at most one additional draw per grounded battler. No new
texture, shader, external dependency or browser integration is required.

Eight `contact_shadow` regressions pass in 1.05 seconds; one unrelated dense
forest benchmark remains ignored. Existing narrow Sentret-tail, Articuno-foot,
Lapras-belly, floating-footprint and source/scene-visibility checks retain
their tolerances. New checks cover exact clipping, high-plume rejection,
tessellation invariance, invalid bounds, broader low armor, bounded ellipse,
independent opacity, unchanged tight registration, floating exclusion and both
layers hiding with the battler or classic view.


Both native review examples build in 44.15 seconds. Three final same-state
production 2D/3D battle pairs were inspected inline, headless and muted, for
Pineco, Forretress and Sentret. Full original pack heights (0.6096 / 1.1938 /
0.7874 m), uniform scales, zero physical support and no fallback are verified
by the existing native capture receipts. Both Pokémon names remain visible.
The camera/terrain/full-envelope gates remain enabled throughout.

| Species | State frame / hash | LCD frame | Virtual seconds |
| --- | --- | --- | --- |
| pineco | 1378 / 658023508 | 2005 | 34.311581583 |
| forretress | 1206 / 724937435 | 1826 | 31.318481627 |
| sentret | 1278 / 1797606167 | 1868 | 32.067574833 |

The ignored complete receipt is
`target/pokemon-native-review/ground-shade-capture-receipts.json`. Both binaries
were checked for the new low-body shading implementation, headless/mute tags
and exact compressed current GLBs. Their streamed hashes, sizes and modification
times remain unchanged before and after capture:

- `render_at_location`: `a67a3007b787fadb8a14117414a7e04f00906fc2c47080bcb0632efba32b7c67` (407,533,488 bytes).
- `immersive_battle_3d`: `4f78f4a46f6517a761fbec6b5f11ce8699763e9d6ff2159de83ec14a7cbafc58` (407,298,952 bytes).

The soft floor cue beneath Pineco's overhang is broader without becoming an
opaque disk. Forretress's lower shell retains a readable subtle ground edge.
Sentret's actual curl remains the support; the faint layer does not invent
support beneath its elevated body/feet. These are visual reviews of the new
implementation, not pixel-identical before/after comparisons: prior captures
have different game clocks and encounter/camera choices. The footprint tests
provide the controlled support, opacity and bounding comparisons.

World shadows, browser rendering and optional hardware-shadow combinations
remain outside this reviewed scope. Small details and contact shading can
still be occluded by the environment. No shiny verification, grass-height,
AA/filtering, audio, pack generation or gameplay changes belong to this work.
There remain 26 static species files, alongside broader roster visual review;
the full model and rigging goal is active.
