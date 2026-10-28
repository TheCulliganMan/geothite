# Snow-ground signs and partial tree edges

This separate append follows the corrected winter art and main Rainy River
region publication. It extends the existing two private aliases; it creates no
new tileset copies and edits no terrain, actors or programs.

| Alias | New block | Exact native source | Collision |
|---|---|---|---|
| `mn_winter_outdoor` | 41 | `mn_border_pines:13` sign | FLOOR/FLOOR/FLOOR/WALL |
| same | 42 | `0e` | original Headbutt/FLOOR quadrants |
| same | 43 | `10` | original Headbutt/FLOOR quadrants |
| same | 44 | `11` | original Headbutt/FLOOR quadrants |
| same | 45 | `12` | original Headbutt/FLOOR quadrants |
| same | 46 | `14` | original Headbutt/FLOOR quadrants |
| same | 47 | `15` | original Headbutt/FLOOR quadrants |
| same | 48 | `16` | original Headbutt/FLOOR quadrants |
| `mn_winter_north_shore` | 88 | `mn_north_shore:45` sign | WALL/FLOOR/FLOOR/FLOOR |

Original lawn glyphs are replaced at their exact eight-pixel positions with
existing snow-block glyphs. Tree pixels use the reviewed winter canopy profile.
Sign pixels retain native timber/gray middle ink and dark outline on snow-white
ground. Colors come from the pack's native day bank and existing winter
foreground file, with exact source hashes. No extra pixel catalog is exported.
There are eight new glyphs F6–FD in the forest alias, and four F6–F9 in the coast
alias. These fit the ten unused slots after the reviewed five-recipe winter art.
Original65/136 blocks, original224 palette entries and every existing used glyph
address/pixel/foreground remain unchanged. Icy tall-grass zero holes are intact.

Cut source blocks0f/5b/5f are deliberately excluded: they need a separately
reviewed replacement mapping. Every new Headbutt collision is copied from the
source. Mixed trees remain partial blockers, preserving the original path width.

The frozen contract is `winter-mixed-edges-v01.json` (file SHA256
`a6d8f46a813555c5daff1deffe0d2cb0682570f244971c28fc19f985593df40c`).
Use `derive_edges(pack, spec)` in `preview_winter_edges_v01.py` on a pack already
containing the corrected winter art. Its CLI can derive that art in memory when
previewing from the earlier source pack. It never writes a game pack.

`renders/ice/mixed-edges-v01/source-v02/` contains the native source comparison
and a source160 player camera. The player stands on legal FLOOR at core(5,4),
north of the sign. This is a material study and source camera, not a published
route, native GPU receipt or proof of earned progression. Rejected/earlier
images and full-resolution originals are retained in their own epochs/history.
The v01 camera placement was corrected in v02; the earlier image remains.

`winter_mixed_edges_art.rs` provides
`append_winter_mixed_edges_art(base, specification_json)`. Its CLI is
`append_winter_mixed_edges_art BASE SPEC NEW_OUTPUT`. It requires a fresh output,
performs strict written readback, and preserves the rest of the full pack.
Three external asset tests require `CRYSTAL_WINTER_MIXED_ART_TEST_PACK` to point
at the completed corrected winter-art/main-region parent. They check exact
source-preview metatile/2bpp hashes, prefix preservation, unsupported Cut,
changed source preimages and corruption of existing palette, pixels, collision,
program or report data. Native publication is owned by the parent worker.
