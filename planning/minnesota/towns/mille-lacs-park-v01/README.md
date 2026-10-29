# Mille Lacs park

A real 24×20-metatile outdoor destination now closes the missing eastern end of
the Mille Lacs shore. Its winding wooded walk reaches a grassy bird lookout,
short downward ledge return, railed fishing dock, sheltered picnic bend and
public lake launch. An offshore rocky roost holds one PP Up. All five residents
have reachable native conversations. There are no decorative building doors.

The old R18 Shore Watch remains the sole Good Rod authority. Its volunteers,
reports, badge condition, ranger, rewards and all existing route events remain
exact. The park supplies no rod, story result, badge, train ride or healing
service. Its grass and water inherit the current regional R18 encounter tables;
Gillpup, Flossail, Rivvit, Notchbill, Ticklet and other active Minnesota species
remain native encounters. No original species, art or audio is replaced.

- [Complete park](../../renders/mille-lacs-park-v01/published-v06/whole-park.png)
- [Seven actual native cameras](../../renders/mille-lacs-park-v01/native-v04/production-decisions.png)
- [Current connected outdoor atlas](../../renders/mille-lacs-park-v01/atlas-v04/full-state-native-current.png)
- [Independent whole-pack preservation/source audit](../../renders/mille-lacs-park-v01/published-v06/receipt.json)
- [Production text-client receipt](../../renders/mille-lacs-park-v01/mcp-v04/receipt.json)

## Authoritative source and traversal

[town.json](town.json) owns actors, dialogue, signs, the pickup and named review
positions. [layout.json](layout.json) contains explicit editor strokes and
native block placements. [native.contract.json](native.contract.json) binds
both sources and the exact reviewed R09 parent/native R18 preimage. All coordinates
below are 16px gameplay tiles. Layout positions are 32px metatiles instead.

| Feature | Exact contract |
| --- | --- |
| R18 road | MnR18C01R00east(15,10/11) ↔ MnMilleLacswest(0,12/13); offsets−1/+1 metatiles |
| Clear park arrival | (2,12); both lanes join the same winding path |
| Entrance board | (2,10), read from(2,11) |
| Park ranger | (7,13), approach(6,13); directions to the existing western Shore Watch |
| Bird lookout | Birder(17,16), approach(16,16); at least one native grass step, with no dry/Surf bypass |
| Short drop | Approaches(16/17,17), HOP_DOWNlip(16/17,18), solid course(16/17,19), landings(16/17,20) |
| Ordinary return | Landing → western path → grass neck → upper approach; at least 9 ordinary walking steps |
| Dock | Clear passing row y22 across x18..27; upper rail y20, lower rail y25, closed end x28..29 |
| Angler | (27,23), approach(26,23); actual native fishing remains available on the lake |
| Picnic stop | Visitor(11,30), approach(12,30) |
| North shore | Boater(39,9), approach(39,10); wind board(24,6), approach(24,7) |
| Surf launch | Floor(19,29) → Water(20,29), using the existing native move/badge prompt |
| Lake course | (20,29) → (20,32) → (38,32) → (38,29) → Floor(37,29) |
| Roost reward | PP_UP×1 at(36,29), approach(37,29), `EVENT_MN_MILLE_LACS_ROOST` |
| Lake boundary | Eastern buoy column x45, southern buoy row y37, solid outer woodland; no unbound Water edge |

Both drop lanes have blocked reverse and side routes. The ordinary uphill
return remains available. The player can see the lake from the lookout and
both shore destinations in actual native cameras. The lake, dock and separate
landing support exploration; they are not unconnected decoration. The roost
cannot be reached through ordinary walking. All other public ground remains
reachable without Surf, with residents treated as obstacles.

## Verified native behavior

The verified 705-map publication is
`content-packs/minnesota-mille-lacs-park-v04.crystalpack`, SHA256
`6d5e7a486adc7508c8adf42d43543a02c9a1d279c6521e01e0c4b3a94490ecaa`.
It adds exactly one map and its native registry/scripts/spawn/independent event.
The only old-map change is R18's east connection and mirrored attributes:
all 704 old maps retain exact terrain, actors and programs; 703 complete old
modules remain exact. All original art/runtime files, audio, trainer parties,
species, inventory definitions, encounter tables, HOME and recovery data stay
unchanged. The park uses dynamically allocated map group 101/map 1.

