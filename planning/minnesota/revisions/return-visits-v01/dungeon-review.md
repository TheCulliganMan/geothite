# Duluth Harbor Gym

The west gate used to open after defeating Norek without shortening the way out. From the west side of his court, the original east return took 14 directional inputs across 24 squares. Going through the newly opened west gate took 16 across 25. The same two-input penalty applied from his north and east sides. The gate helped a player revisit Livra, but the original east course remained quicker for leaving the gym. This was true in both the shared Field and local Intro editions of Trail Discoveries.

The revised west lane lets a player use the two skills the gym just taught. From the center rough patch at **(7,8)**, skate west to the newly exposed stop at **(2,8)**. Turn south and pass Livra’s platform in one eight-square glide to **(2,16)**. Turn east and the ordinary entrance aisle stops the player at **(6,16)**. The original red carpet is three squares south. Norek describes the return after the battle.

| Position beside Norek | Previous quickest exit | Revised west exit |
|---|---:|---:|
| West (7,3) | 14 inputs | 9 inputs |
| North (8,2) | 16 | 11 |
| East (9,3) | 18 | 13 |

An input here is a directional choice. It can move one square on regular ground or carry the player across several ice squares. The west line saves five choices from all three positions. From (7,3) it crosses 25 map squares. The original 24-square route remains available.

## Before the badge

Duluth’s gym doorway at **(48,31)** leads to warp 3 on the blue landing at **(6,17)** in `MnDuluthGym`. Both exit carpets return to the public doorstep at **(48,32)**. The gate scripts read the results of the native battles.

| Stage | What the player can reach |
|---|---|
| Arrival | The guide explains that rough patches stop a slide. The lower patch at (6,12) lets the player turn west to (2,12), then approach Livra. Entering (3,11) puts the player in her sightline. |
| After Livra | The east opening at (10,10) leads to the upper rink. Tordel sees the player from (5,6). Anver watches a different landing at (3,7). Either fight has an approach that stays clear of the other trainer’s sightline. |
| After Livra and Tordel | The court opens at (6,4). Norek waits for conversation at (8,3) and checks the seventh badge. Anver can still be left unbeaten. Both established ways around the two rinks work, and every reachable stop has an exit. |
| After Norek | The west lane opens before the TM handoff, including when the TM pocket is full. Livra retains a dry standing square at (3,10) and can be spoken to from (3,9). The other two trainers, the guide and Norek remain approachable. Anver can still be challenged on sight. |

The original and revised pre-badge rooms have the same resting spaces and shortest routes at each stage: 33 on arrival, 53 after Livra and 76 once the court opens. These counts exclude the two warp carpets. The revised post-badge rink has 67 resting spaces. Several former walking squares now carry the player to the rough landing. Every reachable stop still leads to the door, and the leader’s court remains accessible on a return visit.

## Art and evidence

Crystal’s Mahogany Gym uses diagonal ice, contrasting rough stops and a dry entrance apron. Polished Crystal retains these cues and gives the ice its own cyan palette. Duluth keeps its blue-brick platforms, existing three teachers, red carpet and two separate rinks. The west return uses the same rough-stop drawing that the player encountered in the first puzzle. All four changed blocks come from Duluth’s existing gym bank. [Crystal room and trainers](https://github.com/pret/pokecrystal/blob/f91d74b9c383da9a36f69f79c366852b5d0525e9/maps/MahoganyGym.asm) · [Polished Crystal room](https://github.com/Rangi42/polishedcrystal/blob/f8e67fbd53ad7a99870d8ab87fd73612e6c6a679/maps/MahoganyGym.asm) · [Polished Crystal gym colors](https://github.com/Rangi42/polishedcrystal/blob/f8e67fbd53ad7a99870d8ab87fd73612e6c6a679/maps/MahoganyGym.pal)

Run `python3 check-harbor-gym.py CHILD --parent PARENT` with paths to the matching external packs. The graph check and native regression both passed on Field `6828d7002748` and local Intro `3cf55d724621`. The graph check follows the actual gate instructions, stops slides against native collision and actor bodies, and treats exit carpets as terminal. It covers all 32 combinations of the three teacher receipts, Norek’s win and a potentially stale west-gate receipt. The optional native Rust test `harbor_exit_runs_on_reward_reentry_and_continue` runs the reward’s actual gate refresh, checks the live and saved blocks, and exercises battle reload, warp entry and Continue with contradictory stored tiles. Earlier editions retain their own save identities. These checks do not assess how long a new player takes to learn the rink or the difficulty of winning the battles. [Rink and gate checks](check-harbor-gym.py) · [Native reward and Continue regression](../../../../crates/crystal-assets/src/northstar_harbor_gym_exit.rs)
