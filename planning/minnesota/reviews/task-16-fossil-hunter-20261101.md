# Route 1 fossil hunter review

The selected local revision is `r06_pine_fork_revision`, whose pack is
`content-packs/minnesota-reviewed-maps-v11.crystalpack`. Its SHA-256 is
`d3c16929aade3df2e4b26406dd1bd1c0ac61f05b05be5f569c41e1657c3899d9`.
It still places `MnFossilBluff` at `(39, 21)` and has the older text. The
checked-off task does not establish an installed fix in that selection.

The unselected candidate `content-packs/minnesota-story-review-fixes-v09.crystalpack`,
SHA-256 `175e419ac0abd0261d6417fdb825eabd3de0a7223b51f45baf1390ea8f29259a`,
places the scientist at `(26, 14)` and contains the revised shell, cliff,
and fossil-choice pages from `story.json`. Its Route 1 block array and map
attributes exactly match the selected pack, and its object count is unchanged.
An inspection of the candidate's source-art composite shows the scientist on
the raised bluff with a clear adjacent floor tile. This is not a native client
capture.

The existing production-controller test
`rochester_new_game_walks_r01_delivers_acknowledged_originals_and_returns_home`
passed against the candidate with `RUST_MIN_STACK=67108864`: it walked Route 1,
completed the delivery, and returned home. A separate collision-graph check
found paths from the hunter's approach to both route exits, excluding actor
bodies and trainer sightlines. The new
`fossil_hunter_cliff_conversation_and_route_remain_accessible` controller test
checks the three opening pages, refusal, a fossil choice, route traversal, and
Start. Its queued Cargo run was stopped at the user's direction; the
coordinating chat owns the remaining validation. The native client view and
installed dialogue still require verification.

```sh
RUST_MIN_STACK=67108864 CARGO_INCREMENTAL=0 \
CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-story-review-fixes-v09.crystalpack" \
cargo test --locked -p crystal-bevy --lib \
  fossil_hunter_cliff_conversation_and_route_remain_accessible -- --ignored --nocapture
```

The coordinator should also run the relevant fossil-choice regression and
inspect the actual native client at the hunter, including readable pages and
traversal. Verify the final integrated pack rather than promoting this candidate
solely on the source-art composite.

The older early-research fossil controller test in
`minnesota_research_access.rs` still starts next to the historical `(39, 21)`
position. Update its fixture for the candidate when coordinating with the
owner of the active changes to that file.
