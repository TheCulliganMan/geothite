# Current Winona–St Cloud river review

The actual current waterfront candidate supports continuous native Surf from
Winona through St Paul and Minneapolis to St Cloud, and the complete return.
The controller follows1,247 route actions each way, uses all seven bridge
apertures, climbs St Anthony Falls and returns through its native downward
current. Save/Continue beneath a bridge works in both directions. Walking on
the bridge deck remains a separate, working surface.

This closes the pending current-pack river replay from the frontage revision.
It preserves the two long, quiet wooded reaches instead of adding a different
route or changing geometry that already satisfies the request. No pack, map,
source layout, encounter, story or audio data was changed in this review.

- [Whole connected current river](../../renders/current-river-v01/verified-v01/whole-current-corridor.png)
- [Verified receipt](../../renders/current-river-v01/verified-v01/receipt.json)
- [Exact native input itinerary](../../renders/current-river-v01/native-v01/all-bridges-input-plan.json)
- [Physical/seam/boundary proof](../../renders/current-river-v01/native-v01/physical-proof.json)
- Reached mouths: [Winona](../../renders/current-river-v01/native-reached-v01/winona-mouth.png), [St Paul](../../renders/current-river-v01/native-reached-v01/st-paul-east-mouth.png), [Minneapolis south](../../renders/current-river-v01/native-reached-v01/minneapolis-south-mouth.png), [Minneapolis north](../../renders/current-river-v01/native-reached-v01/minneapolis-north-mouth.png), [St Cloud](../../renders/current-river-v01/native-reached-v01/st-cloud-arrival.png).
- [Ordinary upper wooded river](../../renders/current-river-v01/native-reached-v01/ordinary-upper-river.png)
- Upper bridge reached states: [walking on the road](../../renders/current-river-v01/bridge-planes-v01/foot-road.png), [Surf below the deck](../../renders/current-river-v01/bridge-planes-v01/surf-underpass.png).

All eight native frames and the whole corridor were inspected. The city mouths
join real water on neighboring maps; no invented connecting terrain fills the
atlas gaps. The ordinary river camera has a readable bend and continuous pine
banks. The bridge deck intentionally occludes the rider below it; native
movement, Surf mode and Continue are independently verified. The two bridge
images have different reached positions, rather than posing as matched
before/after cameras. All captures use their actual saved clock and facing,
without map, position or hour overrides.

## Exact candidate and native geometry

Use the ignored `content-packs/minnesota-waterfront-frontages-v01.crystalpack`,
file SHA256 `3862c34b044a456721b5f7ee5464b42dd123f07e3bd12d16e7e503bec972ffa2`.
It contains691 maps. The corridor image shows26 actual exterior footprints
at a single scale and retains each full-resolution native map in `native-v01/maps/`.
The current whole-state atlas remains linked from the waterfront runbook.

The physical proof checks6,923 river-component water tiles and276 directed,
reciprocal water seam lanes. There are no reachable unconnected Surf edges in
that component. Removing either wooded reach, St Paul or the bridge underpasses
breaks the trip; these are actual negative checks. The lower and upper woods
retain361 and374 interior route actions in each direction. This is a connected
river-component proof, not an audit of every separate lake in Minnesota.

For static planning only, `audit.py` opens a private copy of the three existing
`MnMPUpperPoolBuoyOpen` blocks. The native controller receives the declared
initial badges and independently executes the real callback. The script never
publishes a pack or writes map terrain. It verifies every input/seam against
native collision and inventories actual bridge components rather than reusing
old city dimensions. Its itinerary's inherited `parent_pack_sha256` is historical
manifest ancestry; `pack_sha256` binds the exact current installed pack.

## Fixture and executed commands

The explicit initial fixture is CHRIS on Winona's real bank at core(25,11),
with level80 SLIDEWAKE, Surf/Waterfall and Plain/Fog/Rising badges. SLIDEWAKE is
the active Mississippi river-otter family member; its native species definition
legally supports both HMs. The shared fixture now accepts
`CRYSTAL_RIVER_FIXTURE_SPECIES` and checks actual HM compatibility. Other tests
retain their default MEW fixture. Initial party, badges, position and deterministic
divider are setup, not earned progression. No subsequent position, flags,
mode, party or RNG state is injected. Incidental wild encounters use native RUN.

