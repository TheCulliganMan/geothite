# Duluth harbor ice — review candidate

The selected room's fixed stops and trainer-operated gates looked alike, and
its two ice lanes were almost parallel. This unselected candidate frames the
lanes with native blue ice wall art and makes the four moving gates gold while
keeping the fixed stops silver. The east lane now has an additional real rough
stop: from `(11,16)`, slide up to `(11,12)`, left to `(9,12)`, then up via
`(9,10)` to `(9,7)`. The original west approach and gate order remain intact.
The guide explains the new turn and retains the Waterfall directions.

The active-roster requirement replaces retired Winkfin with Thawbonnet at
level 39 and retired Tipulord with its evolution Vernacape at level 45. Their
legal level-up moves are recorded in the source. No active Ice/Water family
was available in the supplied pack; this changes those two slots to
Ice/Psychic. Other team slots, held items, DVs, trainers and gate conditions
remain unchanged. Ordinary-party balance has not been established.

The immediate ignored review pack is
`content-packs/minnesota-eight-gyms-review-v01.crystalpack`, identity
`14f4840757c869f1f1aa4edc6ab71dbc2d587089f94f7a37e1d055c75621aa9e`.
Only `MnDuluthGym` changes among maps. Source and compiled initial terrain
pixels and collision quadrants match, and the only deliberate walking
collision change is the east rough stop. See the whole room and native
entry/east-gate views in `../renders/gyms/duluth-harbor-ice-published-v01/`,
and the source fixtures in `duluth-harbor-ice-cross-v01/` and
`duluth-harbor-ice-court-v01/`. The flag-seeded native view is visual evidence,
not an earned save.

The literal production-controller run began at the current Duluth facade
`(48,32)` with Mew100 and seven prior badges as fixtures. It tested the closed
east gate, both live ice approaches, opposing inputs, the new stop, an east
retreat, all three trainer wins, Norek, Continue, one reward, and both city
exit carpets. Separate loss and missing-seventh-badge tests passed. All 602
audio programs passed PCM hash, frame-count and loop checks. These tests do
not establish an earned regional journey or ordinary-party balance.

Build from the exact seven-gym review parent with a fresh ignored filename:

```sh
cargo run --locked -p crystal-assets --bin pack_duluth_harbor_ice -- \
  content-packs/minnesota-seven-gyms-review-v03.crystalpack \
  planning/minnesota/gyms/duluth-harbor-ice-v01.json \
  content-packs/minnesota-eight-gyms-review-v01.crystalpack
```

The later active-roster repair is described in
`active-roster-repair-v01.md`. Neither candidate is selected or deployed;
saved in-room tile overrides need migration review.
