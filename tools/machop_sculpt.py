#!/usr/bin/env python3
"""Original Machop-family volumes, audited against game orbits and front/back idles."""
import argparse,math
from pathlib import Path
from functools import lru_cache
from sculpt_geometry import part,loft,tube,ellipsoid,translate,curved_profiles,crease_normals,fitted_triangle_contour
from bovine_sculpt import curved_tube,oval
from bulldog_sculpt import rounded_loft
from raptor_bird_sculpt import paint
from abra_sculpt import surface
from skin_glb import COORDINATES
from chikorita_sculpt import f32

HEIGHTS={'machop':.85,'machoke':1.15,'machamp':1.45}
PALETTE={'machop':(.53,.66,.68),'machoke':(.61,.56,.66),'machamp':(.48,.60,.64),
         'ridge':(.85,.70,.33),'ink':(.07,.065,.085),'white':(.97,.96,.87),
         'gold':(.94,.71,.26),'red':(.75,.23,.29),'lip':(.86,.75,.41),'tongue':(.63,.29,.36)}


def arm_paths(species):
    """Authored shoulder/elbow/wrist paths shared with the actual rig pivots."""
    rows=('upper','lower') if species=='machamp' else ('upper',)
    out=[]
    for row in rows:
        for side in (-1,1):
            if species=='machop':
                points=[(.13,.70,-.015),(.25,.65,.01),(.32,.59,.08),(.33,.62,.15),(.30,.66,.21)]
                radii=[.046,.064,.040,.046,.040]
            elif species=='machoke':
                points=[(.18,.91,-.025),(.345,.855,-.035),(.425,.705,-.010),(.43,.595,.055),(.38,.49,.10)]
                radii=[.078,.135,.075,.095,.058]
            elif row=='upper':
                points=[(.175,.94,-.040),(.38,.94,-.075),(.48,.92,-.005),(.50,1.035,.09),(.46,1.14,.15)]
                radii=[.068,.1125,.05355,.0782,.058]
            else:
                points=[(.19,.755,.025),(.36,.69,.00),(.455,.60,.035),(.40,.515,.12),(.32,.50,.18)]
                radii=[.078,.1125,.0544,.07905,.058]
            out.append((row,side,[(side*x,y,z) for x,y,z in points],radii))
    return tuple(out)


def solid_foot(species,side,spread):
    """One occupied sole with three/five blunt toe lobes, instead of round beads."""
    young=species=='machop';rx=.080 if young else .112;rz=.145 if young else .166
    rows=((0.,.88),(.016,1.),(.05,.98),(.087,.65),(.114,.36))
    segments=96;vertices=[];teeth=3 if young else 5
    for y,size in rows:
        for i in range(segments):
            a=math.tau*i/segments;u=math.cos(a);front=max(0.,math.sin(a))
            z=.045+size*(rz*math.sin(a)+.017*math.cos(teeth*math.pi*u)*front**4)
            vertices.append((side*spread*1.09+rx*size*u,y,z))
    # The toe outline is concave. A fan from its boundary can overlap a toe
    # valley; fans from the occupied sole center cover each cap once.
    bottom=len(vertices);vertices.append((side*spread*1.09,rows[0][0],.045))
    top=len(vertices);vertices.append((side*spread*1.09,rows[-1][0],.045))
    faces=[(bottom,(i+1)%segments,i) for i in range(segments)]
    for j in range(len(rows)-1):
        for i in range(segments):
            k=(i+1)%segments;faces.append((j*segments+i,j*segments+k,(j+1)*segments+k,(j+1)*segments+i))
    faces += [(top,(len(rows)-1)*segments+i,(len(rows)-1)*segments+(i+1)%segments) for i in range(segments)]
    return crease_normals(part('Foot / solid '+str(teeth)+' toe sole '+str(side),PALETTE[species],vertices,faces,True),45.)


