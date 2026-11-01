# Physical companion spacing and first-person eye height

The world renderer keeps every Pokémon's original geometry, skin, clips and
pack-derived physical size. `pokemon_follow::pokemon_following_distance` now
uses the same model reference span and pack dimension as the actor renderer
to choose a presentation trail distance. It includes the animated model's
forward extent, a half-metre surface gap, and standing-view room for its height
and width. The ordinary one-game-tile gap remains the minimum. Camera controls
never enter the calculation, so looking around does not relocate a companion.
Missing dimension/model data retains ordinary sprite-card spacing.

First-person eye height now includes the same 1.25 player presentation scale
as the displayed trainer. Chris's authored 1.20 m height therefore gives a
1.275 m eye, rather than placing the camera at the unscaled 1.02 m height.
The camera still follows the production foot, terrain and traversal support;
user yaw, pitch, zoom and FOV remain authoritative.

`publish_party_follower` uses the distance for the recorded trail only. Its
sprite/card size and foot anchor remain at the original one-tile dimensions.
Initial placement probes every crossed gameplay tile with the production
movement predicate on a copy of the player, rejecting walls, occupied tiles
and transitions. If that path is blocked, the companion waits for sufficient
real walking history. A larger newly selected companion also waits instead
of inventing older history. Teleport detection stays tied to gameplay step
size, independently of the animal's longer gap.

Fractional gaps exposed a trail precision defect: after seeding or trimming the first
point, coordinate rounding could leave a stopped trail fractionally shorter
than the requested distance and hide the companion. The segment comparison
now allows the same 0.001 visual-pixel precision used by the existing trail,
with interpolation clamped to the recorded endpoint. It cannot extrapolate
beyond recorded history. An isolated extraction of the actual Rust trail and
new stopped-follower regression fails with the old strict comparison and
passes with the correction. No gameplay state or collision rules are changed.

## Verification

Current passing checks:

- Three focused renderer tests: matching the eye to the rendered trainer scale;
  projecting every original-size vertex of Nidoking, Nidoqueen, Arcanine,
  Feraligatr, Slowking and Pikachu inside the ordinary standing frustum over
  32 world-gait phases; real-dimension validation and map-pixel scaling.
- Eighteen camera tests, including the existing battle-camera tests selected by
  the camera module filter.
- Eight follower tests: corners, bounded history, blocked spawns, real occupied
  first/second tiles, larger gaps, stopped fractional trails, teleports, real
  pack dimensions, card size, production input and authoritative state. The
  Arcanine test isolates the production publisher after initial app settling
  and compares the complete serialized game state before and after it.
- Two real-pack Surf ownership tests: the non-lead field-move user and Goldeen
  as the actual Surf user.

These are 31 unique passing checks. The temporary old/new precision-control
executables supplement the official regression and are not counted again.
Use the ignored `target/pokemon-native-review/follower-clearance-` logs for
exact test names and results.

```sh
CARGO_TARGET_DIR=target/pokemon-models \
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/text-tui.crystalpack" \
  cargo test --locked -p crystal-voxel-view pokemon_follow --lib -j1 \
  --features bevy/webgl2,bevy/bevy_winit,bevy/x11,bevy/bevy_gilrs,bevy/zstd_rust,crystal-assets/test-fixtures \
  --config profile.dev.package.crystal-voxel-view.opt-level=0 \
  --config profile.dev.package.crystal-bevy.opt-level=0 \
  --config profile.dev.package.crystal-assets.opt-level=2 \
  --config profile.dev.package.crystal-core.opt-level=0
CARGO_TARGET_DIR=target/pokemon-models \
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/text-tui.crystalpack" \
  cargo test --locked -p crystal-bevy party_follower --lib -j1 \
  --features location-tester,bevy/zstd_rust,crystal-assets/test-fixtures \
  --config profile.dev.package.crystal-voxel-view.opt-level=0 \
  --config profile.dev.package.crystal-bevy.opt-level=0 \
  --config profile.dev.package.crystal-assets.opt-level=2 \
  --config profile.dev.package.crystal-core.opt-level=0
```

