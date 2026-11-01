# Native R12 / inland connector dependency inventory

Inspected native parent: `content-packs/minnesota-superior-inland-winter-v01.crystalpack`; file SHA256 `65cf6f98e65ada839ec5afaa57728bf8289d3a50ba53ba6b9b50f08d3620bb17`.
Content identity: `f34280a5fc5d6ab8ccc3ec4ef4317a6ea9321070df342ea95aca5745514a0223`. No pack writes, source promotions or Cargo runs are performed by this inspection.

This is a bounded relocation planning inventory, not a replacement program catalog. Coordinates below are 16px gameplay tiles unless dimensions explicitly say metatiles. Hashes use sorted-key compact JSON of each existing native module. Geometry and retained/retired choices are not approved by this inventory.

## Native identities and preimages

| Map | Constant | Group:map | Metatile extent | Tileset | Whole module SHA256 |
| --- | --- | --- | --- | --- | --- |
| MnR12C00R00 | MN_R12_C00R00 | 42:1 | 32×24 | mn_winter_gunflint | 28b7690d3297c5cff8463db0301d0cdc3d58588dabd121d4abf7bef307b4aae9 |
| MnR12C00R01 | MN_R12_C00R01 | 42:3 | 32×24 | mn_winter_gunflint | ae6f6bc02b551d3f989a35eb750dcd1fdd8ff0d5cda48663199ac25f2a60b99c |
| MnR12C00R02 | MN_R12_C00R02 | 42:5 | 32×8 | mn_winter_gunflint | 6f3ce69ff5334f2f7e992c5ef0cc608779395daad0ab411b117a15b30e2dbac0 |
| MnR12C01R00 | MN_R12_C01R00 | 42:2 | 32×24 | mn_winter_gunflint | dfa0aae404ec4667e2a0547281a1abec0ab666f03ec6d9084d3023a9ddf83d4f |
| MnR12C01R01 | MN_R12_C01R01 | 42:4 | 32×24 | mn_winter_gunflint | 7e4394b1e157785b4d0c7ea1067843763f34cbeb8f45d2220f81806c3e23c52e |
| MnR12C01R02 | MN_R12_C01R02 | 42:6 | 32×8 | mn_winter_gunflint | cc2fb80bbe3781fbbf9d5f1fddbdf040085fcee083c211f2024e53c2b73944ab |
| MnR12Cave1F | MN_R12_CAVE_1F | 42:7 | 16×12 | cave | a9fcdb81bd71e0e4c2865661f24df8afa5a7cea247b4548c0bbaea28afca4dd1 |
| MnR12RangerHut | MN_R12_RANGER_HUT | 42:8 | 6×5 | house | cb3f9ab5884290c10e72b5ee06df55b1e40d8099196cee51b7a8dc04edf90c86 |
| MnMaraisInlandNeck | MN_MARAIS_INLAND_NECK | 40:8 | 65×19 | mn_winter_north_shore | 6881f92cbadafc1925acc60d068f47d08fe64882dfb089faa208c9ebbac2042a |
| MnMaraisCedarApproach | MN_MARAIS_CEDAR_APPROACH | 83:2 | 32×19 | mn_winter_north_shore | 719cca8740762261a0b1b5f1eda919ebf8aceff9460a749c19428cf0a668ddc4 |

## Every incoming and outgoing native connection

| Source | Direction | Target | Offset |
| --- | --- | --- | --- |
| MnBoundaryWaters | east | MnR12C00R01 | 8 |
| MnGrandMarais | north | MnMaraisCedarApproach | -19 |
| MnMaraisCedarApproach | west | MnMaraisInlandNeck | 0 |
| MnMaraisCedarApproach | south | MnGrandMarais | 19 |
| MnMaraisInlandNeck | west | MnR12C01R01 | 0 |
| MnMaraisInlandNeck | east | MnMaraisCedarApproach | 0 |
| MnR12C00R00 | south | MnR12C00R01 | 0 |
| MnR12C00R00 | east | MnR12C01R00 | 0 |
| MnR12C00R01 | north | MnR12C00R00 | 0 |
| MnR12C00R01 | south | MnR12C00R02 | 0 |
| MnR12C00R01 | east | MnR12C01R01 | 0 |
| MnR12C00R01 | west | MnBoundaryWaters | -8 |
| MnR12C00R02 | north | MnR12C00R01 | 0 |
| MnR12C00R02 | east | MnR12C01R02 | 0 |
| MnR12C01R00 | south | MnR12C01R01 | 0 |
| MnR12C01R00 | west | MnR12C00R00 | 0 |
| MnR12C01R01 | north | MnR12C01R00 | 0 |
| MnR12C01R01 | south | MnR12C01R02 | 0 |
| MnR12C01R01 | west | MnR12C00R01 | 0 |
| MnR12C01R01 | east | MnMaraisInlandNeck | 0 |
| MnR12C01R02 | north | MnR12C01R01 | 0 |
| MnR12C01R02 | west | MnR12C00R02 | 0 |

