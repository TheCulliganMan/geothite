# Route 19 field observations

Detroit Lakes’ north gate leads north to Prairie Forest. Leave the route at the east end to reach Lake Itasca. The same trip works in reverse. Both gatehouse aisles are public before and after the Lake Badge. The previous gate had no guard, yet two floor squares stopped a northbound player and warned them about entering the town they had just left.

The three observations can be recorded in any order. From Detroit, the prairie bend comes first, then the spring to its northeast, then the pine clearing near the Itasca road. Both sides of the prairie board now have blooms in view. They use the same ordinary-ground flowers as Crystal’s Route 30 Trainer Tips sign. The ranger lists only the places still missing from the player’s notes.

| Stop | Installed map and position | Accessible reading position |
| --- | --- | --- |
| Prairie bend | `MnR19C00R00` (22,44) | (23,44), face west |
| Spring pond | `MnR19C00R00` (40,34) | (40,35), face north |
| Pine clearing | `MnR19C00R00` (54,14) | (54,13), face south |
| Detroit Lakes ranger | `MnDetroitLakesRangerCabin` (6,3) | (6,4), face north |

With all three readings and the Lake Badge, the ranger gives one Max Repel. A full pocket leaves the reward available. An existing stack with room can accept the item even when all slots are occupied. After a successful report, the ranger tells the player how the observations changed the spring map.

Run [check-loop.py](check-loop.py) with a Floodwatch pack and its matching Lake Observations successor. The static check follows all six reading orders from each town through the archived map connections, warp destinations, actors and dry terrain. It exercises all 32 combinations of badge, notes and prior receipt against four inventory states, including retry and repeat visits. It also compares the installed words and native flower pixels in morning, daylight and night palettes. The check measures walking distances and event outcomes independently of encounters, battle timing and controller input.
