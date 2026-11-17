# Brainerd wooden docks — review candidate

This is an unselected candidate on the five-gym review lineage, not the
world selected by `world.json`. The room replaces the old brick deck with
native timber, puts the pier control on the south deck, and retains the
installed Minnesota trainer teams, rewards, exterior warps and story scripts.
Native Port buoy art marks two barriers across the pool and the exposed side
approaches. The center and left winch settings open the corresponding walks.

The Rust publisher checks six combinations of trainer wins and control
positions using both walking and walking-plus-Surf reachability. It rejects a
water route to Bravel or Sivren before the intended pier opens. Source and
published terrain pixels and collision quadrants were compared. Review the
closed, center and left whole-room renders in `../renders/gyms/` under
`brainerd-buoy-published-v02`, `brainerd-buoy-center-v01`, and
`brainerd-buoy-left-v01`; the first directory also contains the native entry
capture. These are retained visual evidence, not a selected release.

The canoe-equipped ignored review pack is
`content-packs/minnesota-six-gyms-buoys-canoe-review-v01.crystalpack`.
Controller fixtures proved that a canoe and Surf each launch into the lower
pool and stop at the closed pier, and that a Mew100/prior-badge fixture can
reverse the winch, win both trainer battles and the leader battle, save, and
leave and re-enter through both carpets.
The fifth-badge denial fixture also passed. These fixtures do not establish
earned regional progression or ordinary-team balance. The older ordinary-party
fixture lost against the current Minnesota teams; it assumes a selection
strategy for Johto species and needs a separate representative balance audit.
The pack's 602 audio programs passed PCM hash, frame-count and loop checks.

The review pack was composed with:

```sh
cargo run --locked -p crystal-assets --bin pack_brainerd_wooden_docks -- \
  content-packs/minnesota-five-gyms-review-v01.crystalpack \
  planning/minnesota/gyms/brainerd-wooden-docks-v01.json \
  content-packs/minnesota-six-gyms-buoys-review-v01.crystalpack
cargo run --locked -p crystal-assets --bin pack_canoe -- \
  content-packs/minnesota-six-gyms-buoys-review-v01.crystalpack \
  content-packs/minnesota-six-gyms-buoys-canoe-review-v01.crystalpack
```

The first command requires a fresh output filename. The controller tests use
`CRYSTAL_GYM_TEST_PACK` set to the absolute canoe-equipped pack path and
`RUST_MIN_STACK=67108864`.

Do not select this candidate until the remaining gyms, regional integration,
save migration, and full end-to-end review have been completed.
