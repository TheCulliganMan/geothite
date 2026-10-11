#!/usr/bin/env python3
"""Original Horsea and Seadra with swept coils, hollow muzzles and sculpted fins."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,tube,fitted_patch,fitted_contour,crease_normals
from bovine_sculpt import oval,curved_tube,reverse_front
from abra_sculpt import surface
from pitcher_sculpt import funnel
from wing_insect_sculpt import contour
from skin_glb import COORDINATES
SPECIES=('horsea','seadra');HEIGHT=1.07
BLUE=(.35,.70,.84);DEEP=(.28,.59,.76);CREAM=(.96,.91,.67);FOLD=(.64,.60,.43)
WHITE=(.98,.98,.94);CORAL=(.90,.44,.42);INK=(.08,.13,.16)

def fin(name,color,outline,sign):
    xy=[(sign*x,y) for x,y in outline]
    def depth(y,x):
        base=-.075-.10*abs(x)+.018*math.sin((y-.35)*8)
        return base+.045*math.sin(9*(abs(x)-.09))*math.sin(10*(y-.40)) if color==DEEP else base
    return crease_normals(fitted_contour(name,color,xy,depth,.009,subdivisions=3),40)

def fin_paint(name,outline,wing):
    front=surface(wing);back=surface(reverse_front(wing))
    q=fitted_contour(name,CREAM,outline,front,.003,subdivisions=3)
    vertices=[]
    for i in range(0,len(q['positions']),3):
        x,y,z=q['positions'][i:i+3]
        if z<front(y,x):z=-back(y,-x)-.003
        vertices.append((x,y,z))
    return crease_normals(part(name,CREAM,vertices,[q['indices'][i:i+3] for i in range(0,len(q['indices']),3)],True),35)

def sculpture(species):
    if species not in SPECIES:raise ValueError(species)
    adult=species=='seadra';color=DEEP if adult else BLUE
    body=loft('Body / continuous curved chest neck and tail root',color,curved_profiles(((.19,.035,.059,.025),(.26,.11,.095,.023),(.38,.15,.125,.017),(.50,.139,.113,.005),(.63,.086,.080,-.025),(.75,.067,.069,-.030),(.86,.060,.058,.003)),5),64)
    head=ellipsoid('Head / rounded seahorse hood',color,(0,.913,.005),(.202,.170,.174),64,36);p=[body,head]
    points=[(0,.245,.020),(-.056,.130,.029),(-.163,.092,.035),(-.257,.156,.039),(-.252,.257,.041),(-.159,.300,.042),(-.082,.243,.043),(-.120,.186,.044),(-.170,.209,.045)]
    p.append(curved_tube('Tail / continuous tapered inward spiral',color,points,[.075,.068,.057,.046,.037,.029,.022,.014,.005],40))
    skin=surface(body)
    belly_outline=[(-.037,.757),(.037,.757),(.050,.66),(.082,.56),(.105,.44),(.084,.33),(.023,.275),(-.029,.295),(-.085,.37),(-.088,.47),(-.061,.60)]
    belly=fitted_contour('Belly / fitted curved cream breast',CREAM,belly_outline,skin,.008,subdivisions=3);p.append(belly);bs=surface(belly)
    # Shallow transverse folds wrap the actual front of the fitted breast.
    for j,(y,w) in enumerate(((.348,.060),(.432,.080),(.516,.074),(.604,.050),(.690,.034))):
        pts=[]
        for k in range(33):
            x=w*(2*k/32-1);yy=y-.008*(1-(x/w)**2);pts.append((x,yy,bs(yy,x)+.003))
        p.append(crease_normals(tube('Belly fold / supported transverse crease '+str(j),FOLD,pts,[.0018 if adult else .0015]*len(pts),8),35))
    p.append(funnel('Snout / hollow flared blue muzzle',color,[(.116,.076,.068),(.20,.064,.056),(.30,.067,.061),(.422,.082,.073)],(0,.872,0),thickness=.016,tilt=.035,segments=64))
    p.append(funnel('Snout / recessed dark bore lining',INK,[(.235,.050,.043),(.30,.053,.047),(.418,.068,.059)],(0,.872,0),thickness=.004,tilt=.035,segments=64))
    p.append(ellipsoid('Snout / deep enclosed dark chamber',INK,(0,.878,.235),(.051,.044,.008),32,22))
    hs=surface(head)
    for sign in (-1,1):
        x=sign*.122;y=.945
        xy=[(-.056,-.009),(-.049,.020),(.025,.004),(.054,-.002),(.032,-.040),(-.022,-.036)] if adult else None
        outline=[(x+sign*a,y+b) for a,b in xy] if adult else oval(x,y,.043,.070,40)
        socket=fitted_contour('Eye socket / fitted dark outline '+str(sign),INK,outline,hs,.005,subdivisions=3);p.append(socket)
        cx=sum(a for a,b in outline)/len(outline);cy=sum(b for a,b in outline)/len(outline)
        white=fitted_contour('Eye white / fitted ivory '+str(sign),WHITE,[(cx+(a-cx)*.88,cy+(b-cy)*.88) for a,b in outline],surface(socket),.003,subdivisions=3);p.append(white)
        if adult:
            pupil=fitted_patch('Pupil / narrow watchful black '+str(sign),INK,oval(x-sign*.012,y-.014,.004,.012,24),surface(white),.0025);p.append(pupil)
        else:
            iris=fitted_patch('Iris / fitted coral oval '+str(sign),CORAL,oval(x-sign*.008,y,.022,.052,32),surface(white),.003);p.append(iris)
            pupil=fitted_patch('Pupil / narrow black '+str(sign),INK,oval(x-sign*.010,y,.006,.033,24),surface(iris),.002);p.append(pupil)
            p.append(fitted_patch('Glint / tiny warm light '+str(sign),WHITE,oval(x-sign*.011,y+.015,.0025,.006,16),surface(pupil),.0015))
        if adult:
            outline=[(.136,1.024),(.298,1.128),(.270,1.030),(.382,1.021),(.285,.944),(.397,.860),(.267,.856),(.290,.740),(.153,.815)]
            outline=[(x*.83,y) for x,y in outline]
            p.append(fin('Head fin / serrated blue cheek web '+str(sign),color,outline,sign))
            outline=[(.076,.61),(.472,.902),(.353,.669),(.592,.647),(.378,.518),(.467,.379),(.210,.421),(.084,.456)]
            wing=fin('Back fin / broad pointed blue sail '+str(sign),color,outline,sign);p.append(wing)
            for j,xy in enumerate(([(.270,.750),(.448,.864),(.361,.703)],[(.381,.650),(.566,.642),(.409,.565)],[(.280,.437),(.448,.400),(.381,.469)])):
                p.append(fin_paint('Fin tip / fitted ivory sail point '+str(sign)+' '+str(j),[(sign*a,b) for a,b in xy],wing))
            for j,(y,r) in enumerate(((.345,.028),(.442,.026),(.551,.021))):
                p.append(curved_tube('Armor / short rooted flank thorn '+str(sign)+' '+str(j),color,[(sign*.117,y,-.020),(sign*.173,y+.025,-.020),(sign*.221,y+.056,-.026)],[r,r*.7,.002],24))
        else:
            for j,(y,end_y) in enumerate(((1.024,1.122),(.927,.960),(.833,.778))):
                p.append(curved_tube('Head fin / rounded tapered ray '+str(sign)+' '+str(j),color,[(sign*.140,y,-.045),(sign*.207,(y+end_y)/2,-.077),(sign*.292,end_y,-.108)],[.037,.026,.0035],28))
            outline=contour([((.060,.551),(.16,.68),(.32,.77),(.40,.735)),((.40,.735),(.48,.668),(.39,.555),(.27,.519)),((.27,.519),(.17,.504),(.085,.523),(.060,.551))],12)
            wing=fin('Back fin / rounded ivory paddle '+str(sign),CREAM,outline,sign);p.append(wing)
            ws=surface(wing)
            for j,end in enumerate(((.378,.714),(.406,.647),(.326,.556))):
                pts=[]
                for k in range(25):
                    t=k/24;xx=sign*(.089+(end[0]-.089)*t);yy=.553+(end[1]-.553)*t+.014*math.sin(math.pi*t);pts.append((xx,yy,ws(yy,xx)+.002))
                p.append(crease_normals(tube('Fin ray / supported curved fold '+str(sign)+' '+str(j),FOLD,pts,[.0016]*len(pts),8),35))
    for i,q in enumerate(p):
        if q['part'].startswith(('Belly /','Eye','Iris','Pupil','Glint','Fin tip')):p[i]=crease_normals(q,35)
    floor=min(v for q in p for v in q['positions'][1::3]);height=max(v for q in p for v in q['positions'][1::3])-floor
    for q in p:
        q['positions']=[round((v-(floor if i%3==1 else 0.))*HEIGHT/height,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--out',type=Path,default=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species');a=a.parse_args()
    for n in SPECIES:
        m=sculpture(n);(a.out/(n+'.mesh.json')).write_text(json.dumps(m,separators=(',',':')));print(n,len(m['primitives']),sum(len(q['indices'])//3 for q in m['primitives']),flush=True)
if __name__=='__main__':main()
