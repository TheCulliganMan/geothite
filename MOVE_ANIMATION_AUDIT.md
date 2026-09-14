# Move animation parity

Full-move LCD parity is not yet established. Object callback, frameset, and OAM
parity now have independent cartridge evidence; that is narrower than proving
every rendered move frame.

## Implemented

The renderer now advances persistent object state by executing the pinned
cartridge's 80 battle-object routines in a self-contained SM83 call-boundary
runner. The old age-based motion reconstructions have been removed. Frameset
playback, reinitialization, duration counters, byte wrapping, enemy coordinate
conversion, object commands, and OAM construction use the source instructions.
The runtime embeds its instruction image and needs no external emulator or ROM.
Pack-owned frameset and OAM data are installed into the runner's data memory.

Object allocation uses creation IDs, first-free slots, the source's partial
clear, and command-before-callback ordering. A callback can deinitialize an
object before its final OAM update. Filling 40 OAM entries stops callbacks for
later objects that frame. Rendering masks rows after the tenth sprite on a
scanline and uses the emitted coordinates and flips. Playback retains its
machine between frames rather than replaying its history each render.

The normal Rust pack exporter now follows authored OAM counts across adjacent
labels and preserves all attribute bits. Previously truncated sets omitted
pieces (including Focus/Razor Wind). It checks the instruction image and source
hashes before exporting. Object palette writes are retained with their update
frame; source ordering resolves them against command/background writes.

## Independent checks

- The PyBoy oracle executes the pinned ROM's unmodified routines.
- 552 function/parameter/side scenarios cover all 80 callbacks, including
  unused routines, for 384 updates each: 211,968 checkpoints per corpus.
- One corpus compares raw object state and relevant global registers.
- A second compares raw state, registers, framesets, and all shadow OAM bytes.
- Both corpora pass against the Rust instruction runner.
- Installing the exported bundle also passes the complete OAM corpus.
- Every exported OAM piece is compared with the cartridge table, separately
  from the callback samples.
- All 251 move scripts are swept for parameters 0–4 on both sides, including
  every intermediate object update. This is coverage, not pixel proof.
- Incremental playback has a command-history comparison against uninterrupted
  playback, including repeated render frames, skipped renders, set, inc, clear,
  and slot reuse.

## Still unproven

Background effects and graphic transfers are still separate implementations.
The current object renderer resolves graphics by object sheet; it does not yet
model every VRAM overwrite from successive anim_Ngfx commands. Complete LCD
compositing, including OBJ/BG priority, needs cartridge-frame comparison.
Battle-dependent branches and audio need end-to-end synchronized reference
captures. The isolated corpus does not establish arbitrary nonzero initial
state or every command history. Do not describe all moves as ASM-perfect based
only on these checks.

## Reproduction

From the repository root, after building the pinned reference ROM:

```
uv run --project tools/asm-oracle python tools/asm-oracle/export_battle_program.py
uv run --project tools/asm-oracle python tools/asm-oracle/battle_machine_trace.py
uv run --project tools/asm-oracle python tools/asm-oracle/battle_machine_trace.py --oam
./export
cargo test --manifest-path rust/Cargo.toml -p crystal-bevy battle_anim_ --lib
cargo test --manifest-path rust/Cargo.toml -p crystal-bevy battle_anim_machine --lib
cargo test --manifest-path rust/Cargo.toml -p crystal-assets --bin pack_core
```

Generated image/source provenance lives beside the instruction bank. The ROM
SHA-1 is pinned in the oracle tooling. Corpus metadata records case definitions
and output hashes. The earlier motion-only fixture remains a regression corpus,
not a production runtime path.

## Latest validation

The final integrated animation run passed 32 tests, including the installed
bundle corpus, CGB palette image regression, and incremental command history.
The standalone cartridge runner passed both 211,968-checkpoint corpora. The
integrated pack exporter passed three tests, and both packs rebuilt successfully.

The broader battle run passed 127/130 before the command-history fixture was
corrected; that fixture passes in the final animation run. Two failures remain
outside rendering: `runtime_battle_item_heals_active_party_pokemon_from_exact_pack_effect`
expects active combat during setup, and `runtime_battle_turn_uses_compiled_item_payload`
receives `BattleItemRequiresPartyTarget` for POTION. They were not changed by
this animation pass.
