# Devil's Kettle discovery

This is an unselected native 2D branch with broader map visual review still open.
The original 731-map scene review
is `content-packs/devils-kettle-rainy-review-v01.crystalpack`. Its exact hash is
recorded in the final verification section below.
It inherits accepted Upstream Woods, Confluence and Rainy terrain, plus released
species art, all 602 audio programs, 272 moves, Verdigrim and the League repair. The first
R10 candidate is retained history. All 14 combined native cases pass on the Rainy child, including selected-PC
capture, knockout/retry, real clinic recovery and nurse registration. The full
post-blackout return now passes in the follow-up below; further map visual
acceptance remains open.

The authored valley has two actual native WATERFALL columns, a divided pool,
a longer dry approach and a grass crossing, a west-bank lookout, and stairs
back to the cedar clearing near Grand Marais. All 148 perimeter tiles are solid
native tree/boulder collision. Source collision review reaches all five arrival,
exit, encounter and lookout anchors over 231 walking tiles. The whole terrain
preview was inspected; it uses existing native source pixels. The large boulder
banks should receive another composition review before visual acceptance.

The publisher appends three cedar warps without changing the existing eight.
It modifies one existing clearing block into native stairs. The added map uses
the existing native cedar habitat, without carrying unrelated coordinate zones
or swarm overrides into the new valley. The branch adds no Fly/recovery spawn.
Kettlurk appears only after both its story release and independent restoration.
The ranger's clue reacts to restoration. The voluntary level-60 battle uses the
native optional legendary battle type. Capture sets a dedicated flag; RUN and
knockout hide it for the visit. Physical reentry should restore an uncaught
creature; Continue should retain a hidden visit. The initial controller cases now prove gating, refusal, RUN, ordinary capture
and physical return/Continue. Full-party, knockout and blackout need separate evidence.

Executed build and publication:

```sh
CARGO_TARGET_DIR=/Users/ryanculligan/GitHub/geothite/target/verdigrim-scene \
RUST_MIN_STACK=67108864 cargo build --locked -p crystal-assets \
  --bin pack_devils_kettle
/Users/ryanculligan/GitHub/geothite/target/verdigrim-scene/debug/pack_devils_kettle \
  content-packs/verdigrim-current-scene-upstream-v01.crystalpack \
  planning/minnesota/pokemon/acquisition-v01/devils-kettle/site.json \
  content-packs/devils-kettle-upstream-review-v03.crystalpack
```

The source reviewer accepts PACK SITE FRESH_OUTPUT_DIRECTORY:
`planning/minnesota/tools/review_devils_kettle.py`. It is read-only and uses the
existing source-art renderer. Source and published pack inputs remain external
or tracked as appropriate; generated packs, previews and logs stay ignored.

Before acceptance: prove physical stairs in both directions and legal recovery;
absent creature before either requirement; refusal, RUN, knockout, blackout,
full-party capture, caught removal, hidden and caught Continue/reentry; real
native animation and 2D captures; actual production MCP with empty stderr;
strict unchanged-content/roster/audio verification; final lint and formatting.
The remaining seven rare species and the earned campaign stay separate work.

## Current investigation

The newest written review is `devils-kettle-upstream-review-v03.crystalpack`,
now inheriting accepted Upstream Woods. Earlier candidates are retained history.
The first controller run rejected the proposed cedar return because it was
existing grass. The return is now guarded native FLOOR at14,9. The next run
proved that native stone stairs are ordinary Floor/Wall, so warp records alone
do not activate them. Source-authored coordinate `warp` scripts now connect
both stair landings through the production controller. No engine shortcut or
state rewrite is used. The two original stair graphics/collision blocks remain
unchanged; the new coordinate events are explicit additions.

Physical entry reaches the new map. The retained native failure save showed
that a floor-only shortest-path helper walked left across the active exit stairs
and returned to cedar. This was a test-route error. The test now ascends two
steps into the valley before planning its approach, and deliberately walks onto
the exit when returning. The first actual capture case passes: a real native
level60 battle, Master Ball consumption, caught Dex membership, Continue and
physical reentry preserve removal and exactly one caught Kettlurk. Other outcome
and broader acceptance checks remain pending.