## Every incoming and outgoing native warp

| Source | Warp index | Local tile | Target | Target warp |
| --- | --- | --- | --- | --- |
| MnR12C01R00 | 1 | (2,23) | MnR12Cave1F | 1 |
| MnR12C01R01 | 1 | (21,35) | MnR12RangerHut | 1 |
| MnR12Cave1F | 1 | (17,23) | MnR12C01R00 | 1 |
| MnR12RangerHut | 1 | (4,9) | MnR12C01R01 | 1 |
| MnR12RangerHut | 2 | (5,9) | MnR12C01R01 | 1 |

The cave exterior mouth currently belongs to `MnR12C01R00` (global R12 `(66,23)`), with native passive dry return `(66,24)`. The hut mouth belongs to `MnR12C01R01` (global `(85,83)`), with native passive return `(85,84)`. Both hut carpets return to the same exterior warp index 1. Moving either mouth requires rebinding its existing room return(s) to the new owning map and exterior warp index; the interior identities and programs can remain unchanged.

## Spawns, recovery and non-map registries

Runtime spawn/recovery entries targeting these maps: **0**. The native spawn catalog was checked by both map name and map constant. No spawn identifier needs migration for these maps in this exact parent.

Additional exact map-name/constant references in title, Fly, playability, field-box items, story, phone/global scripts and special routines:

None in those registries.

## Script map references and coordinate-bearing instructions

| Source path | Map reference |
| --- | --- |
| global split map_scripts/MnR12C01R00_MapEvents/2/args/2 | MN_R12_CAVE_1F |
| global split map_scripts/MnR12C01R01_MapEvents/2/args/2 | MN_R12_RANGER_HUT |
| global split map_scripts/MnR12Cave1F_MapEvents/2/args/2 | MN_R12_C01R00 |
| global split map_scripts/MnR12RangerHut_MapEvents/2/args/2 | MN_R12_C01R01 |
| global split map_scripts/MnR12RangerHut_MapEvents/3/args/2 | MN_R12_C01R01 |

Coordinate-bearing program instruction/table scan (event-header coordinates are inventoried separately below):

No coordinate-bearing script instructions or nonempty typed map/object/block/movement tables occur in the ten inventoried modules. There are no map scenes or coordinate events in these modules. Terrain interactions such as Cut use native collision, not a bespoke coordinate program.

## Existing actors, encounter flags and pickup identities

