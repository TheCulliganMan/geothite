# Gym comparison against Crystal — 2026-11-05

This is an inspection and design audit, not a completed redesign or a pack
publication. The installed baseline is the `r06_pine_fork_revision` selected by
`world.json`: `content-packs/minnesota-reviewed-maps-v11.crystalpack`, SHA-256
`d3c16929aade3df2e4b26406dd1bd1c0ac61f05b05be5f569c41e1657c3899d9`.
Do not use the older gym review branches as its parent.

The original Crystal room contact sheet was inspected alongside an atlas
rendered from the selected pack's actual blocks and objects:

- `../renders/gyms/originals/original-gyms-contact-sheet.png`
- `../renders/gyms/selected-20261105/installed-all.png`

The installed atlas renders initial blocks and standing objects; it does not
simulate event-driven block changes or establish traversal. It also renders
objects facing south for comparison. Native gameplay and each gate state need
separate review. The two quick Rochester experiments in
`../renders/gyms/rochester-crystal-study-v01/` and `...-v02/` were rejected:
they copy Violet's blue walkway vocabulary without a convincing room or course.
They are visual experiments, not installable layouts.

Crystal's best rooms have a clear dominant setting and a legible journey through
it: Violet's suspended winding walk, Azalea's living garden, Cianwood's timber
training hall, Mahogany's ice and Blackthorn's two floors. Some are simple,
but their walls, floor, furnishings, trainers and puzzle belong to the same
place. The current regional rooms often show one mechanic on a large blank
floor or repeat hard rectangular barriers. A palette swap alone will not fix
that. Recompose each entire room at player scale and make the route and return
read naturally from the setting.

| Gym | Installed visual issue | Direction for a complete redesign |
| --- | --- | --- |
| Rochester | The first room is still the previously rejected pink planter zigzag; it reads as Goldenrod's materials without Goldenrod's recognizable shape. | Give the first Gym a coherent identity and a readable first challenge, with trainer order and a short return. Avoid another planter maze or an arbitrary switch sequence. |
| Mankato | A very large timber floor is cut up by repetitive rock lines; terrace height and intended stairs are difficult to read. | Compose a real tiered quarry or bluff hall, with coherent cliff edges, obvious ascent and meaningful descent. |
| Minneapolis | Rows of identical bins dominate an otherwise empty lobby, while a narrow stack of barriers separates the leader. | Make an electrical works with visible machinery, a comprehensible circuit and a room organized around its actual controls. |
| St. Paul | Four almost empty checkerboard rectangles contain isolated pads; the room partitions carry nearly all visual identity. | Make the pad network part of a distinct steel or civic interior. Give each chamber a recognizable role and landmarks so destinations and return can be learned. |
| St. Cloud | Three blue platforms and repeated identical bands sit over a large featureless black void. | Make wind lanes readable as one believable elevated structure, with clear entry direction, landings and safe returns. |
| Brainerd | The room closely resembles Cerulean's pool path; its piers look like a generic brick maze and the old installed controls are not apparent. | Design a lake-and-dock space with legible movable hardware, meaningful pier choices and a clearly marked leader approach. |
| Bemidji | Continuous repeated tree bands and a nearly empty rectangular garden flatten the Cut route. | Build an inhabited northwoods garden with varied native planting, two meaningful Cut decisions and a visible earned return. |
| Duluth | A large blue room splits into two near-identical ice lanes; repeated grey slabs obscure which barriers change. | Compose distinct ice approaches with visible stops, readable fixed versus changing barriers and a strong leader court. |

## Integration warning

`planning/minnesota/gyms/current.json` is a later design index but is **not** a
source description of the selected region's installed rooms. Running
`review_gyms.py --verify-published` against the selected pack fails immediately:
the current-index Rochester design is 7×7 metatiles while the installed room
is 7×10. This is a concrete mismatch, not proof that the other seven rooms
match. The selected pack also contains later regional content and script
repairs. Any replacement must be built over that exact selected lineage,
preserve current parties, progression and exterior warp records, and account
for existing in-room save positions and block overrides. Source-file party data
in old gym candidates includes non-Minnesota species and must not overwrite the
active roster-derived parties.

Before selecting an improved pack, inspect each complete room and native camera
at all meaningful puzzle states; test the actual course, trainer and leader
battles, loss/retry, badge/TM, exits, re-entry and Continue through production
controls. Verify unrelated maps and pack payloads remain intact. Separately
prove the earned regional journey where claimed. None of those checks has been
run for a new design in this audit.
