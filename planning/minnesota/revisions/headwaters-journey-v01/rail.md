# Returning by the Glacier Line

Entering a station hall registers it as a return destination. Each of the five halls sets its own visit flag on entry. On a first visit with no other stops registered, the conductor explains how to unlock one. Thereafter the menu lists the four other stops in line order. A previously visited destination boards immediately after the announcement. An X marks a stop the player still needs to visit. Choosing that row explains the restriction and leaves the player on the concourse. Return travel is free, including while the St. Paul courier case is open.

| Station | Public road used for the check | Front door | Road to porch, either way | Train arrival to an exit |
| --- | --- | --- | ---: | ---: |
| Detroit Lakes | West to Detroit Lakes Approach | (11,5) | 25 | 10 |
| Staples Rail Junction | South to Route 15 | (13,7) | 17 | 8 |
| St. Cloud | South to Route 5 | (37,5) | 44 | 11 |
| St. Paul Union Depot | West to Route 4 | (8,7) | 58 | 14 |
| Winona | West to Route 1 | (11,5) | 23 | 6 |

Distances count ordinary directional steps. The road count starts on the city side of the map connection and ends one square south of the depot door. The hall count starts where a train leaves the player and includes the final step onto an exit carpet. Winona’s west trail assumes the player has received a starter. Its callback moves the two wardens aside, and both trail checks then allow passage. The other four public walks avoid unrelated story triggers.

The [rail check](check-rail.py) reads a compiled pack and follows the installed visit tests, menus, movement scripts and warp targets. Each edition has 20 directed rides. The check covers all 80 combinations of origin and previous visits. Seventy-five combinations display a menu, giving 450 reachable selections across the four destination rows, CANCEL and the B button. The remaining five combinations produce the first-stop message. Each destination flag has one setter across Minnesota’s map scripts, in the corresponding hall callback.

Each conductor can be addressed from one public square. For every ride the conductor clears the opening, the player reaches the native train door, and the conductor walks home before the scene changes. All five train arrivals face south on unoccupied floor. The two exit carpets in each hall return to its own front porch. The street checks reserve authored passenger positions and alternate story staging squares, including Sera’s station position. These findings come from static inspection of the installed scripts, collision and actors.