```sh
export PYTHONPATH="$PWD/.local/ely-backroute-python${PYTHONPATH:+:$PYTHONPATH}"
python3 planning/minnesota/connections/current-river-v01/audit.py \
  --pack content-packs/minnesota-waterfront-frontages-v01.crystalpack \
  --expected-sha256 3862c34b044a456721b5f7ee5464b42dd123f07e3bd12d16e7e503bec972ffa2 \
  --atlas-audit planning/minnesota/renders/minneapolis-waterfront-v01/state-continuity-v01/native-connection-audit.json \
  --output planning/minnesota/renders/current-river-v01/native-v01

CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=3 RUST_MIN_STACK=33554432 \
cargo test --profile wasm-dev --locked -p crystal-bevy --lib --no-run \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0 \
  --config profile.wasm-dev.package.crystal-bevy.opt-level=0 \
  --config profile.wasm-dev.package.crystal-bevy.codegen-units=256

export CRYSTAL_MINNEAPOLIS_FALLS_CONTROLLER_PACK="$PWD/content-packs/minnesota-waterfront-frontages-v01.crystalpack"
export CRYSTAL_RIVER_FIXTURE_SPECIES=SLIDEWAKE
export CRYSTAL_MISSISSIPPI_ITINERARY="$PWD/planning/minnesota/renders/current-river-v01/native-v01/all-bridges-input-plan.json"
export CRYSTAL_MISSISSIPPI_CAPTURE_SAVES_DIR="$PWD/planning/minnesota/renders/current-river-v01/reached-v01"
export RUST_MIN_STACK=33554432

target/runtime-fps/wasm-dev/deps/crystal_bevy-01787b9da148a652 \
  bevy_shell::tests::minneapolis_falls::mississippi_journey:: \
  --ignored --nocapture --test-threads=1
target/runtime-fps/wasm-dev/deps/crystal_bevy-01787b9da148a652 \
  bevy_shell::tests::minneapolis_falls::minneapolis_falls_controller_ \
  --ignored --nocapture --test-threads=1
```

Both journey/bridge cases passed in16.94 seconds. All five Falls/seam/badge
cases passed in6.47 seconds, including denied uphill Waterfall and the downstream
missing-Plain-badge bypass attempt. The journey retains45 native checkpoint
saves and the original real Winona dismount. Route actions exclude extra
turning, text acknowledgement, menu and animation inputs; they are not a count
of every button press.

The renderer was freshly built with `location-tester` and the same two package
optimization overrides. Each capture uses the actual reached save, for example:

```sh
CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_RENDER_TEST_WINDOW_SIZE=640x576 RUST_MIN_STACK=33554432 \
target/runtime-fps/wasm-dev/examples/render_at_location \
  --pack content-packs/minnesota-waterfront-frontages-v01.crystalpack \
  --save planning/minnesota/renders/current-river-v01/reached-v01/MnStPaul-79-52-Left.crystalsave \
  --view 2d \
  --screenshot planning/minnesota/renders/current-river-v01/native-reached-v01/st-paul-east-mouth.png
```

All output epochs are immutable; use fresh names when repeating commands.
The initial Python syntax error was corrected for the installed Python version
before the successful audit. Its failure log remains saved. No successful
artifact was overwritten. The receipt binds the input plan, logs, code snapshot,
executables and manually inspected native frames by SHA256.

## Remaining limits

The changed Rust fixture passes rustfmt and diff checks. Required workspace
`cargo fmt --all -- --check` still fails on existing unrelated formatting.
The normal `cargo clippy --locked -p crystal-bevy --all-targets` run with
`location-tester` still hits existing core `bad_bit_mask` and voxel
`approx_constant` errors. A scoped `--no-deps --lib --tests` run with the
existing `never_loop` allowance passes, with no diagnostic in the changed
fixture. Logs remain under `.local/current-river-*`.

This is current native river and named-frame evidence, not an earned full
campaign, final3D acceptance, optional-bank activity replay or certification of
all map quality. Major-city density, Iron Range caves, Boundary Waters activities
and remaining regional integration stay in scope under the broader goal.
