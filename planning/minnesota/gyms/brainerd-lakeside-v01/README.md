# Brainerd swimming beach

Enter through the two cedar posts. The pale central shallows lead straight ahead to Sivren’s sandbar. Lorla stands beside the starting line on the west shore. The red float is just offshore from Bravel’s east casting point. Both trainers can spot a player approaching their stretch of beach. The badge challenge requires the Wind Badge.

The beach uses Brainerd’s private native tileset. The 155 original 8×8 tiles occupy physical positions 27–181. Each uses four colors from one of eight RGB555 palettes in `lake.pal`. The 38 new metatiles in `shallows.json` each borrow the four collision quadrants named by their `from` field. The cedar canoe and rack occupy blocked ground behind the leader.

Run `audit.py PARENT.crystalpack REVISED.crystalpack --render-dir OUTPUT` to compare compiled landings, trainer sightlines, the straight approach and the terrain beneath all four characters. The optional output contains the native full room and three 160×144 camera views with the same framing and character art on both sides.
