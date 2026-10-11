#!/usr/bin/env python3
"""Original clear-wing dragonfly with jointed feet and a rising segmented tail."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,ellipsoid,tube,fitted_patch,fitted_contour,crease_normals
from abra_sculpt import surface
from bovine_sculpt import oval,curved_tube
from wing_insect_sculpt import contour
from skarmory_sculpt import blade
from skin_glb import COORDINATES
HEIGHT=.95
RED=(.89,.36,.27);DARK_RED=(.66,.26,.24);ORANGE=(.96,.45,.23);GREEN=(.51,.75,.36)
IVORY=(.96,.97,.94);VEIN=(.62,.67,.70);INK=(.06,.10,.14);LEG=(.31,.30,.38)

def horizontal(p):
    q=dict(p)
    for field in ('positions','normals'):
        v=p[field];q[field]=[a for i in range(0,len(v),3) for a in (v[i],v[i+2],-v[i+1])]
    return q

def clip(outline,axis,limit,above):
    result=[]
    for a,b in zip(outline,outline[1:]+outline[:1]):
        aa=(a[axis]>=limit) if above else (a[axis]<=limit);bb=(b[axis]>=limit) if above else (b[axis]<=limit)
        if aa:result.append(a)
        if aa!=bb:
            t=(limit-a[axis])/(b[axis]-a[axis]);result.append(tuple(a[k]+t*(b[k]-a[k]) for k in range(2)))
    return result

def wing_parts(sign,rear):
    if rear:
        outline=contour([((.036,.130),(.290,.170),(.620,.210),(.830,.265)),((.830,.265),(.925,.290),(.947,.326),(.884,.338)),((.884,.338),(.720,.325),(.250,.213),(.036,.172)),((.036,.172),(.022,.161),(.022,.141),(.036,.130))],12)
        paths=[[(.082,.159),(.300,.205),(.580,.260),(.856,.310)],[(.206,.170),(.460,.203),(.680,.239),(.864,.286)],[(.234,.204),(.436,.250),(.643,.293),(.842,.325)]]
    else:
        outline=contour([((.036,.010),(.250,-.014),(.610,-.156),(.800,-.152)),((.800,-.152),(.880,-.144),(.930,-.113),(.890,-.083)),((.890,-.083),(.700,-.002),(.270,.071),(.036,.041)),((.036,.041),(.022,.030),(.022,.017),(.036,.010))],12)
        paths=[[(.082,.026),(.300,.006),(.580,-.054),(.861,-.104)],[(.207,-.006),(.456,-.076),(.670,-.123),(.849,-.139)],[(.237,.039),(.443,.002),(.657,-.040),(.845,-.077)]]
    name=str(sign)+' '+str(int(rear));outline=[(sign*x,y) for x,y in outline]
    wing=blade('Wing / curved ivory dragonfly sail '+name,IVORY,outline,lambda x,y:.396+.035*abs(x)+.022*(y-.10)**2,.009);skin=surface(wing);p=[horizontal(wing)]
    band=clip(clip([(abs(x),y) for x,y in outline],0,.638,True),0,.772,False);band=[(sign*x,y) for x,y in band]
    cx=sum(x for x,y in band)/len(band);cy=sum(y for x,y in band)/len(band);band=[(cx+(x-cx)*.998,cy+(y-cy)*.998) for x,y in band]
    p.append(horizontal(fitted_contour('Wing band / fitted orange upper tip panel '+name,ORANGE,band,skin,.003)))
    # Matching lower panel on the closed underside; never a detached tip triangle.
    underside=dict(wing);underside['positions']=[-v if i%3==2 else v for i,v in enumerate(wing['positions'])];underside['normals']=[-v if i%3==2 else v for i,v in enumerate(wing['normals'])]
    underside['indices']=[j for i in range(0,len(wing['indices']),3) for j in (wing['indices'][i],wing['indices'][i+2],wing['indices'][i+1])]
    lower=fitted_contour('Wing band / fitted orange lower tip panel '+name,ORANGE,band,surface(underside),.003)
    lower['positions']=[-v if i%3==2 else v for i,v in enumerate(lower['positions'])];lower['normals']=[-v if i%3==2 else v for i,v in enumerate(lower['normals'])];lower['indices']=[j for i in range(0,len(lower['indices']),3) for j in (lower['indices'][i],lower['indices'][i+2],lower['indices'][i+1])];p.append(horizontal(lower))
    for j,path in enumerate(paths):
        points=[(sign*x,y,skin(y,sign*x)+.001) for x,y in path];p.append(horizontal(tube('Wing vein / fitted slender gray spar '+name+' '+str(j),VEIN,points,[.0016]*len(points),10)))
    return p

def collar(index,center,previous,following,radius):
    tangent=[following[i]-previous[i] for i in range(3)];mag=math.sqrt(sum(v*v for v in tangent));tangent=[v/mag for v in tangent];u=(1.,0.,0.);w=(0.,tangent[2],-tangent[1]);n=32;v=[]
    for along,r in ((-.009,radius+.006),(.009,radius+.006),(.009,radius-.007),(-.009,radius-.007)):
        for j in range(n):
            a=j*math.tau/n;v.append(tuple(center[i]+along*tangent[i]+r*(u[i]*math.sin(a)+w[i]*math.cos(a)) for i in range(3)))
    f=[]
    for k in range(4):
        for j in range(n):a=k*n+j;b=k*n+(j+1)%n;c=((k+1)%4)*n+(j+1)%n;d=((k+1)%4)*n+j;f.append((a,b,c,d))
    return crease_normals(part('Abdomen collar / dark red raised segment joint '+str(index),DARK_RED,v,f,True),35.)

def combined_surface(parts):
    skins=[surface(p) for p in parts]
    def at(y,x):
        hits=[]
        for s in skins:
            try:hits.append(s(y,x))
            except ValueError:pass
        if not hits:raise ValueError(('outside head',x,y))
        return max(hits)
    return at

def sculpture():
    thorax=ellipsoid('Thorax / elongated red dragonfly engine',RED,(0,.373,-.094),(.091,.078,.194),36,24);p=[thorax];heads=[]
    for sign in (-1,1):
        head=ellipsoid('Head / huge green compound eye housing '+str(sign),GREEN,(sign*.084,.439,.150),(.123,.138,.106),40,28);heads.append(head);p.append(head);skin=surface(head)
        eye=fitted_patch('Eye / glossy dark dragonfly lens '+str(sign),INK,oval(sign*.151,.434,.031,.047,32),skin,.006);p.append(eye);es=surface(eye)
        p.append(fitted_patch('Eye glint / broad ivory dragonfly reflection '+str(sign),IVORY,oval(sign*.151-.006,.453,.013,.018,24),es,.002))
        p.append(fitted_patch('Eye glint / tiny lower dragonfly sparkle '+str(sign),IVORY,oval(sign*.151+.008,.414,.005,.006,16),es,.002))
        p.append(curved_tube('Crown / swept red forehead antenna '+str(sign),RED,[(sign*.008,.477,.233),(sign*.036,.545,.232),(sign*.065,.624,.225)],[.026,.023,.001],20))
        for rear in (False,True):p.extend(wing_parts(sign,rear))
        for j,z in enumerate((.007,-.088,-.186)):
            p.append(curved_tube('Leg / angular jointed dragonfly strut '+str(sign)+' '+str(j),LEG,[(sign*.054,.350,z),(sign*(.165+j*.006),.286,z-.029),(sign*(.153+j*.009),.173,z+.013),(sign*(.207+j*.004),.124,z+.052)],[.016,.013,.011,.008],16))
            for fork in (-1,1):
                p.append(curved_tube('Foot / pointed forked dragonfly toe '+str(sign)+' '+str(j)+' '+str(fork),LEG,[(sign*(.207+j*.004),.124,z+.052),(sign*(.208+j*.004)+fork*.011,.100,z+.058),(sign*(.209+j*.004)+fork*.016,.092,z+.063)],[.009,.006,.001],12))
        p.append(ellipsoid('Mandible / rounded red mouth lobe '+str(sign),RED,(sign*.034,.341,.225),(.038,.030,.028),28,18))
    hs=combined_surface(heads);forehead=contour([((-.046,.499),(-.024,.470),(0,.458),(.023,.480)),((.023,.480),(.038,.489),(.041,.502),(.046,.510)),((.046,.510),(.014,.494),(-.012,.490),(-.046,.499))],8)
    p.append(fitted_contour('Forehead / fitted red central V saddle',RED,forehead,hs,.008))
    mouths=[q for q in p if q['part'].startswith('Mandible /')];ms=combined_surface(mouths)
    smile=contour([((-.060,.349),(-.026,.325),(.025,.326),(.060,.349)),((.060,.349),(.022,.332),(-.022,.330),(-.060,.349))],10)
    p.append(fitted_contour('Mouth / fitted dark mandible smile',INK,smile,ms,.0025))
    path=[(0,.375,-.199),(0,.419,-.359),(0,.521,-.554),(0,.658,-.751),(0,.679,-.821)];radii=[.073,.064,.052,.040,.036]
    abdomen=curved_tube('Abdomen / long continuous rising red tail',RED,path,radii,32);p.append(abdomen)
    for j in range(1,4):p.append(collar(j,path[j],path[j-1],path[j+1],radii[j]))
    bulb=ellipsoid('Tail cap / rounded terminal red segment',RED,(0,.682,-.821),(.057,.042,.059),32,20);p.append(bulb)
    for sign in (-1,1):
        fin=ellipsoid('Tail fin / broad lateral red steering paddle '+str(sign),RED,(sign*.123,.682,-.831),(.148,.018,.059),32,18);p.append(fin)
        p.append(curved_tube('Tail tip / tiny ivory posterior prong '+str(sign),IVORY,[(sign*.025,.694,-.848),(sign*.033,.707,-.901)],[.011,.001],12))
    p=[crease_normals(q,35.) if q['part'].startswith(('Eye /','Eye glint /','Forehead /','Mouth /')) else q for q in p]
    low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
    for q in p:
        q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
    return {'name':'yanma','version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
    (args.output/'yanma.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
