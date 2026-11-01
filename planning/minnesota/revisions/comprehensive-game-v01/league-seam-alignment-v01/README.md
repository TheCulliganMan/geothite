# League entrance alignment — unselected implementation

The released dialogue revision makes camp floor `(32/33,47)` meet R13 floor
`(30/31,0)` by changing their reciprocal offsets to `1/-1`. This makes the
local walk work, but contradicts the global outdoor connection cycle through
compact R12. The Rust geography validator correctly rejects composition with
`inconsistent outdoor placement: MnBoundaryWaters/MnR12C00R00`.

The repair retains canonical offsets `0/0` and extends R13's adjacent native
floor into metatile `(16,0)`. On the current reviewed winter source this is
`0x8b → 0x83`. A four-tile turnout joins camp `(32/33,47)` to road `(32/33,0)`;
two horizontal walking tiles join the original road at x30/31. No native
graphics, palette or collision definition changes. The entire camp, including
both League guard actors and all coordinate-event lanes, stays exact.

The original and revised complete R13 terrain previews were rendered from the
supplied pack and visually inspected. The one-cell turnout fits the existing
native snow trail. These are terrain-only source previews, not production
captures or full map acceptance.

`northstar_story_connections::install` now uses this alignment so future
dialogue publication cannot restore the contradictory shifted seam.
`align_northstar_league_entrance` exposes a guarded Rust overlay for existing
packs. It verifies full native floor, an empty wall/floor target, unchanged
border extent, unique reciprocal links and globally consistent resulting
geography. The publisher requires an exact parent SHA256, fresh ignored output
and verified readback. Source parent R13 blocks SHA256:
`55ee9e1809943f7f56eda5b6e423e8d7509c234195f72ac6f47c104b27119b0c`.

Acceptance is pending: compile and publish an ignored candidate; prove strict
preservation, canonical whole-world placement, native two-lane entry/return,
story denial before clearance, proper access after all requirements, Continue,
production stdio and matched native 2D views. Then compose accepted Preserve
Pine and Lower Woods terrain while retaining released dialogue, all225 species
art/cries and both starter signatures/272 moves. No selector or public release
has changed. Final integration must follow any newer accepted world revision.

## Current evidence

The Rust alignment publisher succeeds on the exact released parent. The global
geography validator then accepts it; the previously blocked Preserve Pine and
Lower Woods Rust publishers both succeed. The composed parent SHA256 is
`247afd82511d32395b33c4ee4ec4267285484377b62b66ac392388259a4400f9`.
The full unselected scene child retains730 maps,476 total species,272 moves and
602 audio programs. All five Verdigrim native cases pass on this combined
candidate, and fifteen real stdio MCP calls verify held-dialogue input
ownership, completed exposure, Start/Pack and actual movement with empty stderr.

Independent alignment preservation passes:728 other maps exact; one native
turnout; camp terrain/actors/events exact; all scripts, species, art and audio
exact. The scene overlay separately preserves729 other complete maps and its
parent's entire species/art/audio data. Source scopes remain distinct.

The native League case passes all eight fixtures: both lanes with missing
badges, missing signal, missing restoration, and all requirements present.
It verifies denial text/input ownership, repeated denial after Continue,
outward and reciprocal seam walking, the original two-step inward movement,
unchanged party/bag/money, and Continue on the road and after return. The
combined candidate passes six native cases including all five Verdigrim cases.
Final production captures, all-target Clippy, audio and selection remain pending.

The diagnostic replay proves outgoing press and idle both remain at44 with
facing DOWN and no pending cursor. A fresh stdio client also moves43→44→45
normally. The failure was later, during the return loop: the existing inward
script deliberately walks from44 through the guard lane to42. The test wrongly
expected to stop at44, then kept issuing Up looking for an already-passed tile.
The corrected return sequence asserts46,45,42,41 and retains the original guard
program. Its replay passes. No facing, idle-frame or guard workaround
was added to gameplay.

Fresh-process Continue of the reached scene save passes six further stdio calls
(21 total): same map/position, Start/Pack, left/right authoritative movement,
and empty stderr. The earlier Clippy pass completed for Assets/Bevy with
pre-existing warnings; its one new constant-chunk warning was fixed using
`as_chunks`. Alignment changes still require their final all-target pass.

Final Assets and Bevy all-target Clippy both complete successfully with the
four documented pre-existing lint allowances; no warnings point to this
alignment/scene code. A subsequent animation-cache pixel regression is queued
and will need its own final check. Whole-workspace formatting still fails first
at the unchanged `offset-allocator/src/lib.rs:408`; scoped changed leaves pass.

The three final same-pack native 2D captures (green water, clean bypass and
the reached R13 turnout) were inspected. The gray floor leaves the controls,
copper rubble and contaminated sump legible; the short native turnout connects
the existing snowy trail to camp. Original/rejected previews remain retained.
The final exact scene child also passes Rust desktop/audio verification for
all602 PCM hashes, frame counts and loop ranges. Two production MCP sessions
walk both actual turnout lanes and the reciprocal camp seam, including the
native inward two-step passage and Start/Pack, with22 further calls and empty
stderr (43 functional scene/passage calls total). These remain declared chapter
fixtures; selection, full earned campaign and remaining scene acceptance are
still open.