def hands(parts,species,row,side,points):
    skin=PALETTE[species];young=species=='machop';relaxed=species=='machoke';size=.049 if young else .069
    wrist=points[-1];cx=wrist[0];cy=wrist[1]+(-.025 if relaxed else .026);cz=wrist[2]+.024
    hand=rounded_loft('Hand / '+row+' beveled palm '+str(side),skin,
        [(cy-size*.94,size*.62,size*.48,cz),(cy-size*.65,size,size*.65,cz),
         (cy+size*.60,size,size*.65,cz),(cy+size*.92,size*.65,size*.46,cz)],24,.68)
    parts.append(translate(hand,cx,0,0))
    count=3 if young else 4
    for finger in range(count):
        x=cx+(finger-(count-1)/2)*size*(.65 if young else .47)
        pts=[(x,cy-size*.35,cz+size*.30),(x,cy-size*1.20,cz+size*.40),(x,cy-size*1.32,cz+size*.64)] if relaxed else [(x,cy+size*.70,cz+size*.35),(x,cy+size*.50,cz+size*.89),(x,cy+size*.02,cz+size*.69)]
        parts.append(curved_tube('Finger / '+row+' '+str(side)+' '+str(finger),skin,pts,[size*.27,size*.28,size*.22],12))
    parts.append(ellipsoid('Thumb / '+row+' '+str(side),skin,(cx-side*size*.82,cy-.014,cz+size*.35),(size*.38,size*.59,size*.42),16,10))


def crests(parts,body,young):
    for ridge in (-1,0,1):
        x=ridge*(.10 if young else .095);top=(1.226 if young else 1.359)-.025*abs(ridge)
        yz=[(1.105,.17),(top-.055,.10),(top,-.015),(top-.065,-.15),(1.02 if young else 1.14,-.11),(1.02 if young else 1.14,.13)]
        if not young:yz=[(y+.06 if i==0 else y,z+.025 if i==0 else z) for i,(y,z) in enumerate(yz)]
        widths=(.019,.014,.002,.014,.018,.018)
        vertices=[(x+side*width,y,z) for side in (-1,1) for (y,z),width in zip(yz,widths)];n=len(yz)
        faces=[tuple(reversed(range(n))),tuple(n+i for i in range(n))]
        faces += [(i,(i+1)%n,(i+1)%n+n,i+n) for i in range(n)]
        parts.append(part('Crest / long swept plate '+str(ridge),PALETTE['ridge'],vertices,faces))


def face(parts,body,species):
    young=species=='machop';four=species=='machamp';skin=PALETTE[species];ink=PALETTE['ink']
    def fitted(name,color,outline,parent,thickness):
        return crease_normals(fitted_triangle_contour(name,color,outline,parent,thickness),35.)
    for side in (-1,1):
        x=side*(.112 if young else .111);y=1.032 if young else 1.205
        outline=oval(x,y+.003,.038,.071,32) if young else [(x-side*.051,y-.025),(x+side*.052,y+.002),(x+side*.043,y+.031),(x-side*.037,y+.020)]
        eye=fitted('Eye / fitted ivory almond '+str(side),PALETTE['white'],outline,body,.004);parts.append(eye)
        iy=y-.003 if young else y+.009
        iris=fitted('Eye / red iris '+str(side),PALETTE['red'],oval(x,iy,.021 if young else .009,.047 if young else .013,24),eye,.0025);parts.append(iris)
        pupil=fitted('Eye / dark pupil '+str(side),ink,oval(x,iy,.012 if young else .005,.031 if young else .008,20),iris,.0015);parts.append(pupil)
        if young:parts.append(fitted('Eye / small catchlight '+str(side),PALETTE['white'],oval(x-side*.002,y+.010,.0025,.004,14),pupil,.001))
        else:
            points=[(a,b,surface(body)(b,a)+.008) for a,b in ((side*.060,1.228),(side*.109,1.244),(side*.163,1.234))]
            parts.append(curved_tube('Brow / swept angular ridge '+str(side),skin,points,[.011,.015,.009],14))
    if young:
        # The juvenile's broad, shallow smile stays shut; only mature mouths
        # have a chamber. Its muzzle is part of the continuous head volume.
        pts=[(x,y,surface(body)(y,x)+.002) for x,y in ((-.13,.905),(-.07,.889),(0,.882),(.07,.889),(.13,.905))]
        parts.append(curved_tube('Mouth / fitted closed smile',ink,pts,[.002]*5,8))
        for side in (-1,1):parts.append(paint('Mouth / small nostril '+str(side),ink,oval(side*.025,.937,.006,.003,12),body,.002,2))
        return
    outline=oval(0,1.087,.155,.061,40)
    parts.append(crease_normals(fitted_triangle_contour('Mouth / recessed chamber lining',ink,outline,body,.0015),45.))
    if four:
        # A broad squared lip surrounds an empty chamber, rather than a tiny
        # yellow stripe painted over the old generic head.
        points=[(-.166,1.045,.265),(-.173,1.13,.285),(-.12,1.16,.325),(0,1.169,.34),(.12,1.16,.325),(.173,1.13,.285),(.166,1.045,.265)]
        parts.append(curved_tube('Mouth / broad upper lip',PALETTE['lip'],points,[.022]*7,18))
        jaw=rounded_loft('Mouth / broad lower jaw',PALETTE['lip'],[(1.005,.115,.065,.21),(1.025,.162,.105,.22),(1.055,.16,.10,.22),(1.066,.12,.065,.20)],32,.68)
        parts.append(jaw)
    else:
        parts.append(ellipsoid('Mouth / broad upper muzzle',skin,(0,1.158,.207),(.143,.043,.122),36,18))
        parts.append(rounded_loft('Mouth / broad lower jaw',skin,[(1.008,.10,.065,.16),(1.03,.153,.112,.205),(1.065,.15,.11,.207),(1.075,.115,.065,.17)],32,.72))
        for side in (-1,1):
            for lower in (False,True):
                points=[(side*.108,1.078 if lower else 1.12,.27),(side*.105,1.10 if lower else 1.092,.285)]
                parts.append(tube('Mouth / '+('lower' if lower else 'upper')+' fang '+str(side),PALETTE['white'],points,[.012,.0018],14))
    parts.append(ellipsoid('Mouth / seated tongue',PALETTE['tongue'],(0,1.075,.245),(.075,.012,.05),28,14))


