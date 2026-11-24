# Regional Pokédex ordering

The authored sections already placed ordinary families before the mosquito
pseudo-legendary line, then legendaries and mythicals. The source-number table
had drifted: five active legendaries preceded 31 ordinary replacement species,
and the mosquito line came last. Regenerating the table from its sections keeps
the first 186 active numbers unchanged and corrects the final 39 entries.

The source audit rejects divergent section/table order, missing or repeated
families, separated evolution stages, misplaced starters and ordinary families
after special families. Its publication check compares every active identity
and number. Archived identities remain archived.

The ordering-only Rust builder accepts the exact published Fair presentation
raw and hosted archives. It verifies complete pack preservation after restoring
only the regional-number constants and release metadata. All maps, encounters,
species records, artwork and audio remain identical. Schema 11 adds the two
Fair presentation save identities and retains the 22 inherited identities.
Direct parent saves require no terrain or actor reconciliation.

```sh
CARGO_INCREMENTAL=0 cargo build --locked -p crystal-assets \
  --example pack_northstar_dex_order --bin verify_crystalpack
target/debug/examples/pack_northstar_dex_order \
  content-packs/minnesota-fair-presentation-v01.crystalpack \
  target/fair-presentation-20261122-r2/hosted.crystalpack \
  target/pokedex-order-release/input.crystalpack \
  target/pokedex-order-release/hosted.crystalpack
target/debug/verify_crystalpack target/pokedex-order-release/input.crystalpack --desktop --audio
target/debug/verify_crystalpack target/pokedex-order-release/hosted.crystalpack --desktop --audio
```

Generated packs, verification fixtures, release scripts and browser captures
remain ignored. Builds and publication use an export of the committed source,
excluding unrelated cave work in the shared checkout.
