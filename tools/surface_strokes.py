"""Closed round strokes fitted to original stored front surfaces."""
from sculpt_geometry import tube,crease_normals
from bovine_sculpt import smooth
from abra_sculpt import surface


def surface_stroke(name,color,points,radii,parent,segments=10):
    # Smooth the original XY controls first, then seat every resulting ring in
    # the stored surface. Interpolating sparse depths can leave narrow strokes
    # floating between controls. Preserve their outlines and round radii.
    centers,rr=smooth(points,radii);skin=surface(parent)
    centers=[(x,y,skin(y,x)+r*.35) for (x,y,z),r in zip(centers,rr)]
    return crease_normals(tube(name,color,centers,rr,segments),35.)
