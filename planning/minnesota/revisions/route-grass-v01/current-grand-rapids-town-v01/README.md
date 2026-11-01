# Grand Rapids town exterior review

Accepted locally as `route_grass_grand_rapids_town_revision`. Pack: `content-packs/minnesota-current-grand-rapids-buildings-v01.crystalpack`, SHA256 `ef34a8235325c116e75e72893bd8b096fc79e31673c0ac74f396ee5dcf023085`. See `selection-receipt-v01.json` and `acceptance.json` for exact bindings.

The depot now uses a complete native Goldenrod facade and the mart a complete native Azalea facade. The clinic is preserved. Eleven facade/shoulder cells change; all three door and return positions remain fixed. Three sign events now match the existing visible boards, including typed events, raw event commands and split script payloads. The authored regional connection patches remain intact.

Three meaningful native controller cases pass: building entry/exit and Continue, all four road guard lanes with missing requirements and free inbound travel, and residents/signs/public approaches/river fence. The flat town has no ledges, so no empty ledge case is counted. Production Continue and menus pass all 15 calls with clean stderr. Seven actual 2D captures, including the mart, were visually inspected and have zero source/native pixel mismatches. Scoped Clippy completed successfully. Source/binary/save bindings and original failures remain in this directory.

Independent preservation checks retain all 729 other maps, all State Fair facades, art, audio, encounters and trainers exactly. Canonical layout and sign dossier now match the verified successor. The explicit Mew60/Surf, prior badges and report/depot events are QA fixtures; this is not an earned campaign completion claim.

Thirteen town reviews, other facade repairs and final current-story integration/coverage remain open. Safari paid capture remains unselected. `review-history-before-selection-v01.md` preserves the earlier pending history, superseded visual assessment and rejected first sign child; those historical states do not describe the accepted successor.
