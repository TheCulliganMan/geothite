# Cedar spring shelter on the current Minnesota game

R13's existing split-level refuge is now enclosed in native bedrock. A small
spring fills the bare western approach; the dry eastern bank leads to Harper's
raised care terrace, while a separate short stair reaches the Max Ether notch.
The room stays10×8metatiles. It is a useful rest stop near Ely, with no new
admission requirement, Cut tree, Surf requirement or invented regional exit.

- [Complete room before/after](../../renders/r13-current-shelter-v01/published-v01/whole-before-after.png)
- [Whole published room](../../renders/r13-current-shelter-v01/published-v01/MnR13Cave1F-whole.png)
- [Eight production camera views](../../renders/r13-current-shelter-v01/native-v01/MnR13Cave1F-production-cameras.png)
- [Actually reached spring bank](../../renders/r13-current-shelter-v01/native-reached-v01/spring-bank.png)
- [Whole-pack preservation](../../renders/r13-current-shelter-v01/published-v01/receipt.json)
- [Production stdio replay](../../renders/r13-current-shelter-v01/mcp-v01/receipt.json)
- [Connected outdoors](../../renders/r13-current-shelter-v01/atlas-v01/full-state-native-current.png)

The whole room, production camera sheet, reached spring view and exterior atlas
were visually inspected. Most of the53 changed metatiles replace black void
with blocking bedrock; four form the enclosed16-core-tile spring. The attached
ceiling buttress separates the two climbs. Neither climb requires entering the
water, and no water touches an outer map edge. The original native07 mouth
retains its exact Floor/Ladder edge quadrants; every other edge is solid rock.

## Source and actual native connections

The [canonical source](../johto-quality-v01/MnR13Cave1F.json) is frozen in
`source-v01/`, SHA256
`93af3277fa113a5bf15273b07d232c2a73652532d72b1c651e17051c8b20db24`.
Its previous source has SHA256
`6a10f5102a315398a48c17844bb2dd1fb0fad103d7eff85a76609c9bcc3c7dcc`.
The [contract](native.contract.json) names the exact publication parent and
retained service/reward. Coordinates below are16px gameplay tiles unless
explicitly labeled as metatiles.

| Feature | Native contract |
| --- | --- |
| Inside exit / clear arrival | (7,15) / (7,14), native exit1 |
| Real exterior | MnR13C00R01 |
| Active entrance / passive return | (14,5) / (14,6), exterior indices1/2 |
| Healer / talk approach | Harper(13,5), approach(13,6), face up |
| Original healing | Acknowledge both printed pages, then native HealParty restores HP/status/all learned PP |
| Healing climb | stair metatile(5,3), ordinary right lane x11,y5–8 |
| Supply climb | stair metatile(6,5), ordinary left lane x12,y10–12 |
| Original cache / approach | (15,11) / (14,11), face right |
| Reward | MAX_ETHER×1, EVENT_MN_R13_SHELTER_ETHER |
| Palette | Forced native NITE, unchanged cave pixels and animation |

The native passive return remains stable; stepping down onto(14,7) and back up
to(14,6) cannot reenter. Moving up to the actual CAVE at(14,5) reenters the same
inside ladder, then an ordinary Up reaches(7,14). Both native stair lanes work
in both directions. The entire ordinary floor is connected with actors present.

## Published child and preservation

Ignored output: `content-packs/minnesota-r13-spring-shelter-v01.crystalpack`,
SHA256 `d9b4f2e85061e8e615cc668e68fb05b094bb38773e8dfc08d56af6dca4073816`,
identity `60027135d782170fd7998a1e4ee50ce39170700b4a2bb79129dab2f883707784`.
It is a child of the reviewed R03 grass pack, SHA256
`115a2bfcec328a5d02b76c9b5f5f051836eecb239844bc965b1337719dec91e9`.
It therefore retains the current R12 shelter, complete river activity stage,
compact warm coast, snowy pine Ely, accepted city work and R01–R03 grass work.
After the fresh checks and visual review, `world.json` selects
`r13_spring_shelter_revision`. The exact prior R03 selection and every previous
record remain intact; the guarded selection receipt is retained with the
[bound verification](../../renders/r13-current-shelter-v01/verified-v01/receipt.json).

Only R13's blocks, border01→2e, native encoded block/attribute mirrors and
derived identity change. Independent complete normalized equality proves all
690 other maps and every unowned native program, event, dialogue, item,
species, encounter, art, audio, HOME, recovery and report remain exact.
The guarded Rust publisher's replay is byte-identical. All377 audio PCM
hashes, frame counts and loop ranges pass.

Three fresh controller cases pass: the generic whole-room course exercises
all eight viewpoints, both stairs, the item, Continue and the actual mouth;
the two original service cases separately check Harper's two printed pages,
movement ownership and two injured partners, then Max Ether's actual learned
PP restoration, once-only pickup and persistence. The historical fixed-probe
geometry case is not run against source lacking that retired metadata.

All eight complete640×576 native frames compare with zero differing pixels,
including one coherent water animation phase per image. Stdio MCP independently
plays both stair lanes, Harper's printed pages and movement lock, spring/cache
approaches, passive exterior safety, same-mouth reentry and Start/Pack, with
byte-empty stderr. Position/party fixtures are explicit local setup; they do
not establish earned regional access or a completed campaign.

The actual exterior atlas retains97 maps in one seam component, zero overlaps
and zero inconsistent placement cycles. Its static warp/seam graph is not a
Surf or prerequisite playthrough. Four named hubs remain absent on this branch;
the separate interior rebase is still underway. Four stone cave integrations
and broader map quality remain open. No final3D acceptance is claimed.

See [PUBLICATION.md](PUBLICATION.md) for executed commands and check limits.
