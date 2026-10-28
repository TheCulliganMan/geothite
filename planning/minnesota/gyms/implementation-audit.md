# Eight-gym implementation audit — 2026-10-27

**Superseded as a quality completion claim.** The user subsequently questioned
the puzzles' solvability. The implementation evidence below records the
previous generation; the active improvement pass reviews one room at a time.
[Rochester](rochester-gates.md) now has a two-switch lane revision after the
user rejected the later planter maze too. [Mankato](mankato-terraces.md) now has
distinct sand terraces, visible stairs and native one-way drops. Both ordinary
party control courses pass. The current index now selects revisions for all
eight rooms; the remaining six revision runbooks are linked below. Earlier
automated course passes do not settle design quality or user acceptance.

## Current revision evidence

The cumulative541-map `minnesota-duluth-gold-gates.crystalpack` has identity
`65984d942a6fcf9812ce3d56035a25a6b88c8d100b9ff5233bacf81f6d27154f`.
Its exact publication chain is Rochester → Mankato → Minneapolis → St. Paul →
St. Cloud → Brainerd → Bemidji → Duluth. Every publisher preserves the previous
rooms, city doors, original battle identities and unrelated content. All sixteen
revision asset cases and377 audio hash/frame/loop checks pass. All eight selected
source terrains and collision projections match publication; the combined
whole-room source overview has been inspected at one native scale.

The current room evidence is recorded individually in
[Rochester](rochester-gates.md), [Mankato](mankato-terraces.md),
[Minneapolis](minneapolis-circuit.md), [St. Paul](st-paul-loop.md),
[St. Cloud](st-cloud-arrows.md), [Brainerd](brainerd-winch.md),
[Bemidji](bemidji-return.md) and [Duluth](duluth-gold-gates.md).
Each runbook distinguishes native views, controller behavior and explicit
initial party/badge/position fixtures. Duluth's fixed four-partner course also
passes, using native Pack healing and PKMN switches, in addition to its four
passing existing controller cases. Smaller trial parties lost; this is a
specified-course result rather than universal team balance.

The [Rust regional rebuild](current-regional.md) now installs the complete
selection into the frozen 554-map story and 660-map compact-city branches.
Four full-dataset preservation/source cases, both branches' sixteen actual
carpet/reentry/Continue checks, sixteen exact native room cameras and both
sets of377 audio checks pass. All eight ordinary-party literal courses pass
on the integrated story result. St. Cloud uses explicit carried medicine after
its earlier two-partner medicine-free trial lost; opponents stay unchanged.

The requested city-by-city design remains in `../PROGRESSION.md`, with the
same eight badge slots, route gates and intentional returns. The refreshed
rooms retain those qualifications and preserve all exterior/story data.
This proves the room revisions and their bindings; it does not claim a fully
earned regional campaign or user visual approval. Those are separate from
delivering the eight revised gym designs and progression plan. The older audit
below describes its historical parent, including superseded geometry/counts.

### Coverage of the requested design scope

| Requirement | Current evidence |
| --- | --- |
| All eight gyms, one at a time | `current.json` selects eight individual revision files; their room runbooks retain the individual design, publication and review history. The regional publisher rebuilds that complete selection in both supported branches. |
| Original-style rooms, simple varied puzzles | Native source art, complete-room review and sixteen exact native player-camera comparisons. The mechanics are lane switches, stairs/drops, adjacent electrical switches, a pad loop, wind lanes, shifting piers, Cut hedges and ice stops. |
| Required trainers and a solvable course | Eight fixed-input ordinary-party controller courses earn every required trainer and leader, spend actual PP, receive one badge/TM, save/Continue and use both city exits. Individual source geometry and refusal/loss checks remain recorded in each room's runbook. Initial party/badge/position/supply fixtures are explicit. |
| Understandable walls and controls; no book quizzes | Blue physical switches and gold moving gates, visible stair gates, charged electrical barriers, marked wind directions, one dock control, native Cut trees and rough ice stops. All eight station arrays are empty; the whole-room and player views were inspected. |
| Original names | The eight selected leaders and authored trainers retain their original invented identities; the battle harness checks those exact names/classes. Native generic trainer classes describe roles. |
| City-by-city progression design and usable bindings | `../PROGRESSION.md`, `../world.json` and `../story.json` preserve the city/badge order and explicit gates. Both regional branches pass actual paired carpet/reentry/Continue checks for all eight cities. Full-dataset comparisons preserve every exterior and story field; symbolic plan validation checks regional reachability separately from earned play. |
| Repository/runtime constraints | Rust-only publication and selection, native controller input, unchanged unrelated datasets/payloads/parties/recovery/audio, both sets of377 audio checks and retained ignored renders/packs. No generated pack, PCM, catalog dump or build output was staged or committed. |

