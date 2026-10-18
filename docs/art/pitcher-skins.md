# Bellsprout, Weepinbell and Victreebel botanical skins

The three botanical sculptures now have canonical articulated GLBs. Bellsprout
retains a sinuous stem, two rooted feet, cupped leaves and a hollow trumpet.
Weepinbell gains its missing white sclerae and fitted pupils. Victreebel has
fitted angular eyes, rooted hood and tendril, hollow upper bowl and two fangs.
All facial patches and green flecks follow actual stored supporting triangles.

| Species | Vertices / triangles | Source / runtime parts | Joints | Accessors | GLB bytes | Pack metres |
| --- | --- | --- | --- | --- | --- | --- |
| Bellsprout | 3,001 / 4,728 | 14 / 12 | 10 | 87 | 173,784 | 0.7112 |
| Weepinbell | 3,855 / 5,700 | 20 / 12 | 9 | 83 | 201,696 | 0.9906 |
| Victreebel | 4,116 / 6,232 | 25 / 16 | 10 | 104 | 218,964 | 1.7018 |

Authoring height remains 1.05. The previous static vertex/triangle counts were
2,181/3,824, 5,340/5,668 and 6,158/6,992 respectively. Original vertex,
triangle, part, joint, accessor and byte limits remain intact. Runtime packing
combines only coherent carriers/materials and retains logical source ranges.

Bellsprout has floor root, lower body, upper stem, head, two leaf roots, two
leaf tips and two root-foot controls. Its eyes, rim and recessed throat share
the head carrier. The lower root tips remain fixed while two real support
chains participate in existing world locomotion. The other two species have
no invented legs or arm chains: their lower body tips stay fixed, with body,
mouth and paired leaf controls above. Weepinbell has two hook controls;
Victreebel has a mouth-parented hood and two body-parented tendril controls.
All upper world bindings remain canonical.

Leaf blade and midrib weights use the same XY axis projection. The initial
attachment band blends from the common stalk/body parent to each leaf root,
so coincident leaf bases retain identical weights through production welding.
Further along, weights blend to independent tips. Hook/tendril weights follow
the actual sampled curved-tube rings. Idle lasts 3.4 seconds/85 keys;
attack/hit last one second/65 keys, with rotation-only tracks, neutral endpoints
and source-cue timing.

