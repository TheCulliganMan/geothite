# Prairie road battle sequence

Three ordinary Route 16 encounters for the next combined release. The base is
selected Crossings (`minnesota-crossings-v01.crystalpack`, SHA-256
`caecd60747ee26510651509c6567a3490d919c0d6c9f8c2528d9445de2ad1599`).
This revision does not select a pack or publish a release.

The installed teams on routes 12–18 were inspected directly: 47 trainers across
Gunflint, League Approach, Pine Country, Shiner Narrows, the prairie, Border Pines
and Mille Lacs. The ten prairie trainers rotate a small species pool, usually
three members with their latest four moves. Three short battles give that road
more variation without adding another trainer, reward or compulsory detour.

| Trainer | Authored party, level 31 | Battle idea and counterplay |
| --- | --- | --- |
| Lyle, `MN_R16_T01` | Flossail: Gust, Sludge, Cotton Spore; Cacheek: Headbutt, Dig, Quick Attack | Speed control followed by priority. Switching clears Cotton Spore; a flying partner avoids Dig, but still faces Headbutt. Stun Spore is removed; Sludge retains its usual poison chance. |
| June, `MN_R16_T04` | Zapsap: Spark, Mega Drain, Light Screen; Totlit: Headbutt, Flame Wheel, Defense Curl, Rollout | A screen and a rolling attacker. Physical moves bypass Light Screen; Ground attacks punish Zapsap. Totlit needs setup and consecutive hits. Ground, Water or Fighting pressure can stop the chain. |
| Tom, `MN_R16_T06` | Cacheek: Super Fang, Tail Whip, Quick Attack; Rumblehide: Dig, Spark, Stomp | Chip damage, defense reduction, then stronger physical attacks. Switching clears Tail Whip; Flying avoids Dig but must respect Spark. Super Fang cannot finish a healthy target by itself. |

All three previously had three members. Their ace level remains 31, and each
retained member was already level 31. The selected Crossings pack disables
automatic trainer scaling and general
trainer rematches. This revision preserves those settings and the existing
species, levels, items and DVs. Ordinary AI remains responsible for move
choice; these are supported tactics, not a scripted move sequence.

The lines keep the existing people: Lyle salvages his father's gate for a bench,
June recognizes her winter ski turn among the flowers, and Tom takes a break
from painting. Each has new Seen, Won, Lost and ReturnText pages. All are short,
explicit native pages. `trainers.json` is this revision's focused authoring input;
apply it after broad story overlays so those older lines cannot replace it.

## Reference comparison

