# Complete Minnesota trainer battle portraits

This extends the accepted full-body Gym Leader/professor direction to all 40 remaining portrait identities used by the current Minnesota campaign. It also corrects Sivren's missing-eye appearance. The combined candidate contains 48 battle portraits covering all 337 Minnesota trainer entries, plus Professor Tamarack's opening portrait.

All new artwork and facial corrections use the built-in image generator. Native 56×56 rows are exact whole-image Rust conversions of the generated sources; no native pixels were painted or repaired by hand. Each character directory retains its image sources, exact prompts, selected native JSON, and conversion metadata. `cast-v1.json` is the complete source/encoding index.

Only the 14 private battle classes installed in the current pack are used from the historical regional authoring collection. Its 44 unused historical sets are not introduced into the campaign. The current ordinary classes receive regional battle art by exact trainer ID. Team Lode and the three established ICE roles have separate designs. The existing professor remains opening-only.

## Integration

`publication-private-v1.json` replaces the battle PNG/2bpp resources for the 15 existing private portraits (eight Gym Leaders, four Elite Four members, Reed, Tavel and Tamarack). `publication-overrides-v1.json` adds 34 battle-only portrait banks and maps the exact 308 ordinary-class Minnesota trainer IDs to them. The 29 entries already using private classes retain their classes and use the revised private banks.

The optional pack resource `gfx/trainers/portrait-overrides.json` selects presentation artwork independently of trainer class. Original stock art remains intact for Johto/Kanto, and no trainer is reclassified. Field sprites, maps, scripts, parties, AI, progression, rewards and audio remain unchanged.

The selected source for Sivren is `../portrait-faces-v03/sivren/native-pixels-v2.json`, generated from `image-source-v3.png`. The other eight previously accepted Gym Leader/professor portraits are carried forward from their existing source selections.

## Local candidate

This is a local review candidate; selection and deployment are separate. The parent is the current `fair_release_intro_revision`, `content-packs/minnesota-fair-release-v01.crystalpack`, SHA256 `18003b91597173f1d98937a0f6cc2fa82cfd2c5b3612c9803cd5f615acad2a73`.

The original art review used `content-packs/minnesota-trainer-full-cast-v02.crystalpack`.

- SHA256: `19b48eae691384ef9301aa932a15869ee42947b5a1f745614317d5dab20de9b0`
- Content identity: `b096786c02a13c3b595c2bc74f9b0adcb460166c2e2b1e25c6dda709d88ce4f6`
- Review gallery: `target/trainer-cast-rest/gallery/trainers-{1,2,3}.png`

The local v03 preview gives the full cast its own browser save slot. Rebuilding the same portraits against the same parent keeps that slot. Changing a portrait or its trainer binding creates a new slot. Its SHA256 is `d98c7f6352d6c258af7b98c3284b834bcbae76f6e18be9e9cf467b71e719a722` and its content identity is `b3de9604cfa8c8a681e32847e9f84088b6393fae799423f8c4a80b1a2eb57b46`.

```sh
target/debug/geothite play \
  content-packs/minnesota-trainer-full-cast-v03.crystalpack \
  --save target/trainer-cast-rest/player-review-v03.crystalsave
```

The review save is separate from existing player saves. To reproduce the pack, use fresh ignored output paths:

```sh
target/debug/examples/pack_trainer_portraits \
  content-packs/minnesota-fair-release-v01.crystalpack \
  planning/minnesota/npcs/portrait-cast-v01/publication-private-v1.json \
  target/trainer-cast-rest/rebuilt-private.crystalpack
target/debug/examples/pack_trainer_portrait_overrides \
  target/trainer-cast-rest/rebuilt-private.crystalpack \
  planning/minnesota/npcs/portrait-cast-v01/publication-overrides-v1.json \
  target/trainer-cast-rest/rebuilt-complete.crystalpack
```

## Verification

The v03 archive matches all 2,744 source art rows and all 49 native palettes. Its 8,631 retained runtime files match v02 byte for byte. The Fair save migration policy belongs to the Fair release and is absent from the preview. Local save checks reopened independent Fair and portrait progress with both graphical and terminal browser slot names, preserving the original save and backup.

The independent final pack audit verifies all 337 trainer bindings, every one of the 49 native portrait PNGs/2bpp banks/palettes, active-roster teams, exact gameplay/audio preservation, and no non-Minnesota bindings. Exactly 30 existing battle resources change and 103 resources are added (34 native banks plus the manifest). Evidence: `target/trainer-cast-rest/preservation-final.json`.

`target/trainer-cast-rest/source-provenance-final-v2.json` replays the selected image-to-native conversions for all 40 new identities plus Sivren. All 41 conversions reproduce their native rows; all 144 encoded resources match, including unchanged field banks for the seven full-source characters. The eight previously accepted portraits retain their prior selected sources and match the final pack exactly.

