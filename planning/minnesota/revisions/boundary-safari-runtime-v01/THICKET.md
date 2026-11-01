# Pine Pocket Cut thicket

The Cut tree now gates a useful Safari destination: 52 enclosed grass tiles with
better odds for the forest's rarer Shedstag and Gougecrest. The existing sign
already promises `RARE PINE THICKET / CUT FROM THE LAKE`; the encounter data now
delivers that promise. No terrain, source art or creature stats were changed.

Current unselected review:
`content-packs/minnesota-safari-runtime-review-thicket-confluence-v14.crystalpack`,
SHA256 `c18e90caa089bf72e77c5b51b4a0f2351316d44fd356c8a770ae107407e93eba`.
It directly preserves accepted Confluence parent
`ce04fbe0e78eb0b47502b6f32f62619bf160f35f8ded9b63287cb3dfe8b03226`.
The current [zone.json](zone.json) is bound at
`4b31f5ce29f34f9302e769a4ddc4834c6dc6eea749d7561b607fc3a2671e3e7a`.
Paid main admission remains closed pending the other acceptance items.

## Actual approach and return

Coordinates are global walking tiles. Start beside the real hub ranger at local
`(30,10)` and confirm one 500 fee. The initial previously earned party, permissions,
money and noon clock are declared fixtures before input; no position, RNG,
permission or budget repairs occur during these journeys.

| Stage | Actual native path |
| --- | --- |
| Admission | Court global `(15,105)`, 30 balls and 500 steps |
| Main lake | Walk to `(3,90)` and launch at `(3,89)` |
| Pine Pocket portage | Paddle 111 steps to `(30,21)`; dismount `(29,21)`, walk 7 steps through at least 4 grass tiles to `(22,21)` |
| Isolated pocket | Launch `(21,21)`; paddle 19 steps to `(10,29)`; dismount `(10,30)` |
| Cut gate | Face south toward `(10,31)`; actual Cut/Hive permission and confirmation required |
| Thicket | Walk to target `(9,35)` and encounter actual `PINE_THICKET` wildlife |
| Inside Continue | Native TILES reload restores the tree; walk to `(10,32)` and Cut north to exit |
| Return to main water | Relaunch `(10,29)`, paddle to `(21,21)`, cross the portage east and relaunch `(30,21)` |
| Ordinary map reset | Paddle to `(22,47)`, cross south to `(22,48)` and back; authoritative map data again contains the original tree |
| Retirement | Paddle to `(3,89)`, land `(3,90)`, walk to `(10,104)`, confirm the Reed ranger at `(10,103)` |

Pine Pocket is separate water. A direct Surf path from the main lake to the tree
does not exist; the first itinerary incorrectly assumed that and failed without
changing the player's position. The corrected native journey uses both actual
portage landings. It reaches the Cut approach after 168 movement commits.

Both missing-permission trips return after 334 commits. The successful trip,
including wildlife, inside Continue/recut and the ordinary map connection reset,
returns after 354 commits, leaving 146 steps. All three return with 30 balls,
the original bag and unchanged party HP/status/PP. Only normal walking happiness
is normalized in the party comparison. The fee is charged once per expedition.

## Gate and reward contract

The gate lives in `MnBoundarySafariC00R00`, metatile `(5,15)`. Native block 63
contains one `CUT_TREE` quadrant at `(10,31)`. Its source replacement 3f changes
that quadrant to FLOOR; the other three collision quadrants stay exact, including
the neighboring `HEADBUTT_TREE`. No permanent gate flag is invented.

The conservative graph includes walking, Surf and down-hop candidates, with
actual NPC bodies. Before Cut, none of the52 grass tiles is reachable from the
legal launch. The native replacement opens every one in that graph. Actual
controller input independently traverses the gate, reaches `(9,35)`, produces
wildlife and safely recuts the regenerated tree from inside.

The encounter rectangle is `[[0,30],[16,38]]`; it includes surrounding forest
walls and the approach, but no water. Only its 52 grass tiles use the new grass
odds. The remaining Pine Portage rectangles retain the previous tables. All
14,208 exterior cells outside this rectangle have identical encounter behavior
to the earlier source; the additional rectangle IDs only partition that table.

| Species | Weight among encounters | Level | Flee threshold |
| --- | --- | --- | --- |
| PINIT | 15/100 | 30 | 64/256 |
| NOTCHBILL | 10/100 | 33 | 64/256 |
| SHEDSTAG | 45/100 | 36 | 96/256 |
| GOUGECREST | 30/100 | 38 | 96/256 |

