# Northwoods roster balance

Current authoring restores **LINE087 Jotlet → Memofold → Recallisk** without replacing another family: **223 active species / 101 families**, all three pure Psychic. Approved base-stat totals are **285 / 390 / 500**, with level **24 / 44** evolutions; Recallisk is **90 / 50 / 80 / 45 / 110 / 125**. The source remains 313 species. Reflect and Light Screen are earned level-up moves, with only actual Crystal TM/HM tokens in the machine list.

The final `content-packs/northwoods-sticky-v1.crystalpack` contains **474 species / 270 moves** and passes `--check-pack` with **zero findings**, **22,300 default move sets**, **631 real damage/PP battles** and **122 evolution boundaries**. All six native views and candidate 13 full web integration are verified; all three native production tests pass and release 13 is published, with 2,038 exact public-byte plus six status checks passing. [The restoration runbook](../../../docs/northwoods-sticky-notes.md) records the exact pipeline and preservation checks. The prepack in-memory check also passed and remains retained. Frozen release12 and Fairy-v2 counts below remain historical proof for 220 species.

The frozen published [Fairy pass](../../../docs/northwoods-fairy.md) covers all **220 active
species in 100 families** against authored gym aces at **12, 16, 21, 25, 30, 35,
40 and 45**, followed by the **48–53** League. All 313 authored base-stat spreads
are preserved. Thirteen regional and thirteen returning species gain Fairy;
returning species retain their native moves and gain authored Fairy progression.
The final `northwoods-fairy-v2.crystalpack` has 471 species and 270 moves with exhaustive
per-move categories. Its audit passes with **zero progression findings** across
22,000 default move sets, 623 real damage/PP battles and 120 evolution boundaries.

[The decision record](fairy-balance.json) contains every retype and counter change.
Fairy threatens nine of eleven Dragons, eighteen of twenty-three Dark types and
twenty of twenty-two Fighting types. Poison super-effective targets rise from 25 to 31;
Steel rises from 27 to 37. Fairy itself has practical Poison/Steel answers, including physical
Poison Jab and earlier earned STAB. Existing strong Bugs keep their stat budgets.

The [policy](balance-policy.json) preserves different power bands and evolution
commitments. Ordinary final forms range from **445 to 550** total base stats.
Starters share **318 / 405 / 530**, evolve at **16 / 36**, and use Medium Slow
growth. Legendary and mythical totals remain **580–600**. Mosquitan retains its
**300 / 420 / 600** line, Slow growth, and **30 / 55** evolution levels.
Existing stat spreads were retained after review, including the stronger Bugs.

## Bugs keep their strength

Early evolution remains an advantage. Regalarch keeps **490** total stats and
its level-18 evolution; Dakotaskip keeps **475** and level 22. Their improvements
continue through the campaign instead of ending after the opening routes.

| Family role | Dependable Bug move access | Later payoff |
| --- | --- | --- |
| Regalarch / Dakotaskip, fast special attackers | Signal Beam at 24 | Existing X-Scissor at 44; speed and support utility |
| Hatchrush / Mothosaic, special attackers | Signal Beam at 24 | Water or Psychic coverage and Baton Pass |
| Rivetqueen / Skeeterex / Trapnack, physical attackers | X-Scissor at 28 | Rivetqueen also gets accurate Iron Head at 40 |
| Windhopper / Panoplite / Barricant, physical attackers | X-Scissor at 28 | Electric, Rock or Poison coverage |
| Wrigglet / Pupump / Mosquitan, rare late investment | Signal Beam at 28 / 30; inherited thereafter | X-Scissor at evolution, Megahorn, and Bog Siphon |

## Move and evolution changes

Most ordinary late learnsets were brought into the campaign's level range.
Dependable attacks now arrive before the League; high-power finishers and
special signatures can still reward later training. The lynx starter's authored
curve stays intact. Inheritance checks follow the earliest evolution path and
exclude missed level-one reminder moves.

Pouchid gets special Mega Drain at level9 and Birchit gets physical Aqua Jet at
level9. Both can use their stronger attacking stat before the level-12 first gym,
matching Kindlit's useful early Ember. An explicit starter check protects this
under the permanent split.

Physical Steel specialists get Iron Head instead of relying entirely on
Iron Tail's accuracy. The hockey and frost-fighter lines gain Ice Punch.
Type-changing evolutions get usable attacks for their new types, and physical,
special and mixed attackers retain appropriate options. Support moves remain;
several were rescheduled so the four default moves do not become all status,
conditional damage or unreliable finishers.

Trollure, Castoroar and Liftruss also get physical Waterfall on time, so their
Water attacks can use their higher Attack stat under the permanent split.

Four modest evolution payoffs arrive sooner: Castorock evolves at **32**,
Tamarunt at **30**, and both fossil lines at **34**. The stronger late-evolving
lines retain their longer commitments. Canonical profiles, the editable move and
family mirrors, and the [readable Pokédex](POKEDEX.md) are synchronized.

## Repeatable verification

The final rebuilt pack was verified against current authoring:

```sh
target/agent-northwoods-moves/fairy-ui/debug/audit_pokemon_balance \
  content-packs/northwoods-sticky-v1.crystalpack \
  planning/minnesota/pokemon --check-pack
```

`--check-pack` also rejects published stats, types, growth, TMs, level moves or
evolutions that differ from current authoring. `--check` uses the pack only as a
mechanics reference and tests current profiles in memory, which is useful while
art is being revised. Neither mode publishes a pack or changes source data.

The audit checks every species' role budget, evolution window, early primary
attack, both types' attack access by level 30 or evolution, and a dependable
attack suited to its stronger offensive stat by level 45 or evolution.
Night Shade and Magnitude can fill early type access; they do not satisfy the
stronger-stat attack floor. TMs and reminder-only moves do not satisfy these
natural learnset requirements.

The earlier 219-species production-engine checkpoint covered **21,900 default move sets** across levels 1–100,
**620 battles** requiring real PP expenditure and damage, and all **119 level
evolution boundaries**, including rejection one level early. These checks
establish usable roster data. An earned campaign playthrough is still needed
once regional encounter placement is implemented, to assess availability,
experience pacing, team composition and boss difficulty together.

Local content packs and verification scratch output remain ignored. No returning
species, move power values, maps, encounter tables or audio programs were edited.

## Verified local result

The [balanced local pack](../../../content-packs/northwoods-balanced-v1.crystalpack)
contains **471 maps, 470 species and 264 moves**. All **377 audio programs** match
their PCM hashes, frame counts and loop ranges. Published roster checks pass,
including exact current level moves and all evolution data. A stale-pack control
correctly rejected an earlier Waterfall learnset.

The full authored catalog regression, extension preservation regression, 15
focused catalog/sprite regressions, two split regressions and two audit-tool
regressions pass. These include preserving the original 251 species and their
learnsets, plus original audio and runtime files.

Changed Rust files pass formatting. Workspace `cargo fmt --all -- --check` still
fails on existing formatting beginning in `crates/offset-allocator/src/lib.rs:408`.
Required all-target Clippy stops on the existing `bad_bit_mask` error in
`crates/crystal-core/src/multiplayer/mod.rs:1332`. A diagnostic run allowing that
lint also encounters existing `never_loop` errors in the asset library. With
those two existing lint classes allowed for diagnosis, the audit binary has no
new Clippy warnings. No workspace lint exemptions were added.