def garment(parts,body,body_rows,species):
    ink=PALETTE['ink'];gold=PALETTE['gold']
    parts.append(loft('Trunks / fitted dark hip garment',ink,[(y,rx+.006,rz+.006,z) for y,rx,rz,z in body_rows if .29-1e-8<=y<=.50+1e-8],32))
    parts.append(loft('Belt / fitted dark waist band',ink,[(y,rx+.010,rz+.010,z) for y,rx,rz,z in body_rows if .46<=y<=.53],32))
    # The studs share one connected buried backing, not eight separate beads.
    def section(y):
        for a,b in zip(body_rows,body_rows[1:]):
            if a[0]<=y<=b[0]:
                t=(y-a[0])/(b[0]-a[0]);return [a[k]+(b[k]-a[k])*t for k in range(1,4)]
        raise ValueError('belt outside torso')
    vertices=[];count=128
    for y,relief in ((.479,0.),(.494,.022),(.509,0.)):
        rx,rz,cz=section(y)
        for j in range(count):
            a=math.tau*j/count;bulge=relief*max(0.,math.cos(10*a))**4
            vertices.append(((rx+.005+bulge)*math.cos(a),y,cz+(rz+.005+bulge)*math.sin(a)))
    faces=[tuple(reversed(range(count)))]
    for row in range(2):
        faces += [(row*count+j,row*count+(j+1)%count,(row+1)*count+(j+1)%count,(row+1)*count+j) for j in range(count)]
    faces.append(tuple(2*count+j for j in range(count)))
    parts.append(part('Belt / raised gold studs with buried backing',gold,vertices,faces,True))
    z=surface(body)(.496,0)
    buckle=ellipsoid('Belt / oval championship buckle',gold,(0,.496,z+.025),(.063,.046,.019),32,16);parts.append(buckle)
    parts.append(paint('Belt / inset red P emblem',PALETTE['red'],[(-.013,.473),(-.013,.521),(.014,.521),(.023,.513),(.023,.495),(.009,.49),(-.001,.49),(-.001,.473)],buckle,.002,2))