Shedstag/Gougecrest's combined weight rises from 15/100 to 75/100 in the gated
clearing. Encounter rate stays 48/256; these weights are neither capture chances
nor guaranteed encounters per step. Levels and flee thresholds are preserved.
Water tables and native fishing remain unchanged. Every configured species and
the GILLPUP/VOYAGEAR fixture partners pass the active Minnesota roster/retirement
check. VOYAGEAR's existing native TM/HM compatibility explicitly includes Cut.

## Native controls and review evidence

The production Cut case plays three separately paid expeditions: no Cut move,
no Hive Badge, then all permissions. Both denials retain the tree and resources.
Choosing NO cancels; choosing YES applies63→3f. Field Cut uses no partner PP or
Safari resources. Continue from inside and a real map connection both restore
the temporary vegetation; reverse-side Cut provides the safe exit.

The renderer-neutral controller now exposes the actual contextual field-move
chooser as a read-only `field:move-confirm` menu while preserving its text.
Its YES/NO markers mirror the production cursor. This does not fabricate a
compiled `yesorno` command or add a text-client dispatcher. Input stays on
`VisibleShellController::press`; the same projection serves native Surf prompts.
The current Cut case additionally verifies the visible YES→NO selection and menu
release after cancellation. A separate Surf/portage/cabin/Continue case passes.

Four production stdio sessions on each of the v13 and v14 reviews make 47
successful calls per review with empty stderr and
immutable input saves. They verify both denials, visible YES/NO, actual NO
blocking and YES traversal to `(10,31)`, plus generated Shedstag capture-screen
input ownership, Run and the production Start menu after returning to the field.

The renderer CLI now accepts its existing ordinary keyboard script in native 2D.
This is needed because loading an opened-gate save correctly regenerates the
temporary tree. Opened views therefore start from a genuinely reached closed
checkpoint and execute live keyboard A inputs before capture. Minnesota preview
requests reject unsupported views and omit the F3 view-switch hint. No window
is opened for automated captures.

- [Current closed gate](../../renders/boundary-safari-runtime-v11/native-final-v01/cut-closed.png)
- [Current native confirmation](../../renders/boundary-safari-runtime-v11/native-final-v01/cut-confirmation.png)
- [Current live opened gate](../../renders/boundary-safari-runtime-v11/native-final-v01/cut-opened.png)
- [Current Shedstag encounter](../../renders/boundary-safari-runtime-v11/native-final-v01/cut-thicket-wildlife.png)
- [Inside after Continue](../../renders/boundary-safari-runtime-v11/native-noon-v01/cut-thicket-inside.png)
- [Complete connected area](../../renders/boundary-safari-runtime-v11/connected-upstream-v01/whole-original.png)
- [Current Confluence-parent connected area](../../renders/boundary-safari-runtime-v11/connected-confluence-v01/whole-original.png)
- [Current independent contract/preservation audit](../../renders/boundary-safari-runtime-v11/thicket-contract-confluence-v01.json)
- [Five rejected contract mutations](../../renders/boundary-safari-runtime-v11/thicket-contract-negatives-v01.json)
- [Current production stdio validation](../../renders/boundary-safari-runtime-v11/thicket-confluence-stdio-validation-v01.json)
- [Current matched-pixel verification](../../renders/boundary-safari-runtime-v11/pixel-final-verification-v01.json)

The matched daytime closed/opened frames share the reached save, position and
facing. Only 1395 pixels change inside the 48×48 Cut-tree footprint; all 367,245
other pixels match. Earlier unset-clock and rejected CLI captures are retained
but are not used as matched before/after evidence. All full originals, previews,
native readbacks, failed attempts and reached saves remain immutable.

The original daytime frames and first 47-call stdio receipt bind the v13 Upstream
review (`8c41c9e92e95da267a3d94186a8249c10372d30314ad56e52ef80fede25263ef`).
The new v14 audit independently binds the exact selected Confluence terrain;
its four fresh native frames match those original images pixel-for-pixel.
See [VERIFICATION.md](VERIFICATION.md) for fresh v14 checks and distinct source epochs.
The two denial still images show the retained physical tree after the transient
notice has cleared; the actual denial text/input ownership is verified by the
controller and raw stdio responses, not by those field-only stills.
