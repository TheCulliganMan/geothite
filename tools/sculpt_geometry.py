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


