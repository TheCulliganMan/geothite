# Boundary Waters capture expedition: native implementation

The preserve remains closed to paid admission in the selected game pack. This
revision implements the native core, pack-bound habitats, staged fee/warp and
return transactions, movement accounting and replayable capture commands.
Committed walking/Surf and native rods now generate actual wildlife through
native held-item and DV routines. Saved ranger confirmation and capture cursors
use the shared controller, bitmap renderer and read-only TUI projection.
Production-controller journeys now cover ranger-to-retirement, native fishing,
party and selected-PC capture, Dex/nickname screens, two actual grassy portages,
Surf launches/dismounts, walking/water seams, the real cabin and 500-step expiry.
The earlier nine-case replay and newer ten-case Itasca-parent replay pass using
the default test profile. Final page-persistence changes have separate affected
travel/ending and original Johto controller checks. See [VERIFICATION.md](VERIFICATION.md)
for exact execution epochs and limits.
Remaining movement/travel paths and throw/shake/audio presentation still
require acceptance before enabling admission.
The battle display now has native command/result captures, matching source-art
pixels and a regression for both trainer back sprites and return to exploration.
See [VERIFICATION.md](VERIFICATION.md) for exact inputs and limits. The free Boundary Waters hub,
Q07 shelter and R12/R13/R20 junction remain outside the expedition.

The current unselected review is
`content-packs/minnesota-safari-runtime-review-itasca-v06.crystalpack`, derived from
`minnesota-current-grass-itasca-v01.crystalpack`. Rust publication
verified exact preservation of unrelated data, runtime art and compiled audio,
plus strict readback. All terrain, actors, events and executable commands remain
exact. In addition to the Safari configuration, v2 publication replaces seven
related closure-text bodies and their existing split mirrors with native
permit/check-in directions. It does not change the main regional selector.
See [TRAVEL.md](TRAVEL.md) for exact lake/portage/cabin coordinates, the executed
publication path and remaining travel work. Promotion requires checking the
live [world selector](../../world.json) and rebasing if it has advanced. Older
R15, Detroit, Marais and guidance review packs remain historical evidence.

## Battle presentation contract

Safari encounters use the complete native battle LCD. The selected trainer's
Chris or Kris back sprite occupies the normal lower-left trainer slot throughout
the encounter. The actual wild Pokemon occupies the upper-right front-picture
slot, with its native name, level, gender and HP display. Balls and steps sit
beside the trainer; the lower command window contains Ball, Bait, Rock and Run.
Results keep this same battle composition until acknowledged. Returning to
exploration releases the battle display and removes the trainer back sprite.
Budget exhaustion and ranger retirement keep the field visible with the normal
field-dialogue window; they no longer display an empty trainer battle.
See [BACKSPRITES.md](BACKSPRITES.md) for the retained Chris/Kris renders and
their production-controller/compositor checks.
The capture engine continues to own these commands and saved results; this
presentation does not send out a party member or offer ordinary combat moves.

## Core implementation

- `crystal-core/src/systems/safari.rs` owns the typed expedition, wildlife,
  disposition, action, result and ending phases. Budgets are30 balls and500
  successful travel commits. The separate500 fee constant is reserved for the
  native admission transaction; constructing a session does not admit or charge.
- `safari_encounter.rs` resolves Ball/Bait/Rock/Run against the actual saved
  session. Ball preflights party/current-box capacity and capture rules before
  any random draw or debit. Other empty boxes do not silently redirect captures.
- Ball uses the same native ball multiplier, full-HP integer calculation,
  catch/wobble sequence and party/current-PC/Pokédex storage functions as ordinary
  capture. The supplied Safari Ball must be the unconditional3/2 rule. Bait and
  Rock modify the species base catch rate before the native ball calculation,
  rather than multiplying an already clamped final chance. The shared species
  catalog remains unchanged.
- Wildlife owns its own actual Pokémon. There is no fictitious player
  combatant, ordinary bag item, enemy attack, EXP, PP or combat dispatch.
- Every resolved command enters `Presenting`. A saved last-ball catch has
  already reached storage and queues exhaustion. Continue retains the same
  result; another throw is refused until the result releases input. Acknowledging
  a result never repeats a capture or its random draws.
- An ending permit clears only after an exact ordinary-walking return arrival
  has committed. The core does not fake that warp. Capture phases retain their
  original authoritative map/tile and reject movement commits. Committed outward
  field travel separately ends the permit at its actual chosen destination;
  ranger and budget returns use the configured basecamp landing.

## Native rod encounters

An active expedition routes a real fishing bite into the same capture battle.
It preserves the native map fishing group, chosen rod, day/night slot, species,
level, held-item and DV generation. Fishing does not use challenge-level scaling
or reroll the separate Surf habitat table. The original mod's fish flee threshold
defaults to64/256 and is configurable as `fishing_flee_threshold`; Bait/Rock use
the same disposition rules as other Safari wildlife.

The saved encounter records its bank position and a separate adjacent water
contact, rod, resolved group, source time and native slot roll. Validation checks
the real facing, water collision, habitat and original native rod-slot resolver.
Field-source encounters omit this optional JSON field, preserving their prior
saved representation. Casting spends no travel steps or balls. The existing
cast/hook/pause and bite notice keep input and the field display until native
A/B acknowledgement opens the trainer-backed battle screen. All three rods at
day/night have component coverage; the actual Good Rod approach, native casting
clock/text, capture, saved source and Continue pass through the production
controller. Native command/result screenshots are retained for review.

## Native maps, admission and replay