## Historical parent evidence

The eight interiors requested for the Minnesota circuit are implemented in this
repository and published in the external ignored room and regional packs.
The current regional candidate is the 541-map Leech shore pack, internal
identity `43b3d65bda72a8904cfad17781cc0da408045c64d91ee22fb5c6b3b1634006db`.
This audit distinguishes implemented gym rooms and the city progression design
from a complete earned regional campaign, which remains unfinished.

| Requested requirement | Authoritative evidence inspected |
| --- | --- |
| All eight gyms, developed individually | Eight full row/art/actor/party/flag contracts in `interiors.json`; eight individual design sections and source/native review records in `../GYMS.md`; every current complete module matches the original regional publication exactly. |
| Original-style rooms with visual judgment | Fresh whole-room source render of all eight at one native scale, visually inspected; every published terrain pixel and collision quadrant matches source. Individual original-style/native entrance and court reviews remain retained under `../renders/gyms/`. |
| Simple, varied physical puzzles | Rochester planter navigation; Mankato rock stairs and one-way drops; Minneapolis adjacent switches; St Paul reciprocal pads; St Cloud wind lanes; Brainerd shifting piers; Bemidji Cut hedges; Duluth ice courses. The combined production course solves all eight. |
| Trainer work is required | All eight course tests earn their required trainer victories before the leader; eight focused required-trainer/puzzle-gate cases and sixteen fresh loss/prior-badge/court refusal cases pass. Rochester's reachable court refuses practice skips; Mankato/Bemidji physically seal their courts. Other individual gate and loss checks remain in the same test module. |
| No book/quiz-room filler | Every current full room was inspected by sight; all eight station arrays are empty. Props and terrain support the individual physical mechanism. Rejected earlier room generations remain archived. |
| Original character names | Current leader roster is Varel, Torven, Avrin, Brannic, Saven, Sivren, Orlis and Norek. Authored trainer names and battle identities are checked by the controller suite; native generic trainer classes remain roles. |
| Functional rooms, rewards and persistence | The fresh combined actual-city course passes all eight gyms, their native battles, spent PP, badge/single-TM rewards, Continue and city returns. The final approach additionally walks mine/R09/depot/R10/Duluth, uses actual clinic care, wins Harbor and returns home. |
| City-by-city progression design | `../PROGRESSION.md`, `../world.json` and `../story.json` retain the full city/badge order and explicit gates. Fresh plan validation passes20 routes,19 hubs,57 chunks,86 adjacency/gate contracts and164 planned maps, with symbolic reachability of all badges/hubs/champion. This does not execute the whole campaign. |
| Repository/runtime constraints | Implementation and publication remain Rust. Gameplay checks use `VisibleShellController::press`; continuous approaches do not assign downstream progress or resources after their labeled initial setup. Packs, source art, audio and generated renders remain external or ignored. All377 audio hash/frame/loop checks passed in the border audit. No build output or pack was staged or committed. |

The combined eight-city controller course passes in76.19 seconds after the
battle test helper gained available-move selection and reserve-switch support.
Eight focused required-trainer/puzzle-gate cases and sixteen exact negative
cases pass on the canonical 480-map interior pack;
the current regional pack retains the same authored behavior and its tested
real city returns. The [final-gym approach](../playable/gym-eight-approach.md)
passes in25.76 seconds from explicit upstream report/seven-badge/two-partner
fixtures. It earns the later custody steps, road clearance and eighth badge.

The [border audit](../playable/gym-border-audit.md) records native collision,
stdio traversal, audio preservation and earlier connected chapter checks.
The return helper follows the actual compiled carpet and passive exterior
arrival, then proves re-entry to the same gym; it no longer assumes every
focused test uses review-hall door zero. The combined review-hall test still
checks each of its eight known door positions independently.

Changed Rust leaves pass rustfmt. Scoped all-target Bevy Clippy passes with
the pre-existing `never_loop` allowance and no warning in the changed gym
modules; workspace-wide formatting and full dependency lint retain the
unrelated failures recorded there.

## Evidence limits

Review parties, initial placements and prior badges are explicit fixtures.
Isolated negative cases can construct missing-badge or completed-practice
states to test refusals; they do not claim those states were earned.
This is not an ordinary-party balance claim, complete earned Winona-to-League
playthrough, or final visual approval from the user. The regional story task
still has its remaining bindings and earned-story audit. Every earlier render,
original and conversion is retained; these limits do not hide an unfinished
gym mechanism or replace one room's evidence with another room's result.
