# Private Gunflint winter art

The frozen `native.contract.json` binds the exact installed `mn_winter_outdoor`
art and the independent `mn_gunflint` behavior. Its SHA-256 is
`7b36b1fd9b8b6e0ddf55f8b03355bd46cbb774c8391a89b300916ff9d26e3ee6`.
This stage adds `mn_winter_gunflint`; it changes no map or existing alias.
Existing Border Pines winter behavior stays as authored. Gunflint retains its
native Cut tree `0f` → `17`, flower/water animations and restless forest trees.
The new alias has the same complete 73 native/mixed metatiles, 256 glyphs,
palettes and authored foregrounds as the existing cold forest art.

The focused asset leaf and CLI were handed to Root for integration:

```
crates/crystal-assets/src/winter_gunflint_alias.rs
crates/crystal-assets/src/winter_gunflint_alias_tests.rs
crates/crystal-assets/src/bin/build_winter_gunflint_alias.rs
```

Root registers `build_winter_gunflint_alias_pack` and publishes with four explicit
arguments: `BASE CONTRACT EXPECTED_PARENT_FILE_SHA NEW_OUTPUT`. The CLI validates
the actual approved parent file SHA and reloads the written pack for complete
typed equality. Output must be fresh. Future Ely/pine/coast additions are allowed
when source art and behavior remain exact; historical baseline file/identity
annotations are provenance, not a global rollback target.

Five asset tests exist: one source-only contract test and four ignored external
tests. Set `CRYSTAL_WINTER_GUNFLINT_ALIAS_TEST_PACK` to the approved parent.
They exercise exact assets/Cut, unchanged maps, stale source/contract/identity,
occupied targets, full inverse residual counterexamples, and unrelated valid
parent composition. The residual removes only the exact alias definition,
six exact files, matching inherited behavior and one expected manifest, then
requires the entire pack to equal its actual parent.

The production renderer helper recognizes this exact private alias. The forest
tree registration predicate uses the resolved animation source. Existing R17
does not gain forest animations. The winter presentation tests now include all
three aliases; set `CRYSTAL_WINTER_RENDER_TEST_PACK` to the completed alias pack.
They compare original animation frame pixels, schedules and registrations in
four time banks, and verify authored icy-grass rustle colors with unchanged
native OAM opacity and separate green/icy caches. An optional fresh
`CRYSTAL_WINTER_RUSTLE_CAPTURE_DIR` retains 32 production frame assets and receipt.
These are CPU renderer assets, not GPU captures or controller proof.

Root published the actual composed Ely/pine/private-alias art parent at
`content-packs/minnesota-superior-inland-art-v01.crystalpack`, file SHA-256
`1c6ba5d59097546ff8b94e09c809d270727ce1272acb93759b89f3b3c7d9b90a`,
identity `317e9c039914d27c83dfed01ef55b4720023867fd9d8d5aef9ea1b1a452aec3e`.
Root reports strict written equality and all five alias asset tests passing in
42.59s. The author consumes this verified art parent for detached source renders.
The author formatted the focused Rust files and checked the scoped diff; Root
owns Cargo, publication and native renderer/controller execution. No GPU or
controller acceptance of new winter terrain follows from this art publication.

The substantial R12/R13 winter terrain candidate is a separate stage. Final
module preimages await the actual coast geography, including its new R12 east
connection. North Shore R11 and Grand Marais remain entirely warm.
