# Minneapolis cycling conversations

All 18 existing optional cyclists now have distinct conversations about ordinary
city rides: commuting, market errands, repairs, hills, the river, meeting friends
and heading home. Native source lines fit 18 columns. Actor placement, patrols,
teams, levels, optional challenges and defeat flags remain unchanged.

`refresh_minneapolis_cycling_dialogue` is a Rust text-only publisher for an
already-installed cycling cast. It requires all 18 distinct native bike actors
and replaces only their Seen/Won/Lost text programs and parsed text caches.
It preserves the free rebuilt Bicycle reward at the Minneapolis workshop after
the first badge. Applying the same source again produces the same content.
The original actor-adding builder is not an update path for installed riders.

Executed in the isolated review worktree:

```sh
CARGO_TARGET_DIR=/Users/ryanculligan/GitHub/geothite/target/verdigrim-scene \
  cargo build --locked -p crystal-assets --bin refresh_minneapolis_cycling_dialogue
/Users/ryanculligan/GitHub/geothite/target/verdigrim-scene/debug/refresh_minneapolis_cycling_dialogue \
  content-packs/devils-kettle-rainy-cabin-volunteer-v01.crystalpack \
  planning/minnesota/trainers/minneapolis-cyclists.json \
  content-packs/minneapolis-cycling-rainy-review-v03.crystalpack
```

The ignored result reopened successfully: 731 maps, 476 global species,
272 moves and 602 audio programs; SHA256
`e9f9adc1f01a9922f3eafccc51c364e6b491d0f3ef4e66033ec89c9ee3e5e7a4`.
This includes the separate Kettle, clinic and cabin revisions and preserves the
Rainy mapping lineage. It is unselected; this document does not authorize changing
the active mapping or public release pointers.

The read-only `audit_minneapolis_cycling_dialogue.py BEFORE AFTER SOURCE` passed:
54 intended text programs, all other city fields and 730 other maps exact,
including every team, patrol, battle program, workshop reward, art and audio.
Four negative mutations reject patrol, battle, reward and compiled-text changes.
All 602 PCM hashes, frame counts and loop ranges passed the audio verifier.

Five existing production-controller regressions passed on this exact pack:
`minneapolis_cyclist_victory_runs_the_authored_return_script`,
`minneapolis_cyclists_controller_decline_keeps_battle_closed`,
`minneapolis_cyclists_controller_patrol_stays_on_authored_lane`, and
`minneapolis_cyclists_controller_talk_turn_and_continue` (three distinct riders,
real turn/PP mutation and battle Continue), plus
`progression_rewards_bike_badge_denial_real_gift_and_continue_no_duplicate`.
The Bicycle case uses a native Pouchid, checks unchanged money, earns the gift
through dialogue, continues without duplication, registers the actual Key Item,
walks outside and rides using SELECT. These use declared local parties and
placement; they do not establish full earned-campaign difficulty.

The v01 native render exposed an orphan word on the next page. The v03 source
uses deliberate two-line introduction pages. Original packs and rejected images
remain retained. The held v03 save then exposed a real controller Continue bug:
the saved script has an active yes/no prompt, but loading leaves its production
cursor absent. MCP `move down` fails with `menu ui:yes-no has no valid cursor`;
the headless renderer rejects `yes/no prompt is active without a valid cursor`.
The earlier native case continued later, inside battle, and missed this boundary.

A focused repair in `field_dialogue_resume.rs` rebuilds legacy prompt
cursors and persists new selections on the checkpoint copy with script/index
validation. The cyclist regression now saves an actual selected NO without
answering, checks saving leaves live state unchanged, continues the held prompt,
and requires that NO selection before accepting through ordinary input. This
passes for three riders in the five-case follow-up
`.local/cycling-core-fix-native-v04.log`, together with both cabin cases, the
complete Kettlurk recovery return and the Bicycle case.

The freshly built production client now passes nine MCP calls from the original
legacy held save: default YES is visible, field movement remains blocked, a
single `press down` selects the displayed NO, a new process preserves that NO,
and confirming actually declines without battle or money changes. Both stderr
files are empty. UI selection uses `press`; the field `move` tool can retry a
blocked directional step and must not be treated as a single menu key.

The freshly built saved-session renderer also passes headless native 2D capture.
The settled v04 image was inspected: the complete question fits two lines and
the restored YES/NO cursor is visible. The earlier partial, orphan-page and
runtime-error captures and receipts remain retained. This accepts the scoped
dialogue/held-Continue repair, not a full campaign or deployment.

Publisher compilation and scoped formatting passed. Asset-crate all-target
Clippy passes with the documented baseline allowances and no diagnostics in the
new publisher/cabin leaves. Bevy all-target Clippy also completes with the
documented baseline allowances and no diagnostics in the new save-resume leaf.
The rebuilt controller regressions and fresh production-client verification pass.
Only public API comments were clarified afterward; behavior is unchanged.
Whole-workspace formatting still has the recorded unrelated baseline differences.
Generated packs, saves, captures and verification logs remain ignored.

The selected mapping parent has since advanced through the State Fair facade
and Duluth stair revisions. This Rainy-based candidate remains unselected and
must be composed with those accepted changes before campaign integration.