The strict residual audit on v03 passes: all729unrelated maps, native art/audio,
species catalog, all eight original cedar warps and original scripts survive.
Only one existing block, two coordinate events, paired trail scripts/mirrors,
three appended warps and the one new map plus its declared metadata/habitat/flag
are introduced. The habitat copies the base cedar tables and removes old
coordinate zones/swarm overrides; no foreign species or new audio is added.
All602audio PCM hashes, frame counts and loop ranges also pass on this exact
v03 child. The Upstream composition separately passes758checks and five
negative mutations. Generated proofs, saves and previews stay ignored.


## Initial native and production proof

All three `devils_kettle_controller_` cases pass on the current child in
`.local/devils-kettle-controller-v08.log`. Four declared chapter-state fixtures
prove that both independent story conditions are required. Actual input enters
and leaves on the stairs, approaches the native actor, refuses without spending
items, starts a real level60 battle, selects RUN, preserves hidden-visit Continue,
physically returns and starts another battle. Capture consumes the Master Ball,
adds Kettlurk to party and caught Dex, and preserves exactly one capture/removal
through Continue and physical reentry. These are local fixtures, not earned
campaign access or general difficulty evidence.

The same-pack reached pre-capture save drives14production MCP calls. They prove
actual movement, Start/Pack, opening the encounter text, movement denial during
text, and fresh-process held Continue, with byte-empty stderr. A headless native
2D capture from that reached save was visually inspected: the two native falls,
rock divide, blue current and original Kettlurk sprite are visible. The camera
is at night; this is not a same-time source pixel-comparison or whole-map visual
acceptance. The source and all rejected candidates/renders remain retained.

The final residual audit also rejects four deliberate mutations: another map,
native art, a foreign wild species and a reused capture-event identity. Both assets and Bevy all-target Clippy complete with the documented baseline
allowances and no diagnostics in the new leaves. Scoped Rust formatting and
Python syntax checks pass. Whole-workspace formatting still fails first at
the pre-existing offset-allocator formatting differences.

## Full-party and recovery expansion

The next six-case native run (`.local/devils-kettle-controller-v09.log`) passes
five cases, including full-party delivery to selected box4 with every other box
and the original party exact, ball consumption, caught Dex, Continue and actual
reentry without duplication. Two legal active level60 partners, Beetynamo and
Tiltawisp, also earn a real knockout and preserve uncaught hidden Continue,
physical return and another native encounter. These are declared local teams,
not earned party or general balance evidence.

The sixth case exposes a real service gap: Grand Marais's nurse exists, but
its clinic has no registered recovery spawn and its script does not select a
blackout destination. Declaring that prior clinic therefore falls back to
Winona's home. The proposed repair registers passive floor3,5 and wires the
existing nurse through native blackoutmod plus its original care script. It
adds no new map or Fly destination and preserves the existing town arrival.
A separate actual-nurse test will prove HP/PP restoration and registration;
the weak Cindore case will prove real defeat, recovery and Continue. The battle
test helper now classifies defeat by the authoritative result instead of a
Pokecenter1F name suffix. The repaired candidate remains unverified and must
not be selected until these checks and updated preservation pass.


## Rainy-parent verification and clinic repair

Current ignored review pack: `devils-kettle-rainy-review-v01.crystalpack`, SHA256
`fd05f90ce9f0a18ba3a6dbc1c7e801a53eca4fd0159c18d16c59d911a3f0f04c`.
It is unselected. All 14 combined Kettlurk, Verdigrim and League cases pass in
`.local/rainy-combined-native-v01.log`. This supersedes the earlier pending
full-party/KO/blackout status: selected-PC capture, KO/retry, real blackout to
the prior clinic, native HP/PP care, actual nurse recovery registration and
Continue now have passing cases. All setup parties are active Minnesota species;
no direct party, HP, inventory, progression or RNG writes occur after declared setup.

