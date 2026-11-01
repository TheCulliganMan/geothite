# Original facility and radio desk kit

Six compact runtime meshes are built from independently authored closed,
faceted parts. Their muted paper, enamel and teal palette uses the shared
interior face-light bake after runtime fitting. No original game art, actor,
collision or event data is embedded in a model.

Radio placement adapts the named parts in Rust to the live source palette and
projected proportions: worktops are eight pixels high and eight pixels deep;
service returns keep their full length and join the rail. Phones, memo faces,
and the attached terminal keep separate fitting transforms. Native gray, white,
and ink swatches replace the authored preview colors. Operator stools retain
their geometry and support while sampling their own cap and dark-side source
swatches through the room’s live palette.
The facility workbench and instrument-bank presentation paths are unchanged.

- `facility_workbench`: observed offset monitor and keyboard on the left,
  upright secondary apparatus on the right, common bench with open legs
- `radio_phone_desk` and `radio_memo_desk`: physical telephone, shallow drawer,
  folded worktop and lower knee recess; the latter also carries the live memo
- `radio_reception_u` and `radio_reception_l`: coherent connected service
  counters whose full-height native staff openings remain empty
- `radio_counter_extension`: short telephone counter adjoining the existing
  Radio 5F broadcast terminal

The paired gauge cabinets reuse the exact existing
`special_rooms/facility_instrument_bank` mesh. Radio block `08` stools reuse
`interiors/arcade_stool`; neither fixture receives a redundant mesh.

Bindings in `mesh/facility_radio_source.rs` and `facility_radio_bindings.rs` verify
complete source drawings before ownership. Mixed selector plots are not object
footprints: floor cells, entry carpets, stairs, studio side-wall edges, staff
openings and neighboring furniture remain outside sparse claims. Native floor
samples stay live, as do meaningful cropped readout and memo faces. Custom live
profiles, clipped drawings and absent native backing all cause safe fallback.

```sh
python tools/build-facility-radio.py target/facility-radio --runtime-only
blender -b --threads 2 --python tools/build-facility-radio.py -- target/facility-radio
python tools/check-facility-radio.py --pack content-packs/core-modular.browser.crystalpack
cargo test -p crystal-voxel-view facility_radio
```

Editable `facility-workbench.blend` and `radio-desks.blend` sources use the
repository's hash-verified 64 KiB chunk format. They retain named geometric parts,
materials, hidden canonical asset collections and separate front/rear preview
arrangements. Runtime exports and editable objects share the same generator.

Radio terminal finishing uses complete source drawings separately from the mixing
desk model. The standalone `1c` terminal and rear `12` PCs reuse the selected
computer's CRT and keyboard over a closed eight-pixel cabinet, with native gray,
white and ink. The rear drawing's keyboard stays upward; only its bottom plinth
art faces forward. The U reception monitor remains owned by the complete counter.
Its LCD uses the actual source display across two tile rows, below the white cap.
Unknown materials or a missing display/desktop retain source-art fallback.