The additional `bevy_shell::tests::surf_mount_keeps_actual_surf_user_after_party_reorder_and_save`
check fails before app/follower initialization in
`tests/surf_anchor.rs::surf_anchor_preview_shell`, at
`prepare_surf_encounter_preview`'s unchanged-source water-table guard. It
requires Route44 water rate 5 and morning/day/night slots POLIWAG 25,
POLIWAG 20, POLIWHIRL 25. The current supplied local pack does not meet that
guard. The attempted direct test command uses the current test executable,
`--exact bevy_shell::tests::surf_mount_keeps_actual_surf_user_after_party_reorder_and_save --test-threads=1`
and the absolute TUI pack in `CRYSTAL_RENDER_TEST_PACK`. This remains unresolved;
no source encounter tables, packs or fixture guards are changed to bypass it.
No claim that the full gameplay suite or Surf save/reorder path passes follows
from the model and presentation checks.

## Offscreen production review

The current native example builds in 21.41 s. Every capture uses
`CRYSTAL_RENDER_TEST_HEADLESS=1`, `CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`,
`CRYSTAL_PREVIEW_MUTE=1`, `CRYSTAL_RENDER_TEST_PARTY_SHINY=0`, a 1000×700 image
target and synthetic walking input. Both the strict headless guard and mute
path are present in the executable. Its exact embedded compressed canonical
Nidoking, Arcanine and Feraligatr GLB payloads were checked before launching;
all model assets, dimensions, rig data and support offsets are unchanged.
No windows, focus changes or application activation are used.

The completed shared `content-packs/core-modular.browser.crystalpack` runs
through the production `render_at_location` example. Each primary review
starts at Route36 `(20,8)`, hour 12, `--view both`, with `--walk D`.
`CRYSTAL_RENDER_TEST_PARTY_SPECIES` selects each named species. First-person
controls are `CRYSTAL_RENDER_TEST_CAMERA='0 0 20 1'` with ordinary default FOV;
no FOV override is used. Orbit controls are `5 2 30 0`. Each traversal records
32 real ticks through five authoritative coordinates and ends at `(20,12)`.
Arcanine and Feraligatr additionally use wider orbit `2 2 30 0`. A stationary
Nidoking pair at `(20,8)` repeats the original horn-clipping location without
walking. All nine final 3D/2D pairs were inspected inline at native resolution.

Nidoking remains visible after stopping and its full horn, face and feet fit
both reviewed first-person locations. Arcanine's mane, face and feet fit its
standing view, with clear space between the creature and trainer. Feraligatr's
eyes are now visible above the jaw in its ordinary first-person view, with
crest and feet retained. Original sizes remain 1.3970, 1.9050 and 2.3114 m.

The zoom-5 orbit views crop the bodies of the two larger companions at the
right edge. Wider orbit views retain their full silhouettes; foreground
foliage still obscures some feet. Extra actual moving-00 frames for all three
species were inspected: model/trainer separation is retained, but signs and
foliage can still overlap the larger creatures. Those scenery overlaps,
turns, other cameras, browser and broader moving-world acceptance remain open.
The extra moving readbacks are asynchronous, not additional same-time 2D
pairs. This does not claim collision-free full-body placement on every path.

All final frozen pairs preserve authoritative state:

| Species / view | State frame / hash | LCD frame | Virtual seconds |
| --- | --- | --- | --- |
| nidoking / front | 2236 / 2361297603 | 2245 | 38.416681208 |
| nidoking / world | 2280 / 3882133955 | 2287 | 39.218660291 |
| nidoking / standing | 1747 / 3039290705 | 1747 | 29.683294 |
| arcanine / front | 2890 / 305234734 | 2898 | 49.871746416 |
| arcanine / world | 1879 / 2337221710 | 1886 | 32.391741708 |
| arcanine / wide | 2364 / 2029835443 | 2376 | 41.073706501 |
| feraligatr / front | 2343 / 1694217296 | 2347 | 40.18563775 |
| feraligatr / world | 1867 / 139238056 | 1875 | 31.872861166 |
| feraligatr / wide | 2373 / 1542364290 | 2384 | 41.104815999 |

Executable SHA-256: `2e3e4fd34247c16a6815a2599c695ca3881f4200533f14200bd6f242a473cd68`.
Identity is unchanged through the complete capture sequence. The ignored
`target/pokemon-native-review/follower-clearance-` evidence prefix retains
current PNGs, paired receipts, movement CSVs, executable identities, build/test
logs and the failed Surf fixture report. `follower-clearance-before/` retains
the initial fractional-gap visibility failure, separately from final images.
Temporary compiler control sources/binaries and helper scripts are removed.

No shiny verification, grass, AA/filtering, game content, audio or pack changes
are part of this correction. The full model and rig goal remains active with
216 registered skins and 59 static species files remaining.