@lru_cache(maxsize=6)
def sculpture(species,normalize=True):
    if species not in HEIGHTS:raise ValueError('unsupported fighting species')
    young=species=='machop';four=species=='machamp';skin=PALETTE[species];parts=[]
    if young:
        profiles=((.24,.065,.07,-.015),(.34,.136,.107,-.015),(.44,.156,.117,-.015),(.54,.148,.108,-.01),(.64,.17,.114,-.01),(.72,.182,.111,-.008),(.79,.104,.094,.005),(.84,.166,.145,.035),(.91,.222,.179,.06),(1.015,.213,.17,.035),(1.10,.163,.14,.015),(1.15,.07,.07,0.))
    else:
        chest=.375 if four else .328
        profiles=((.29,.105,.112,-.02),(.40,.209,.157,-.02),(.50,.195 if not four else .212,.151,-.017),(.60,.161 if not four else .175,.13,-.015),(.72,.245 if not four else .274,.169,-.02),(.83,chest,.18,-.025),(.925,chest*.98,.18,-.028),(.99,.248 if not four else .285,.14,-.02),(1.035,.181 if not four else .193,.143,.035),(1.10,.201,.174,.06),(1.19,.195,.16,.035),(1.235,.174,.14,.015),(1.27,.11,.10,-.005),(1.28,.028,.035,-.015))
    body_rows=curved_profiles(profiles,4)
    if young:
        # Resolve each rib's crest and trough in actual geometry. A narrow
        # relief field evaluated only on distant torso rings erased the ribs.
        extra=[]
        for y in (h+dy for h in (.535,.573,.611) for dy in (-.015,-.008,0.,.008,.015)):
            for a,b in zip(body_rows,body_rows[1:]):
                if a[0]<y<b[0]:
                    t=(y-a[0])/(b[0]-a[0]);extra.append((y,*(a[k]+t*(b[k]-a[k]) for k in range(1,4))));break
        body_rows=sorted({f32(row[0]):(f32(row[0]),*row[1:]) for row in body_rows+extra}.values())
    body=loft('Body / joined pelvis waist chest neck jaw and cranium',skin,body_rows,48);vertices=[]
    for x,y,z in zip(body['positions'][::3],body['positions'][1::3],body['positions'][2::3]):
        front=max(0.,min(1.,(z+.015)/.13));back=max(0.,min(1.,(-z-.015)/.13))
        if young:
            pec=.019*math.exp(-((y-.69)/.065)**2)*(1-math.exp(-(x/.07)**2))
            ribs=sum(.020*math.exp(-((y-h)/.008)**2) for h in (.535,.573,.611))*math.exp(-((abs(x)-.075)/.065)**2)
            z+=(pec+ribs)*front**3
        else:
            pec=(.095 if four else .085)*math.exp(-((y-.85)/.105)**2)*(math.exp(-((x-.155)/.115)**2)+math.exp(-((x+.155)/.115)**2))
            z+=pec*front**3
            z-=(.04*math.exp(-((y-.9)/.10)**2)*(1-math.exp(-(x/.07)**2))-.012*math.exp(-(x/.035)**2)*math.exp(-((y-.84)/.16)**2))*back**3
            if four:z+=.115*math.exp(-((y-1.115)/.12)**4)*front**3
            chamber=(x/.162)**2+((y-1.087)/.068)**2
            if z>.04 and chamber<1:z-=(.24 if four else .16)*(1-chamber)**1.3
        vertices.append((x,y,z))
    body=crease_normals(part(body['part'],skin,vertices,[tuple(body['indices'][i:i+3]) for i in range(0,len(body['indices']),3)],True),48.);parts.append(body)
    spread=.13 if young else (.22 if four else .205)
    for side in (-1,1):
        parts.append(curved_tube('Leg / upright thigh knee and ankle '+str(side),skin,[(side*spread*.36,.43 if not young else .35,-.015),(side*spread,.315 if not young else .245,-.005),(side*spread*1.07,.205,-.003),(side*spread*1.09,.105,-.004)],[.093 if not young else .069,.10 if not young else .071,.064 if not young else .050,.051 if not young else .041],22))
        parts.append(solid_foot(species,side,spread))
    for row,side,points,radii in arm_paths(species):
        arm=curved_tube('Arm / '+row+' shaped deltoid biceps elbow forearm '+str(side),skin,points,radii,28);parts.append(arm)
        hands(parts,species,row,side,points)
        if species=='machoke':
            for lower in (False,True):
                for band in range(3):
                    x=side*((.302 if not lower else .41)+band*.018);y=(.856 if not lower else .651)-band*.026
                    outline=[(x-side*.006,y-.04),(x+side*.006,y-.04),(x+side*.006,y+.015),(x-side*.006,y+.015)]
                    parts.append(paint('Arm marking / '+('lower' if lower else 'upper')+' '+str(side)+' '+str(band),PALETTE['red'],outline,arm,.003,2))
    if young:parts.append(curved_tube('Tail / young curved taper',skin,[(0,.31,-.105),(0,.30,-.24),(.035,.32,-.36),(.07,.355,-.45)],[.053,.042,.025,.005],22))
    else:garment(parts,body,body_rows,species)
    face(parts,body,species);crests(parts,body,young)
    top=max(v for p in parts for v in p['positions'][1::3]);scale=HEIGHTS[species]/top if normalize else 1.
    for p in parts:
        p['positions']=[round(v*scale,7) for v in p['positions']];p['normals']=[round(v,7) for v in p['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':parts}


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--only',choices=HEIGHTS);p.add_argument('--out',type=Path,default=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species');a=p.parse_args()
    for name in ([a.only] if a.only else HEIGHTS):
        from fighting_glb import export_fighting
        a.out.mkdir(parents=True,exist_ok=True);blob=export_fighting(name);path=a.out/(name+'.glb');path.write_bytes(blob);print(name,len(blob),'bytes')

if __name__=='__main__':main()