[zone.json](zone.json) binds the six exterior chunks covering64×56metatiles and
Dorothy's actual cabin to four geographic habitats. Its45 weighted entries use23 active
Minnesota species with separate grass/water tables. Reeds include mudpuppies,
crayfish and mosquito wildlife; pines include deer, bears and woodpeckers;
granite shores include caddisflies and agates; quiet lakes include otters,
turtles, duckweed and perch/walleye. Rare Mosquitan55, Gougecrest38 and
Tipulord44 meet their actual evolution thresholds. No source Johto replacements,
unavailable Cragcloak or underlevel rare evolutions are silently substituted.
Grass checks are authored48/256 and water12/256; these rates are configuration
for the pending encounter generator, not already executed encounters.

`safari_zone.rs` validates native maps/collision/actors, reciprocal seams,
interior warps, complete habitat coverage, hundred-weight tables and evolution
levels. Runtime save validation now rejects missing/changed Safari config,
invalid species/level/surface/habitat/caught-location references and unpermitted
arrivals beyond the free court. An absent optional config preserves existing
pack serialization and exact identity; the actual R11 parent still loads with
native identity verification.

The existing24×16 `MnPreservePineEntry` approach and its walking connection to
the southwest gate court remain free. Inactive saves may occupy the safe court
at local bounds`[8,5)..[18,16)`, with real FLOOR/actor/warp checks. The actual
two permit guards seal the lake neck. Admission derives their native
disappear/appear behavior from the typed session; it does not merely make a
colliding actor visually transparent or introduce a separate stale flag.
Continue restores the same permit/visibility and the closed bodies return
after retirement.

`safari_admission.rs` preflights the actual facing ranger, native Surf partner
and badge, ownership, money and party/current-box space. Fee500, new30/500
session and native transition commit together. Decline, denial, duplicate
confirmation and native warp/music failure cannot charge. The return similarly
stages the actual map transition before clearing the permit. Neither transaction
changes the party. The production controller dispatches the actual facing ranger
into saved native admission/retirement choices; pending choices survive Continue.

Native successful movement now decrements before trainer/connection/warp/coord
events can bypass normal `CountStep`. Turns, bumps and idle frames do not spend
steps; capture/result/ending input cannot continue walking. Scripted Surf's
actual committed movement has the corresponding hook. The focused native test
proves ordinary movement, zero-cost turning/waits, final-step refusal and real
return. Surf, ledge, seam and outward-field-travel coverage remains required.

`safari_mutation.rs` and `runtime_safari.rs` bind admission, capture action,
acknowledgement, retirement and return to the existing mutation/checksum/journal
path. Live action staging records the actual hardware-divider samples, commits
the native random state with the result, and replays the same trace. Empty or
extra traces roll back. There is no alternative TUI/MCP dispatcher.

## Save compatibility

The full `SafariSession` is serialized as one strictly validated, versioned
value in `GEOTHITE_SAFARI_SESSION_V1` within the existing saved extension map.
It is not a group of loosely coupled ball/step script variables. Core save
validation parses the complete typed state, rejects unknown versions and
impossible phases/budgets, checks ownership and the committed caught Pokémon's
actual storage slot and metadata, and never resets malformed active resources.
The authoritative game RNG remains the existing native random state; the
runtime action adapter must commit its supplied random stream with the staged
action, using the same transaction/replay path as other native mutations.

No positional `GameState` field or binary save schema changes. An inactive
legacy save has no extension value and retains its exact encoding/checksum.
The historical `wSafariBalls` and `wSafariSteps` hospitality sentinels are not
read, rewritten or treated as an expedition. No Bug Contest party backup,
timer or capture replacement is reused.

## Saved catch presentation

The native trainer-backed battle now continues through caught text, applicable
new Pokédex pages, the original nickname prompt/grid and actual party or
selected-PC-box transfer. The saved presentation has its own versioned extension
value, without changing the binary GameState schema. Text pages, YES/NO choice,
draft, grid cursor and letter case survive Continue. Renaming commits to storage
and the retained capture result together; only final acknowledgement releases
pending exhaustion. The ordinary native widgets still handle input through
`VisibleShellController`; TUI/MCP project the same saved state.

Default-profile controller and clean-stdio regressions pass. See
[VERIFICATION.md](VERIFICATION.md) for exact scope, inputs and reviewed images.
All habitat and existing rod/time/swarm species, including their evolution
targets, must belong to the active Minnesota roster after retirement curation.

## Remaining integration before opening admission

1. Prove the complete before/after walking and paddling gate boundaries while
   preserving the free pine approach and court. Actual ranger admission,
   refusals, retirement and generated encounters already have controller tests.
2. Complete native throw/shake animation and audio presentation around the
   existing saved capture resolver and trainer-backed battle display.
3. Prove native Surf launch/dismount, ledges, internal seams, cabin travel and
   final-step encounter suppression. Settle outward court seams, Fly/Teleport/
   Dig/Escape Rope and other outward warps without leaving a live permit outside
   or displaying a field-move destination that differs from its actual arrival.
4. Exercise the completed flow: admission, budgets, all four commands, storage,
   pending result Continue, final step/ball, retirement/outward travel and safe
   free Q07/route traversal through the production controller and clean stdio.
   Inspect the actual native views, verify preservation/audio, then publish a
   fresh external pack from the latest selected parent. Keep the gate closed
   until this complete path passes.

Core tests are component fixtures, not earned regional progression or evidence
that paid admission already works. Verification results belong beside this
runbook when executed.
