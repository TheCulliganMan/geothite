# Special-environment extension

Eleven additional original full-volume models are authored by
`tools/build-special-environments.py`. They are separate from the frozen initial
nineteen-model dungeon kit. The generator imports no game artwork or content
pack and leaves named editable parts in `special-environments.blend`.

## Rebuild

```sh
blender -b --python tools/build-special-environments.py -- target/special-environments
python3 tools/check-special-environments.py
cargo test -p crystal-voxel-view dungeon_extension
```

Copy only the eleven validated JSON files to `models/dungeon_extensions/`.
The shared editable-source packaging workflow consumes the generated uncompressed
Blender scene. Preview renders and unpacked source scenes remain build output.

## Actual source roles

- Ruins and all four word-room atlases: modeled sandstone friezes retain each
  original live 8px clue glyph as a face sample. Four-quadrant floor letters stay
  on their exact source plots in stone inlays. Changed or incomplete letters
  remain unclaimed. Puzzle consoles receive original stone daises and unchanged
  live puzzle-panel artwork. Guardian sculptures also cover the exact
  mixed guardian/console drawings used in puzzle chambers
- Elite Four: native complete League podiums retain their sphere silhouettes;
  new ceremonial wall panels follow complete north-wall courses
- Champion: exact two-by-four drawings receive original horned, winged dragon
  sculptures and the original central carpet remains at the zero-height datum
- Fast Ship: closed corridor volumes consume the rear cap and exactly two face
  courses. Front panels select physical portholes, recessed doors or plain steel
  according to their source drawing; upper void remains untouched
- Ports: the full eight-block ship drawing receives an original ocean-going
  ferry with bow, cabin, bridge, funnel, deck rails and the north-side gangway
  opening. Full metatile phase plus sixteen bow/cabin/keel art anchors are
  required. Separate native mooring-post and pipe-rail drawings remain separate
  fixtures. Animated sea cells remain on the water path
- Port passages, Underground Path and Saffron Gym: connected walls use only the
  existing source-art wall vocabulary. Floor, stair and teleport-pad drawings
  cannot become walls. Saffron's complete guardian drawing is a sculpture
- Hall of Fame: its exact six-tile terminal becomes a physical recording console
- Tin Tower Roof: complete verified roof bays form continuous pitched wings
  sloping away below the central walkway. Native ridge tiles become rounded
  geometry; central planks retain their exact zero-height top. A clipped wing
  without a known ridge/landing neighbor remains fallback

Stone, ceramic, carpet and timber surfaces are genuine thin volumes below their
existing actor datum. Only known repeated floor vocabulary is matched. They do
not classify arbitrary unknown art as floor, nor turn collision into geometry.

## Integration and verification

The extension is a child of the existing dungeon resolver and shares its one
claim mask. It emits source-index labels through the existing authored-coverage
contract. Every model include is installed before the registry references it.
The original nineteen mesh JSON files are unchanged.

Geometric validation covers bounds, triangles, finite values, unit normals,
winding, opaque materials and asset budgets. Rust regressions cover live glyph
UVs, incomplete letter rejection, complete ship courses, ship signature/anchor
rejection, teleport protection, below-datum floors and continuous roof seams.
Pack audit and native captures remain necessary before claiming every source
asset is complete. Unmodeled artwork is reported as fallback rather than hidden
by a broad map-support flag.
