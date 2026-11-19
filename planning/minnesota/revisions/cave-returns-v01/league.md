# Leaving the League in Ely

The registrar used to say “A loss or an exit ends the run.” A player in Casey’s chamber can walk all the way back to Fern, turn around, and return to Casey with four wins. The run ends upon entering the lobby or losing a battle. The revised sign-in question tells players both rules before they choose Yes or No.

| Player action | What the installed events do |
| --- | --- |
| Decline registration | The registrar points to Ely’s clinic and Mart. The player stays at the desk. |
| Accept with eight badges and the restored survey | The registrar starts a run at zero, sets the blackout destination to Ely’s clinic, and admits the player to Fern’s chamber. |
| Walk south from Casey through the four opponents’ chambers | Each room returns the player to the preceding chamber. Already earned northern doors reopen on re-entry. A returning Champion can also decline Casey’s rematch and stay in the chamber. |
| Leave Fern’s room for the lobby | The lobby’s arrival callback clears the run and tally. The player can register again. |
| Lose to any of the four opponents or Casey | The clinic’s arrival callback clears the run and tally. The native blackout heals the party and halves the carried money. The player can return to the registrar on foot. |
| Continue from a saved run or after a loss | In a chamber, earned wins persist. In the clinic the previous run is over. Survey restoration and earned badges survive both branches. |

The two Gym Walk parent editions have identical scripts, warps and actors in the lobby, four opponent rooms, Casey’s room, Hall of Fame and Ely clinic. The only authored League change is the registrar’s explanation. The existing [League rooms review](../league-journey-v01/rooms.md) covers the two-wide south carpets and Casey’s rematch choice.

The [controller regressions](../../../../crates/crystal-bevy/src/bevy_shell/tests/league_return_trip.rs) accept either written Cave Returns edition in `MN_BADGE_TRAILS_PACK`. Each loss starts with declared earlier wins and one level-one Cindore at one HP. The cases cover each first-time opponent plus an accepted rematch with Casey. After each battle, the controller walks out of the clinic at Ely (11,22), returns to the League at (25,14), tries both registration answers and approaches Fern. A separate case starts at Casey as a previous Champion, declines the rematch, saves and continues in the preceding chamber, returns to Casey, and walks all the way back to the lobby.
