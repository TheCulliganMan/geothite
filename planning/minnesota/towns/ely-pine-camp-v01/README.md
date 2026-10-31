# Ely pine camp

A refinement of the existing 20×18 snowy town. Native pine windbreaks frame the
League lodge, clinic, mart and existing roads. A packed snow loop behind the
lodge reaches a once-only Full Restore nook. Three ordinary residents explain
shelter, the loop and League preparation. The original ranger, signs, Fly
callback, real doors and north/east connections remain.

`world.json` is the selection authority. Authored sources are `before.json`,
`after.json`, `activities.json`, `contract.json`, and the complete `plan.json`.
No art is copied into these sources. The native `mn_ely_snow` alias, original
façades, seasonal palette and falling snow remain installed. All gameplay uses
ordinary walking; no Cut, Surf, ledge or slide is added as a town gimmick.

The original north lanes `(24,0)/(25,0)` join R13 at `(34,31)/(35,31)`.
The east lanes `(39,14)/(39,15)` join Ely Shore Pass at `(0,14)/(0,15)`.
The original clinic, mart and both lodge doors retain all eight active/passive
warp records. The clinic still heals PP; Full Restore heals HP/status only.

Eino stands at `(21,5)` with approach `(21,4)`. The initial rejected placement
at `(20,5)` occupied the old sign approach. The final builder checks both
approach tiles under each original sign with actor bodies included. Its
negative test specifically rejects restoring that occupied-sign layout.

## Fast full-town source loop

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_ely_pine_camp.py \
  --pack content-packs/minnesota-iron-crosscut-bends-v01.crystalpack \
  --source planning/minnesota/towns/ely-pine-camp-v01/after.json \
  --activities planning/minnesota/towns/ely-pine-camp-v01/activities.json \
  --output planning/minnesota/renders/ely-pine-camp-next-source
```

Use a fresh output directory each time. The measured warm whole-town and six
view render is recorded in its receipt, separately from pack-loading time.
Every original, converted image, rejected design and untouched GPU capture is
retained under `planning/minnesota/renders/ely-pine-camp-v958`.

## Rust publication

The guarded seven-argument `rebuild_ely_pine_camp` builder consumes the completed
parent pack and its SHA256, original terrain, reviewed terrain, activities,
contract and a fresh ignored output path. It rejects stale source/module hashes,
changed original anchors, disconnected floors, sealed ports, open borders or
blocked original sign lanes. Strict compiled-pack readback is mandatory.
Build jobs use the shared `.local/minnesota-cargo.slot.lock`; packs remain ignored.
The publication receipt records the exact executed arguments and binary hash.

## Acceptance

The initial candidate's new gameplay checks passed, but the inherited sign
check rejected Eino's original position. That pack and all its evidence remain
retained. Final corrected-pack acceptance is recorded separately and must cover:
three resident conversations, the original signs and four regional seam lanes,
all three original building returns, Full Restore collection and actual native
use, full-pocket denial/retry, once-only claim and production Continue.

Native screenshots are captured from actually reached saves with falling snow
enabled. No equality to a static snow-free source image is claimed. The complete
parent/roster/audio audits are separate from visual and controller proof.
Local starting position and one depleted Minnesota partner are explicit initial
fixtures; they do not establish earned regional or League progression. Existing
saved terrain overrides are not migrated. Wider regional map review stays open.

Final corrected pack: `content-packs/minnesota-ely-pine-camp-v02.crystalpack`,
SHA256 `b3e91e32af2de9bfebd1116d40f891fa611368001bd53b45e1e650963b65c3eb`.
Three controller cases pass, including the inherited original-sign/seam test.
A two-process production stdio replay passes 389 calls with empty stderr.
Five actual reached snowy native images and the whole town were inspected.
The full source preview and installed map agree in all 368,640 pixels; live
snow particles are reviewed separately. All 734 other maps, original Ely
programs/events, native assets, wildlife, trainer teams and audio remain exact.
The active 225-species regional policy and all 377 PCM/frame/loop checks pass.
Rebuilding from the same sources gives the identical pack SHA256. Both affected
crates pass all-target Clippy; owned formatting passes. Workspace formatting
still reports unrelated existing differences starting in offset-allocator.

The exact executed Rust publication is recorded in
`renders/ely-pine-camp-v958/publication-v974.json` relative to Minnesota planning:

```sh
CARGO_TARGET_DIR=target/verdigrim-scene CARGO_BUILD_JOBS=2 \
  cargo build --locked -p crystal-assets --bin rebuild_ely_pine_camp
.local/ely-pine-camp-publisher-v974 \
  content-packs/minnesota-iron-crosscut-bends-v01.crystalpack \
  a69310b47d2e3583d9fb8378784c919584591d6e260d5f548ae10ea0cd3409ce \
  planning/minnesota/towns/ely-pine-camp-v01/before.json \
  planning/minnesota/towns/ely-pine-camp-v01/after.json \
  planning/minnesota/towns/ely-pine-camp-v01/activities.json \
  planning/minnesota/towns/ely-pine-camp-v01/contract.json \
  content-packs/minnesota-ely-pine-camp-v02.crystalpack
```

The frozen publisher is a local build artifact. A fresh build supplies the same
CLI at `target/verdigrim-scene/debug/rebuild_ely_pine_camp`. Retain the existing
outputs and choose a new ignored output filename for another rebuild. This
revision changes no existing indoor reservations because all doorway bindings
and room modules are exact. Rain and connected central-north winter expansion
are later work, not acceptance claims of this Ely-only revision.

Selection remains pending: the strict publication check detected that the
selected world advanced to `northstar_title_revision` before selection. It
rejected the old-parent promotion and did not change world.json or the canonical
Ely source. Preserve the new title/intro resources, rebuild the Ely candidate
on that exact selected parent, and rerun the pack-bound acceptance checks
before selecting. The verification above describes the retained Iron-parent
candidate only.
