#!/usr/bin/env python3
"""Original distinct six-limbed spider anatomy, fitted dorsal markings."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,curved_profiles,tube,ellipsoid,fitted_patch,crease_normals
from abra_sculpt import surface
from skin_glb import COORDINATES
from chikorita_sculpt import add,sub,mul,dot,cross,unit

SPECIES=('spinarak','ariados');HEIGHT=.90
GREEN=(.58,.75,.38);RED=(.77,.23,.31);GOLD=(.97,.77,.35)
DARK=(.19,.30,.30);PURPLE=(.52,.44,.65);INK=(.09,.085,.12)
IVORY=(.94,.95,.85);CORAL=(.91,.38,.32)

def upper_skin(mesh):
    m=dict(mesh);m['positions']=[v for i in range(0,len(mesh['positions']),3)
        for v in (mesh['positions'][i],mesh['positions'][i+2],mesh['positions'][i+1])]
    return surface(m)

def dorsal_patch(name,color,outline,skin):
    dense=[]
    for a,b in zip(outline,outline[1:]+outline[:1]):
        dense.extend((a[0]+(b[0]-a[0])*i/5,a[1]+(b[1]-a[1])*i/5) for i in range(5))
    p=fitted_patch(name,color,dense,skin,.007)
    p['positions']=[v for i in range(0,len(p['positions']),3)
        for v in (p['positions'][i],p['positions'][i+2],p['positions'][i+1])]
    # Swapping axes is a reflection: keep winding and normals consistent.
    p['normals']=[v for i in range(0,len(p['normals']),3)
        for v in (p['normals'][i],p['normals'][i+2],p['normals'][i+1])]
    p['indices']=[v for i in range(0,len(p['indices']),3)
        for v in (p['indices'][i],p['indices'][i+2],p['indices'][i+1])]
    return p

def banded_limb(name,path,radii,colors):
    # Transport a single ring frame through the complete limb. Adjacent bands
    # use identical rings; independent tube frames left gaps at bent joints.
    rings=[];side=None;n=14
    for k,p in enumerate(path):
        tangent=unit(sub(path[min(k+1,len(path)-1)],path[max(0,k-1)]))
        if side is None:side=cross(tangent,(0,1,0) if abs(tangent[1])<.9 else (1,0,0))
        side=unit(sub(side,mul(tangent,dot(side,tangent))));other=cross(tangent,side)
        rings.append([add(p,mul(add(mul(side,math.cos(i*math.tau/n)),mul(other,math.sin(i*math.tau/n))),radii[k])) for i in range(n)])
    f=[tuple(reversed(range(n))),tuple(n+i for i in range(n))]
    f.extend((i,(i+1)%n,(i+1)%n+n,i+n) for i in range(n))
    return [crease_normals(part(name+' band '+str(k),c,rings[k]+rings[k+1],f),35.) for k,c in enumerate(colors)]


def legs(big):
    p=[];count=2 if big else 3
    for side in (-1,1):
        for j in range(count):
            z=.16-j*(.36 if big else .20);spread=.60 if big else .48
            hip=(side*.15,.29,z);knee=(side*(spread*.72),.40,z+(.07 if j==0 else -.03))
            foot=(side*spread,.032,z+(.23 if j==0 else -.16))
            def blend(a,b,t):return tuple(x+(y-x)*t for x,y in zip(a,b))
            # Four contiguous closed sections carry the bands; every section
            # shares its end ring with its neighbor, without floating cuffs.
            path=[hip,blend(hip,knee,.55),knee,blend(knee,foot,.38),blend(knee,foot,.73),foot]
            colors=(PURPLE if big else GOLD,PURPLE if big else DARK,GOLD,PURPLE if big else DARK,GOLD)
            radii=(.048,.046,.052,.044,.030,.004)
            p.extend(banded_limb(f'Walking leg / {side} {j}',path,radii,colors))
        if big:
            path=[(side*.17,.47,-.18),(side*.29,.58,-.27),(side*.35,.80,-.31),(side*.39,.98,-.34)]
            p.extend(banded_limb(f'Raised leg / {side}',path,(.046,.043,.038,.003),(GOLD,PURPLE,GOLD)))
    return p

def sculpture(species):
    if species not in SPECIES:raise ValueError('unsupported spider')
    big=species=='ariados';color=RED if big else GREEN
    abdomen=loft('Abdomen / tapered egg-shaped carapace',color,
        curved_profiles(((.13,.075,.15,-.15),(.20,.20,.27,-.18),(.32,.265,.33 if big else .28,-.19),
                         (.43,.225,.29 if big else .245,-.19),(.51,.13,.18,-.19),(.54,.012,.018,-.19)),3),28)
    head=loft('Head / rounded tapered cephalothorax',color,
        curved_profiles(((.15,.04,.048,.20),(.20,.115 if big else .17,.12,.23),
                         (.30,.16 if big else .21,.154,.235),(.40,.15 if big else .19,.13,.215),
                         (.46,.09 if big else .11,.08,.20),(.48,.016,.014,.20)),3),28)
    p=[abdomen,head]+legs(big);skin=surface(head);top=upper_skin(abdomen)
    for side in (-1,1):
        x=side*(.10 if big else .135);y=.325;z=skin(y,x)
        p.append(ellipsoid('Eye socket / '+str(side),PURPLE if big else INK,(x,y,z+.004),(.039 if big else .046,.055 if big else .047,.016),20,12))
        p.append(ellipsoid('Eye pupil / '+str(side),INK,(x,y+.004,z+.040),(.025,.037,.009),16,10))
        p.append(ellipsoid('Eye glint / '+str(side),IVORY,(x+side*.012,y+.018,z+.031),(.015,.019,.009),12,8))
        p.append(tube('Mandible / hooked '+str(side),IVORY if big else CORAL,
            [(side*.074,.215,.34),(side*.086,.15,.39),(side*.052,.114,.445)],
            [.031 if big else .034,.026,.003],18))
    if big:
        p.append(tube('Horn / forward ivory lance',IVORY,[(0,.426,.29),(0,.445,.42),(0,.449,.60)],[.056,.042,.002],18))
        # Segmented diagonal ribbons fit the top of the abdomen, rather than
        # duplicating Spinarak's false face on a red egg.
        for row,z in enumerate((-.055,-.265)):
            for side in (-1,1):
                outline=[(side*.015,z-.047),(side*.170,z-.088),(side*.170,z-.006),(side*.015,z+.040)]
                p.append(dorsal_patch(f'Abdomen stripe / fitted chevron {row} {side}',INK,outline,top))
    else:
        p.append(tube('Horn / upright ivory spike',IVORY,[(0,.442,.205),(0,.515,.217),(0,.605,.220)],[.036,.025,.002],18))
        for side in (-1,1):
            x=side*.09;z=-.18
            outline=[(x+.027*math.cos(a),z+.025*math.sin(a)) for a in (math.tau*i/14 for i in range(14))]
            p.append(dorsal_patch('False eye / fitted dorsal oval '+str(side),DARK,outline,top))
        line=[(-.072,-.042),(-.038,-.063),(0,-.069),(.038,-.063),(.072,-.042)]
        p.append(tube('False mouth / fitted dorsal frown',DARK,[(x,top(z,x)+.003,z) for x,z in line],[.009]*5,12))
        outline=[(-.098,-.40),(.098,-.40),(.075,-.36),(-.075,-.36)]
        p.append(dorsal_patch('Abdomen stripe / fitted rear bar',DARK,outline,top))
    low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
    for q in p:
        q['positions']=[round((v-(low if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
        q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    for s in SPECIES:(args.output/(s+'.mesh.json')).write_text(json.dumps(sculpture(s),separators=(',',':')))

if __name__=='__main__':main()
