# Route 5 grass pocket and low bank

The Great Ball pocket now has twelve additional native grass quadrants and two
short downward ledge lanes. Ordinary return crosses at least two grass tiles;
the forest cap closes the dry side bypass. Existing grass and tall-cliff stairs
are preserved. One old sign now says STAIRS BOTH WAYS.

Eight metatiles change. The three-chunk route has 220 grass quadrants in its
upper chunk. Source layouts v03 include the required reject_cliff_cuts field.
The negative control in cache-bank-source-proof-v02.json removes the forest cap
and exposes a zero-grass bypass.

production-cache-bank-v03.json records 2305 production calls, nine wild battles,
three in the added grass, both drops, reverse denials, the genuine once-only
Great Ball pickup and Continue. Earlier harness attempts stopped on x01 quantity
formatting and remembered Pack pocket; their actual reached saves were retained.
Five native 2D views were inspected with zero source/native pixel differences.

The tested v01 pack was rebased using the guarded Rust publisher on the latest
selected Route 17 talus successor. current-rebase-inheritance-v03.json proves
complete Route 5 map/art/encounter/runtime inheritance; selection additionally
checks battle data and five trainer teams. Earlier runtime logs are inherited
through exact inputs, not represented as fresh runs on v03.

Reproduce publication with portable-r05-cache-bank-tools-v01/revise_r05_cache_bank
under .local/route-grass-review, passing the recorded parent pack and SHA256,
cache-bank-before-v03.layout.json, cache-bank-after-v03.layout.json and a fresh
output path. All-target affected-crate Clippy and owned Rust formatting passed;
workspace formatting still reports unrelated existing differences.

The broader goal remains active. native-hop-model-followup-v01.json records the
required statewide graph correction: ledge lips permit lateral walking and a
jump starts from the current lip tile. Older statewide grass minima need a
fresh audit using those native semantics, including inland ice sliding.
