# Optional Vergloom encounter

Ownership: `northstar_ancient_swamp.rs`, `pack_northstar_ancient_swamp.rs`,
`encounter.json`, and `tests/northstar_ancient_swamp.rs`. The terrain owner owns
R08 layout/dossier and the original northern-lakes tests. Do not modify those
files to implement this event.

The active legendary is Vergloom, MN_099 / MN_F048 (Dark/Grass). This is an
optional cryptid visit to the ancient mire; it does not replace its established
Vergas forest identity, Pepivern at Lake Pepin, or Kettlurk's story/side encounter.

Native clue targets and dry adjacent approaches on `MnR08C00R00`:

| Observation | Target | Approach | Facing |
| --- | --- | --- | --- |
| Petrified timber beside cedar remains | 47,33 | 48,33 | West |
| Peat footprint | 55,29 | 56,29 | West |
| Moss scrape | 60,25 | 59,25 | East |
| Vergloom, after three observations | 59,23 | 59,24 | North |

All three clue targets must be WALL markers. HEADBUTT_TREE would intercept the
first clue for players who know Headbutt; terrain-v03 fixes that conflict while
preserving its adjacent cedar remains. The publisher validates the exact
anchors and walkable approaches, requires the active roster family and reviewed
native Vergloom sprite, and preserves terrain, other maps, teams and audio.

Each fully acknowledged inspection sets its own persistent event. The third
reveals Vergloom with a cry and a separate acknowledged scene. Interaction then
offers a base-level-40 static wild battle. Existing wild-challenge rules may
raise its level to match the carried party; the event does not bypass those rules. Refusal retains the creature. Fleeing or
knockout hides it temporarily until map return. Only actual native capture sets
`EVENT_MN_ANCIENT_MIRE_VERGLOOM_CAUGHT`; later entries keep it absent.
The opt-in `BATTLETYPE_OPTIONAL_LEGENDARY` mode keeps normal RUN behavior and
records the caught bit only after actual party/PC capture storage. Native
`CheckCaughtCelebi` reads that bit. Canonical NORMAL and CELEBI rules remain
unchanged; CELEBI mode itself prevents escape, so it is not suitable here.
The actor has no bound event flag: ordinary disappear/appear commands must never
mutate its permanent capture flag. No main-route or Northstar completion event
is awarded.

## Publication and acceptance

The integration base is `minnesota-ancient-mire-terrain-v03.crystalpack`, already
region-bound with 475 species. Do not apply the regional binder a second time.
Install the reviewed native overworld bank first, then apply the event overlay:

```sh
RUST_MIN_STACK=67108864 target/northstar-story/wasm-dev/pack_minnesota_overworld \
  content-packs/minnesota-ancient-mire-terrain-v03.crystalpack \
  content-packs/minnesota-ancient-mire-art-v02.crystalpack \
  planning/minnesota/pokemon/overworld/20261027-v1/sources/mn_f048/vergloom.manifest-native-v5-light-window-proof.json
RUST_MIN_STACK=67108864 target/northstar-story/wasm-dev/pack_northstar_ancient_swamp \
  content-packs/minnesota-region-polish-rock-cut-v01.crystalpack planning/minnesota \
  content-packs/minnesota-region-polish-vergloom-events-v02.crystalpack
```

The final combined terrain/art parent is
`content-packs/minnesota-region-polish-rock-cut-v01.crystalpack`; it already
contains the reviewed sprite. Applying only `pack_northstar_ancient_swamp` to it
produced `content-packs/minnesota-region-polish-vergloom-events-v02.crystalpack`
(SHA-256 `1d444e70500cff6b83d4951c53dcbd9ed214241ab760a42c9adc0e264130d830`).
Earlier ancient-mire event packs v01-v04 were diagnostic builds with incomplete
battle metadata or unsuitable battle modes. Never migrate their saves.

The subsequent combined river/terrain integration is
`content-packs/minnesota-region-polish-final-v03.crystalpack`
(SHA-256 `67fb7ee40c022f0de78b561bbb144767d1483fe97a7dba9a2e6ec111d8c3360e`).
The coordinator reran both native chapter cases on that exact pack: 2 passed
in 41.38 seconds, `.local/region-polish-final-mire-native-v03.log`. Its own
fresh checkpoints live in `.local/region-polish-final-mire-v03-saves/`.

Both native chapter tests passed on that exact final pack using the shared
`target/runtime-fps/wasm-dev/deps/crystal_bevy-01787b9da148a652` executable:
three acknowledged clues, reveal, refusal, native RUN, seam reentry, knockout
with actual PP use, native loss, Master Ball consumption, capture, actor removal
and Save/Continue. The final-pack log is
`.local/northstar-ancient-mire-final-v02-native.log` (2 passed, 14.01 seconds).
Chapter fixtures declare their initial party
and one Master Ball for the capture branch and a held Smoke Ball for untrapped RUN verification; they are not earned-campaign proof.
After setup, movement, reads, RUN, FIGHT, item consumption, capture and Continue
must all occur through the production controller. No RNG or result injection
is allowed. Retain terrain render originals and the reached sprite captures.

The reached revealed save is
`.local/northstar-ancient-mire-final-v02-saves/ice-mire-revealed.crystalsave`.
Its headless native 2D capture is
`planning/minnesota/renders/ancient-swamp-v01/story-native-final-v02/revealed.png`.
The creature is visible immediately north of the player on the dry bank, beside
the pool and moss marker; the approach and surrounding terrain remain legible.

Replay the real stdio client from that same-pack checkpoint with:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_ancient_swamp_mcp.py \
  --pack content-packs/minnesota-region-polish-final-v03.crystalpack \
  --save .local/region-polish-final-mire-v03-saves/ice-mire-revealed.crystalsave \
  --captured-save .local/region-polish-final-mire-v03-saves/ice-mire-captured.crystalsave \
  --client target/northstar-story/wasm-dev/geothite
```

Fresh final-v03 stdio RUN/refusal/capture passed with clean process exit and
byte-empty stderr; a third fresh process continued the native captured
checkpoint, retaining Vergloom in the party and its absence from the map.
Log: `.local/northstar-ancient-mire-final-v03-mcp.log`.
The matching revealed native 2D image, inspected at the
actual reached save's position/time/facing, is
`planning/minnesota/renders/ancient-swamp-v01/story-native-final-v03/revealed.png`.

MCP presses clock individual production frames. Capture animation must finish
before its nickname/storage boundary; the replay advances bounded original
joypad frames rather than modifying capture results or stored state.

Focused core regressions passed for NORMAL/CELEBI/optional capture-result bits,
full-storage rejection, protected/trapped escape and held-item escape. The
current event-overlay preservation test passed in 15.78 seconds. Scoped Rust
format checks passed. Workspace formatting has unrelated existing differences;
core all-target Clippy is blocked by ten missing radio oracle fixture includes
in `crystal-core/tests/radio_text.rs`. No raw fixture catalogs were restored.
Core library-only Clippy and Assets/Bevy all-target Clippy passed with the
repository's existing warnings and baseline lint allowances.
