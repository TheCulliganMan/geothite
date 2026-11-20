# Gym Leader and professor portrait revision

**Rejected by the user.** The professor's head is too large and chibi-like;
the native portraits still do not have sufficiently readable eyes, noses or
defining facial features. The nine sources have been removed from the regional
authoring selection. Mechanical test results below do not establish visual
acceptance. Retained packs and artwork are rejected review history, not the
replacement to use in new builds.

Nine image-generated replacements: Varel, Torven, Avrin, Brannic, Saven,
Sivren, Orlis, Norek, and Professor Tamarack. The work focuses on readable
eyes, brows, noses, mouths, hair and facial silhouettes in the 56×56 battle
format. Tamarack uses the same portrait resource in the New Game introduction;
this revision does not give her a battle or party.

The user rejected the initial hand-edited faces. Those drafts are excluded.
The `image-source` PNGs are the replacement artwork from the built-in image
tool, with the final prompts beside them. Every `battle_rows` grid here is
unaltered Rust converter output from its recorded generated image. The original
palette and all six overworld frames are copied unchanged from `previous_source`.

`conversion.json` records the precise source image, native source, original
source, crop, palette, sampling and thresholds. Source-center sampling retains
the leaders' facial features best. Tamarack's second image-generation pass uses
a larger face and simpler glasses; area voting with a black threshold of 12
retains the thin rims and smile. Coverage-preserving and default area-vote
candidates were reviewed and rejected for speckling or lost pupils. No manual
native face painting follows conversion.

## Portrait sources

| Character | Source | Native conversion |
| --- | --- | --- |
| Varel | [image-source-v1.png](varel/image-source-v1.png) | [56×56 source](varel/native-pixels-v1.json) |
| Torven | [image-source-v1.png](torven/image-source-v1.png) | [56×56 source](torven/native-pixels-v1.json) |
| Avrin | [image-source-v1.png](avrin/image-source-v1.png) | [56×56 source](avrin/native-pixels-v1.json) |
| Brannic | [image-source-v2.png](brannic/image-source-v2.png) | [56×56 source](brannic/native-pixels-v1.json) |
| Saven | [image-source-v1.png](saven/image-source-v1.png) | [56×56 source](saven/native-pixels-v1.json) |
| Sivren | [image-source-v1.png](sivren/image-source-v1.png) | [56×56 source](sivren/native-pixels-v1.json) |
| Orlis | [image-source-v1.png](orlis/image-source-v1.png) | [56×56 source](orlis/native-pixels-v1.json) |
| Norek | [image-source-v1.png](norek/image-source-v1.png) | [56×56 source](norek/native-pixels-v1.json) |
| Tamarack | [image-source-v2.png](tamarack/image-source-v2.png) | [56×56 source](tamarack/native-pixels-v1.json) |

## Local pack and selection

- Parent: `content-packs/minnesota-intro-fair-patrols-v01.crystalpack`
- Parent SHA256: `cfb028aec4080314247063a3c8686fa2f1b361a7c84bc7df3a4f5e3413aab0bd`
- Review pack: `content-packs/minnesota-trainer-portrait-faces-v02.crystalpack`
- SHA256: `d2e8f3624ec9f30254c08ba41bc07609d7f1331e594a4ce15b55018e06a3b5f0`
- Content identity: `acf147c20042483ddf3c520bc59b474f819094b9a7fbe8869408751178e7be70`

The regional trainer asset plan has been restored to the previous sources;
these nine rejected sources must not be used for future builds. `world.json` records the review pack without changing the active
local pack. This is a local review candidate, not a public deployment or a claim
of user visual approval. Existing saves remain bound to their existing packs.

## Rebuild and run

From the repository root, with the pinned Rust toolchain and existing external
parent pack:

```sh
CARGO_INCREMENTAL=0 cargo build --locked -p crystal-assets \
  --example npc_sprite_candidate --example pack_trainer_portraits
target/debug/examples/pack_trainer_portraits \
  content-packs/minnesota-intro-fair-patrols-v01.crystalpack \
  planning/minnesota/npcs/portrait-faces-v01/publication.json \
  content-packs/minnesota-trainer-portrait-faces-v03.crystalpack
target/debug/geothite play \
  content-packs/minnesota-trainer-portrait-faces-v02.crystalpack \
  --save target/trainer-face-review/player.crystalsave
```

The publisher requires a fresh ignored output path and verifies each installed
portrait against its recorded original source before replacing it. It refuses
stale or missing resources, duplicate identities, palette or overworld changes,
and unchanged portraits. The example output above uses `v03` to preserve the
already-built `v02` review pack.

To reproduce a generated image's grid, use the crop, light/middle palette values
and conversion settings in `conversion.json`:

```text
npc_sprite_candidate SOURCE.png OUTPUT X Y WIDTH HEIGHT LIGHT_RGB555 MID_RGB555 SAMPLING MIN_BLACK_SAMPLES MIN_WHITE_SAMPLES
```

The output `battle.candidate.json` contains the 56 rows. Encoding the combined
native source uses `npc_sprite_candidate --native SOURCE.json OUTPUT`.
Generated banks, packs, test saves and review images remain ignored.

## Verification

The fresh pack was written and read back successfully. Exactly 18 resources
changed: each character's trainer PNG and 2bpp bank. All palettes, overworld
banks, gameplay data, map/script metadata, trainer parties and rewards, audio,
and other resources match the parent. Native banks pass an independent tile
roundtrip.

Three publisher unit tests and the final nine-portrait pack-preservation test
passed. `cargo clippy --locked -p crystal-assets --all-targets` passed with
existing warnings. `cargo fmt --all -- --check` passes.

The real `target/debug/geothite mcp` client loaded the revised pack and exercised
Start, actual movement, Start-menu saving, and a fresh process resuming that
save with movement and Start available. Traces are under
`target/trainer-face-review/mcp-v02/`.

All eight leaders pass the production-controller encounter check: dialogue,
trainer intro, a selected move and PP change, battle save/load, legal victory,
the native badge and exactly one TM, repeat conversation and saved progress.
Tamarack's production title/New Game opening also passes, reaching CHRIS's
bedroom with Start available. All nine actual rendered portraits were reviewed.

Controller evidence is recorded in the ignored
`target/trainer-face-review/gameplay/` directory. Four historical fixture
approaches were stale; the review uses unoccupied FLOOR tiles adjacent to the
actual compiled actors: Varel `(11, 6)` west, Avrin `(5, 3)` west, Brannic
`(14, 4)` south, and Sivren `(8, 4)` south. Original failed fixture logs are
retained separately. No gameplay or encounter script was changed to pass a test. Its fixture party is three
legal level-60 Minnesota starters with maximum DVs, adjacent placement and
explicit prior-access flags. These checks establish local encounter behavior,
not an earned campaign journey or intended-level balance.

## Review material

- `target/trainer-face-review/index.html`: generated source and actual native art.
- `target/trainer-face-review/before-after.png`: old/new sprites at native 1× and 3×.
- `target/trainer-face-review/in-game.png`: all nine production-rendered portraits.
- `target/trainer-face-review/native/`: encoded native banks and 8× review images.
- `target/trainer-face-review/gameplay/`: production-rendered encounter captures.

The source PNGs are larger than the game art. Judge the native comparison and
actual game captures when assessing the installed result.
