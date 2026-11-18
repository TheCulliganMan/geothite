# St. Cloud and Duluth return conversations

Four optional residents talk about Minnesota Pokemon they know and care about.
The user rejected the first version's spoon/painting material and generic
work-related observations. This revision replaces both the setup and return
pages, so none of that material remains in these four conversations.

`dialogue.json` owns four existing first-visit labels and four new return labels.
`northstar_town_responses::install` preserves the residents, terrain, rewards,
flags and normal facing interaction. It requires Pendulop, Birchit, Lakernaut
and Loobit to exist in both the curated active roster and the supplied pack.
No creature actor, encounter, party, movement, or compulsory scene is added.
These are residents telling the player about their partners and familiar wildlife,
not new animated demonstrations.

## Scene and knowledge trace

| Resident | First visit / incomplete state | Return condition and response |
| --- | --- | --- |
| St. Cloud stonecutter `(18,13)` | Trusts Pendulop to detect a wobbling paving stone. | `EVENT_MN_REED_WORKER_RELEASED`: Reed's shouting frightened Pendulop; the resident waits while it sleeps in their coat. No mine-rescue knowledge. |
| St. Cloud river watcher `(29,24)` | Misses Birchit's canoe races and triumphant tail splashes while the river is low. | `EVENT_MN_SURVEY_RESTORED`: Birchit races again, reaching the far bank before the resident starts paddling. Safe-launch advice remains in both branches. |
| Duluth dock hand `(20,73)` | Loves working with gentle, strong Lakernaut, which carefully handles cargo. | `EVENT_MN_SURVEY_RESTORED`: Lakernaut can reach the boats again and eagerly offers to help every arriving crew. |
| Duluth island resident `(76,58)` | Loves guessing where diving Loobit will surface, but has seen few lately. | `EVENT_MN_SURVEY_RESTORED`: the Loobit return; one surfaces beside the resident while they are looking the wrong way. No private campaign knowledge. |

Partner behavior follows the active roster concepts: Pendulop's vibration-sensitive
stone balance, Birchit's swimming and paddle tail, Lakernaut's cargo-lifting arms,
and Loobit's loon identity. The residents' individual experiences are original
fiction. They do not change the species' mechanics or create regional encounters.

The selected input at inspection was `content-packs/minnesota-crossings-v01.crystalpack`,
SHA-256 `caecd60747ee26510651509c6567a3490d919c0d6c9f8c2528d9445de2ad1599`.
The actual installed `MnArcSCScene` checks the St. Paul Reed victory, offers a
refusal, moves Sera away from Reed, dismisses Reed, displays the release pages,
then sets `EVENT_MN_REED_WORKER_RELEASED`. A St. Paul victory alone does not earn
the stonecutter response. Sera returns at `(24,9)`. This leaves her existing radio
and rescue-report branches intact.

The outdoor student was deliberately excluded: the installed root already jumps
to `MnWayPineRoad`, a newer wayfinding conversation. Its older ambient text is
still in the pack but does not establish the current interaction.
The installed `MnVerdigrimBypass` requires counter-current, Reed's Boundary defeat,
drill shutdown and outlet closure. Only the accepted bypass, retreat, Kettlurk
release and crew aftermath eventually set `EVENT_MN_SURVEY_RESTORED`. A harbor
signal, mine victory or drill shutdown alone leaves both Duluth replies unchanged.
Completed saves select the return branches immediately; repeated visits do not
change state. Every branch faces the listener and ends with ordinary native text.

## Craft comparison

