# Granite Narrows and the Voyageurs stamp

Three dark openings share one ridge on Route 20. From the west they stand at (62,31), (70,33) and (76,31), measured continuously across the two route chunks. The outer pair connect through Granite Passage. The middle entrance returns to the same outdoor landing after a trip through the two floors of Voyageurs Ice Cavern. From the eastern fork, the ice door and the eastern through-door appear together and share the same dark arch.

The Granite Narrows post now identifies the outer pair and the middle entrance. The Sheltered Meadow post keeps its clue about rare nighttime encounters and gives the order a westbound traveler will encounter the caves. The [route dossier](../../routes/MN_R20.json) supplies both compiled messages.

| Optional trip | Installed movement and rewards |
| --- | --- |
| Cross Granite Passage from the west | The cave takes 44 directional inputs east and 31 west. Skirting the ridge outside takes 18 in either direction. The east shelf has stairs, two one-way westward drops and an ordinary climbing route back. |
| Take the shelf | Leif faces south at (21,17). His unobstructed three-square sightline reaches walkable floor. Another dry route passes him. The upper wall holds a Max Revive. A Revive and a visible Hyper Potion are also reachable on foot. |
| Search the seep | The scout describes a dry lip across the water. Surf opens two adjacent search positions for the Full Restore at (11,19). Neither cave exit needs a field move. |
| Enter the ice cavern | The other fork is eight walking inputs from the eastern granite landing. The 1F ladder needs an ice slide. All 215 reachable stops on 1F and 238 on B1F can lead back to an exit. The optional interior items include a hidden Full Heal and PP Up. The basement PP Up has its own collection flag. |
| Return to the ranger | After registering, a qualified eastbound traveler entering Boundary Waters at (0,24) or (0,25) earns the stamp. The conditions are all eight badges and the harbor signal. The Falls clinic grants one PP Up on return and keeps the reward available when the Pack is full. |

The two entrances in Crystal’s Union Cave join Routes 32 and 33 on the first floor, with separate ladders extending the trip underground. Polished Crystal retains those destinations, gives the northern and southern basements separate entrances and rewards searching three first-floor stops. In Minnesota the distinction matters one screen earlier, where a traveler has to choose an exterior cave mouth. [Crystal’s Union Cave](https://github.com/pret/pokecrystal/blob/f91d74b9c383da9a36f69f79c366852b5d0525e9/maps/UnionCave1F.asm#L203-L222) · [Polished Crystal’s Union Cave](https://github.com/Rangi42/polishedcrystal/blob/f8e67fbd53ad7a99870d8ab87fd73612e6c6a679/maps/UnionCave1F.asm#L6-L31)

The [passage check](check-passage.py) reads a compiled pack. Run it with the pack path, adding `--baseline` for a Station Return parent with the original signs. It follows map seams, warp targets, slide endpoints, ledges and adjacent search squares, then reads the stamp callback and reward branch. Eight static production-camera views cover the western sign, the fork, the granite shelf and seep, and the ice floors. The distances count directional inputs in a fixed map state. Battles, random encounters and inventory collection were outside the static check.