The combined final-pack controller receipt passes 49/49: 48 trainer battle journeys and Tamarack's real title/opening flow. Every actual native intro crop has zero differing pixels from its encoded reference. All 48 battles also pass the shared painted desktop/mobile asset-resolution checks. A later narrow-screen layout adjustment gives a single trainer the full available width; its focused desktop/phone/landscape renderer test and actual browser checks were rerun on the final code. The earlier verified TUI binary is retained with the 49-entry receipt. Evidence: `target/trainer-cast-rest/final-review-20261120T174458Z-vfabzo_t/aggregate-final-verification.json`. The initial Boarder fixture started on grass; its failed attempt is retained separately and the corrected native-floor fixture passed on the same final pack.

The rebuilt production client passes real movement, Start-menu Save with confirmation, fresh-process exact resume, further movement and Start. Evidence: `target/trainer-cast-rest/mcp-final-v2/verification.json`. A real native PTY also reaches the new Beauty portrait through ordinary A input; this is terminal protocol evidence, not a GUI screenshot.

The adjacent-position/prerequisite fixtures use legal level-60 Minnesota starters. They prove local rendering and controller behavior, not an earned campaign journey or intended-level balance. The actual rebuilt WASM also passes desktop Chrome with registered WebMCP and phone WebKit in portrait/landscape with touch A inputs. Each reaches the real trainer intro, preserves state across PAINT/TEXT, and reaches FIGHT. Screenshots were visually reviewed, including the enlarged, centered phone portrait with clear name, dialogue and touch controls. Desktop and landscape captures remain unchanged by that narrow-layout adjustment. This is a focused Beauty browser check; the 48 distinct TUI portrait resources were checked through the shared Rust renderer. Browser evidence: `target/trainer-cast-rest/browser-final-v2/browser-verification.json`.

`cargo fmt --all -- --check`, all-target Clippy for the four affected crates, the focused publication/rejection tests, and the standard `sh tools/tui-build.sh` pass. Build logs retain the pre-existing cfg-dependent `content_pack.rs` unused-mut warning and `menu_rendering.rs` unused-variable warnings. No new portrait-module warnings remain.

Superseded intermediate packs were removed after the final combined pass, recovering 122.94 MiB. The inactive task-owned WASM profile was also removed after preserving the verified preview. Combined cleanup recovered about 1.46 GiB. Final review evidence, sources, the final pack, native client and shared debug cache remain available. Cleanup details are ignored in `target/trainer-cast-rest/cleanup.json` and `target/trainer-cast-rest/implementation/wasm-cleanup.json`. The final build, browser and focused-layout validation index is `target/trainer-cast-rest/implementation/validation-final.json`.


## Generated sources and prompts

