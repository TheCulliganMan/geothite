# Deployed Minnesota dialogue audit — exact hosted pack

Pack: `target/northstar-next-hosted-cdd4da77.crystalpack`  
SHA-256: `03ad92a98538f810809e3b547fffe4642c3fadbc82a9c778da91870258486aef`  
Base comparison: `target/northstar-next-cdd4da77.crystalpack`, SHA-256 `4071b8ab72bd40c71a40a888c9b9dda6d14309dac37568cd7326d8ac8e9068c1`.

Both packs contain the same text and script bodies for the labels below. This is a read-only audit of decoded pack data and branch rows; no builder, pack, or gameplay state was changed.

## Priority fix: bluff researcher claims unsourced news

`MnFossilExchangeText` on `MnR01C00R00`

Current pages:

1. `Your first fossil` / `is alive! Take the`
2. `other specimen for` / `free research?`

The speaker is the Route 1 bluff researcher. The `MnFossilExchange` branch shows this offer when either `EVENT_MN_TRILUME_RESTORED` or `EVENT_MN_SIPHLET_RESTORED` is set and `EVENT_MN_FOSSIL_EXCHANGED` is not. The restoration happens in Rochester. The pack has no dialogue, phone, or message handoff that tells the bluff researcher the result. This line leaks an event to an NPC who has no shown way to know it.

Replace the two pages with:

1. `I found another` / `fossil in the rock.`
2. `My colleague can` / `study it with us.`
3. `Would you like it?`

These lines fit the existing yes/no offer and its unchanged item-selection/gift path. The colleague is already introduced in `MnFossilDirectionsText`. The existing `EVENT_MN_FOSSIL_EXCHANGED` set occurs only after `verbosegiveitem` succeeds; a full key-item pocket returns through `MnFossilBagFull` without consuming the one-time offer. Keep those gates and retry semantics.

`MnFossilDirectionsText` is a separate post-gift handoff and already says the colleague works by the Rochester clinic courtyard, identifies the Care Badge requirement, and explains that the fossil is the thing to bring. Keep it; it gives timely, relevant directions.

## Rochester gym: remove repeated Flash spoiler, keep course instructions

`MnRochesterGymGuideText` current pages include:

- `Follow the gaps` / `between planters.`
- `Beat TAVEL, then` / `ELVIK. VAREL waits`
- `at the far end.`
- `For HM FLASH, ask` / `the hollow guide`
- `on DRIFTLESS` / `TRAIL.`
- `The CARE BADGE` / `allows field use.`

The first five lines are useful directions for the visible gym course. The Flash lead is a later-route spoiler repeated in multiple leader replies. Replace the guide's complete page sequence with:

1. `Follow the gaps` / `between planters.`
2. `Beat TAVEL, then` / `ELVIK. VAREL waits`
3. `at the far end.`
4. `The CARE BADGE` / `allows field use.`

Remove only the Flash lines.

`MnRochesterGymLeaderWonText` current pages:

1. `You noticed when` / `the battle turned.`
2. `You earned this!`
3. `For HM FLASH, ask` / `the hollow guide`
4. `on DRIFTLESS` / `TRAIL.`
5. `The CARE BADGE` / `allows field use.`

The battle reaction is specific and should stay. Replace the sequence with:

1. `You noticed when` / `the battle turned.`
2. `You earned this!`
3. `The CARE BADGE` / `allows field use.`

This is the trainer's win-text, selected by the leader's `winlosstext` pair; the script then sets `ENGINE_ZEPHYRBADGE` and `EVENT_MN_GYM_1_DEFEATED` before running `MnRochesterGymLeaderReward`. It is safe for VAREL to state the badge's field use here.

`MnRochesterGymLeaderGiftText` current pages:

1. `You earned a BADGE` / `and this TM!`
2. `Ask the city guide` / `where to go next.`
3. `For HM FLASH, ask` / `the hollow guide`
4. `on DRIFTLESS` / `TRAIL.`
5. `The CARE BADGE` / `allows field use.`

Replace with:

1. `You earned a BADGE` / `and this TM!`
2. `Prairie Farm Road` / `leads to MANKATO.`
3. `The CARE BADGE` / `allows field use.`

This text is reached only when the gym reward script successfully gives `TM_HEADBUTT`; the badge has already been awarded after VAREL's victory. The actual route is a reasonable next destination; the Flash/Hollow Guide lead is the spoiler to remove.

`MnRochesterGymLeaderAfterText` is used on later interactions once `EVENT_MN_GYM_1_TM_RECEIVED` is set. Current pages repeat the Mankato road/rest hint and the Flash spoiler. Replace with:

1. `Rest up, then take` / `Prairie Farm Road`
2. `to MANKATO.`

Keep trainer dialogue otherwise. The TAVEL and ELVIK seen/win/loss pages are short battle exchanges. Their defeat-only replies point out gaps and the last planter; those are useful, locally visible course directions. `MnRochesterGymBeatTavelText`, `MnRochesterGymBeatElvikText`, and `MnRochesterGymCourtRequirementsText` are also reached by the leader's explicit trainer-defeat checks and only tell the player which visible trainer is required next.

## Fossil Lab: accurate, unspoiled first visit

`MnRochesterFossilLabAssistantText` current pages:

1. `We study fossils` / `and restore the`
2. `life inside them.` / `If you bring one,`
3. `our scientist can` / `explain the work.`

The map has two separate scientist actors: `MnLabFossilResearcher` and `MnRochesterFossilLabAssistant`. This assistant line is reached when neither restoration nor the second-fossil exchange branch applies. It does not assume a Route 1 discovery, but “our scientist can explain the work” is vague and sounds like it addresses a visitor who already has a specimen. A more direct first-visit explanation is:

1. `We study fossils.`
2. `Bring one if you` / `find a specimen.`
3. `Our researcher can` / `try to restore it.`

Keep `MnLabFossilNeedSpecimenText`: it is the researcher's conditional response when the player has no accepted specimen, and it asks for a fossil rather than assuming one was found. Keep `MnLabFossilRestoredText` and `MnLabSecondOfferText`: their branches occur in the lab after that lab restores the first fossil, so the staff have direct evidence.

## Fishing instructions

The exact hosted pack includes the proposed fishing labels. Keep the practical Old/Good/Super Rod directions in `Mn*FishingGuruHouseGuruText`: these explain how to cast and point to the next Guru. Do not add further future-route or later-gear spoilers.

The repeated post-gift `Mn*FishingGuruHouseGuruAfterText` is the same in all three houses (“Try every shore! Different RODS catch different kinds of POKEMON.”). It does not leak hidden progression; it is merely generic. No change is needed for the omniscience fix. The existing full-pocket branch is retryable and should remain unchanged.

## Scope and verification limit

This audit verifies source labels, exact deployed text bodies, and branch/flag wiring in the named pack, including identity hash. It does not claim controller replay or rendered-page verification. The proposed lines are ready for a text-only overlay; rerun the pack reader and affected interaction checks after integration.

