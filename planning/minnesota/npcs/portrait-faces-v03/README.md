# Full-body Gym Leader and professor battle sprites

All eight Minnesota Gym Leaders and Professor Tamarack have new image-generated
full-body battle sprites. The previous waist-up format was rejected. These use
actual native Crystal trainer sprites as references for complete poses, head and
body proportions, outlined faces, clothing and feet within the 56×56 frame.

This is a separate local review candidate. It is not selected or deployed.
The current regional source selection and all overworld frames are unchanged.
Tamarack remains an older woman and a field-only professor; her portrait is used
in the opening. No professor battle was added.

## Sources

Each directory retains its generated images, exact prompts, native JSON and
conversion metadata. Corrections were made with the built-in image tool.
Native rows come directly from whole-image Rust conversion, with no manual
pixel edits. Non-battle JSON fields exactly match their original sources.

| Character | Generated source | Final prompt | Native source |
| --- | --- | --- | --- |
| Varel | [Image](varel/image-source-v4.png) | [Prompt](varel/image-prompt-v4.txt) | [56×56](varel/native-pixels-v1.json) |
| Torven | [Image](torven/image-source-v1.png) | [Prompt](torven/image-prompt-v1.txt) | [56×56](torven/native-pixels-v1.json) |
| Avrin | [Image](avrin/image-source-v3.png) | [Prompt](avrin/image-prompt-v3.txt) | [56×56](avrin/native-pixels-v1.json) |
| Brannic | [Image](brannic/image-source-v1.png) | [Prompt](brannic/image-prompt-v1.txt) | [56×56](brannic/native-pixels-v1.json) |
| Saven | [Image](saven/image-source-v1.png) | [Prompt](saven/image-prompt-v1.txt) | [56×56](saven/native-pixels-v1.json) |
| Sivren | [Image](sivren/image-source-v2.png) | [Prompt](sivren/image-prompt-v2.txt) | [56×56](sivren/native-pixels-v1.json) |
| Orlis | [Image](orlis/image-source-v1.png) | [Prompt](orlis/image-prompt-v1.txt) | [56×56](orlis/native-pixels-v1.json) |
| Norek | [Image](norek/image-source-v1.png) | [Prompt](norek/image-prompt-v1.txt) | [56×56](norek/native-pixels-v1.json) |
| Professor Tamarack | [Image](tamarack/image-source-v3.png) | [Prompt](tamarack/image-prompt-v3.txt) | [56×56](tamarack/native-pixels-v1.json) |

`conversion-all-v1.json` records exact original and generated sources, palettes,
sampling methods and settings. `publication-all-v1.json` is the full pack plan.
The authentic trainer references remain external ignored review inputs.

## Native review

The complete sheet is `target/trainer-full-body/all-nine-native-v2.png`, showing
the actual encoded 56×56 sprites at 1× and nearest-neighbor 6×. Canonical encoded
files are under `target/trainer-full-body/final-native-v2/NAME/encoded/`.

All nine retain complete bodies, legs and feet. Facial review checks eyes, noses,
mouths and adult character identity at game resolution. Avrin uses area32
sampling: the earlier area24 mouth outline merged into her jaw, creating an
unintended beard-like impression. The selected conversion retains her eyes,
small nose and separate toothy grin. Four-color conversion
still simplifies detail: Norek's separate tiny mouth dash is lost and the lower
moustache contour reads as his mouth; Sivren's far eye is narrow; a few accent
pixels remain around Orlis's beard. These are visible art limitations, not
mechanical failures. Visual review does not imply user acceptance.

## Local review pack

- Parent revision: `fair_release_intro_revision`
- Parent: `content-packs/minnesota-fair-release-v01.crystalpack`
- Parent SHA256: `18003b91597173f1d98937a0f6cc2fa82cfd2c5b3612c9803cd5f615acad2a73`
- Candidate: `content-packs/minnesota-trainer-full-body-faces-v02.crystalpack`
- Candidate SHA256: `41f90c12cf1e3356a74f9e3976248c722aee28a2d7ebab8fbb77192b4b157ae2`
- Content identity: `003f0043367a4a29ae713759a02cfb24205ae67339a484ef6a14f84d328d5c13`

The current parent was resolved again immediately before publication because
the shared checkout's selected campaign revision changed during art authoring.
Exactly 18 resources change: nine trainer PNGs and their nine 2bpp banks.
The publisher checks installed preimages, unchanged gameplay and audio, and
exact pack readback. Palettes, field sprites, maps, teams, AI, rewards and
progression remain intact.

With the existing built Rust tools, reproduce to a fresh ignored output:

```sh
target/debug/examples/pack_trainer_portraits \
  content-packs/minnesota-fair-release-v01.crystalpack \
  planning/minnesota/npcs/portrait-faces-v03/publication-all-v1.json \
  content-packs/minnesota-trainer-full-body-faces-v03.crystalpack
target/release/geothite play \
  content-packs/minnesota-trainer-full-body-faces-v02.crystalpack \
  --save target/trainer-full-body/player.crystalsave
```

Native encoding uses `npc_sprite_candidate --native SOURCE OUTPUT`. Whole-image
point conversion uses `trainer_portrait_palette IMAGE OUTPUT LIGHT ACCENT
--native PHASE_X PHASE_Y`; Sivren additionally uses `--chroma-weight 4` to avoid
mapping neutral source-edge pixels to teal. This optional authoring parameter
defaults to the previous behavior, changes no sample positions and preserves
exact palette inks. Area sampling parameters are recorded per character.

## Verification

Final verification receipts are retained under `target/trainer-full-body/`.
They cover source reproducibility, production controller portrait rendering,
Gym dialogue and battle flow, professor opening, and a fresh production-client
Save/Continue check. Explicit adjacent-position Gym fixtures test mechanics;
they do not establish earned campaign progression or battle balance.

- `source-provenance-final.json`: all nine source conversions reproduce every
  native row; all 54 canonical files match byte-for-byte; all 27 overworld files
  match the originals.
- `controller-20261120T155328Z-41f90c12-4_bvc4um/controller-verification.json`:
  all nine production-controller checks pass on the exact final pack. All
  rendered portraits have zero differing pixels from their native references.
  Gym tests cover dialogue, intro, move/PP, save/load, victory, badge/TM and
  repeat interaction; Tamarack uses the normal title/opening flow.
- `preservation-final-v2.json`: independent republication matches the final pack
  byte-for-byte, with exactly 18 changed trainer resources. Only Avrin's source
  differs from the first full-body candidate.
- `mcp-v2/verification.json`: actual production release executable, Start,
  movement, ordinary Save confirmation, fresh-process exact resume, further
  movement and Start; both processes exit successfully with empty stderr.
- `independent-full-body-review.md`: complete-pose and facial review against
  authentic native Crystal sprites, including remaining visual limitations.
- Authoring-tool checks: all six focused palette tests, workspace formatting,
  and `cargo clippy --locked -p crystal-assets --all-targets` passed. Clippy
  reported existing warnings outside the changed example. Logs remain under
  `chroma-study-current/`; default conversion outputs are unchanged.

The initial MCP attempt used an outdated debug executable that did not recognize
the current parent pack's save-compatibility metadata. That failure is retained
under `mcp/first-attempt-stale-debug`; the current release executable passed
both complete MCP runs. No user saves were touched.

Both duplicate preservation-check packs were removed after verification,
recovering about 81.9 MiB. Final disk check: 338 GiB free. Review evidence and
source inputs are retained; active shared build caches were not removed.