Source fixes include selecting the outermost actual Bellsprout head surface
for eyes, attaching dark mouth floors to occupied walls, moving Victreebel's
hood/tendril roots into actual supporting geometry and fitting its inner bowl
lining. An optional ear-clipped cap closes these concave leaf cups without
the old fan's self-intersections. Other callers retain their existing helper
default. The later [Oddish cap correction](oddish-sculpture.md#concave-cap-correction)
confirms and fixes that species' legacy cap crossings separately.

The existing gold/coral/green/brown palette remains. Local normal sprites were
inspected from the external pack and use brighter green/red/white/black, so
exact sprite palette parity is not claimed. No shiny verification belongs to
this batch.

Canonical assets, the direct CLI and the normal background builder produce
identical bytes. Both Python readers/checker and Rust registry/GPU/world
dispatch use the same GLBs. Rebuild into ignored output:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 tools/pitcher_sculpt.py \
  --output target/pitcher-rebuild
/Applications/Blender.app/Contents/MacOS/Blender --background --threads 2 \
  --python tools/build-battle-species.py -- \
  target/pitcher-rebuild --only BELLSPROUT,WEEPINBELL,VICTREEBEL
```

Eighteen stored Python checks pass in 14.330 seconds: ten source and eight
independently decoded skin checks. Coverage includes exact recipes, closed
connected outward solids, finite normals, real roots and vein/cup contacts,
stored-triangle facial support and visibility, open mouth axes reaching dark
floors, negative detach controls, uncrossed neutral/posed stalks and leaves,
hierarchy/inverse binds, independent controls, coherent carriers, fixed ground,
positive posed winding, bounded strain, neutral endpoints and mutation rejection.

Five focused Rust checks pass in 0.62 seconds: original-size GPU admission for
all three, registry/canonical support across all clips, and world locomotion
with two Bellsprout supports and zero legless support chains. Every world
vertex stays above the floor and within full motion bounds; support stays
within 0.005 authoring units. Shared schema/physical-size and exact animation
cue checks pass in 82.40 and 91.10 seconds. This is 25 scoped checks, not a
full-suite or gameplay-completion claim. Both native review examples build
in 47.92 seconds.

| Species | Source SHA256 | Packed SHA256 | GLB SHA256 |
| --- | --- | --- | --- |
| Bellsprout | `dcacef4f2684505c937f76c4f5e7b66fc268da1b528d16a3723855efd618bf67` | `34c7d0c2b0acffea6b36f16c6ec988186e5bb2c1b5906c6781c805600ed1ae32` | `6466270ad48b17809f77ef060946f283524a0763960abd0d5a61b18750b0be6d` |
| Weepinbell | `2e9cb117e0c0ed3b15493d7dce8756bad76511f9f87af773cab74bd9a605a6f3` | `2f702734e363aaa15b2bc88e89e5ca88b6ad31a7517a6980bdae11926eb3c9ab` | `41929ea29dd783186730ac68f8b2a18035a564c62d456a7e4a06fe52594ac6b9` |
| Victreebel | `42140432e5eabe120833d48cb23b83490161ecb5044ffa90cd67e4a7171314c3` | `efb9f3f09ee0e691fc9a3acd3675e9c23b86e86ff0ecbaccdc97411cd2958923` | `e295dda63e59c2b3a63b90142d05ffb370bcb883ebd37bb20bb678d6453e996d` |

Full-roster model and rig improvement remains active.

Nine final same-state 2D/3D production pairs and three moving world frames
were inspected inline. Every capture used headless image targets and mute;
no window was opened and no focus changed. Six front/world traces have 32
synthetic frames across five Route36 positions. Route29 battles admit all
three complete models at their original pack sizes with uniform scale, zero
support, unchanged full-envelope/camera/terrain gates and no fallback. Both
Pokémon names are visible in each battle HUD. Victreebel required more camera
candidate searches and took 156.161 seconds; no gate was relaxed.

| Species / view | State frame / hash | LCD frame | Virtual seconds |
| --- | --- | --- | --- |
| bellsprout front | 1562 / 3664710315 | 1566 | 26.602627791 |
| bellsprout world | 1561 / 3167624538 | 1568 | 26.741879459 |
| bellsprout battle | 1959 / 3921479512 | 2580 | 44.202115333 |
| weepinbell front | 1611 / 2041331501 | 1614 | 27.471619667 |
| weepinbell world | 1488 / 4219474906 | 1494 | 25.553226666 |
| weepinbell battle | 2373 / 3829102887 | 3083 | 53.132722042 |
| victreebel front | 2235 / 539194122 | 2241 | 38.91963925 |
| victreebel world | 1889 / 1712507535 | 1897 | 33.1821685 |
| victreebel battle | 4144 / 1782641200 | 4979 | 92.231066084 |

Complete ignored receipts:
`target/pokemon-native-review/pitcher-capture-receipts.json`. Both binaries
contain the exact compressed canonical GLBs and headless/mute controls.
Streamed SHA256, size and modification times stayed identical before/after:

- `render_at_location`: `4104193cfc562ec4073eb1d32e8dedee7476dbb64e3b78f52a36e924fa7b9196` (407,450,864 bytes; mtime_ns 1792330655270673366).
- `immersive_battle_3d`: `ef0d542f0430ccfadb371d435bd8b18c7f0de5df9f888301395207b990e412bf` (407,199,816 bytes; mtime_ns 1792330656265350600).

Bellsprout’s eyes and hollow trumpet read clearly from the front; its narrow
stem and leaf edges become thin in side/world views. Weepinbell’s white gaze
and recessed mouth remain readable in battle, while side views hide its far
eye/leaf. Victreebel’s hood, vine, flecks and angular gaze remain distinct;
the hood/rim occlude the upper bowl from low angles, and high world views
hide much of the far gaze. Fitted details fade at normal world scale. Body
tips/root feet meet the ground with soft shading; broader world/browser/
camera/material coverage and full-roster visual acceptance remain open. No
grass-height, AA/filtering, audio, pack publication or gameplay changes
belong to this batch. Registration reaches 252 skin variants and 23 static
species files.
