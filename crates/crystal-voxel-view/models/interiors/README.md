# Original full-volume interior kit

68 independently authored reusable assets for domestic, care, retail and civic
interiors. Models contain only original geometry and flat material colors. They
contain no game pixels, source textures, animation commands or content-pack data.

## Assets

- Seating: timber stool, arcade stool, chair, padded link seat, floor cushion,
  station bench
- Tables: dining table, traditional low table, pedestal café table, laboratory
  display table
- Domestic: four bed colors, potted foliage, long planter, flowering stand,
  bookcase, laboratory bookcase, paneled cupboard, drawers, pendulum clock,
  computer desk, laboratory workstation, TV, radio, game console, keyboard,
  refrigerator, kitchen stove and sink, open book, carved memorial
- Retail/care: stocked shelf, gift shelf, glass refrigerator, vending machine,
  service counter, laboratory counter, healing machine, treatment console
- Civic: arcade cabinet and paired row, station turnstile, information terminal,
  broadcasting mixer, laboratory restoration machine, clinical and timber screens
- Architecture: plaster panel, framed window panel, open sliding-door frame,
  wall-mounted radio panel

The generator preserves separately named objects, modeled keys, controls, books,
slats, cloth seams, vessels and other editable geometry. Material-group exports
are compact JSON using right-handed +Y up / +Z front coordinates. Source Blender
coordinates are +Z up / -Y front. No generated GLB is required at runtime.

## Reproduction and verification

From the repository root:

```sh
blender -b --threads 2 --python tools/build-johto-interiors.py -- target/johto-interiors
python3 tools/check-interior-models.py
```

`--skip-preview` omits the optional studio render. The output includes an editable
`johto-interiors.blend` and `interiors-kit.png`; these build products are kept under
ignored `target/`. Published editable source is packaged by the repository's art
source manifest, not by committing an extra binary build output.

`src/interior_models.rs` validates and imports these meshes.
`src/mesh/modeled_interiors.rs` owns source-complete placement and labels every
successfully appended source cell. Ground sampling is verified before reserving
cells; incomplete drawings and changed live profiles retain their legacy path.

Second-pass additions: source-distinct feathery/polkadot/Pikachu quilts; magna,
tropical and jumbo plants; domestic/traditional/clinical/acoustic wall panels;
mitered picture frame; closed stair flight with nosings and handrails; office
telephone. The original artwork is retained inside poster frames at runtime.

Final civic refinements: `broadcast_rack` and `tower_reception`. Their role-specific
geometry avoids substituting a retail vending machine for radio equipment or an
empty counter for a staffed service workstation. The Rust surface-finishing pass
is separate from this mesh catalog and does not count floor materials as objects.
