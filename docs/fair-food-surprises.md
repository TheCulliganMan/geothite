# Fair purchase surprises

A successful purchase at one of the counters below gets one 20% roll per
transaction, regardless of quantity. The item and normal payment commit together
with the resolved encounter. Cancelling, failed purchases and browsing do not
roll. The receipt appears first, followed by a short food reveal. A winning
purchase closes the shop before starting one normal catchable wild battle.

| Item ID | Counter IDs (prefixed `MART_`) | Basic form |
| --- | --- | --- |
| `FAIR_CHEESE_CURDS` | `MnFairCurdsVendor` | Curdlet |
| `FAIR_HOTDISH` | `MnFairCurdsVendor` | Totlit |
| `FAIR_COOKIE_PAIL` | `MnFairCookieVendor`, `MnFairBakerVendor` | Doughlin |
| `FAIR_KETTLE_CORN` | `MnFairKettleVendor`, `MnFairGardenSnack` | Pipcorn |

These mappings require `MnStateFairgrounds`. Encounters stay at level 10 with
the basic form, even with a high-level party. Existing grass tables are unchanged.
Each win immediately ends that shop visit after its reveal; several purchases
cannot queue a chain of encounters. If no living non-Egg partner is available,
the purchase still succeeds and no battle starts.

The deterministic Crystal RNG rejects byte values 250–255, then treats 0–49 as
a win: exactly 50 of 250 accepted results. The complete Pokémon is materialized
and saved with the purchase, so continuing a paid pending surprise never rerolls
its outcome or attributes. Consuming that pending state and activating battle
form one mutation. The normal catch, escape, victory and blackout paths own the
result, without additional purchase charges or item delivery.

Other Fair food IDs are deliberately excluded because the roster does not give
a direct match: sweet corn, pretzel, root beer, ice cream, pickle, honey, pronto
pup, mini donuts, funnel cake, fried candy, fried walleye and beer. For example,
Cinderfin's smoked-fish identity is not a fried-walleye match, and Chislet is
butter rather than ice cream. Standard berries, milk, drinks, supplies and
non-Fair shops retain their ordinary behavior.

## Focused checks

Set `CRYSTAL_FAIR_TEST_PACK` to the completed Fair pack to run pack-backed tests;
fixtures supply party, money, location and DIV samples explicitly. The native
controller cases use production input for shopping and battle exit.

```sh
cargo test --locked -p crystal-core --lib fair_food
cargo test --locked -p crystal-assets --lib fair_food_purchase -- --include-ignored --test-threads=1
cargo test --locked -p crystal-bevy --lib fair_food_surprise -- --include-ignored --test-threads=1
```

These are focused transaction and controller regressions, not an uninterrupted
native campaign or acceptance of a newly published executable/pack.
