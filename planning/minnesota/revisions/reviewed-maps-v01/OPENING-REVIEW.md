# Opening and recovery on the selected map pack

The preceding selected `minnesota-reviewed-maps-v03.crystalpack` (SHA256
`f79f8a3226ef2b4d97d25b546871359797f006265beaa5dcded344024ec5e89d`)
passes eleven current opening/research/recovery cases. No game content changed
for these checks. The whole Winona–first-route–Rochester source view and the
actual home captures were inspected.

The preceding v04 selection (SHA256
`47d45a3bb934e4ad863d3e92950e29dd55bdceb62d5824ff23952d55e11e7b48`)
changes only Rochester's exterior rows and moved door/guide/sign anchors. Its
native opening/rooms/programs remain exact. The first-route delivery and full
home return, three early-research cases and earned first badge have now passed
on v04 as well. The complete current evidence and scope are in the
[Rochester review](../../towns/rochester-care-green-v01/README.md).
The later Grand Rapids, Mankato and selected Brainerd v07 successors preserve
this complete opening and every Rochester map/program byte. These earned opening
saves retain v04's identity; an uninterrupted earned campaign on v07 is not
claimed. Early native zero-badge fishing has separately passed again on the
current byte-identical Brainerd child.

The later v08 R16 and current v09 R02 revisions preserve Winona, R01 and
Rochester in full, including all opening/research/recovery programs. R02 adds
an independent native Berry stop after its existing Care departure gate, with
no story unlock. Both early zero-badge fishing/guru courses pass again on v09
bytes. The earned opening saves below retain their original identities.

- [Complete connected opening maps](../../renders/selected-opening-whole-v1336/connected-opening-original.png)
- [Actual home introduction](../../renders/selected-home-native-v1335/mom-speaking.png)
- [Actual home after fresh-process Continue](../../renders/selected-home-native-v1335/continued-home-free-input.png)

## Matching the actual story revision

The selected pack has the stationary native home introduction at (9,0) and the
standard field station. It does not contain `MnOverhaulMomScene` or
`MnOverhaulLabEntry`. The separate animated-overhaul proposal is not its source
contract. Its direct test was initially chosen incorrectly; both failures are
retained in `selected-story-v1333/` and `selected-story-v1337/`. A temporary test
change was fully undone; no Rust source or game behavior remains altered.

`story.json` contains both current native milestone text and the unselected
`dialogue_overhaul` proposal. The canonical test helper prefers the latter by
default. The first native delivery replay therefore stopped on a text mismatch,
not a broken delivery. Its three successful starter cases and failed delivery
remain in `selected-story-v1338/`.

For the matching replay, the existing
`CRYSTAL_NORTHSTAR_CANONICAL_STORY_SOURCE` interface points to the ignored
`.local/selected-native-story-milestones-v1339.json`. It preserves the authored
`/milestones` values from `planning/minnesota/story.json` and records that source
hash, excluding the separate proposal. Expectations are not copied from the
game pack or relaxed to arbitrary text. Its SHA256 is
`2f5f21217183b600465413bb5ea7d0e5b8670b254688ef24b4ac26a490731787`.

## Verified current behavior

Three real New Game courses select the native Pouchid, Kindlit and Birchit
through the existing legacy-named starter script slots. They check home dialogue
ownership, starter refusal/retry, naming, level-5 gift and bundle, no duplicate
partner, ordinary walking and Continue. Their successful results are the first
three checks in `renders/selected-story-v1338/receipt.json`; the later failed
delivery in that receipt is not counted.

Five matching cases pass in `renders/selected-story-v1340/receipt.json`:

- Earn the opening/rival, walk the real first route to Rochester, preview the
  parcel-locked Gym, acknowledge Ivy's actual delivery, repeat safely and walk
  the complete return home with Continue.
- Earn that delivery again, return to the field station, decline/retry and
  actually receive the Gillpup egg, retain it through Continue and refuse a
  duplicate.
- Lose the real first-rival battle with a declared weak native partner, retain
  the parcel and recover without repeating the scene.
- Knock out the intake encounter with declared native partners, retreat,
  Continue and actually retry without claiming a catch.
- Black out at the intake with declared weak native partners, recover at the
  prior Rochester clinic with restored HP, Continue and retain containment
  without claiming a catch.

The final old League-only starter-gift case in that receipt is **not a current
contract and did not pass**: the current fauna revision intentionally makes
other starters available after the first delivery. The correct three cases in
`renders/selected-research-v1341/receipt.json` all pass: each initial starter's
two alternatives, refusal/capacity/Continue; an earned first-delivery return
before any badge; and both fossil choices, real lab travel/revival without a
champion flag. No production unlock was changed to satisfy the historical test.

The earned courses use production New Game/input without injected party,
position, progression, result or RNG. Explicit capacity, loss, cleared-chapter
and fossil fixtures remain separately labeled. This proves those mechanisms and
the earned opening/local round trips, not an uninterrupted full campaign.

Two real stdio sessions in `renders/selected-home-mcp-v1334/` pass 50 calls with
empty stderr and unchanged inputs. Fresh New Game reaches the actual home
stair arrival, movement is blocked during dialogue, acknowledgment releases it,
Start works, and a new process resumes the speaking checkpoint correctly.
The two headless GPU views use those actual saves without position, clock,
facing, party or state overrides. All originals and rejected passes remain.

## Tighter Rochester is selected

The full-map inspection found Rochester's long central lawn and southern walks
still sparse. The [care-green revision](../../towns/rochester-care-green-v01/README.md)
now has a published, inspected whole town with shorter useful trips and exact
roof/door/sign contracts. Its nine successful current cases include played
doors/roads/services, early research/fishing and the earned first badge, followed
by two actual stdio sessions. The selected v04 pack retains every other map.
Broader regional story/recovery, ordinary-street and cave-quality review remains
active; the earned opening does not establish a complete campaign.
