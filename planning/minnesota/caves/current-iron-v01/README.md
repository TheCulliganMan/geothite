# R10 rock gallery on the current Minnesota lineage

The active compact-coast/waterfront candidate now contains the complete revised
R10 cave: a pinched upper gallery, split rock spine, stepped spring bank and
separate Revive shelf. Native stone fills the old black ceiling/border gaps.
The original reader, reward, stairs, short drop and same exterior mouth remain.
This integrates the existing hand-authored Johto-quality source without
replacing the whole candidate with its older, separately published cave pack.

- [Whole cave before and after](../../renders/iron-cave-quality-v01/verified-v01/whole-before-after.png)
- [Whole published cave](../../renders/iron-cave-quality-v01/published-v02/MnR10Cave1F-whole.png)
- [Nine actual native cameras](../../renders/iron-cave-quality-v01/native-v01/MnR10Cave1F-production-cameras.png)
- Reached frames: [upper gallery](../../renders/iron-cave-quality-v01/native-reached-v01/pinched-gallery.png), [spring bank](../../renders/iron-cave-quality-v01/native-reached-v01/spring-bank.png), [widest entrance fork](../../renders/iron-cave-quality-v01/native-reached-v01/widest-fork.png).
- [Verified receipt](../../renders/iron-cave-quality-v01/verified-v01/receipt.json)
- [Production stdio client check](../../renders/iron-cave-quality-v01/mcp-v01/receipt.json)

The whole room, source camera sheet, native camera sheet and three reached
frames were visually inspected. The entrance fork keeps useful passing room;
the upper corridor has close rock shoulders rather than an empty broad lane.
The lower bank bends around the spring and rock nose. The unchanged two-lane
drop and ordinary returns create different outward/return movement. This is
one cave improvement, not completion of every regional cave or map.

## Native contract

The ignored output is `content-packs/minnesota-r10-rock-gallery-v01.crystalpack`,
SHA256 `a69aa145e6ea870b459d79859958806170b56036af4f85e0c1e007629a2f67d6`,
identity `4d826107eede5bd551d45987dbd107460a474e152b6a08707974047372a07168`.
It is a child of the waterfront pack, not the older eleven-cave branch.

The room remains18×14 metatiles, using unchanged native cave art and forced
NITE. Exactly139 metatiles change and the repeating border changes from01 to
solid native rock2e. All40 original water quadrants remain;24 explicit new
water quadrants extend the spring's stepped head, for64 total. Two old wet
metatiles change to fit the new water join. The complete outer perimeter is
blocking rock; no new water reaches a map edge.

All690 other native maps and all actor/event/program/warp data remain exact.
The only data changes are this room's blocks, border and their encoded/attribute
mirrors, plus the derived pack identity. Runtime art, palettes, species,
encounters, fishing, audio, reports, HOME and recovery data remain exact.
Thus the compact North Shore/Grand Marais geography, snowy pine Ely, back
route, waterfront fronts and parent river geometry are retained. The parent's
river replay is evidence on its original bound pack; it is not relabeled as a
new earned journey on this child.

| Retained gameplay contract | 16px core coordinates |
| --- | --- |
| Original mouth / clear indoor arrival | (3,13) / (3,12) |
| Exterior active entrance / passive return | MnR10C00R00(36,17) / (36,18) |
| Reader, facing east | (9,5) |
| Revive×1, original flag EVENT_MN_R10_ITEM_04 | (29,23) |
| West upper stair lane | x7, y7–10 |
| East two stair lanes | x20, y5–8 and y7–10 |
| West lower stair lane | x7, y13–16 |
| Two downward lips / landings | (12/13,6) → (12/13,8), across blocked y7 |
| Ordinary uphill returns | 13 and15 native walking steps |

The source is [../johto-quality-v01/MnR10Cave1F.json](../johto-quality-v01/MnR10Cave1F.json),
SHA256 `edd3c9f12223cae55aa59ea8e6a69d610522fa13fa2728ffc2374c895c6b46cd`.
Its exact prior room is [the previous source](../johto-quality-v01/MnR10Cave1F.previous.json),
SHA256 `f9cbd7beede8b38d9957667c6cc247198a5df3a7ee6f58f655e6f37af73a93a0`.
The current R10 route dossier already has these rows. Only an explicit native
revision link is added to the dossier; no other route geometry is promoted.

## Verification and iteration

Four production-controller cases pass: every declared camera/four stairs/
both lips/reverse denial/returns/mouth/Continue; the retained original traversal;
fully printed reader pages and Revive once-only persistence; full-bag refusal,
native Potion use and pickup retry. Nine source-to-production comparisons
have zero differing pixels across3,317,760 pixels, including coherent native
spring animation. Three additional captures load actually reached saves without
position, facing or clock overrides. The nine-camera checks are explicitly
positioned visual fixtures; neither fixture type earns regional access.

The current Rust publisher reproduces the same file byte for byte. It rejects
a walkable border, unpaired ladder and changed Revive quantity before writing
any output. Complete normalized native equality and all377 audio PCM/hash/
frame/loop checks pass. See [PUBLICATION.md](PUBLICATION.md) for commands and
the retained rejection/check limits.

The production stdio client also loads the actual reached entrance-fork save,
opens the real Start/Pack menu and walks left/back through native movement.
It exits successfully with byte-empty stderr.

Both source and native review tools now accept `--room MnR10Cave1F`; their
default remains all nine caves. The measured warm full-room/source-camera loop
takes0.171 seconds after loading the pack. Use a fresh output directory for
every edit/review. All original renders, comparisons, animation phases, earlier
failed attempts and reached checkpoints are retained.

The other eight Johto-quality cave sources remain a separate integration task
on this lineage, including R09's prerequisite-checked foreman/cache story.
Wider city density, Boundary Waters activities, rail/fairgrounds and remaining
regional requirements also remain active. Final3D and an earned campaign are
not claimed.