| Trainer | Selected image | Exact prompt | Native source |
| --- | --- | --- | --- |
| Beauty | [Image](beauty/image-source-v1.png) | [Prompt](beauty/image-prompt-v1.txt) | [56×56](beauty/native-pixels-v1.json) |
| Bird Keeper | [Image](bird_keeper/image-source-v1.png) | [Prompt](bird_keeper/image-prompt-v1.txt) | [56×56](bird_keeper/native-pixels-v1.json) |
| Blackbelt | [Image](blackbelt_t/image-source-v2.png) | [Prompt](blackbelt_t/image-prompt-v2.txt) | [56×56](blackbelt_t/native-pixels-v1.json) |
| Boarder | [Image](boarder/image-source-v1.png) | [Prompt](boarder/image-prompt-v1.txt) | [56×56](boarder/native-pixels-v1.json) |
| Bug Catcher | [Image](bug_catcher/image-source-v1.png) | [Prompt](bug_catcher/image-prompt-v1.txt) | [56×56](bug_catcher/native-pixels-v1.json) |
| Camper | [Image](camper/image-source-v2.png) | [Prompt](camper/image-prompt-v2.txt) | [56×56](camper/native-pixels-v1.json) |
| Casey | [Image](champion/image-source-v2.png) | [Prompt](champion/image-prompt-v2.txt) | [56×56](champion/native-pixels-v1.json) |
| Cooltrainer F | [Image](cooltrainer_f/image-source-v2.png) | [Prompt](cooltrainer_f/image-prompt-v2.txt) | [56×56](cooltrainer_f/native-pixels-v1.json) |
| Cooltrainer M | [Image](cooltrainer_m/image-source-v1.png) | [Prompt](cooltrainer_m/image-prompt-v1.txt) | [56×56](cooltrainer_m/native-pixels-v1.json) |
| Lode Chief | [Image](executive_m/image-source-v2.png) | [Prompt](executive_m/image-prompt-v2.txt) | [56×56](executive_m/native-pixels-v1.json) |
| Firebreather | [Image](firebreather/image-source-v1.png) | [Prompt](firebreather/image-prompt-v1.txt) | [56×56](firebreather/native-pixels-v1.json) |
| Fisher | [Image](fisher/image-source-v1.png) | [Prompt](fisher/image-prompt-v1.txt) | [56×56](fisher/native-pixels-v1.json) |
| Gentleman | [Image](gentleman/image-source-v1.png) | [Prompt](gentleman/image-prompt-v1.txt) | [56×56](gentleman/native-pixels-v1.json) |
| Team Lode | [Image](lode/image-source-v3.png) | [Prompt](lode/image-prompt-v3.txt) | [56×56](lode/native-pixels-v1.json) |
| Guitarist | [Image](guitarist/image-source-v3.png) | [Prompt](guitarist/image-prompt-v3.txt) | [56×56](guitarist/native-pixels-v1.json) |
| Hiker | [Image](hiker/image-source-v1.png) | [Prompt](hiker/image-prompt-v1.txt) | [56×56](hiker/native-pixels-v1.json) |
| Juggler | [Image](juggler/image-source-v3.png) | [Prompt](juggler/image-prompt-v3.txt) | [56×56](juggler/native-pixels-v1.json) |
| Lass | [Image](lass/image-source-v1.png) | [Prompt](lass/image-prompt-v1.txt) | [56×56](lass/native-pixels-v1.json) |
| Fern | [Image](fern/image-source-v2.png) | [Prompt](fern/image-prompt-v2.txt) | [56×56](fern/native-pixels-v1.json) |
| Flint | [Image](flint/image-source-v1.png) | [Prompt](flint/image-prompt-v1.txt) | [56×56](flint/native-pixels-v1.json) |
| Halden | [Image](halden/image-source-v1.png) | [Prompt](halden/image-prompt-v1.txt) | [56×56](halden/native-pixels-v1.json) |
| Veil | [Image](veil/image-source-v2.png) | [Prompt](veil/image-prompt-v2.txt) | [56×56](veil/native-pixels-v1.json) |
| Reed | [Image](reed/image-source-v1.png) | [Prompt](reed/image-prompt-v1.txt) | [56×56](reed/native-pixels-v1.json) |
| Tavel | [Image](timekeeper/image-source-v2.png) | [Prompt](timekeeper/image-prompt-v2.txt) | [56×56](timekeeper/native-pixels-v1.json) |
| Officer | [Image](officer/image-source-v2.png) | [Prompt](officer/image-prompt-v2.txt) | [56×56](officer/native-pixels-v1.json) |
| Picnicker | [Image](picnicker/image-source-v1.png) | [Prompt](picnicker/image-prompt-v1.txt) | [56×56](picnicker/native-pixels-v1.json) |
| Pokefan F | [Image](pokefan_f/image-source-v1.png) | [Prompt](pokefan_f/image-prompt-v1.txt) | [56×56](pokefan_f/native-pixels-v1.json) |
| Pokefan M | [Image](pokefan_m/image-source-v1.png) | [Prompt](pokefan_m/image-prompt-v1.txt) | [56×56](pokefan_m/native-pixels-v1.json) |
| Pokemaniac | [Image](pokemaniac/image-source-v2.png) | [Prompt](pokemaniac/image-prompt-v2.txt) | [56×56](pokemaniac/native-pixels-v1.json) |
| Sailor | [Image](sailor/image-source-v1.png) | [Prompt](sailor/image-prompt-v1.txt) | [56×56](sailor/native-pixels-v1.json) |
| Schoolboy | [Image](schoolboy/image-source-v1.png) | [Prompt](schoolboy/image-prompt-v1.txt) | [56×56](schoolboy/native-pixels-v1.json) |
| Scientist | [Image](scientist/image-source-v2.png) | [Prompt](scientist/image-prompt-v2.txt) | [56×56](scientist/native-pixels-v1.json) |
| Skier | [Image](skier/image-source-v1.png) | [Prompt](skier/image-prompt-v1.txt) | [56×56](skier/native-pixels-v1.json) |
| Super Nerd | [Image](super_nerd/image-source-v1.png) | [Prompt](super_nerd/image-prompt-v1.txt) | [56×56](super_nerd/native-pixels-v1.json) |
| Swimmer | [Image](swimmer_f/image-source-v1.png) | [Prompt](swimmer_f/image-prompt-v1.txt) | [56×56](swimmer_f/native-pixels-v1.json) |
| Teacher | [Image](teacher/image-source-v2.png) | [Prompt](teacher/image-prompt-v2.txt) | [56×56](teacher/native-pixels-v1.json) |
| Youngster | [Image](youngster/image-source-v1.png) | [Prompt](youngster/image-prompt-v1.txt) | [56×56](youngster/native-pixels-v1.json) |
| ICE Agent | [Image](ice_agent/image-source-v2.png) | [Prompt](ice_agent/image-prompt-v2.txt) | [56×56](ice_agent/native-pixels-v1.json) |
| ICE Operative | [Image](ice_operative/image-source-v1.png) | [Prompt](ice_operative/image-prompt-v1.txt) | [56×56](ice_operative/native-pixels-v1.json) |
| ICE Enforcer | [Image](ice_enforcer/image-source-v2.png) | [Prompt](ice_enforcer/image-prompt-v2.txt) | [56×56](ice_enforcer/native-pixels-v1.json) |
