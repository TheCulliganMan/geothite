# Red River prairie seed bank

This terrain revision gives the existing seed grower and flower plot a short
downhill approach and two yellow-grass return gates. The open farm landscape,
two river crossings, original hayfield drop and cottonwood approach remain.
It introduces no new quest, reward, trainer, fishing restriction or field gate.

## Authoritative layout and connections

The complete 48×24-metatile route is bound in [`source-v03.json`](source-v03.json),
SHA256 `2c0cb252df3534b42595afc6c547ad2fa72a867067727bb769b14148dba53bac`.
The western chunk is `MnR16C00R00`, 32×24; the eastern chunk is
`MnR16C01R00`, 16×24. The source pins both whole native modules, their original
rows and final rows. Only fifteen western metatiles change. Native blocks57,
61 and84 supply the low hop, permanent cottonwood shrubs and yellow grass;
no pixels, palettes, collision definitions or tileset aliases are added.

Walking coordinates below are 16px tiles, twice the metatile coordinates.

| Feature | Native coordinates and behavior |
| --- | --- |
| Seed-bank left lane | Upper(42,25), lip(42,26), landing(42,28), reverse wall(42,27) |
| Seed-bank right lane | Upper(43,25), lip(43,26), landing(43,28), reverse wall(43,27) |
| Side closure | Permanent Headbutt shrubs at(41,27)/(44,27); Headbutt does not remove them |
| Ordinary return | At least four yellow-grass entries; shortest returns13/15steps |
| Western grass gate | Core rectangle[36,22,38,34); joins the existing northern meadow |
| Eastern grass gate | Core rectangle[50,24,52,32); joins the existing encounter field |
| Existing destination | Seed grower(47,29), approached from(47,30), original dialogue/rewards exact |
| Existing hayfield | Both upper(52/53,15), lip(52/53,16), landing(52/53,18) lanes remain exact |
| River crossings | Both rows23/24 atx10–27 and rows29/30 atx60–67 remain exact |
| Western road | Route(0,24) ↔ Moorhead(31,12), existing connection |
| Eastern road | Route global(95,24) ↔ Detroit Lakes approach(0,18), existing connection |
| Southern branch | Existing western-chunk south connection to Former Shores, offset14 |

Both hops are dry downhill. Walking uphill uses either grassy end of the
windbreak; neither the reverse face nor its adjacent shrubs provides a bypass.
The complete ordinary route retains its eastern cottonwood encounter crossing.
The existing nine trainers, three residents, four caches, signs, current
Minnesota parties and habitat tables retain their original programs and access.
The western-chunk battle-versus-grass alternative is unchanged. Water and all
shores remain exact, including the starting region's open fishing banks.

## Review and native verification

Source previews take about0.3seconds after loading the pack. Both complete
route renders and six decision cameras are retained in
`renders/r16-seed-bank-source-v1400/`. Earlier sources remain immutable:
v01 had a two-grass return and v02 retained a three-grass corner bypass.
Those failed proposals were corrected without weakening the four-grass rule.

Rust `revise_r16_seed_bank` publishes only the native block array, encoded
mirror, stage marker and derived identity. Its independent whole-root audit
in `renders/r16-seed-bank-preservation-v1401/` confirms all742other maps,
actors, scripts, services, water, fishing catalogs, native attribute tables,
regional fauna, art and audio remain exact.

The standalone candidate is
`content-packs/minnesota-r16-seed-bank-v01.crystalpack`, SHA256
`38366f43092e040d40e1084ac4437cef967d812029d4e0c9e04d160b06139df7`,
content identity`c9c4af424190022ac77a1588010edc8edade235ea9fad5f6973f2a93a2e25d7d`.
The nineteen-stage `recipe-v08.json` reproduces these bytes as
`content-packs/minnesota-reviewed-maps-v08.crystalpack`, now selected locally in
`world.json`. Its source index binds both complete R16 chunks; default source
resolution now uses v08. The previous v07 rebuild remains byte-identical.
Both affected crates pass all-target Clippy, all602audio checks pass, nine
publisher refusals and six corrupted-index cases reject. Owned Rust formatting
passes; workspace formatting retains pre-existing offset-allocator differences.
The selected receipt is `renders/selected-r16-seed-bank-v1410/receipt.json`.

All six complete native640×576frames in `renders/r16-seed-bank-native-v1402/`
match the source exactly and were visually inspected. Three production
controller cases in `renders/r16-seed-bank-controller-v1403/` pass both new
hop lanes, reverse/side refusal, grassy returns, original grower dialogue,
both existing bridges in both directions, both original hayfield lanes,
the chunk seam, a real new-grass wild encounter/native Run and Save/Continue.
Geometry uses an empty-party local fixture with prior route trainers defeated;
the wild case declares an original level40Borealynx with Smoke Ball and zero
badges. Neither fixture claims an earned regional journey or early-team balance.
There are no later position, party, progress, bag, HP/PP or RNG writes.

Two production stdio processes pass186calls with empty stderr in
`renders/r16-seed-bank-stdio-v1404/`, including both hops, grassy returns,
closed sides, dialogue input ownership, Start and fresh-process Continue.

```sh
CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-assets --bin revise_r16_seed_bank
target/verdigrim-scene/debug/revise_r16_seed_bank \
  content-packs/minnesota-reviewed-maps-v07.crystalpack \
  d94e6ccfa6460f58730fff0e5b23a4167a050cc0709214625af9199718e70a7f \
  planning/minnesota/routes/r16-seed-bank-v01/source-v03.json \
  content-packs/minnesota-r16-seed-bank-FRESH.crystalpack
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_r16_seed_bank.py \
  --source planning/minnesota/routes/r16-seed-bank-v01/source-v03.json \
  --output planning/minnesota/renders/r16-seed-bank-FRESH
```

Use fresh ignored outputs and serialize Cargo/publication with
`.local/minnesota-cargo.slot.lock`. All source renders, rejected renders,
production frames, native checkpoints and transcripts are retained.
The full regional quality and earned-progression objective remains active.

Both early zero-badge fishing courses pass on the same bytes in
`renders/r16-seed-bank-early-fishing-v1405/`. Two actual played night/rain views
in `renders/r16-seed-bank-played-native-v1406/` were inspected independently of
the clear-day fixtures. The108-map outdoor atlas in
`renders/r16-seed-bank-state-v1407/` retains the same actual connections,
footprints and water-boundary findings as v07. The full regional225-species
placement audit passes5,454references, including all regional fishing groups.

Run all three current native courses against the selected pack with a fresh
ignored save directory:

```sh
CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-reviewed-maps-v08.crystalpack" \
CRYSTAL_SEED_BANK_REVIEW_DIR="$PWD/planning/minnesota/renders/r16-native-FRESH" \
CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo test --locked -p crystal-bevy --lib seed_bank_controller_ \
  -- --ignored --nocapture --test-threads=1
```
