# Gym battle lessons

| Gym | What the player sees |
| --- | --- |
| Rochester | Tavel's level 8 Nibbin uses Quick Attack. Varel's level 10 Nibbin can use Headbutt, and the faster Crookit keeps Quick Attack. |
| Mankato | Sodcalf can open with Mud-Slap. The 20-power hit previews the TM and gives the player a reason to switch and clear the accuracy drop. |
| Minneapolis | Pipulse can throw Thunder Punch. Sparkveil pairs Thunder Wave and Light Screen with a lighter attack, leaving the stronger Thunderbolt to Tapacitor. |
| St. Paul | Patchebee can use Steel Wing for damage and a chance to raise its Defense. The Rustlebee, Vanechick and Burrowbill families can use the prize after encounters on Routes 1, 2 and 3 respectively. |
| Duluth | Lutebrisk can lower Speed with Icy Wind ahead of Plowzer and Rimeguard. Vernacape still closes with Ice Beam. |

The other three leaders already carry their prize moves. Minneapolis's own flour-mill family can use its local reward. Oatlet appears on Route 2 and Millimp on Route 4. Seventeen compatibility additions give active regional parties nine Thunder Punch learners and ten Steel Wing learners.

Tavel's stopwatch, bent arm and raised sneaker give him a silhouette visible against the Rochester floor. His native source contains a 56×56 portrait and six 16×16 field poses sharing four RGB555 inks. The reference review examined how the Schoolboy separates his limbs, Danny holds his upper body at an angle and the Camper leaves daylight at his elbows. [Crystal Legacy Schoolboy](https://github.com/cRz-Shadows/Pokemon_Crystal_Legacy/blob/77e6b3859349698ad0a7a8a93cd27f78860d3e4a/gfx/trainers/schoolboy.png), [Orange Danny](https://github.com/PiaCarrot/pokeorange/blob/5b4a95ec2d4638c7b8f125b6e1644390d52d4a5b/gfx/trainers/danny.2bpp), [Polished Crystal Camper](https://github.com/Rangi42/polishedcrystal/blob/f8e67fbd53ad7a99870d8ab87fd73612e6c6a679/gfx/trainers/camper.png).

Run `audit.py PARENT.crystalpack CANDIDATE.crystalpack` with Python, cbor2 and Pillow to compare the packed trainers, machine compatibility and Rochester's two entrance sightlines. To prepare the matching website, run `publish_pokedex.py PRIOR_RELEASE NEW_DIRECTORY` with Python. It compares the canonical regional profiles to the existing release and updates the machine tables, tab counts, searchable names and TM numbers.
