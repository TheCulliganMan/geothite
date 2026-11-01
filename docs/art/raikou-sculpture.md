# Raikou authoring work

The original recipe in `tools/raikou_sculpt.py` replaces the intended Raikou
proxy with a yellow tiger body, fitted black flank and leg markings, white
cheeks and saber fangs, forehead armor, a cyan muzzle mask, a purple storm
mantle, and an angular lightning tail. Anatomy is checked against the
[official Raikou artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/243.png)
and the front and back sprites in the local verified content pack.

The pack's normal palette anchors are RGB555 yellow `(31,31,0)` and purple
`(19,6,18)`. The verified Pokédex height is 6 feet 3 inches, or 1.9050 metres.
The skin is authored at a normalized silhouette height of 1.30; production
integration must apply the verified physical height.

`tools/raikou_glb.py` writes a standard glTF skin with sixteen joints. The
torso, neck, and armored head form one chain; ears, the three-part storm
mantle, four legs, and a three-part lightning tail have dedicated joints.
Paws and toes stay attached to the identity floor root. Leg markings use the
same weight field as the legs. Idle, attack, and hit clips use rotations and
return to the neutral pose at both endpoints. Production battle cues must
continue to own their timing.

Identical normal and animation accessors share exact packed bytes, preserving
signed zero as well as all authored geometry. Existing byte, vertex, and
accessor limits remain in force. Stored-model tests independently check parent
triangle clearance, closed geometry, shading, source colors, support ownership,
floor clearance, winding, deformation strain, and animation endpoints.

The current stored GLB has 62 parts, 18,482 vertices, 28,632 triangles, 16
joints, and 255 accessors in 704,528 bytes. Its pinned neutral digest is
`06890d739507ac0cd3add1a6da44b954daafcb44f897f9e96e30ea039a597c6f`.
Reproduce the authoring export and run the independent stored-model checks:

```sh
PYTHONPATH=tools python3 tools/raikou_glb.py
PYTHONPATH=tools python3 -m unittest test_raikou_glb -v
```

The export was executed locally and all seven stored-model regressions passed.
Front and oblique neutral renders were inspected after lowering the head,
shortening the throat, shaping the legs, and correcting forehead clearance.
Both focused Blender entry points were executed in temporary directories and
their outputs matched the reviewed GLB byte for byte:

```sh
blender -b -t 2 --python tools/build-battle-species.py -- \
  target/pokemon-native-review/raikou-builder-check/battle_species --only raikou
blender -b -t 2 --python tools/build-actor-props.py -- \
  target/pokemon-native-review/raikou-actor-builder-check/actor_props \
  --only=raikou --skip-preview
```

Those temporary export directories were removed after comparison.
The two focused canonical-routing regressions also passed, including the
250-body catalogue count and shared battle/world source.

The first broader `PYTHONPATH=tools python3 tools/check-johto-models.py` attempt
stopped in `johto_art_sources.read_source` with a compressed source identity
mismatch for `actor-props-totodile-refinement.blend`. A later read-only Blender
export reproduced the runtime Totodile neutral digest exactly. The source had
been saved with native Zstandard compression while its manifest expected gzip.
The existing `store_source` helper normalized that container and updated its
identity record, preserving raw source bytes, all other source records, and
the runtime Totodile GLB. `tools/johto_art_sources.py --check` subsequently
passed. The later full model-authoring check cleared the stale Totodile
part-count and two icon-name expectations, then stopped at a noncanonical
Bayleef GLB. This remaining source/export drift prevents claiming a passing
whole-repository authoring check.

The production registry resolves `RAIKOU` through `Species::Raikou`. Battle
and world prop routing share this canonical GLB, and both build entry points
publish it instead of regenerating the old proxy. The duplicate world JSON
has been removed. The focused
production tests passed at 1.9050 metres, preserving planted paws while the
head, storm mantle, and tail move. The registry schema check, all 42 GPU
skinning regressions, and all five actor-prop regressions also passed.

The native capture examples were built and paired normal/shiny views were
inspected against their matching 2D states. The complete body fits the frame;
the paws stay on the ground, fitted stripes follow the torso and legs, and
the wrapped mask, saber fangs, storm mantle, and lightning tail remain readable.
The normal pair preserved checksum `2461421346`, LCD frame 63, at virtual time
`1.6544675`. The shiny pair preserved checksum `2865683432`, LCD frame 77,
at virtual time `2.719734626`. Each pair reports `state_preserved=true`.
Both palettes use the same stored geometry and rig. These are model and
renderer checks, not evidence of a completed playthrough or whole-game
acceptance. References and review output remain under ignored `target/`.
