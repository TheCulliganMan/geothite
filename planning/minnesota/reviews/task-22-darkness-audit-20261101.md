# Task 22: cave darkness and Flash progression audit

This is a partial, read-only audit of the local story review candidate
`content-packs/minnesota-story-review-fixes-v09.crystalpack`, SHA-256
`175e419ac0abd0261d6417fdb825eabd3de0a7223b51f45baf1390ea8f29259a`.
It is not evidence of the selected or deployed pack. Task 22 remains open.

The compiled field-move rule requires Johto badge index 0. The installed
`MN_R01_HOLLOW_GUIDEFieldHM` handoff comes from the HM progression builder,
which checks `ENGINE_ZEPHYRBADGE` before giving Flash. The planned regional
mapping identifies that flag as the Rochester Care Badge. Thus a player must
reach and defeat the Rochester Gym, return to the guide, receive Flash, and
teach it to a compatible party member before using it. Ownership, permission,
and a compatible party member are separate conditions.

In this candidate `MnR01Cave1F` is `PALETTE_NITE`, not dark. Its only world
exit returns to `MnR01C00R00`; its other destination is the lower gallery.
`MnDriftlessLampGallery` uses `dark` and its only non-self return leads back to
the hollow. The existing gallery plan describes it as optional, with a fossil
clue and cache, not a campaign gate. The candidate's R01 map attributes link
east to Winona and west to Rochester. The historical earned New Game route in
[the Rochester chapter](../playable/rochester.md) records walking this outdoor
route without a badge; it used an older pack and does not prove the current
candidate's traversal. The independently reported cave topology issue remains
open under tasks 13 and 21.

A scan of all `Mn` map attributes in this candidate found the gallery as the
only map whose palette is `dark` or `PALETTE_DARK` (case-insensitive). The
maps marked `CAVE` besides the gallery use `PALETTE_NITE`: `MnInnerBasin`,
`MnIronDrillApproach`, `MnR01Cave1F`, `MnR03Cave1F`, `MnR05RockCut`,
`MnR09Cave1F`, `MnR10Cave1F`, `MnR12Cave1F`, `MnR13Cave1F`,
`MnR17Cave1F`, `MnR20Cave1F`, `MnVoyageursIce1F`, and `MnVoyageursIceB1F`.
This is an attribute inventory, not a proof of navigability, renderer output,
or complete earned progression. It also does not cover later pack changes.

Before closing task 22, inspect the actual release lineage and play the current
opening through the Rochester badge, then return to the guide and teach/use
Flash with a compatible Minnesota Pokémon. Verify the outdoor route and any
new or revised connecting caves with ordinary inputs, inspect the dark and lit
rendering, and repeat the palette and topology inventory on the final pack.
Do not treat historical fixtures or this candidate as an earned campaign.

A repeatable pack-level regression is available with
`uv run --offline --with cbor2 python planning/minnesota/tools/check_flash_progression.py PACK`.
It checks Minnesota map palette and both native darkness selectors, rejects any
new dark regional map or a new gallery exit until reviewed, checks the surface
map links and Flash gift prerequisites, and inventories compatible R01 grass
encounters. On the candidate above it checked 355 regional maps. STRIPILLAR is
in ordinary grass by day and morning; TESSLARV is in the Driftless Seep habitat
at night. The test does not demonstrate access to that habitat or a capture.
Deliberately changing the hollow's palette to dark and adding a Rochester gallery
exit each caused the guard to fail. The guard is a manual check; it is not yet
wired into pack publication or CI.

On this same candidate, the existing prebuilt native controller test
`fossil_gallery_controller_hm_gift_teaching_held_fossils_reward_and_original_return`
passed with a declared Care Badge and compatible partner. The companion
`fossil_gallery_controller_dark_badge_denial_both_stairs_and_passive_pair`
passed with a declared partner that knew Flash but no badge. These establish the
local gift, teaching, illumination, denial, and return mechanics; they do not
earn the badge or prove the full opening. The older
`native_field_gallery_teaches_flash_holds_acknowledgment_and_preserves_reward`
failed at its obsolete exit coordinates, which do not match the redesigned
12-by-10 gallery. The newer tests above use its current source contract. The
prebuilt test executable predates subsequent working-tree changes.

The existing prebuilt `rochester_earned_starter_route_trainers_first_badge_and_continue`
controller test also passed against this candidate. It begins at New Game,
travels the surface R01 route, completes delivery, earns the route and gym
battles and the Care Badge, and continues from save. This is an earned badge
on the candidate, but it does not continue the same save to the Flash gift or
prove the final composed release. The local gallery tests above independently
cover the gift and use with declared prior state.

The audit has a synthetic regression suite in `tools/test_flash_progression.py`.
Run it from the repository root with:

```sh
uv run --offline --with cbor2 python -m unittest discover -s planning/minnesota/tools -p test_flash_progression.py
```

The suite and the added requirement that R01 has all three grass encounter
periods were authored after the checks recorded above and have not yet been
run. The guard deliberately fails when hollow or gallery connections change;
review it alongside the concurrent cave redesign, and update its topology
contract only after verifying the resulting passage and earned progression.