Original [Crystal's Azalea elder](https://github.com/pret/pokecrystal/blob/master/maps/AzaleaTown.asm)
uses `EVENT_CLEARED_SLOWPOKE_WELL` to switch from concern over the missing animals
to a familiar, slightly dismissive remark about their return. The elder does not
recite the well battle. Here, restored water is noticed through a partner's activity and the return of
familiar wildlife, without copying that dialogue.

The local Polished Crystal reference at
`/Users/ryanculligan/Documents/Geothite-References/rom-hacks/polishedcrystal`,
revision `f8e67fbd53ad7a99870d8ab87fd73612e6c6a679`, adds an Azalea teacher branch:
changed weather affects her own partner, then she gives an item, and later speaks
about that partner affectionately. Its separate event and reward checks matter.
These residents instead have no reward or one-time receipt; the meaningful
change is in how each person talks. No extra walk or emote is needed for it.

## Verification and reproduction

The ignored Rust test reads the verified external parent, applies the real
installer, compares every unrelated map and every untouched script, preserves
actors/events/terrain and runtime files/audio, validates rebuilt native tables,
rejects duplicate installation, and can write/read back an ignored review pack.
No gameplay inputs or user saves are involved.

```sh
CARGO_INCREMENTAL=0 \
CRYSTAL_TOWN_RESPONSES_PACK=content-packs/minnesota-crossings-v01.crystalpack \
CRYSTAL_TOWN_RESPONSES_OUTPUT=target/town-responses-20261118/town-responses.crystalpack \
cargo test --locked -j 1 -p crystal-assets --lib \
  town_responses_preserve_content_and_bind_real_milestones -- --ignored

uv run --with cbor2 --with pillow python \
  planning/minnesota/revisions/town-responses-v01/review.py \
  target/town-responses-20261118/town-responses.crystalpack \
  target/town-responses-20261118/installed --installed
```

Use a fresh ignored output for the pack. The storyboard reads actual pack terrain,
resident art, font and frame, using the production 160x144 dimensions, a 20x6-tile
field box, 18-character lines and baselines at rows 14 and 16. It checks every
page and, with `--installed`, requires exact text read back from the pack. It
stages only the selected resident, not a fabricated player session or guessed
callback visibility. These are static native-asset storyboards, not production
client screenshots or an earned playthrough. The user's no-play instruction
precludes interaction and timing verification here. The broad open dialogue
complaint in `TASKS.md` remains open.

### Superseded technical result (2026-11-18)

The user rejected this prose. These earlier passes are historical evidence only,
not acceptance or verification of the Pokemon-focused rewrite below.

The exact test passed on both Crossings and Lakeside: one test run for each,
zero failures. The separate review pack written/read back by the Crossings test
is `target/town-responses-20261118/town-responses.crystalpack`, SHA-256
`51936b82c45eb82d93540cbd45a8fe76108c0f10793eb3ba160ff5933ac27582`.
Installed-page inspection passed all four raw branches, native flag rows and
resolved conditional targets. All 26 native pages fit; all 31 page/sheet PNGs
match the reviewed source pixels. The final river-watcher sheet was inspected
after its last pagination adjustment. Evidence is under
`target/town-responses-20261118/installed/`; exact test logs are
`target/field-review-20261118/towns-{crossings,lakeside}-final.log`.

Workspace formatting and the leaf's final rustfmt check passed. The integration
owner coordinates the current-tree Clippy check and the combined candidate.
Shared Cargo output remains in use by that owner; no shared cache was removed.
Superseded student storyboards were removed. The review pack is unselected and
unpublished. No gameplay inputs were sent, so timing, interaction and earned
playthrough acceptance are explicitly outside this proof.

### Pokemon-focused rewrite

All 32 source pages were rendered with the actual pack font and frame and
visually reviewed under `target/town-responses-20261118/pokemon-source/`. The
spoon material is removed from the replaced first-visit label as well as the
return. The installer and preservation test now explicitly allow changes to
those four first-visit labels, preserving every other existing script. New
installed-pack verification will use a fresh output; previous pack hashes do
not describe this rewrite.

The rewritten version passed the exact named test on Crossings and Lakeside
(one test on each, no failures). The written/readback standalone pack is
`target/town-responses-20261118/pokemon-rewrite-v02.crystalpack`, SHA-256
`ec4733fce8022d46d96d700572b8a1e33cc6eb765558ddacc00767bbd1a3944c`.
`review.py --installed` checked all eight installed text bodies plus the four
raw/native flag branches. All 32 pages fit, and all 37 page/sheet PNGs match the
reviewed source pixels exactly. All four final sheets were visually inspected
under `target/town-responses-20261118/pokemon-installed/`. Test logs are
`target/field-review-20261118/v02-towns-{crossings,lakeside}.log`.
This is technical and native-page verification of the rewrite, not a claim of
user approval or a played interaction. No gameplay inputs or deployment occurred.