Three production-controller cases pass in 10.84 seconds: both road lanes and
directions; all five conversations and both signs with input ownership; both
ledge lanes/reverse blocks/ordinary returns; dock barriers; a real DEWDANT wild
encounter and native RUN; Surf launch/paddle/dismount; actual PP Up quantity/event,
once-only pickup/body removal; and Save/Continue. The water fixture uses a legal
Gillpup at level 60 with Surf and Birchit/Pouchid reserves at level 30 plus an explicit initial
Fog Badge. No progression, party, position, RNG or resource writes occur after
setup. Empty-party walking is a separate explicit local fixture. These cases
do not earn regional access or a full campaign.

Two asset cases pass in 11.22 seconds, including deterministic publication,
complete native preservation, duplicate/stale-source rejection and four unsafe
editor variants: missing dock rail, open water edge, grass bypass and open dock
end. The native item-ball binding is checked directly. All 377 audio PCM hashes,
frame counts and loop ranges pass. A fresh guarded replay is byte-identical.

All seven complete 640×576 native frames match source pixels exactly, including
the actual R18neighbor at the entrance and one coherent water animation phase.
The complete park, all native cameras and reached captures are inspected.
Four actual stdio processes pass with byte-empty stderr: roads/residents/ledge/
dock/Start and Continue, plus the native PP Up notice, collected tile and
once-only Continue. Quantity/event and full paddle effects are separately
controller-asserted, rather than inferred from text output.

The final atlas has 98 outdoor maps in one connected seam component, with zero
overlaps or inconsistent placement cycles. Detroit Lakes, Moorhead and
International Falls remain absent from this selected map lineage; their
separate authored branches are not silently imported.

`world.json` records the accepted `mille_lacs_park_revision`, including native
group 101/map 1/spawn 282, the actual road pairing and all proof references.
The former gatehouse-warp reservation is replaced by the verified direct seam.
The R10 terrain handoff consumes this exact accepted park payload; later
selected descendants must preserve this revision record and the park itself.
The newly selected R10 child was independently read back: only12terrain cells
in `MnR10C00R00`, its encoded mirror and identity differ. The complete park,
R18 road, split payloads, all programs, roster, art and audio remain exact.
Its [preservation receipt](../../renders/mille-lacs-park-v01/r10-descendant-v01/receipt.json)
keeps that check separate from the original park's gameplay/render bindings.

## Fast review loop and retained revisions

```sh
PYTHONPATH="$PWD/.local/ely-backroute-python" python3 \
  planning/minnesota/towns/mille-lacs-park-v01/review.py \
  --pack content-packs/minnesota-mille-lacs-park-v04.crystalpack --published \
  --output planning/minnesota/renders/mille-lacs-park-v01/NEW_FRESH_EPOCH
```

The measured warm source render/check takes 0.309 seconds after pack loading.
The loop never publishes a pack or regenerates terrain. It rejects mismatched
source hashes, disconnected ground, open boundaries, incidental Cut quadrants,
grass bypasses and unowned native changes. Rust publishes; production input
proves play. See [PUBLICATION.md](PUBLICATION.md) for actual commands.

Every original, crop, phase comparison, rejected render and failed log is kept.
`source-v01` retains the overly open first lawn; `source-v02` the tighter woods
before closing two isolated ground patches; `source-v03` freezes the initial
R03parent; `source-v04` binds accepted R09; `source-v05` fixes the roost camera
from its boulder to actual Floor(37,29). The camera correction changes no native
game bytes. Park v03 and v04 packs are byte-identical.

The first play check exposed the omitted typed item-ball handler. The publisher
now parses and requires it, and actual pickup tests pass. The first ledge helper
incorrectly rejected the normal intermediate lip; the corrected course allows
the lip, rejects standing on its wall and requires the real landing. The grass
helper now advances native presentation frames after every battle input. These
are recorded repairs, not skipped checks. No engine rule was weakened.

Broader Minnesota map quality, missing hubs, passenger trains, Safari capture
rules, full earned campaign progression and final 3D acceptance remain separate
unfinished work.
