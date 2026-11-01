# Rochester: switch the lanes

The user rejected both planter mazes: their walls forced walking without an
interesting decision. The current source is [rochester-gates.json](rochester-gates.json).

The smaller 7 × 7 room has two blue wall switches and three gold gates. Ordinary
walls stay fixed. The lower switch opens one lane and closes the other. The
upper switch operates the gate below Varel. A changes the switch immediately;
there is no quiz or selection menu.

![The room's gate states](../renders/gyms/rochester-gates/gate-sequence.png)

1. Battle Tavel in the entrance hall. Press the lower switch to open the left lane.
2. Enter the left bay and battle Elvik. Press the upper switch: Varel's gate
   opens across the divider, on the right side.
3. Return to the entrance hall. Press the lower switch again. The left gate
   closes and the right gate opens. Take that lane to Varel.

That return is necessary. Taking the right lane before opening the upper gate
leads to a closed gate and an ordinary return. Neither battle moves a gate by
itself; each switch requires its nearby trainer's win. The upper gate can be
closed and reopened. The lower switch remains reachable in every configuration,
so the left control can always be regained.

| Feature | Walking-tile coordinate / behavior |
| --- | --- |
| Arrival, indexed warp3 | `(6,11)` |
| Tavel | `(4,10)`, Sentret 8 |
| Lower switch | `(2,9)`, operate from `(2,10)` facing up |
| Left/right lane gates | Metatiles `(2,4)` / `(4,4)`; mutually exclusive |
| Elvik | `(4,6)`, Hoothoot 9; requires Tavel |
| Upper switch | `(2,5)`, operate from `(2,6)` facing up |
| Court gate | Metatile `(4,2)`; upper-open event and both victories required |
| Varel | `(10,2)`, original Sentret 10/Furret 12, moves and parcel check |
| Reward | Care and one Headbutt TM; original once-only claim behavior |
| Carpets1/2 | `(6,13)` / `(7,13)` → existing Rochester return6 `(30,24)` |

The room uses original native flooring, wall buttons, gold sliding-door art,
statues and carpet. The diagram and animation are source-state views; gameplay
is proved separately through the controller.

## Executed publication and control proof

Ignored `content-packs/minnesota-rochester-gates.crystalpack`: 541 maps, identity
`cd560f97f332b1f81070cbf82bb79d6d692c443ba49b609b4eb3ee6028ed6967`.
It extends the readable-maze parent; later story/city successors still need
this revision applied to their lineage. Installation grants no progress.
Existing in-room saves need position migration.

```sh
CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  run --locked --profile wasm-dev -p crystal-assets --bin revise_rochester_gates -- \
  content-packs/minnesota-rochester-readable.crystalpack \
  planning/minnesota/gyms/interiors.json \
  planning/minnesota/gyms/rochester-gates.json \
  content-packs/minnesota-rochester-gates.crystalpack

RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/gym-controller \
CRYSTAL_GYM_TEST_PACK="$PWD/content-packs/minnesota-rochester-gates.crystalpack" \
cargo \
  --config 'profile.wasm-dev.package.crystal-core.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-runtime.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-bevy.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-bevy --lib \
  rochester_gates_literal_controls -- --ignored --nocapture --test-threads=1

CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-assets --test rochester_gate_revision \
  -- --ignored --nocapture --test-threads=1

target/gym-interiors/wasm-dev/verify_crystalpack \
  content-packs/minnesota-rochester-gates.crystalpack --audio --desktop
```

The literal control test passes in 15.94 seconds with Totodile 16/Hoothoot 14 and
native level-up moves. Initial placement, parcel and deterministic test RNG are
supplied. It earns all three wins, spends PP, tries the wrong lane, checks both
disabled switches, closes/reopens the upper gate, checks Continue, verifies the
single TM and uses both real city returns. Walking uses fixed directions;
reachability checks inspect collision without steering.

The test caught a real bypass: a central half-wall left a third column beside
the lower gate. The barrier is now closed. The Rust publisher checks all six
lane/upper-gate configurations and rejects that corruption. Both asset tests
pass, including changed-party, malformed-toggle and repeat-install negatives,
preservation of all other maps/battles/numbers/destinations/art/audio, and
uncached split-module reconstruction. All 377 PCM hashes/frame/loop checks pass.

Native closed/open captures at `(6,11)`, noon, facing down were visually
reviewed. Independent source cameras match all 368640 pixels in each capture,
including borders and actors. Rejected drafts and every render generation stay
archived. Changed Rust passes scoped rustfmt. Assets and Bevy all-target
Clippy passes with existing warnings (`--no-deps`, existing `never_loop`
allowance). Workspace `cargo fmt --all -- --check` still first fails at the
unrelated `crates/offset-allocator/src/lib.rs:408`.

[current.json](current.json) selects this room. The current town binding is
[MnRochester.gates.json](../towns/MnRochester.gates.json); world/interior
reservations use its 7 × 7 extent and updated arrival/carpets. The old town dossier
stays unchanged for the historical build chain. `check_plan.py` passes the
current bindings. Other seven-room improvements and user visual acceptance
remain open.
