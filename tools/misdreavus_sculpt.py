#!/usr/bin/env python3
"""Original flowing Misdreavus with swept pink-tipped locks and ruby necklace."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,ellipsoid,curved_profiles,fitted_patch,fitted_contour,crease_normals
from abra_sculpt import surface
from bovine_sculpt import oval,smooth,curved_tube
from wing_insect_sculpt import contour
from skin_glb import COORDINATES
SPECIES=('misdreavus',);HEIGHT=1.02
# Local Crystal normal RGB555: blue (9,9,17), pink (30,6,19).
BLUE=(74/255,74/255,140/255);HAIR=BLUE;PINK=(247/255,49/255,156/255)
RUBY=PINK;CORD=(0.,0.,0.);WHITE=(1.,1.,1.);CREAM=WHITE;IRIS=PINK;INK=CORD;GLOW=PINK

def lock_sections(points,widths,depths):
    path,ws=smooth(points,widths,6);ds=smooth(points,depths,6)[1];rows=[];n=24
    for k,(x,y,z) in enumerate(path):
        a=path[max(0,k-1)];b=path[min(len(path)-1,k+1)];dx=b[0]-a[0];dy=b[1]-a[1];length=math.hypot(dx,dy);nx=dy/length;ny=-dx/length
        rows.append([(x+nx*ws[k]*math.cos(j*math.tau/n),y+ny*ws[k]*math.cos(j*math.tau/n),z+ds[k]*math.sin(j*math.tau/n)) for j in range(n)])
    return rows

def lock_part(name,color,rows):
    n=len(rows[0]);v=[p for row in rows for p in row];f=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+j for j in range(n))]
    for k in range(len(rows)-1):
        for j in range(n):a=k*n+j;b=k*n+(j+1)%n;f.append((a,b,b+n,a+n))
    return crease_normals(part(name,color,v,f,True),35.)

def dress():
    n=60;rows=curved_profiles(((.35,.183,.136,-.020),(.43,.148,.122,-.020),(.49,.108,.090,-.010),(.575,.097,.082,.008),(.615,.046,.042,.016)),4);v=[]
    for j in range(n):
        a=j*math.tau/n;wave=math.cos(5*a);v.append(((.20+.026*wave)*math.sin(a),.280-.045*wave,-.020+(.146+.014*wave)*math.cos(a)))
    for k,(y,w,d,z) in enumerate(rows):
        fade=.45*max(0,1-(y-.35)/.11)
        for j in range(n):
            a=j*math.tau/n;wave=math.cos(5*a);v.append((w*(1+.10*fade*wave)*math.sin(a),y,z+d*(1+.08*fade*wave)*math.cos(a)))
    bottom=len(v);v.append((0,.30,-.020));f=[(bottom,(j+1)%n,j) for j in range(n)]
    for k in range(len(rows)):
        for j in range(n):a=k*n+j;b=k*n+(j+1)%n;f.append((a,b,b+n,a+n))
    f.append(tuple(len(rows)*n+j for j in range(n)))
    return crease_normals(part('Body / continuous five-lobed flowing ghost hem and neck',BLUE,v,f,True),38.)

def sculpture(species='misdreavus',normalize=True):
    if species not in SPECIES:raise ValueError('unsupported flowing ghost')
    body=dress();head=ellipsoid('Head / broad rounded spectral face',BLUE,(0,.760,.020),(.282,.235,.199),40,26);p=[body,head];hs=surface(head)
    recipes=[
      ('crown',[ (0,.887,-.055),(-.064,1.024,-.055),(-.217,1.105,-.045),(-.264,1.177,-.040),(-.248,1.247,-.040)],[.147,.135,.078,.029,.0016],[.105,.096,.052,.023,.0016],12),
      ('left upper',[(-.178,.813,-.063),(-.336,.881,-.060),(-.470,.819,-.045),(-.509,.703,-.020),(-.601,.671,.002),(-.658,.722,.012)],[.117,.124,.093,.063,.027,.0016],[.088,.085,.066,.043,.022,.0016],18),
      ('left lower',[(-.197,.700,-.105),(-.301,.632,-.145),(-.398,.511,-.148),(-.500,.475,-.126),(-.552,.520,-.116)],[.089,.084,.052,.020,.0016],[.071,.066,.043,.019,.0016],12),
      ('right upper',[(.170,.860,-.080),(.326,.919,-.084),(.455,.848,-.071),(.475,.776,-.038),(.420,.717,-.009)],[.106,.119,.087,.060,.008],[.089,.087,.069,.045,.008],None),
      ('right lower',[(.212,.678,-.109),(.323,.591,-.098),(.337,.459,-.070),(.420,.407,-.049),(.488,.436,-.033)],[.088,.095,.056,.023,.0016],[.079,.075,.045,.020,.0016],12),
      ('rear',[(0,.807,-.115),(.109,.824,-.279),(.231,.696,-.350),(.277,.575,-.326),(.359,.553,-.306)],[.138,.141,.109,.059,.0016],[.104,.099,.071,.041,.0016],18)]
    for name,points,widths,depths,seam in recipes:
        rows=lock_sections(points,widths,depths)
        p.append(lock_part('Hair / swept volumetric '+name,HAIR,rows if seam is None else rows[:seam+1]))
        if seam is not None:p.append(lock_part('Hair tip / tapered pink curl '+name,PINK,rows[seam:]))
    for s in (-1,1):
        outline=[(s*.048,.848),(s*.208,.864),(s*.204,.809),(s*.166,.756),(s*.114,.744),(s*.075,.766)]
        border=fitted_contour('Eye / fitted dark almond socket '+str(s),INK,outline,hs,.007);p.append(border)
        cx=sum(x for x,y in outline)/len(outline);cy=sum(y for x,y in outline)/len(outline)
        white=fitted_contour('Eye / warm gold sclera '+str(s),CREAM,[(cx+(x-cx)*.88,cy+(y-cy)*.88) for x,y in outline],surface(border),.003);p.append(white)
        iris=fitted_patch('Iris / large ruby oval '+str(s),IRIS,oval(s*.128,.802,.033,.039,28),surface(white),.003);p.append(iris)
        pupil=fitted_patch('Pupil / dark inward vertical oval '+str(s),INK,oval(s*.118,.809,.014,.026,24),surface(iris),.002);p.append(pupil)
        p.append(fitted_patch('Eye glint / ivory catchlight '+str(s),WHITE,oval(s*.117,.818,.006,.008,20),surface(pupil),.002))
    smile=contour([((-.050,.722),(-.032,.683),(.018,.688),(.051,.706)),((.051,.706),(.016,.680),(-.036,.675),(-.050,.722))],10)
    p.append(fitted_contour('Mouth / delicate fitted spectral smile',INK,smile,hs,.004))
    bs=surface(body);gems=[]
    for j,(x,y,r) in enumerate(((-.153,.600,.030),(-.089,.565,.047),(0,.535,.076),(.089,.565,.047),(.153,.600,.030))):
        try:z=hs(y,x)
        except ValueError:z=bs(y,x)
        z+=r*.46;gems.append((x,y,z));bead=ellipsoid('Necklace gem / graduated ruby orb '+str(j),RUBY,(x,y,z),(r,r,r*.87),28,18);p.append(bead);skin=surface(bead)
        p.append(fitted_patch('Gem glint / soft rose highlight '+str(j),GLOW,oval(x-r*.25,y+r*.32,r*.20,r*.24,20),skin,.002))
        p.append(fitted_patch('Gem glint / tiny ivory sparkle '+str(j),WHITE,oval(x-r*.39,y-r*.22,r*.09,r*.12,16),skin,.002))
    p.append(curved_tube('Necklace cord / curved front gem chain',CORD,gems,[.010]*len(gems),14))
    p.append(curved_tube('Necklace cord / fitted rear loop',CORD,[gems[0],(-.176,.609,-.019),(-.129,.610,-.105),(0,.611,-.135),(.129,.610,-.105),(.176,.609,-.019),gems[-1]],[.010]*7,14))
    p=[crease_normals(q,35.) if q['part'].startswith(('Eye /','Iris /','Pupil /','Eye glint /','Mouth /','Gem glint /')) else q for q in p]
    lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
    if normalize:
        for q in p:
            q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'misdreavus.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
