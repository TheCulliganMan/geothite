"""Standard-library closed surface helpers for original static model recipes.

Geometry only: species palettes, anatomical profiles and features live in their
own recipes. Coordinates are +Y up, +Z front. No generated mesh catalog.
"""
import math
from chikorita_sculpt import mesh,linear,add,sub,mul,dot,cross,unit

def part(name,color,vertices,faces,smooth=False):
    p=mesh(name,'body',vertices,faces,smooth)
    p['base_color']=linear(color);return p


def loft(name,color,profiles,segments=20):
    v=[(rx*math.sin(math.tau*i/segments),y,z+rz*math.cos(math.tau*i/segments))
       for y,rx,rz,z in profiles for i in range(segments)]
    f=[tuple(reversed(range(segments))),tuple((len(profiles)-1)*segments+i for i in range(segments))]
    for k in range(len(profiles)-1):
        for i in range(segments):
            a=k*segments+i;b=k*segments+(i+1)%segments
            f.append((a,b,b+segments,a+segments))
    return part(name,color,v,f,True)


def ellipsoid(name,color,center,radii,segments=16,rings=10):
    # Small nonzero polar rings retain a closed, nondegenerate authored solid.
    return translate(loft(name,color,[(center[1]+radii[1]*math.cos(math.pi*(i+.025)/(rings+.05)),radii[0]*math.sin(math.pi*(i+.025)/(rings+.05)),radii[2]*math.sin(math.pi*(i+.025)/(rings+.05)),center[2]) for i in reversed(range(rings+1))],segments),center[0],0,0)


def translate(p,x,y,z):
    p['positions']=[value+(x,y,z)[i%3] for i,value in enumerate(p['positions'])];return p


def tube(name,color,points,radii,segments=12):
    # Parallel-transport ring frame prevents abrupt striped-tail twists.
    v=[];f=[];side=None
    for k,p in enumerate(points):
        tangent=unit(sub(points[min(k+1,len(points)-1)],points[max(0,k-1)]))
        if side is None:side=cross(tangent,(0,1,0) if abs(tangent[1])<.9 else (1,0,0))
        side=unit(sub(side,mul(tangent,dot(side,tangent))));other=cross(tangent,side)
        v.extend(add(p,mul(add(mul(side,math.cos(i*math.tau/segments)),mul(other,math.sin(i*math.tau/segments))),radii[k])) for i in range(segments))
    f=[tuple(reversed(range(segments))),tuple((len(points)-1)*segments+i for i in range(segments))]
    for k in range(len(points)-1):
        for i in range(segments):
            a=k*segments+i;b=k*segments+(i+1)%segments;f.append((a,b,b+segments,a+segments))
    return part(name,color,v,f,True)


def fitted_patch(name,color,outline,surface,thickness=.008):
    """Closed shallow marking follows a real surface instead of floating on it.

    The XY outline must be convex. Three concentric front/back rings preserve
    the underlying curvature, and shared edges give a closed oriented solid.
    """
    outline=list(outline)
    if sum(a[0]*b[1]-b[0]*a[1] for a,b in zip(outline,outline[1:]+outline[:1]))<0:
        outline.reverse()
    cx=sum(x for x,y in outline)/len(outline);cy=sum(y for x,y in outline)/len(outline)
    xy=[(cx,cy)]
    for radius in (.33,.67,1.):xy.extend((cx+(x-cx)*radius,cy+(y-cy)*radius) for x,y in outline)
    front=[(x,y,surface(y,x)+thickness) for x,y in xy]
    back=[(x,y,surface(y,x)-thickness*.35) for x,y in xy]
    n=len(outline);count=len(front);faces=[]
    for i in range(n):faces.append((0,1+i,1+(i+1)%n))
    for ring in range(2):
        for i in range(n):
            a=1+ring*n+i;b=1+ring*n+(i+1)%n;faces.append((a,a+n,b+n,b))
    faces += [tuple(i+count for i in reversed(f)) for f in list(faces)]
    last=1+2*n
    faces += [(last+(i+1)%n,last+i,last+i+count,last+(i+1)%n+count) for i in range(n)]
    return part(name,color,front+back,faces)
