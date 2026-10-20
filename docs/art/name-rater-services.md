# Name Rater service input

Both `LavenderNameRater` and `GoldenrodNameRater` use the authored NPC script,
including `faceplayer`, `opentext`, `special NameRater`, and its subsequent text
wait/closure. The runtime runner returns a `NameRaterPrompt` boundary before the
special opcode. `VisibleShellController` then uses its existing service
interceptor to present the introduction, choices and party selection. A stale
`_party_slot` from a previous service cannot bypass a new visit's introduction.

The introduction and better-name text pages own A/B until they finish. The
question then uses the existing source YES/NO geometry, joypad history and
15-VBlank closing wait. The naming screen begins with an empty entry, matching
the external source's `NamingScreen_InitNameEntry`. Blank submissions preserve
the old nickname; unchanged and newly accepted names retain their separate
source completion pages. Egg and foreign-OT refusals do not open the editor.

Renderer-neutral name editing adapts the eight game buttons to the existing
native naming-key handler. That handler remains authoritative for cursor
movement, A selection, B deletion, Select case, Start/END and completion. There
is no second editor or service dispatcher in the terminal/MCP frontend.

Presentation uses the selected species' existing two-line nickname heading in
both the LCD and text observation. The logical `POKéMON'S NAME?` service label
remains the completion owner. Long heading lines stay inside both border
columns; ordinary short labels keep their established source offset.

The external-pack regression in `tests/name_rater_services.rs` starts from an
explicit initial room/party fixture. Every subsequent service action uses
`VisibleShellController.press`. Each house covers new, blank, unchanged and
ten-character names; deletion/case editing; egg, foreign-ID and foreign-name
refusals; NO/B at both questions; party cancellation; inventory/party invariants;
walking and Start-menu return. The renamed Pokémon is also reviewed through a
second ordinary visit with the prior service variables still present. This is
service reachability proof, not proof of acquiring the fixture Pokémon.

For native screenshots, `service:name-rater-entry` observes an actually reached
editor and releases ordinary input. It never creates a naming screen, picks a
party slot, changes a name, or advances a text page. The disposable initial
saves are emitted only when `NAME_RATER_REVIEW_DIR` is set. Use headless image
capture, synthetic walk input and `CRYSTAL_RENDER_TEST_WALK_PC_PULSE=1`; the
ordinary `DU` approach establishes movement before A pulses reach the service.

Ignored evidence lives under `target/kanto-resume/name-rater-build/` and
`name-rater-review/`. The later independent nickname LCD review below closes
sampled source-pixel/icon acceptance for this service. Broader room/actor
fidelity and other party-selection specials remain separate acceptance work.

Initial 2026-10-19 private verification passed 13 selected checks,
including all 26 service cases and two repeat visits. Both final native/source
pairs were inspected; their complete RGBA images match, their source owner
receipts retain empty entries, and session state is preserved. Native
transitions now pass without the previously observed stale-presenter errors.
The renderer records early retirement and skips later ownership cleanup for
that entity. Live ownership cleanup retains the configured error handler.

Seven additional focused lifecycle/presentation checks pass, including a real
deferred CommandQueue regression that failed before the retirement guard. The
cold-handoff fixture clears the newer boot splash along with intro/title; its
layer assertions are unchanged. Both final headless naming pairs preserve the
session and are RGBA-byte-identical to the accepted captures. Evidence is in
`target/kanto-resume/lcd-lifecycle-review/review.json` and
`lcd-lifecycle-build/`. Other optional-renderer/empty-mesh warnings remain.
The full library suite and complete room art are not certified by these checks.

The standard test entry point is:

```sh
export CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/core-modular.crystalpack"
export NAME_RATER_REVIEW_DIR="$PWD/target/name-rater-review"
cargo test --locked -p crystal-bevy name_rater -- --include-ignored
```

Actual verification used the private test executable and saved rustc command
under `target/kanto-resume/name-rater-build/`. Runtime/host/example commands and
individual test/capture logs are retained there; no full Cargo build is claimed.
The fixture source-asset test now materializes the supplied pack instead of
requiring raw assets in a sibling checkout. No generated fixture content is a
tracked deliverable.

The independent nickname corpus now covers male/female Cyndaquil and genderless
Magnemite: 32 idle frames plus eight editing states each. All 120 full LCD
comparisons match the original ROM in RGB5 at the sampled icon/cursor phases.
The generator uses the external pinned ROM and null-window PyBoy, never this
renderer. See [the reference runbook](../menu-oracle.md#nickname-screens).

That comparison exposed missing Pokémon icons and gender markers, plus a
permanently visible cursor. Name Rater presentation now reads the selected
species' existing pack icon mapping and party icon palette. The gender rule
uses the source Attack/Speed DV byte and inclusive ratio comparison. Its icon
occupies original LCD coordinates (16,12). Icon frames change every eighteen
VBlanks and the cursor alternates every four. The naming cache includes these
bounded phases; source clock transitions invalidate presentation so the idle
renderer actually displays them. Gameplay state does not own a second editor.

The final idle regression runs ordinary native clock updates without physical
keys: all four icon/cursor combinations reach the retained LCD using one entity,
one texture allocation and four cached compositions. Party, bag, money, Pokédex
and position stay unchanged; the original VBlank GameTimer may advance.

Final current-source verification passes 21 selected checks, including the nine
Name Rater checks, the 26 service cases/two revisits, fullscreen handoff/retention,
three existing player naming checks, Pokémon heading, Mom/New Bark, live Oak
question and PC fifteen-frame controls. Both native service-entry pairs were
individually inspected and preserve exact session/pair pixels. Each native LCD
also matches an independent ROM frame at its actual icon/cursor phase, across
all pixels of the 5× LCD expansion. Evidence:
`target/kanto-resume/naming-pixels-review/review.json`, `comparison.json`,
`naming-pixels-native/` and `naming-pixels-build/`. Captures are headless image
targets with mute. Other optional-renderer/empty-mesh warnings remain, but the
previous stale-presenter errors do not recur.

These checks do not certify initial ROM entry-clock synchronization, every
joypad subframe, all species or other naming owners (player, rival, gifts and
hatching), complete room art or the full library suite. Compiled QA products,
ROM/reference images and game content remain ignored or external.