Grand Marais's missing service wiring is repaired in a focused leaf. The new
nurse wrapper selects its registered passive floor3,5 and calls the original
care routine. The actual nurse case stands on floor3,3 across the native counter,
earns HP/PP healing, checks the registered destination and continues that save.
The dedicated residual auditor proves all 730 other complete maps and all
prior spawn records exact, plus unchanged art/audio/species/terrain. It rejects
four corruptions, including extra compiled nurse operations and recovery drift.
The shared battle-test helper uses native defeat result1 instead of clinic-name
spelling; the six original Verdigrim cases still pass with that change.

The Confluence composition passes764checks/five negative cases; Rainy passes
757checks/five negative cases. All 602 PCM hashes, frame counts and loop ranges
pass on this exact Rainy child. Fresh production MCP passes14calls covering
actual movement, Start/Pack, encounter text ownership and held Continue, with
empty stderr. The Confluence blackout checkpoint also proves a physical clinic
exit through production MCP. Both affected crates pass all-target Clippy with
baseline allowances; changed Rust leaves (including the shared helper) and
Python syntax checks pass. Workspace formatting still fails at pre-existing
offset-allocator formatting. Only the final API comment was clarified afterward;
no production behavior changed after these checks.

A structural Confluence roster pass covers316regional trainers,81wild tables,
one field table,22fishing groups,12time groups and18creature actor definitions,
all in the active225-species roster. These tables remain exact in the Rainy
composition. Acquisition remains a structural upper bound of218; the other
seven rare sources and earned full-Dex/campaign proof remain open.

That original weak-party case stopped after recovery. The real Marais north
street guard requires all eight badges plus `EVENT_MN_LIGHTHOUSE_SIGNAL`.
The follow-up below declares those prior late-story prerequisites before input
and completes the physical return. The guard is unchanged; no continued party,
flags, coordinates or save identity are rewritten.


Fresh Rainy-pack production MCP also proves blackout Continue and the physical
clinic exit from the actually reached weak-party recovery save, with empty
stderr. That receipt is `.local/rainy-kettle-recovery-exit-v01.json`; it does not
claim that the zero-badge local fixture can pass the real north street guard.
The broader game remains unselected pending integration and outstanding work,
including full campaign/Dex acquisition and visual acceptance. The duplicate
Dorothy identity was subsequently corrected in a separate cabin revision that
keeps Dorothy outdoors and supplies a distinct indoor volunteer.

## Physical recovery return verified

The five-case follow-up in `.local/cycling-core-fix-native-v04.log` passes on
the combined `minneapolis-cycling-rainy-review-v03.crystalpack` (same Kettle
terrain/scripts; see the cycling verification document for its exact hash).
The level1 Cindore genuinely loses, returns to the registered Grand Marais
clinic, heals and continues. Ordinary input then leaves the clinic, passes the
real north street guard, crosses the Shore Gate and cedar trail, continues at
the clearing, descends the stairs and starts another native Kettlurk encounter.
RUN leaves it uncaught and preserves the money remaining after the blackout.

The initial attempt exposed a floor-only test-planner restriction on the cedar
grass trail. The final case reuses the existing campaign walker, which handles
actual grass encounters through production battle inputs. Only two test-helper
visibility declarations changed; no map, gate, collision or game-state shortcut
was added. All eight badges and the lighthouse signal are declared fixture
prerequisites before input, not earned-campaign proof.

The same five-case run passes both cabin care/door cases, cyclist battle/Continue
with a held NO selection, and Bicycle gift/register/ride/Continue. The fresh
production client subsequently passes nine held-prompt MCP calls with empty
stderr, and the native 2D dialogue/cursor capture was inspected. This closes
that scoped cursor repair. The comprehensive game goal remains open, and the
candidate must inherit the newer State Fair/Duluth selected terrain before
campaign integration.