| Owner | Actor/script identity | Local tile | Whole R12 tile | Facing/movement | Activity | Event flag / native number |
| --- | --- | --- | --- | --- | --- | --- |
| MnR12C00R00 | MN_R12_T05 | (53,41) | (53,41) | SPRITEMOVEDATA_STANDING_RIGHT | PICNICKER/MN_R12_T05 | EVENT_MN_R12_TRAINER_05=2146 |
| MnR12C00R00 | MN_R12_T10 | (35,13) | (35,13) | SPRITEMOVEDATA_STANDING_LEFT | COOLTRAINERF/MN_R12_T10 | EVENT_MN_R12_TRAINER_10=2151 |
| MnR12C00R00 | MN_R12_N03 | (53,39) | (53,39) | SPRITEMOVEDATA_STANDING_UP | existing dialogue/service | -1=not global / no flag |
| MnR12C00R00 | MN_R12_N06 | (11,39) | (11,39) | SPRITEMOVEDATA_STANDING_LEFT | existing dialogue/service | -1=not global / no flag |
| MnR12C00R00 | MN_R12_ITEM_01 | (53,19) | (53,19) | SPRITEMOVEDATA_STANDING_DOWN | HYPER_POTION×1 | EVENT_MN_R12_ITEM_01=2152 |
| MnR12C00R00 | MN_R12_ITEM_04 | (45,17) | (45,17) | SPRITEMOVEDATA_STANDING_DOWN | REVIVE×1 | EVENT_MN_R12_ITEM_04=2155 |
| MnR12C00R01 | MN_R12_T08 | (37,31) | (37,79) | SPRITEMOVEDATA_STANDING_DOWN | FISHER/MN_R12_T08 | EVENT_MN_R12_TRAINER_08=2149 |
| MnR12C00R01 | MN_R12_T09 | (55,35) | (55,83) | SPRITEMOVEDATA_STANDING_LEFT | CAMPER/MN_R12_T09 | EVENT_MN_R12_TRAINER_09=2150 |
| MnR12C00R02 | MN_R12_N04 | (43,1) | (43,97) | SPRITEMOVEDATA_STANDING_RIGHT | existing dialogue/service | -1=not global / no flag |
| MnR12C00R02 | MN_R12_ITEM_03 | (45,1) | (45,97) | SPRITEMOVEDATA_STANDING_DOWN | MAX_REPEL×1 | EVENT_MN_R12_ITEM_03=2154 |
| MnR12C01R00 | MN_R12_T03 | (19,43) | (83,43) | SPRITEMOVEDATA_STANDING_RIGHT | COOLTRAINERF/MN_R12_T03 | EVENT_MN_R12_TRAINER_03=2144 |
| MnR12C01R00 | MN_R12_T04 | (7,37) | (71,37) | SPRITEMOVEDATA_STANDING_DOWN | HIKER/MN_R12_T04 | EVENT_MN_R12_TRAINER_04=2145 |
| MnR12C01R00 | MN_R12_N02 | (37,13) | (101,13) | SPRITEMOVEDATA_STANDING_DOWN | existing dialogue/service | -1=not global / no flag |
| MnR12C01R00 | MN_R12_ITEM_02 | (49,13) | (113,13) | SPRITEMOVEDATA_STANDING_DOWN | ULTRA_BALL×1 | EVENT_MN_R12_ITEM_02=2153 |
| MnR12C01R01 | MN_R12_T01 | (47,35) | (111,83) | SPRITEMOVEDATA_STANDING_UP | HIKER/MN_R12_T01 | EVENT_MN_R12_TRAINER_01=2142 |
| MnR12C01R01 | MN_R12_T02 | (43,3) | (107,51) | SPRITEMOVEDATA_STANDING_RIGHT | BIRD_KEEPER/MN_R12_T02 | EVENT_MN_R12_TRAINER_02=2143 |
| MnR12C01R01 | MN_R12_T06 | (9,33) | (73,81) | SPRITEMOVEDATA_STANDING_RIGHT | COOLTRAINERF/MN_R12_T06 | EVENT_MN_R12_TRAINER_06=2147 |
| MnR12C01R01 | MN_R12_N01 | (53,9) | (117,57) | SPRITEMOVEDATA_STANDING_LEFT | existing dialogue/service | -1=not global / no flag |
| MnR12C01R01 | MN_R12_N05 | (45,29) | (109,77) | SPRITEMOVEDATA_STANDING_LEFT | existing dialogue/service | -1=not global / no flag |
| MnR12C01R02 | MN_R12_T07 | (23,3) | (87,99) | SPRITEMOVEDATA_STANDING_LEFT | BUG_CATCHER/MN_R12_T07 | EVENT_MN_R12_TRAINER_07=2148 |
| MnR12Cave1F | MN_R12_OTTA | (5,5) | not R12 | SPRITEMOVEDATA_STANDING_RIGHT | existing dialogue/service | -1=not global / no flag |
| MnR12Cave1F | MN_R12_CAVE_CACHE | (27,5) | not R12 | SPRITEMOVEDATA_STANDING_DOWN | MAX_ETHER×1 | EVENT_MN_R12_CAVE_ETHER=2156 |
| MnR12RangerHut | MN_R12_JUNE | (7,3) | not R12 | SPRITEMOVEDATA_STANDING_DOWN | existing dialogue/service | -1=not global / no flag |
| MnMaraisInlandNeck | MN_INLAND_CAMP_SUPPLY | (63,27) | not R12 | SPRITEMOVEDATA_STILL | BERRY×3 | EVENT_MN_GUNFLINT_CAMP_SUPPLY=2387 |
| MnMaraisCedarApproach | MN_COHESION_MARAIS_KEEPER | (29,19) | not R12 | SPRITEMOVEDATA_STANDING_DOWN | BIRD_KEEPER/MN_COHESION_MARAIS_KEEPER | EVENT_MN_COHESION_MARAIS_KEEPER_DEFEATED=2380 |
| MnMaraisCedarApproach | MN_COHESION_MARAIS_WALKER | (53,31) | not R12 | SPRITEMOVEDATA_STANDING_DOWN | LASS/MN_COHESION_MARAIS_WALKER | EVENT_MN_COHESION_MARAIS_WALKER_DEFEATED=2381 |

## Background signs and text-program guards

| Owner | Sign script | Local tile | Native text label(s) |
| --- | --- | --- | --- |
| MnR12C00R01 | MN_R12_SIGN_05 | (35,27) | MN_R12_SIGN_05Text |
| MnR12C00R01 | MN_R12_SIGN_06 | (7,7) | MN_R12_SIGN_06Text |
| MnR12C01R00 | MN_R12_SIGN_02 | (17,25) | MN_R12_SIGN_02Text |
| MnR12C01R00 | MN_R12_SIGN_03 | (5,25) | MN_R12_SIGN_03Text |
| MnR12C01R01 | MN_R12_SIGN_01 | (55,5) | MN_R12_SIGN_01Text |
| MnR12C01R01 | MN_R12_SIGN_04 | (17,39) | MN_R12_SIGN_04Text |
| MnR12RangerHut | MN_R12_HUT_MAP | (4,0) | MN_R12_HUT_MAPText |
| MnMaraisInlandNeck | MN_INLAND_CAMP_SIGN | (69,25) | MN_INLAND_CAMP_SIGNText |
| MnMaraisCedarApproach | MN_COHESION_MARAIS_SIGN | (7,9) | MN_COHESION_MARAIS_SIGNText |