[Crystal's primary trainer data](https://github.com/pret/pokecrystal/blob/master/data/trainers/parties.asm)
contrasts Ralph's single Goldeen with Justin's repeated Magikarp. Later Ralph
and Tully rematches use deliberately selected support moves and finishing moves.
The lesson here is a recognizable battle idea and varied length, with a stronger
identity than repeating a full generic team. Its [Route 32 event source](https://github.com/pret/pokecrystal/blob/master/maps/Route32.asm)
also separates the challenge, defeat line and return conversation through the
normal defeated flag.

The existing external reference clones were read in place with `git show`:

- Polished Crystal `f8e67fb`, `data/trainers/parties.asm`: Ralph5 adds Lanturn
  between Qwilfish and Seaking; Tully4 gives Poliwrath Rain Dance and Seaking
  Agility. Distinct member jobs can develop a familiar local trainer.
- Crystal Legacy `77e6b38`, the same file: Justin uses one Tentacool; Henry uses
  Marill and Poliwag; Edgar's Octillery has selected coverage. An ordinary route
  battle can be compact and recognizable without becoming a miniature boss.

No source parties, species, dialogue or assets from those games were imported.

## Integration and verification

`install_northstar_route_challenges(&mut pack)` is exported by `crystal-assets`.
Call it after regional story overlays and before the combined release derives
its identity/save policy. It deliberately rejects changed parent teams.

The installer validates native move legality, active roster membership, every
scaled team through ace level 100, and exact first-battle move retention. It
checks existing sight rays against walkable collision and connected-road reachability.
The standing actors remain at `(13,16)` left/range 1, `(38,39)` right/range 2,
and `(11,11)` down/range 2 on the next chunk. No actor or lane is moved.

The static external-pack test compares every map's terrain, events, trainer
requests and actors, all unrelated trainer teams, audio and runtime files. It
also compares the complete remaining data set and verifies pack write/readback.
Defeated flags and callbacks are identical. A defeated ordinary trainer retains
the return conversation; this revision does not add a repeat-battle invitation.
The static scaling sweep also checks compatibility with the existing helper;
it does not enable that optional policy in the selected pack.

```sh
CARGO_INCREMENTAL=0 cargo test --locked -j 1 -p crystal-assets --lib route_challenge_pages_fit_native_text
CARGO_INCREMENTAL=0 \
CRYSTAL_ROUTE_CHALLENGE_BASE="$PWD"/content-packs/minnesota-crossings-v01.crystalpack \
CRYSTAL_ROUTE_CHALLENGE_OUTPUT="$PWD"/target/route-challenges-v01/review.crystalpack \
cargo test --locked -j 1 -p crystal-assets --lib route_challenges_preserve_world_and_check_scaled_teams -- --ignored --nocapture
```

The output must be fresh; omit `CRYSTAL_ROUTE_CHALLENGE_OUTPUT` for a repeat data
check. Native-art crops in `target/route-challenges-v01/three-trainers.png` show
the actual pack tiles and standing trainer frames. They are static art studies,
not gameplay screenshots or proof of rendered battle dialogue. No controller
inputs or public deployment are part of this review.

### Verified candidate, November 18

Both named tests passed (2 passed, 0 failed) against Crossings using the shared
current-source executable `target/northstar-next-combined/field-tests`. Its test
list was checked before execution: a frozen Lakeside cache had previously
returned zero tests and was not accepted as evidence. The successful run is
`target/field-review-20261118/trainers.log`.

The written and reread candidate is `target/route-challenges-v01/review.crystalpack`:

- File SHA-256: `9ff53c50b6fbc980db246352b2dfc9ad6d986c1859835b3c67c81bdd61b2431f`.
- Content hash: `5f24417ba9579d0be92558e4093ccb479305a6e89e1adf549b8d055ebbfafc1e`.
- All twelve installed speech programs independently match their authored pages.
- Native-art crops were regenerated from these exact candidate bytes and inspected.
- Both `minnesota_trainer_scaling` and `trainer_rematches` remain false.

The production damage formula, using the pack's level-31 trainer stats/DVs,
no badges, no critical hit, neutral weather and minimum/maximum random rolls,
gives Totlit's Rollout into Flossail **15–18** damage normally, **30–36** after
Defense Curl and **61–72** on the next curled hit. Against Cacheek the same
ranges are **3–4**, **6–8**, and **13–16**. This supports retaining the modest
30-power attack with a visible setup turn: a resistant switch greatly reduces
the pressure, while ignoring the accumulating hits is costly. These are static
damage calculations, not an AI-turn simulation or a player-team balance sweep.

No gameplay inputs, save edits, pack selection or deployment were performed.
The combined release still owns final integration and save compatibility.

The final working-tree `cargo fmt --all -- --check`, `git diff --check` and
`cargo clippy --locked -j 1 -p crystal-assets --all-targets` passed. The only new
lint warning (a test helper placed after its module) was corrected by moving
the helper, then `--lib --tests` passed with no diagnostics in the new files.
Fresh dependency listings verified all five current workstreams and the combined
builder/CLI; final source hashes stayed unchanged across the follow-up. Existing
warnings remain elsewhere. See `target/route-challenges-v01/lint-receipt.md`.
