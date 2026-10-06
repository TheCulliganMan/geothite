# Source-specific articulated human kit

This kit supplies one original, solid 3D interpretation for each of the 74
resolved human sprite-family IDs present in the supported external pack. The
retained `outdoorsman` development look is the 75th rig; no source family is
silently routed to it. `SOURCE_KINDS` in `src/johto_characters.rs` is the exact
runtime contract. `chris` and `kris` use the established `trainer` and
`trainer_female` filenames. All other families have their own matching filename.

## Authoring and identity

`tools/build-johto-characters.py` contains the complete original geometry and
explicit per-family art direction. Shared anatomy is intentional; an outfit,
hair, hat or accessory difference is real geometry, rather than a recolored
generic role alias. Examples include nurse cap and hair rolls; clerk apron;
Rocket uniform and raised insignia; sailor collar, cap and anchor; martial-arts
wraps; kimono and obi; long coat and spectacles; grandparent hair, shawl and cane;
named cast hair, capes, masks, swimwear, uniforms and personal accessories.

The source family identifies a sprite design, not necessarily an individual
person. `cooltrainer_f` does not establish a unique named trainer, for example.
The authored 3D palettes are deliberate art direction, not a claim that every
runtime palette substitution or user customization is known from a source ID.
Unknown, malformed, icon, effect, mount and `remote_player` sources return no
human model. A bike must use a posed rider/mount composite, never a walking rig.

## Runtime contract

- Sixteen rigid articulated meshes and their named joint hierarchy
- Blender source uses +Z up, front -Y; runtime uses +Y up, front +Z
- Floor-centered actor root, with unchanged production hip/knee/ankle pivots at
  0.66, 0.39 and 0.12 units, respectively
- Smooth authored corner normals and linear-space vertex material colors
- One immutable shared geometry table, plus lazy per-family rig assembly; viewing
  one family does not assemble all rigs
- Appearance data cannot modify game collision, movement, script or warp state

The bones and root contract deliberately stay identical across the kit. Body
width, tailored volumes, garment hems, hair and accessories provide shape
variation without changing production-speed locomotion or foot placement. The
rigs are original stylized interpretations, not extracted game geometry.

## Regeneration and checks

```sh
blender -b -t 2 --python tools/build-johto-characters.py -- \
  target/johto-character-kit --skip-preview
cp target/johto-character-kit/*.rig.json \
  target/johto-character-kit/shared.geometry.json \
  crates/crystal-voxel-view/models/johto_characters/
python3 tools/check-johto-characters.py
python3 tools/test_johto_character_geometry.py
cargo test -p crystal-voxel-view johto_characters
```

Omit `--skip-preview` for the studio lineup. `--only=nurse,rocket,erika` builds a
small iteration set. Ten generated editable Blender files each hold at most eight designs, bounding
export-time memory. They contain semantic mesh objects parented to real joint
empties, with one collection per design and source/design custom properties. Regenerated Blender sources must also be
repacked through the repository's source-transport workflow before publication.
Preview images and temporary output directories are verification artifacts.

The standalone geometry check rejects missing families, alias-only designs,
duplicate complete shapes, mismatched Rust references, malformed primitives,
invalid normals, altered bind pivots and incorrect shoe grounding. Rust tests
exercise exact source matching, unknown refusal, hierarchy parsing and every
rig's foot and normal contract. These tests establish asset and identity
contracts; rendered in-game visual review and gameplay regressions remain
separate required integration checks.

## Lossless shared storage (runtime schema 2)

`shared.geometry.json` stores each distinct complete position/normal/index array
exactly once. Each primitive in a v2 rig retains its own material color and
references one geometry index. Joint names, hierarchy, translations, primitive
order, vertex/index order and all numeric values are unchanged. This is exact
storage deduplication: no mesh simplification, welding, quantization, decimation,
normal regeneration or palette reduction takes place. The 75 rigs use 977 shared
geometries and occupy 6,444,167 bytes together with the table, down from
33,650,186 inline bytes (80.85% smaller). Editable Blender sources are unchanged.

The table is ordered by the SHA-256 of canonical geometry arrays, and each rig
identifies the complete table by its SHA-256. The runtime rejects wrong library
identities, missing/out-of-range references, mixed schemas, malformed geometry,
and invalid materials. It retains the original v1 inline reader. The exact v1
normalization path is used after expansion, preserving rendered normal bits.

The Blender generator automatically runs the dependency-free packing step after
export. Always copy the complete set of generated rigs **and** its matching
shared table together. `--only` is for a separate preview/iteration directory;
its small table is not compatible with the full catalog. To update an existing
full export directory with `--only`, retain the existing complete rig set and
shared table there: the packer reads the unchanged compact rigs and new inline
rigs, then repacks the complete set consistently.

The storage tool can also operate independently of Blender:

```sh
python3 tools/johto_character_geometry.py pack INLINE_DIRECTORY COMPACT_DIRECTORY
python3 tools/johto_character_geometry.py expand COMPACT_DIRECTORY INLINE_DIRECTORY
python3 tools/johto_character_geometry.py verify BEFORE_DIRECTORY AFTER_DIRECTORY \
  --report /tmp/character-equivalence.json
```

`verify` compares the complete canonical expanded documents and separately hashes
the exact little-endian f32 bits for positions, normals, material colors and
joint translations, along with integer indices, parent bindings, names and
ordering. Verification reports are scratch artifacts, not shipped catalogs.
The regular character checker also requires byte-identical deterministic
repacking, while the focused storage tests cover signed zero, material instances,
input-order independence, corrupt references, library identity and round trips.
