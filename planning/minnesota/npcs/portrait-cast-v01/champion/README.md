# Casey’s overworld sprite

The six 16×16 poses in [field-pixels-v1.json](field-pixels-v1.json) carry
the battle portrait’s swept copper hair, dark high collar and copper cape
into the field. The native renderer mirrors the left-facing poses for right.
The field bank takes its four colors from the portrait already in the input pack.

Build a local candidate over the Fair Presentation pack or a successor with the
same Champion-room actor. Run from the repository root with fresh ignored paths.

```sh
mkdir -p target/casey-field-review
cargo run --locked -p crystal-assets --example pack_northstar_casey_field -- \
  path/to/fair-presentation.crystalpack target/casey-field-review/candidate.crystalpack
cargo run --locked -p crystal-assets --bin verify_crystalpack -- \
  target/casey-field-review/candidate.crystalpack --desktop
```

The builder binds `SPRITE_MN_CASEY` to the south-facing Champion-room actor at
`(5, 7)`. The output is a local pack with its own content identity.
