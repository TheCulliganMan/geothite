# Field YES/NO windows

The default Crystal `YesNoBox` has inclusive source-tile borders `(14,7)`
through `(19,11)`: six columns and five rows. Its cursor occupies column15,
labels start at column16, and YES/NO print on rows8 and10. Row9 separates the
answers. Both labels and their cursor remain inside the border.

This comes from `home/menu.asm` in the pinned external source
`3bc8daa4173e96a7f4011dad3922eb6fa5dad5c6`: `_YesNoBox`,
`GetMenuTextStartCoord`, and `PlaceVerticalMenuItems`. The border coordinates
are inclusive. A four-row window or adjacent answer rows are incorrect.
Callers of `PlaceYesNoBox` can supply another origin; retain that caller's
source coordinates rather than forcing the ordinary default position.

The shared field renderer must use the same positions for initial drawing and
retained glyph updates. A cursor change must erase its old marker and repaint
the new marker without shifting labels or rebuilding the border. The ordinary
speech window remains20×6; together the two frames contain66 perimeter tiles.

`FIELD_YES_NO_HEIGHT_TILES` defines the shared height. Both
`spawn_visible_yes_no_prompt_box` and
`update_scene_dialog_text_content_in_place` in `overworld_rendering.rs` place
the labels. `retained_field_dialog_structure_matches` in `menu_rendering.rs`
validates both complete frames. Source-paletted item/Oak PC boxes and nickname
prompts have separate presenters; they also retain the source five-row layout.

The focused regression `shared_yes_no_` checks all18 default border tiles,
all seven cursor/label glyph positions, phone/day/PC/Save cursor owners, and
pixel equality between a fresh NO draw and its retained update. A production
controller regression opens Save through Start and declines with NO/B in all22
clinic lobbies, proving no save is written and walking returns. The existing
full Save and Mom lifecycle regressions check their actual input and scripts.

Render review stays offscreen with `CRYSTAL_RENDER_TEST_HEADLESS=1` and
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`. The ignored
`target/kanto-resume/shared-yes-no-review/` evidence contains a matched native
Save pair and production UI rasters. These are renderer comparisons, not
independent ROM pixel or frame-timing goldens. This layout correction does not
certify every custom two-option menu, its palette, or its closing delay.
