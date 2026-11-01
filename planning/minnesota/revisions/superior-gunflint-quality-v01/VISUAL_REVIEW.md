# Compact coast and Gunflint review

Status: final native v03 published and reviewed. Thirteen controller cases,
two asset cases, canonical Rust parity, audio verification and byte-identical
replay pass. Earlier detached-source stages below are retained review history.

The latest user correction removes the straight approach between North Shore
and Grand Marais, then shortens Gunflint. This is a replacement for the former
composition, not acceptance of its old technical baseline as the desired map.

## Coast

Reviewed `north-shore-ely-geography-v01/coast-quality-v01/compact-bend-v02/`
whole connected crop by sight. The terminal chunk is four rather than twenty-four
metatiles wide, placing town twenty metatiles closer to the existing northern
coast trail. North Shore stays north–south. The short north-end turn reaches the
west town entry, and the actual open Superior extends through both maps.

The first candidate used native buoy block30 on its unsupported east water
edge; its fourth quadrant was still WATER. That rejected epoch is retained.
Version02 uses visible native wall rocks there. Both walking lanes and the
shared water seam remain open, with all other perimeter cells closed.

## Gunflint

Reviewed source-v01, source-v03 and source-v06 whole maps and player contact
sheets by sight. The selected 32×32 source is substantially shorter than64×56.
It replaces the large route-wide rectangular circuits with one asymmetric lake,
a short bog circuit and a shelter/frost branch. The principal east-bank camera
now shows the lake; the initial candidate's broad empty east approach did not.

The first frost shape covered the cave front. The later native cave mouth has
a clear snow landing and ice beside the granite flank. Static traversal includes
straight native ice transitions, with a dry return around the basin.

An intermediate candidate overwrote two columns of the ranger facade with pine
strokes. Source-v06 restores the complete original three-column building, with
its native door and a clear return footprint. The source-native door is on the
last row of the upper chunk, so actual room return across the lower chunk seam
must be tested; a source picture cannot establish it.

The northwest Cut pocket remains genuinely sealed before Cut, and the short bog
grass neck lies between two water arms. Original green native Cut pads are
retained. The winter art is native; no source image is repainted to hide errors.

## Short snowline

Root inspected `superior-gunflint-quality-v01/cedar-source-v02/` whole map and all
four player cameras. The former 97-metatile inland connector span is replaced
by a20×12 approach. Two original optional trainers and the sign remain, with a
short grass bend, a downward native ledge and a short grassy uphill return.
The first pass accidentally opened extra southern border cells; the rejected
static result is retained and v02 brings that return inward.

Cedar is a brief forest transition rather than another major destination.
Grand Marais remains warm. The old long snowy Neck is retired; its original
BERRY×3 activity and flag migrate to Gunflint's ranger turnout.

## Completed native acceptance

The exact native v03 pack passes actual Foot/Surf/gate/room/Cut/ledge/encounter/
pickup inputs and Save/Continue. Root inspected the final connected atlas, all
twenty matched source/native player views, and seven actual reached-Save views.
The independent source raster and Rust canonical parity agree on every published
cell. See VERIFICATION.md for exact identities, outputs and local fixture scope.

## Bounded integration corrections retained as new epochs

Coast source-v03 also closes the *lower neighbor's* north overhang after the
upper chunk is cropped. Both sides of an unequal-width seam must be audited.
Root viewed its complete upper/lower coast and town image: the visible rock belt
frames genuinely unmapped space below town. That dark gap is retained honestly;
no water terrain is fabricated there. The real western lake passage remains
connected. The whole reachable Surf component and neighboring perimeters are
checked, in addition to the new controller regression for the overhang.

Gunflint source-v07 gives the second exterior door its unique native warp2;
source-v08 records the Python rasterizer's existing cliff-cut policy explicitly
for the Rust schema without changing any terrain. Source-v09 moves the original
REVIVE from the unreachable decorative roof to existing snow Floor west of the
cave mouth. Root inspected its updated cave camera. Walking audits must treat
CAVE/DOOR quadrants as terminals rather than paths through a building or ridge.
These changes preserve original items/flags/programs and all earlier epochs.

The final bounded v10 hut move solves the observed production connection-door
block: original northbound input could not enter a DOOR placed on the upper
chunk edge. Moving the complete hut one metatile north gives a normal Floor
approach and return in the upper chunk. Both carpets and actual PP-resource use
now pass. Coast v04 replaces three wrongly repeated west-bank corners with
horizontal native bank76; root inspected the corrected actual GPU shoreline.

Final camera-only corrections preserve the old occupied west-bank and ICE
preview positions as history while choosing legal unoccupied Foot starts. One
rejected capture plan retained the old hut chunk label after the position moved;
that failed capture/log is preserved. Cedar edge comparisons now use an exact
centered connected source view instead of a clamped small-map crop. All20final
fixture captures succeeded; native water animation can differ from the still.

Actual reached captures preserve their saved night clock. Native night tint
changes warm coast colors; its terrain remains the unchanged warm alias and
source art. No new visual defect was found in these retained reached views.
