# Minnesota fishing guru houses

## Geography status: historical lineage withdrawn

`minnesota-field-progression-v03.crystalpack` and its
`minnesota-interiors-fishing-life-v02.crystalpack` parent retain obsolete northern
geometry: R11's terminal is 24×24 rather than 4×24, and the four retired Gunflint
chunks remain. They are historical verification artifacts, not current build
parents or recommended playable outputs. Do not branch another pack from them.

New fishing/HM publication must consume the canonical compact geography in
[the Superior/Gunflint verification](../revisions/superior-gunflint-quality-v01/VERIFICATION.md).
The canonical reference is `minnesota-superior-compact-v03.crystalpack` (689 maps).
Rebasing the furnished homes is coordinated with the interior owner; shared
geometry belongs to the map owner. No replacement fishing pack has been
published by this thread. The gameplay/visual receipts below are historical
and do not establish compact-lineage acceptance.

Crystal's pinned native sources were reviewed: `Route32Pokecenter1F.asm`,
`OlivineGoodRodHouse.asm` and `Route12SuperRodHouse.asm`. Minnesota uses three
route gurus and brings the Super Rod forward to the road before Brainerd's
sixth gym and the northern lakes.

| Rod | House and stage | Exterior door | Safe return |
| --- | --- | --- | --- |
| Old | Driftless Trail, lower creek bank, first route | `MnR01C00R00` (25,33) | (25,34) |
| Good | River Valley, middle crossing, before Minneapolis | `MnR03C00R00` (31,29) | (31,30) |
| Super | Pine Country Road, above the lake deck, after St Cloud/before Brainerd | `MnR06C00R00` (37,39) | (37,40) |

The Rust publisher reads [fishing-gurus.json](../fishing-gurus.json). Every
facade uses the complete native Johto house drawing, porch and sign: six
explicit metatile edits. Original rivers, waterfalls, ledges, caves, actors and
events are protected. Each porch connects to both main route-corridor edges
on foot without an HM. Both interior exit carpets return to passive dry floor;
actual entry, exits and re-entry passed through the production controller.

The three gurus give free, independent, repeat-safe gifts, including retry
after a full Key Items pocket. Owning a stronger rod does not block an earlier
one. Old/Good tutorials identify the next guru. The existing Mille Lacs shore
watch quest retains its original ownership-aware Good Rod reward.

All 27 installed Minnesota FISHER trainers give guru-house directions after
battle and in their referenced repeat text. Original dialogue remains first;
trainer roots, loss/seen text, defeat flags, callbacks and rewards are preserved.
Existing route anglers and the first creek child also advertise the houses.
All 62 members of the touched fisherman parties belong to the current active
original Minnesota roster, with legal moves and retained levels/DVs/items.

## Verification

The historical v03 output passed all three rod controller cases and all five HM
controller cases. These cover every house, both exits, re-entry, free one-time
gifts, repeats, Continue, full-pocket retry, independent rods, and an actual
fisherman battle followed by Super Rod directions. Position/party/badge/pocket
setup is explicit; this does not claim an earned full campaign.

Both publication-preservation regressions passed. That historical HM composition
preserves the furnished guru maps byte-for-byte, all unrelated data sections,
source assets/audio, field permissions and original recovery/home spawns.
The production stdio MCP executable loaded v03, moved the authoritative Winona
position, exposed PACK/SAVE through Start, and kept stderr byte-for-byte empty.

Native 2D remains the composition reference. The accepted furnished room frames
are in `renders/interior-life/fishing-houses/native-2d-v02/`; their parent hash
was verified and the rooms remain identical in v03. Additional paired exterior
3D review passed at the same position and noon: Driftless `current-v03`, River
Valley and Pine Country `current-v04` under `renders/fishing-rods/`.
The Headwaters tileset's unchanged Johto houses/signs now select native volume
models while retaining their real atlas. Three model regressions and the native
sign-profile regression pass. Earlier flat/timeout captures remain historical.

Workspace formatting still fails in pre-existing `offset-allocator` code.
Required Clippy attempts encounter existing core `bad_bit_mask`, asset
`never_loop`, voxel `approx_constant`, and test `erasing_op` failures. No
warnings were introduced in the new rod/HM/Fly files; unrelated failures were
left unchanged. The actual commands and final pack hash are in the HM runbook.
