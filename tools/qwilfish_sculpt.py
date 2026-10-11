#!/usr/bin/env python3
"""Original inflated Qwilfish with rooted radial spines and a spotted fan tail."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import loft,ellipsoid,tube,fitted_patch,fitted_contour,crease_normals
from abra_sculpt import surface
from bovine_sculpt import oval,curved_tube,reverse_front
from wing_insect_sculpt import membrane,contour
from skin_glb import COORDINATES
SPECIES=('qwilfish',);HEIGHT=.91
TEAL=(.27,.52,.56);CREAM=(.88,.92,.64);PINK=(.92,.56,.66);INK=(.065,.085,.105);WHITE=(.98,.98,.94)
CENTER=.45;RADII=(.34,.30,.31)

def hemisphere(lower):
    a=math.pi-.012 if lower else math.pi/2;b=math.pi/2 if lower else .012
    rows=[]
    for i in range(17):
        t=a+(b-a)*i/16;rows.append((CENTER+RADII[1]*math.cos(t),RADII[0]*math.sin(t),RADII[2]*math.sin(t),0.))
    p=crease_normals(loft('Body / closed cream lower hemisphere' if lower else 'Body / closed teal upper hemisphere',CREAM if lower else TEAL,rows,48),35.)
    # Shared seam side normals come from the common ellipsoid. Keep internal
    # closing disk normals separate so the color boundary does not pinch.
    for i in range(0,len(p['positions']),3):
        n=p['normals'][i:i+3]
        if abs(n[1])>.99999 and abs(n[0])+abs(n[2])<.00001:continue
        x,y,z=p['positions'][i:i+3];v=(x/RADII[0]**2,(y-CENTER)/RADII[1]**2,z/RADII[2]**2);length=math.sqrt(sum(q*q for q in v));p['normals'][i:i+3]=[q/length for q in v]
    return p

def rotated_oval(x,y,w,h,angle,n=28):
    c=math.cos(angle);s=math.sin(angle)
    return [(x+c*(xx-x)-s*(yy-y),y+s*(xx-x)+c*(yy-y)) for xx,yy in oval(x,y,w,h,n)]

def sculpture(species='qwilfish'):
    if species not in SPECIES:raise ValueError('unsupported puffer fish')
    lower=hemisphere(True);upper=hemisphere(False);p=[lower,upper]
    lower_skin=surface(lower);upper_skin=surface(upper)
    def skin(y,x):return (lower_skin if y<CENTER else upper_skin)(y,x)
    directions=[(-.33,.94,-.07),(.44,.84,.16),(.04,.96,.25),(-.96,.19,.10),(.96,.19,.10),(-.64,.49,-.59),(.64,.49,-.59),(0,.64,-.77),(-.82,-.49,.28),(.82,-.49,.28),(0,-.81,.58),(-.65,-.55,-.53),(.65,-.55,-.53),(0,-.97,-.24)]
    for j,d in enumerate(directions):
        length=math.sqrt(sum(v*v for v in d));d=tuple(v/length for v in d);root=tuple(d[k]*RADII[k]+(CENTER if k==1 else 0) for k in range(3));extension=.173 if d[1]>.75 else .139 if d[1]<-.75 else .154
        points=[tuple(root[k]-(.028*d[k]) for k in range(3)),root,tuple(root[k]+extension*.38*d[k] for k in range(3)),tuple(root[k]+extension*d[k] for k in range(3))]
        p.append(tube('Spine / rooted '+('cream' if d[1]<0 else 'teal')+' radial cone '+str(j),CREAM if d[1]<0 else TEAL,points,[.020,.039,.030,.0012],14))
    for s in (-1,1):
        outline=[(s*.104,.524),(s*.211,.539),(s*.191,.621),(s*.161,.664),(s*.123,.598)]
        border=fitted_contour('Eye / fitted pointed dark socket '+str(s),INK,outline,skin,.006);p.append(border)
        cx=sum(x for x,y in outline)/len(outline);cy=sum(y for x,y in outline)/len(outline)
        white=fitted_contour('Eye / ivory triangular sclera '+str(s),WHITE,[(cx+(x-cx)*.84,cy+(y-cy)*.88) for x,y in outline],surface(border),.003);p.append(white)
        p.append(fitted_patch('Pupil / small wary black oval '+str(s),INK,oval(s*.163,.601,.008,.012,20),surface(white),.003))
        fin_outline=[(s*.107,.385),(s*.101,.294),(s*.192,.323)]
        fin=fitted_contour('Fin / small raised cream pectoral '+str(s),CREAM,fin_outline,skin,.014);p.append(fin)
        fold=[(s*x,y) for x,y in ((.110,.365),(.108,.303),(.164,.318),(.166,.322),(.112,.309),(.114,.365))]
        p.append(fitted_contour('Fin crease / fitted V fold '+str(s),TEAL,fold,surface(fin),.002))
    lips=ellipsoid('Lips / pursed soft pink oval',PINK,(0,.455,.306),(.076,.057,.043),36,22);p.append(lips)
    smile=contour([((-.047,.452),(-.021,.475),(.021,.475),(.047,.452)),((.047,.452),(.018,.469),(-.018,.469),(-.047,.452))],10)
    p.append(fitted_contour('Mouth / fitted short pursed lip seam',INK,smile,surface(lips),.003))
    p.append(curved_tube('Tail / curved teal caudal peduncle',TEAL,[(0,.386,-.250),(-.104,.306,-.431),(-.208,.265,-.596)],[.066,.038,.033],24))
    outline=contour([((-.200,.302),(-.275,.331),(-.412,.345),(-.493,.277)),((-.493,.277),(-.582,.204),(-.574,.103),(-.496,.056)),((-.496,.056),(-.426,.006),(-.341,.025),(-.294,.101)),((-.294,.101),(-.245,.170),(-.224,.249),(-.200,.302))],10)
    fan=membrane('Tail fin / rounded teal paddle border',TEAL,outline,1,root_z=-.570,thickness=.026);p.append(fan);fs=surface(fan)
    cx=sum(x for x,y in outline)/len(outline);cy=sum(y for x,y in outline)/len(outline)
    inset=[(cx+(x-cx)*.81,cy+(y-cy)*.81) for x,y in outline]
    cream=fitted_contour('Tail fin / fitted cream paddle center',CREAM,inset,fs,.004);p.append(cream);cs=surface(cream)
    for j,(x,y,w,h,a) in enumerate(((-.392,.251,.024,.049,-.67),(-.415,.123,.023,.052,-.45))):
        p.append(fitted_patch('Tail spot / fitted teal oblong '+str(j),TEAL,rotated_oval(x,y,w,h,a),cs,.003))
    reverse=reverse_front(fan);back_inset=[(-x,y) for x,y in inset]
    back_cream=fitted_contour('Tail fin / fitted cream reverse center',CREAM,back_inset,surface(reverse),.004);back_skin=surface(back_cream)
    p.append(reverse_front(back_cream))
    for j,(x,y,w,h,a) in enumerate(((-.392,.251,.024,.049,-.67),(-.415,.123,.023,.052,-.45))):
        p.append(reverse_front(fitted_patch('Tail spot / fitted rear teal oblong '+str(j),TEAL,rotated_oval(-x,y,w,h,-a),back_skin,.003)))
    p=[crease_normals(q,35.) if q['part'].startswith(('Eye /','Pupil /','Fin /','Fin crease /','Mouth /','Tail fin / fitted','Tail spot /')) else q for q in p]
    lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
    for q in p:
        q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'qwilfish.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
