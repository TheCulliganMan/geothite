# Southern cave routes

The six replacements in [south.json](south.json) start from new room outlines
and use the original native terrain banks. The map names still connect to the
same places on the region map, but the indoor mouths, ladders, people and finds
move with the new routes. The fossil guide and Castorock dialogue have been
updated where the old directions pointed somewhere else.

| Room | New floor plan | Reason to explore |
| --- | --- | --- |
| Driftless Hollow | A 24×13 limestone traverse with a low shelf below a long rock rib. Near the middle, steps lead into the high notch. A second stair brings the eastern bank into view. | The high notch leads down into the Lamp Gallery. The trainer watches the high bank, leaving a dry crossing for players who bypass the battle. |
| Driftless Lamp Gallery | An 18×15 dark room with a western switchback up to the fossil relief and a descending ledge across its eastern shoulder. | Reading the fossil with Flash reveals the field kit. The ledge skips the long walk around the switchback when leaving. |
| River Pocket | A 21×16 flooded spring running lengthwise beside the dry bank. A constriction between two rock fans leads into the dead-end bedrock chamber, where the eastern bank turns south. | The spring hides a reward on a rock you can only approach by Surf. Checking the far pillar and the branch by the second pool requires the Strength route. |
| Rock Cut | An 11×16 slanting fault with a walkaround along the west face and a high southern drop toward the mouth. | A rest pocket has the healer. On the high side, the supply branch and bright exit sit on different levels of the split. |
| Cedar Shelter | A 15×13 warm stone bowl with a spring at the bottom and a stair into the sheltered upper treatment bay. | Harper waits above the spring. The shelter Ether sits beside the opposite turn in the bank. |
| Granite Den | A 19×15 contour loop around a rock buttress. The high rim gives way to a narrow seep and a ledge that rejoins the entrance approach. | The reader sits on the sheltered side of the buttress. The crest and its hidden tag give the climb a destination. |

On foot, the new Hollow crossing takes 54 steps between the indoor approaches.
Rock Cut takes 32 steps from the south and 31 in reverse. The paired hops each
take one input. Walking between their two sides takes 56 steps in Lamp Gallery,
20–22 in Rock Cut and 8–10 in Granite Den. The ordinary routes remain open in
both directions.

Use `review_south.py CANDIDATE.crystalpack` to check the compiled layouts,
links, actor placement, dialogue text operands, return paths and the river
gate. The route graph treats active warp squares as exits and uses their dry
approaches as standing positions. It checks every possible local item ball
state. Whole-room and player-view images are kept in the ignored review
directory `target/cave-replacement-south/`.
