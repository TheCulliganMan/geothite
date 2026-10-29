# Verdigrim integration handoff

The user requests full story wiring after the overworld sprite and unique cry
are ready. Both asset components now pass their technical checks.

Identity is **VERDIGRIM / MN_319 / MN_F145**, Steel/Poison, a massive Minnesota
float-copper wolf with green patina and exposed copper high points. Keep its
wolf anatomy and native 2D-only presentation. Kettlurk is MN_096, Water/Ghost.
Verdigrim contaminates and pins its returning current; its battle answer is
Crunch, without inventing Water/Ghost immunity.

Use these canonical sources:

- `planning/minnesota/pokemon/roster.json`, `species-profiles.json`,
  `movesets.json`, `palettes.json`, `pokedex-order.json` and `roster-curation.json`.
- Battle art: `sprites/mn-f145/manifest.json`, selected float-copper v4 source.
- Field art: `overworld/20261027-v1/sources/mn_f145/verdigrim.manifest-v3-eye-registration.json`.
  The accepted registry is complete 225/225; actor token is `SPRITE_VERDIGRIM`.
- Original score: `cry-designs.json` family MN_F145, species MN_319.
  The installed runtime cry is **CRY_MN_319**, pitch 0, length 256. Native script
  `cry VERDIGRIM` must resolve that binding.
- Reveal and containment: `planning/minnesota/pokemon/verdigrim-story.json`.

Compose sources onto your latest verified current-world descendant. Do not
select the local asset review pack as the regional parent or regress newer
terrain, scripts or story work. The existing Rust pipeline order is:

1. `pack_minnesota_pokemon` with the current source/native roots and
   `--family MN_F145` installs this species and reviewed front/back pictures.
2. `pack_minnesota_cries` installs the current authored book, including
   CRY_MN_319. Its source profile's donor cry is only a compiler compatibility
   reference; inspect the resulting `pokemon_cries["VERDIGRIM"]` binding.
3. `pack_reviewed_minnesota_overworld` installs the complete accepted banks,
   or use `pack_minnesota_overworld` with just the selected Verdigrim manifest.
4. Implement the chapter and encounter on that composed current parent.

Required story behavior: Team Lode removes glacial overburden and exposes a
sleeping native-copper mass. Wolf eyes open and the original howl plays. Its
forepaw crushes the test line. Green poison enters the local outflow; Verdigrim
pins the channel and Kettlurk's returning wave stalls. The player must separately
stop the drill, close the contaminated outlet and open the clean bypass before
the crew clears the area. Exact scene placement and timing belong to the story
owner; candidate maps in the contract are proposals, not published events.

Green water must be an actual visible, persisted map-scoped transition with
water animation and collision intact. Do not mutate a globally shared palette
or tint unrelated lakes. Actor removal/movement, controls, story flags and water
state must survive map reentry and Save/Continue.

Optional capture is level 60, with Crunch, Poison Jab, Iron Tail and Sludge Bomb.
Story completion must not require capture, a Safari permit or a paid gate.
Refusal, escape, defeat or blackout must retain another attempt while uncaught;
actual capture is once-only and persists. Use production party/PC delivery.

Keep all keyboard/MCP inputs on VisibleShellController. Add regressions for
dialogue ownership, each control action, green-water persistence, cry binding,
the retry/capture states and input parity. Inspect actual native 2D scene captures.
Do not claim earned campaign completion from setup flags or an art fixture.

Verified asset evidence:

- Six normal/shiny 16×16 frames; distinct idle/step pairs; transparent borders;
  384-byte bank and exact source-backed pupils. Eight production terrain views
  pass and were inspected beside the trainer.
- The original howl is 1.524 seconds / 33,596 stereo PCM frames at 22,050Hz.
  Rust gallery renders 225 distinct PCM voices. Current cry-structure/performance
  tests pass; scoped runtime decoding, slow variant, corruption rejection,
  gallery/runtime byte equality, idempotence and unrelated-content preservation
  pass. All 377 prior audio assets remain unchanged; final pack verifies 378 audio
  programs with matching PCM hashes, frame counts and loop ranges.
- Reference asset pack: `content-packs/verdigrim-assets-review-v02.crystalpack`,
  471 maps / 472 species / 270 moves. This is an unselected asset review base.
- Listening: `~/Music/Geothite/minnesota-cries/verdigrim-wolf-v1/index.html`
  and `verdigrim-runtime-v1/VERDIGRIM.wav`. Auditory/aesthetic acceptance has
  **not** been claimed; the original score and technical runtime checks are ready.

No shared world pointer, game service or website was changed by this asset work.

## Current-world composition boundary

The story task has composed the canonical assets, all225 authored voices and
updated research membership/rewards on the selected Marais/Mille world parent.
See [the composition status](../../acquisition-v01/verdigrim-composition.md) for
the exact unselected pack, hashes and acceptance limits. It has602 audio
programs, not the older one-voice reference count378. The excavation scene and
optional encounter remain unfinished; no full story/capture acceptance is
implied. Check the current runbook before treating this as a release input.
