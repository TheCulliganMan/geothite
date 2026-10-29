# Source v10: bounded native hut entry repair

Source v09 remains immutable history. Actual native v02 controller diagnostics repeatedly pressed Up from lower-chunk (49,0), settling60frames each time, and stayed in the lower chunk. The former exterior DOOR (49,47) on the other side of the chunk connection could not be entered. This is an observed gameplay blocker, not a renderer defect.

V10 moves the entire original3×3 ranger hut exactly one metatile north, from rows21..23 to20..22. Former bottom-row cells24..26,23 become native SnowFloor3c. Exterior door (49,45), warp2, and settled clear (49,46) now share the upper chunk. Both original room carpets, scripts, flags, all actors/items/parties, lakes, ice, Cut, and external ports are unchanged. Exactly12metatiles differ, all within24..26,20..23.

The door-terminal source collision/straight-slide audit in physical-proof.json confirms all27 ordinary actor approaches, both real ports, cave and hut clear returns, and Cut-gated reward behavior. This is static source evidence. Actual rebuilt controller verification is publication-owned and pending when this source is frozen.

Source renders: planning/minnesota/renders/gunflint-quality-v01/source-v10/. All earlier views remain preserved. The existing49,49 ranger camera crops the roof after the north shift; whole-map render retains the complete house.
