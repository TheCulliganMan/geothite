# Tamarack, Team ICE and eight trainer types

Tamarack is a woman. The 16×16 overworlds use actual Crystal six-frame bases,
with the original head size, silhouettes, poses and walking animation. The
only logical ink edits are 38 light-hair pixels across Tamarack's six poses;
all other field grids retain the original indices. Clothing uses private
four-color RGB555 palettes. Detailed equipment stays in the 56×56 battle art.

Team ICE follows the user's revised direction: dumb, mean paramilitary. Agents
have crooked helmets, bulky vests and awkward stances; leadership has a severe
haircut and an oversized olive greatcoat based on the public Greg Bovino
reference linked in `04-ice-enforcer/ACCEPTANCE.md`. Agent/Operative field bases
are stock Rocket M/Rocket Girl; leadership uses Surge. Their silhouettes and
indices remain stock Crystal. Earlier police and frightened drafts are
superseded and retained only in ignored history.

`roster.tsv` selects the twelve authoritative native grid sources and their
binding/placement files. Tamarack retains her existing field-only role. The
three ICE art sets cover all 17 existing opponents and 17 corresponding field
actors, preserving their parties, AI, rewards and scripts. Ordinary officers,
Team Lode and Johto/Kanto art remain unchanged.

| New type | Trainer | Location | Authored team |
| --- | --- | --- | --- |
| Maple Tapper | VERN | Mankato (32,24) | Zapsap 18, Pinit 20 |
| Roller Skater | MADDIE | Minneapolis (29,35) | Pipulse 18, Tiltawisp 20 |
| Curler | GORDY | Grand Marais (16,14) | Pucklet 38, Lutebrisk 40 |
| Lifeguard | CAL | Bemidji (27,35) | Rivvit 30, Ticklet 32 |
| Port Pilot | GRETA | Duluth (54,59) | Tugget 36, Loobit 38 |
| Trail Runner | JUNE | St Cloud (14,13) | Bluestip 24, Bunjant 26 |
| Camp Cook | GUS | Brainerd (42,21) | Doughlin 26, Pipcorn 28 |
| Dog Musher | OTTO | Ely (23,18) | Rimepelt 40, Driftpeg 42 |

These eight types are additional to the older cast review. Each has a private
class, matched battle/field art and an actual native trainer encounter. Their
placements passed walkability, event clearance and walking-lane checks; the
Minneapolis placement also leaves existing cycling patrols clear.

The mean paramilitary revision extends the Northwoods grass descendant,
`content-packs/minnesota-current-grass-northwoods-v01.crystalpack`
(`cb212e1a4288b255d8d9ffb739c279fa3f2b6b3f1d679c9f4f8b2b30625e58bd`).
It preserves Cedar geometry, all reviewed creature walking art, Castorock
acquisition and the Northwoods grass/ledge revision. Concurrent map work has
since advanced the world selection again; this is a separate local playable
art pack and does not replace that newer selection.

## Build and run

The native grids are self-contained authoring inputs. The Rust encoder and
publishers produce all PNG, 2bpp and palette banks; no generated banks or packs
belong in Git. The build script preserves its input and refuses existing output.
The following build/publish path was executed locally:

```sh
CARGO_TARGET_DIR=target/tamarack-ice/full-build cargo build --locked \
  -p geothite -p crystal-assets --bin geothite --bin pack_trainer_art \
  --bin pack_original_npc --bin verify_crystalpack
CARGO_TARGET_DIR=target/tamarack-ice/full-build cargo build --locked \
  -p crystal-assets --example npc_sprite_candidate
mkdir -p target/tamarack-ice/bin
cp target/tamarack-ice/full-build/debug/examples/npc_sprite_candidate target/tamarack-ice/bin/
cp target/tamarack-ice/full-build/debug/{geothite,pack_trainer_art,pack_original_npc,verify_crystalpack} target/tamarack-ice/bin/
CRYSTAL_SPRITE_BIN_DIR=target/tamarack-ice/bin \
  bash tools/build-tamarack-ice-sprites.sh \
  content-packs/minnesota-current-grass-northwoods-v01.crystalpack \
  content-packs/minnesota-current-tamarack-ice-mean-r20-v02.crystalpack
```

Choose a fresh ignored output name when rebuilding. The final pack SHA256 is
`368c77ea520c2b2ed2c6a0dbc693e962185df7343e80de21ac14a5fda6680159`.
The local production client is retained at `target/tamarack-ice/bin/geothite`:

```sh
./target/tamarack-ice/bin/geothite play content-packs/minnesota-current-tamarack-ice-mean-r20-v02.crystalpack
./target/tamarack-ice/bin/geothite mcp content-packs/minnesota-current-tamarack-ice-mean-r20-v02.crystalpack
```

## Verification and review

Every set passed its individual native comparison, actual game capture and
production controller encounter before the next was authored. The current combined
pack's 26-case sweep passed under `target/tamarack-ice/mean-final-sweep/`: the
earned Tamarack opening, all 17 revised ICE variants and all eight new trainers.
`accepted-results.json` records every passing case on the exact mean-final
identity. Native comparisons and actual final field/battle captures were
visually inspected. Earlier police-design receipts are historical.

The independent preservation audit passed for all 72 private resources, every
original resource and compiled audio payload, all original trainer parties/AI/
rewards, all terrain/connections/events, existing scripts and unrelated data.
All 324 Minnesota trainer teams and reachable evolution families remain in the
224-species active roster. The Rust audio verifier passed all 377 PCM hashes,
frame counts and loop ranges plus the desktop presentation consumers.

The production stdio client passed Start/Pack, menu input ownership, actual
movement, downstairs dialogue blocking/release, saving and fresh-process reload
in 22 JSON-RPC calls with byte-empty stderr. The four required core controller,
MCP and clean-stdio regressions also passed against `text-tui.crystalpack`.
All four binder preservation/stale-input tests passed against their exact final
parents. A fresh Cargo-built encoder reproduced all 72 final banks byte for byte.

Asset and Bevy all-target Clippy completed with existing warnings. Changed Rust
leaves pass rustfmt; workspace formatting still reports unrelated differences,
including offset-allocator. No user visual approval or deployment is implied.

The trainer tests declare adjacent-position fixtures and three legal level-60
Minnesota starters with maximum DVs. The Enforcer fixture includes the two
existing access badges, with no victory/rescue flag supplied. Opponents and
scaling remain intact. These checks prove inputs, native loading, selected
move/PP, legal victory, defeat/repeat and saves; they do not prove an earned
full journey or ordinary-level balance. Tamarack uses the earned New Game
opening and retains her starter, supplies, dialogue and Continue behavior.

The built-in image tool supplied the selected battle concept references;
`roster.tsv` selects every final native source, and each folder retains its
selected prompt/reference. Camp Cook was authored directly on the native grid.

The review index is `target/tamarack-ice/review-mean-v2/index.html`, with native-size
sprites, actual battle/field sheets and all 17 ICE field captures. The native
sheet is `target/tamarack-ice/review-mean-v2/native-gallery.png`; individual
same-scale Crystal comparisons and actual final captures remain ignored under
`target/tamarack-ice/`. Rejected native drafts and historical stage receipts are
separate from the selected source files. Build caches, obsolete candidate packs
and temporary object files were moved to recoverable Trash; the final pack,
client, native banks and review evidence remain ignored locally.
