#!/usr/bin/env python3
"""Original playful Aipom with a curled tail and radial three-fingered tail hand."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,tube,fitted_patch,fitted_contour as mask
from abra_sculpt import surface
from bovine_sculpt import oval,curved_tube
from wing_insect_sculpt import contour
from skin_glb import COORDINATES
SPECIES=('aipom',);HEIGHT=1.02
PURPLE=(.58,.40,.68);CREAM=(.97,.89,.61);WHITE=(.98,.98,.95);INK=(.14,.13,.19);IRIS=(.37,.54,.58)


def sculpture(species='aipom'):
    if species not in SPECIES:raise ValueError('unsupported monkey')
    body=loft('Body / slim rounded monkey torso',PURPLE,curved_profiles(((.20,.04,.05,0),(.28,.135,.112,0),(.38,.13,.118,-.01),(.48,.115,.098,-.006),(.56,.097,.084,0),(.60,.06,.05,.015)),4),32)
    neck=ellipsoid('Neck / short soft supporting neck',PURPLE,(0,.581,.006),(.089,.093,.078),28,16)
    head=ellipsoid('Head / broad rounded purple crown',PURPLE,(0,.732,.005),(.247,.211,.190),40,28)
    p=[body,neck,head];hs=surface(head)
    p.append(fitted_patch('Belly / fitted cream chest oval',CREAM,oval(0,.434,.051,.099,32),surface(body),.009))
    face_outline=contour([((0,.803),(.052,.800),(.105,.849),(.154,.817)),((.154,.817),(.216,.791),(.219,.708),(.177,.659)),((.177,.659),(.130,.598),(.070,.596),(0,.599)),((0,.599),(-.070,.596),(-.130,.598),(-.177,.659)),((-.177,.659),(-.219,.708),(-.216,.791),(-.154,.817)),((-.154,.817),(-.105,.849),(-.052,.800),(0,.803))],9)
    face=mask('Face / fitted cream mask with crown notch',CREAM,face_outline,hs,.013);p.append(face);fs=surface(face)
    for s in (-1,1):
        ear=ellipsoid('Ear / large upright rounded pinna '+str(s),PURPLE,(s*.273,.800,-.005),(.104,.167,.065),32,22)
        # Tilt the actual pinna and recompute its normals using the same proper rotation.
        angle=-s*.29;c=math.cos(angle);ss=math.sin(angle)
        for field in ('positions','normals'):
            vals=[]
            for i in range(0,len(ear[field]),3):
                x,y,z=ear[field][i:i+3];xx=x-(s*.273 if field=='positions' else 0);yy=y-(.800 if field=='positions' else 0)
                vals.extend((xx*c-yy*ss+(s*.273 if field=='positions' else 0),xx*ss+yy*c+(.800 if field=='positions' else 0),z))
            ear[field]=vals
        p.append(ear);es=surface(ear)
        inner=fitted_patch('Ear / fitted cream inner pinna '+str(s),CREAM,oval(s*.278,.798,.060,.113,32),es,.005);p.append(inner)
        socket=fitted_patch('Eye / dark fitted round socket '+str(s),INK,oval(s*.105,.730,.058,.066,30),fs,.005);p.append(socket)
        white=fitted_patch('Eye / large ivory white '+str(s),WHITE,oval(s*.105,.730,.051,.060,30),surface(socket),.003);p.append(white)
        iris=fitted_patch('Eye / blue grey iris '+str(s),IRIS,oval(s*.104,.729,.034,.037,26),surface(white),.003);p.append(iris)
        pupil=fitted_patch('Pupil / round curious pupil '+str(s),INK,oval(s*.104,.731,.023,.026,24),surface(iris),.002);p.append(pupil)
        p.append(fitted_patch('Eye / tiny ivory glint '+str(s),WHITE,oval(s*.104-.008,.744,.008,.010,18),surface(pupil),.002))
        p.append(curved_tube('Leg / bent supporting monkey shin '+str(s),PURPLE,[(s*.088,.277,-.007),(s*.125,.172,-.020),(s*.14,.072,.023)],[.050,.043,.038],20))
        foot=ellipsoid('Foot / broad grounded cream pad '+str(s),CREAM,(s*.142,.046,.069),(.073,.046,.111),28,18);p.append(foot);fts=surface(foot)
        for j,dx in enumerate((-.023,.023)):
            x=s*.142+dx;p.append(tube(f'Toe fold / fitted rounded toe cleft {s} {j}',PURPLE,[(x,y,fts(y,x)+.001) for y in (.022,.042,.064)],[.0014,.0014,.001],10))
        path=[(s*.085,.521,.012),(s*.185,.459,.023),(s*.265,.458,.079)] if s<0 else [(s*.085,.521,.012),(s*.197,.574,.037),(s*.281,.600,.082)]
        p.append(curved_tube('Arm / curved playful reaching arm '+str(s),PURPLE,path,[.044,.037,.033],20))
        x,y,z=path[-1];hand=ellipsoid('Hand / rounded purple mitten '+str(s),PURPLE,(x,y,z),(.073,.042,.055),28,18);p.append(hand)
        p.append(fitted_patch('Hand / fitted cream palm '+str(s),CREAM,oval(x,y-.010,.056,.021,26),surface(hand),.004))
    grin_outline=contour([((-.082,.664),(-.049,.642),(.049,.642),(.082,.664)),((.082,.664),(.089,.628),(.040,.609),(0,.610)),((0,.610),(-.040,.609),(-.089,.628),(-.082,.664))],9)
    grin=mask('Grin / fitted dark crescent smile',INK,grin_outline,fs,.006);p.append(grin);gs=surface(grin)
    cx=sum(x for x,y in grin_outline)/len(grin_outline);cy=sum(y for x,y in grin_outline)/len(grin_outline)
    teeth=mask('Grin / single broad ivory tooth band',WHITE,[(cx+(x-cx)*.89,cy+(y-cy)*.77) for x,y in grin_outline],gs,.006);p.append(teeth);ts=surface(teeth)
    for j,x in enumerate((-.049,-.018,.018,.049)):
        ys=[]
        for k in range(100):
            y=.608+k*.0006
            try:ts(y,x);ys.append(y)
            except ValueError:pass
        ys=[ys[int((len(ys)-1)*t)] for t in (.12,.50,.88)]
        p.append(tube('Grin / fitted tooth division '+str(j),INK,[(x,y,ts(y,x)+.001) for y in ys],[.0013]*3,8))
    for j,(x,y,z,r) in enumerate(((0,.959,.009,.027),(-.026,.933,.014,.019),(.028,.931,.011,.018))):
        p.append(curved_tube('Tuft / soft pointed crown strand '+str(j),PURPLE,[(x*.4,.918,z),(x,y,z),(x*1.45,y+.067,z-.004)],[r,r*.69,.0015],18))
    tail=curved_tube('Tail / long curled prehensile tail',PURPLE,[(0,.295,-.071),(-.065,.287,-.195),(-.24,.330,-.31),(-.39,.49,-.36),(-.445,.73,-.30),(-.475,.952,-.225),(-.48,1.035,-.20)],[.045,.041,.037,.033,.030,.027,.025],22);p.append(tail)
    palm=ellipsoid('Tail hand / large rounded cream palm',CREAM,(-.48,1.055,-.20),(.118,.111,.077),32,22);p.append(palm)
    for j,a in enumerate((.21*math.pi,.70*math.pi,1.20*math.pi)):
        dx,dy=math.cos(a),math.sin(a);path=[(-.48+dx*.072,1.055+dy*.069,-.20),(-.48+dx*.155,1.055+dy*.155,-.196),(-.48+dx*.203,1.055+dy*.203,-.191)]
        p.append(curved_tube('Tail finger / short radiating cream digit '+str(j),CREAM,path,[.024,.027,.029],18))
        p.append(ellipsoid('Tail fingertip / soft bulbous cream pad '+str(j),CREAM,path[-1],(.053,.051,.045),26,18))
    lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
    for q in p:
        q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
    return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'aipom.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