Program tables to preserve/move with their owning actors (hashes certify content without exporting a duplicate command catalog):

| Owner | Installed content | Retained program tables SHA256 |
| --- | --- | --- |
| MnR12C00R00 | 16 script labels; 2 trainers; 8 text bodies | 5ce50eb8897fe7624963fca492139a94a7a8f10630d48922d99a4ecb38f15216 |
| MnR12C00R01 | 14 script labels; 2 trainers; 8 text bodies | a6f1bcbc5ef9858b5b62656c87f11b2ddea5a4f83c767913b817b1ffaf8d3a1c |
| MnR12C00R02 | 3 script labels; 0 trainers; 1 text bodies | 5aeafc642b4d04713c2b2b64ba790e11ff1022f87776226206564dd17dffd0c4 |
| MnR12C01R00 | 17 script labels; 2 trainers; 9 text bodies | 88c9e6bb88a50de041ba559ef388a54b1e35eb2317f51f62a57bf37d982cedc1 |
| MnR12C01R01 | 23 script labels; 3 trainers; 13 text bodies | 5eff0129b869fe647fbac2b74c6cb5bcfdb1e734b8953d2f9e0de0738ff701f0 |
| MnR12C01R02 | 5 script labels; 1 trainers; 3 text bodies | 6ae171987de3499964000a477d2a6ae80ce45ff429c38e8af256e4ba958b61b5 |
| MnR12Cave1F | 3 script labels; 0 trainers; 1 text bodies | 2ac7a560829ccf1c0f1a5b76565407c225deee330d63cbfb6301beb44de52b5e |
| MnR12RangerHut | 4 script labels; 0 trainers; 2 text bodies | 78c4347eb8b1b6b320c455317fc92e327cdcbe53b4f04e1128f04677261e08a3 |
| MnMaraisInlandNeck | 3 script labels; 0 trainers; 1 text bodies | fab2e9ce7cd1d44dc13d55b6d5a39c1bc200527b258a595409f2a3b6ccec36ce |
| MnMaraisCedarApproach | 12 script labels; 2 trainers; 7 text bodies | 2adbd70112bdd40f9baf2635b14794f166f80e450b07baec147fc0d042614a59 |

## Feasible chunk retirement, without choosing geometry

The current R12 footprint is 64×56 metatiles: six exterior chunks. Rooms `MnR12Cave1F` (42:7) and `MnR12RangerHut` (42:8) remain separately installed and need no internal resizing.

- A 32×32 route using the established 32×24 clipping grid has two exterior chunks: retain `MnR12C00R00` (42:1), resize `MnR12C00R01` (42:3) to 32×8. Retire four exterior chunks: `C01R00`, `C01R01`, `C00R02`, `C01R02`. Existing actors/program labels/flags from retired chunks must migrate to explicit legal new anchors. Both room returns currently target retired right-column chunks and must be rebound.
- A 32×24 route has one exterior chunk: retain `MnR12C00R00` (42:1), retire the other five. Keeping two exterior chunks at that extent would require a different deliberately specified clipping scheme; it is not the existing 32×24 chunk contract.
- Retiring `MnMaraisInlandNeck` is an additional retirement, independent of the R12 chunk count. Its BERRY×3 flag2387 pickup and campsite sign are existing content. Root must explicitly choose relocation or retirement of that activity; removing a map must not silently discard its reward or reset its flag.
- Cedar (83:2) and Neck (40:8) each have distinct existing numeric identities. Neither a proposed direct port nor a revised dimension is frozen by this inventory.

For each retired map, remove its uniquely owned cached module, raw attributes/dimensions, metadata, blocks, NPC wire entry, encounter entry, landmark mapping and owned split labels only after migrating retained programs/events. Keep group/map numbers of retained identities stable; do not renumber unrelated maps. Reject every dangling warp, connection, typed/raw map-command, spawn/recovery and playability reference. Rebuild report map membership/counts only within the explicit removal scope. Full inverse restoration must recover the original parent, including art/audio, programs, catalogs and recovery data.

Old saves may contain visited-map/position/override data for retired identities; resizing is not a save-compatibility claim. Production tests must use fresh fixtures bound to the resulting pack and retain all reached saves.
